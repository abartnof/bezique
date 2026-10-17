// The table's layout, measured as the person sees it: every card inside the
// window and clear of the left-margin panels, no card's index corner hidden
// by a card of its own row or by another row, the piles thick enough to read
// as piles. Seen through the page's own camera (framing.js, EYE) at the
// desktop sizes.
import assert from "node:assert/strict";
import test from "node:test";
import { cameraFor } from "../../kit/js/framing.js";
import { JUMBO } from "../../kit/js/faces.js";
import { cardCorners } from "../../kit/js/kinematics.js";
import { CARD } from "../../kit/js/units.js";
import { layout, PILE_STEP, rowOffsets, spacingFor } from "../src/layout.js";
import { fromState, packModel } from "../src/model.js";
import { EYE, ZONES } from "../src/zones.js";
import { decide, sitting } from "./support/table.js";

const WINDOWS = [[1280, 720], [1440, 900], [1920, 1080]];
const PANEL = 360; // the left margin's panels (style.css), CSS pixels
const STRIP = (JUMBO.strip / JUMBO.w) * CARD.width; // the index's strip, cm

const screenOf = (w, h) => {
  const cam = cameraFor(w / h, PANEL / w, h, undefined, null, EYE);
  return (p) => {
    const v = p.clone().project(cam);
    return { x: ((v.x + 1) / 2) * w, y: ((1 - v.y) / 2) * h };
  };
};
const box = (pts) => ({ x0: Math.min(...pts.map((p) => p.x)), x1: Math.max(...pts.map((p) => p.x)), y0: Math.min(...pts.map((p) => p.y)), y1: Math.max(...pts.map((p) => p.y)) });

// The models of whole sittings, one per reply.
async function models(seed) {
  const { state, send } = await sitting({ seed, level: 2 });
  let rng = seed;
  const random = () => ((rng = (rng * 1103515245 + 12345) >>> 0) / 2 ** 32);
  const out = [];
  for (let i = 0; i < 400; i++) {
    const raw = state();
    out.push(fromState(raw));
    const command = decide(raw, random);
    if (!command || !send(command)) break;
  }
  return out;
}

// The fullest the table gets: a double bezique laid (four cards) among a
// long run of declared cards, and every card the opponent has shown.
function crowded() {
  const m = packModel();
  const codes = ["QS", "JD", "QS", "JD", "AD", "TD", "KD", "QD", "9H", "8H", "7C", "9C"];
  m.hand = ["AS", "KS", "7S", "8S", "AC", "KC", "QC", "JC"].map((code, id) => ({ code, id }));
  m.table = codes.map((code, id) => ({ code, id: 100 + id }));
  m.theirTable = ["QH", "JS", "QH", "JS", "AH", "TH", "KH", "QH", "7D", "8D"].map((code) => ({ code }));
  m.known = ["AD", "TD", "KD", "QD", "JD", "9D", "8D", "7D"];
  m.hidden = 8;
  m.stock = 0;
  m.trump = "D";
  m.trick = [{ who: "them", code: "9S" }, { who: "you", code: "TS" }];
  m.won = { you: 20, them: 14 };
  return m;
}

const sittings = [5, 12].map((seed) => ({ seed, all: models(seed) }));

test("every card lies inside the window and clear of the left panels, in every window and over whole games", async () => {
  const states = [crowded()];
  for (const { all } of sittings) states.push(...(await all).filter((_, i) => i % 3 === 0));
  assert.ok(states.length > 20);
  for (const [w, h] of WINDOWS) {
    const at = screenOf(w, h);
    for (const m of states) {
      for (const s of layout(m)) {
        const b = box(cardCorners(s.pose).map(at));
        assert.ok(b.x0 >= PANEL - 8 && b.x1 <= w && b.y0 >= 0 && b.y1 <= h, `${w}x${h} ${s.zone}[${s.index}]: x ${b.x0.toFixed(0)}..${b.x1.toFixed(0)}, y ${b.y0.toFixed(0)}..${b.y1.toFixed(0)}`);
      }
    }
  }
});

test("no card of a row hides the index corner of another card of it, however many lie", () => {
  for (const name of ["yourTable", "theirTable", "theirKnown"]) {
    const zone = ZONES[name];
    for (const n of [1, 4, 8, 12, 16]) {
      const codes = Array.from({ length: n }, (_, i) => "SHDC"[i % 4] + "AKQJ"[i % 4]);
      const x = rowOffsets(codes, zone);
      for (let i = 1; i < n; i++) assert.ok(x[i] - x[i - 1] >= STRIP, `${name} n=${n}: ${(x[i] - x[i - 1]).toFixed(2)} cm shown, the index strip is ${STRIP.toFixed(2)}`);
      assert.ok(x[n - 1] - x[0] + CARD.width <= zone.width + 1e-6, `${name} n=${n} runs past its width`);
    }
  }
  const z = ZONES.yourHand;
  assert.ok(spacingFor(16, z.spacing, z.width) >= STRIP, "a hand of sixteen shows every strip");
});

test("a card of one row does not cover a card of another, on the screen", async () => {
  const at = screenOf(1280, 720);
  const states = [crowded()];
  for (const { all } of sittings) states.push(...(await all).filter((_, i) => i % 5 === 0));
  const rows = ["your-hand", "your-table", "their-table", "their-known", "their-hand", "trick"];
  for (const m of states) {
    const slots = layout(m).filter((s) => rows.includes(s.zone) && s.zone !== "their-hand").map((s) => ({ s, b: box(cardCorners(s.pose).map(at)) }));
    for (let i = 0; i < slots.length; i++)
      for (let j = i + 1; j < slots.length; j++) {
        const a = slots[i], c = slots[j];
        if (a.s.zone === c.s.zone) continue;
        const dx = Math.min(a.b.x1, c.b.x1) - Math.max(a.b.x0, c.b.x0);
        const dy = Math.min(a.b.y1, c.b.y1) - Math.max(a.b.y0, c.b.y0);
        // A few pixels of shadow and perspective are no hiding.
        assert.ok(dx <= 4 || dy <= 4, `${a.s.zone}[${a.s.index}] covers ${c.s.zone}[${c.s.index}] (${dx.toFixed(0)} x ${dy.toFixed(0)} px)`);
      }
    // The opponent's fan does not run over their known cards' index corners.
    const fan = layout(m).filter((s) => s.zone === "their-hand").map((s) => box(cardCorners(s.pose).map(at)));
    const known = layout(m).filter((s) => s.zone === "their-known" || s.zone === "their-table").map((s) => box(cardCorners(s.pose).map(at)));
    const fanBottom = Math.max(0, ...fan.map((b) => b.y1));
    for (const b of known) assert.ok(b.y0 >= fanBottom - 4 || !fan.length, `a table card's top (${b.y0.toFixed(0)}) is under the opponent's fan (${fanBottom.toFixed(0)})`);
  }
});

test("a pile reads as a pile: the stock and the won piles have visible thickness", () => {
  const m = packModel();
  m.stock = 40;
  m.turnUp = "7D";
  m.won = { you: 12, them: 8 };
  const slots = layout(m);
  const top = (zone) => Math.max(...slots.filter((s) => s.zone === zone).map((s) => s.pose.position.y));
  assert.ok(top("stock") >= 40 * PILE_STEP * 0.95 && top("stock") >= 3, `stock ${top("stock").toFixed(2)} cm`);
  assert.ok(top("won-you") >= 1 && top("won-them") >= 0.7);
  // Piles stand clear of each other and of the play.
  const at = screenOf(1280, 720);
  const piles = ["stock", "turn-up", "won-you", "won-them"].map((z) => box(slots.filter((s) => s.zone === z).flatMap((s) => cardCorners(s.pose).map(at))));
  assert.ok(piles.every((b) => b.x1 - b.x0 > 20));
});

test("the eye's reach covers what the layouts need, and not far more", async () => {
  // reach.up / reach.foot, tangents of the eye's axis, are what framing.js keeps in view.
  const { Vector3 } = await import("three");
  const pos = new Vector3(...EYE.position);
  const f = new Vector3(...EYE.target).sub(pos).normalize();
  const r = new Vector3().crossVectors(f, new Vector3(0, 1, 0)).normalize();
  const u = new Vector3().crossVectors(r, f).normalize();
  const need = { up: -9, down: 9, across: 0 };
  const states = [crowded()];
  for (const { all } of sittings) states.push(...(await all).filter((_, i) => i % 4 === 0));
  for (const m of states)
    for (const s of layout(m))
      for (const c of cardCorners(s.pose)) {
        const v = c.clone().sub(pos);
        const d = v.dot(f);
        need.up = Math.max(need.up, v.dot(u) / d);
        need.down = Math.min(need.down, v.dot(u) / d);
        need.across = Math.max(need.across, Math.abs(v.dot(r)) / d);
      }
  assert.ok(EYE.reach.up >= need.up - 0.005 && EYE.reach.up <= need.up + 0.04, `up ${EYE.reach.up} vs ${need.up.toFixed(3)}`);
  assert.ok(EYE.reach.foot >= -need.down - 0.005 && EYE.reach.foot <= -need.down + 0.04, `foot ${EYE.reach.foot} vs ${(-need.down).toFixed(3)}`);
  assert.ok(EYE.widthTan >= need.across - 0.005 && EYE.widthTan <= need.across + 0.06, `width ${EYE.widthTan} vs ${need.across.toFixed(3)}`);
});
