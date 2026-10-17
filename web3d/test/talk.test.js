// What the opponent says: events in, lines out (src/talk.js).
import assert from "node:assert/strict";
import fs from "node:fs";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { createTalk } from "../src/talk.js";

const words = JSON.parse(fs.readFileSync(fileURLToPath(new URL("../words.json", import.meta.url)), "utf8"));
const lcg = (seed = 7) => () => ((seed = (seed * 1103515245 + 12345) >>> 0) / 2 ** 32);

function table({ mode = "chatty", persona = "balanced", rng = lcg() } = {}) {
  const said = [];
  const talk = createTalk({ words, rng, say: (l) => said.push(l), mode, persona });
  const ev = (e, ctx) => talk.onEvent(e, ctx);
  return { said, talk, ev, groups: () => said.map((s) => s.group) };
}
const dealt = (dealer = "them", card = "KC", hand = 1) => ({ kind: "dealt", hand, dealer, turn_up: { id: 1, card } });
const played = (who, card) => ({ kind: "played", hand: 1, who, card: { id: 1, card } });
const took = (who, points = 10) => ({ kind: "took_trick", hand: 1, who, points });
const declared = (who, text, points = 20) => ({ kind: "declared", hand: 1, who, text: `${who === "you" ? "You declare" : "Your opponent declares"}: "${text}"`, points });

test("the cut: the higher card chooses, and the sitting is greeted", () => {
  const t = table();
  t.ev({ kind: "cut", text: "x", yours: { id: 1, card: "AH" }, theirs: { id: 2, card: "9C" } });
  assert.deepEqual(t.groups(), ["start", "cut_you_high"]);
  const u = table();
  u.ev({ kind: "cut", text: "x", yours: { id: 1, card: "7H" }, theirs: { id: 2, card: "TC" } });
  assert.deepEqual(u.groups(), ["start", "cut_them_high"]);
});

test("the deal, and a seven turned up for the dealer", () => {
  const t = table();
  t.ev(dealt("them", "7D"));
  assert.deepEqual(t.groups(), ["deal_them", "seven_them"]);
  const u = table();
  u.ev(dealt("you", "7D", 2));
  assert.deepEqual(u.groups(), ["deal_you", "seven_you"]);
});

test("the opponent's own declaring is said in the engine's formula, as a call", () => {
  const t = table({ mode: "quiet" });
  t.ev(declared("them", "Forty for bezique."));
  assert.equal(t.said.length, 1);
  assert.equal(t.said[0].text, "Forty for bezique.");
  assert.equal(t.said[0].call, true);
});

test("remarks on your declaring are by combination", () => {
  const cases = [
    ["Twenty for the marriage in clubs.", "decl_you_marriage"],
    ["Royal marriage, forty.", "decl_you_royal"],
    ["Two-fifty in trumps.", "decl_you_sequence"],
    ["Forty for bezique.", "decl_you_bezique"],
    ["Five hundred for double bezique.", "decl_you_double"],
    ["Sixty queens.", "decl_you_four"],
    ["A hundred aces.", "decl_you_four"],
  ];
  for (const [text, group] of cases) {
    const t = table();
    t.ev(declared("you", text));
    assert.deepEqual(t.groups(), [group], text);
  }
});

test("a lucky declaration of the opponent's draws a remark after the formula", () => {
  const t = table();
  t.ev(declared("them", "A hundred aces.", 100));
  assert.deepEqual(t.groups(), ["decl_them", "decl_them_lucky"]);
});

test("quiet says calls only; normal rolls remarks at 0.6", () => {
  const q = table({ mode: "quiet" });
  q.ev(dealt());
  q.ev(took("them"));
  q.ev(played("you", "TH"));
  assert.deepEqual(q.groups(), ["deal_them"]);
  const lo = table({ mode: "normal", rng: () => 0.7 });
  lo.ev(took("them"));
  assert.deepEqual(lo.groups(), []);
  const hi = table({ mode: "normal", rng: () => 0.5 });
  hi.ev(took("them"));
  assert.deepEqual(hi.groups(), ["trick_them"]);
});

test("play-by-play is classified: trump, brisque, high, small", () => {
  const t = table({ rng: () => 0 });
  t.ev(dealt("you", "KH")); // hearts trump
  t.ev(played("them", "9H"));
  t.ev(played("you", "TS"));
  t.ev(played("them", "QC"));
  t.ev(played("you", "8D"));
  t.ev(played("them", "AD"));
  assert.deepEqual(t.groups().slice(1), ["play_them_trump", "play_you_brisque", "play_them_high", "play_you_low", "play_them_brisque"]);
});

test("a rich trick, and three running", () => {
  const t = table();
  t.ev(took("you", 20));
  assert.deepEqual(t.groups(), ["trick_rich_you"]);
  assert.match(t.said[0].text, /^20 /);
  const u = table({ rng: () => 0 });
  u.ev(took("them"));
  u.ev(took("them"));
  u.ev(took("them"));
  assert.deepEqual(u.groups(), ["trick_them", "trick_them", "streak_them"]);
  u.ev(took("you"));
  assert.equal(u.groups().at(-1), "trick_you");
});

test("the seven of trumps", () => {
  const t = table();
  t.ev({ kind: "dix", who: "them", taken: null });
  t.ev({ kind: "dix", who: "you", taken: { id: 3, card: "8S" } });
  assert.deepEqual(t.groups(), ["dix_them", "dix_you"]);
});

test("the stock running low, running out, and the last eight tricks", () => {
  const t = table();
  t.ev(dealt());
  for (let i = 0; i < 20; i++) t.ev({ kind: "drew", card: { id: i, card: "7S" } });
  assert.ok(!t.groups().includes("stock_low"));
  t.ev({ kind: "drew", card: { id: 99, card: "7S" } });
  assert.equal(t.groups().at(-1), "stock_low");
  t.ev({ kind: "stock_out", last_card: "you", turn_up: { id: 1, card: "KC" } });
  assert.deepEqual(t.groups().slice(-3), ["stock_out", "stock_last_you", "last_eight"]);
  for (let i = 0; i < 7; i++) t.ev(took("you"));
  assert.ok(!t.groups().includes("last_trick_you"));
  t.ev(took("them"));
  assert.equal(t.groups().at(-1), "last_trick_them");
});

test("the end of a hand: who is ahead, the double", () => {
  const ends = (you, them, tot) => {
    const t = table();
    t.ev({ kind: "hand_ends", you, them }, { totals: tot });
    return t.groups();
  };
  assert.deepEqual(ends(60, 90, { you: 100, them: 400 }), ["hand_ahead_them"]);
  assert.deepEqual(ends(60, 90, { you: 400, them: 100 }), ["hand_ahead_you"]);
  assert.deepEqual(ends(60, 90, { you: 300, them: 310 }), ["hand_level"]);
  assert.deepEqual(ends(60, 90, { you: 300, them: 820 }), ["double_you_threatened"]);
  assert.deepEqual(ends(60, 90, { you: 820, them: 300 }), ["double_them_threatened"]);
  const t = table();
  t.ev({ kind: "hand_ends", you: 0, them: 0 }, { totals: { you: 300, them: 820 } });
  t.ev({ kind: "hand_ends", you: 0, them: 0 }, { totals: { you: 520, them: 900 } });
  assert.deepEqual(t.groups(), ["double_you_threatened", "double_off_you"]);
});

test("the end of the game, and undo", () => {
  const cases = [
    [{ winner: "you", double: false }, "win_you"],
    [{ winner: "you", double: true }, "win_you_double"],
    [{ winner: "them", double: false }, "win_them"],
    [{ winner: "them", double: true }, "win_them_double"],
  ];
  for (const [e, group] of cases) {
    const t = table({ mode: "quiet" });
    t.ev({ kind: "game_ends", ...e });
    assert.deepEqual(t.groups(), [group]);
  }
  const t = table({ mode: "quiet" });
  t.ev({ kind: "undo" });
  assert.deepEqual(t.groups(), ["undo"]);
});

test("no line comes round again before the rest, nor twice running", () => {
  const t = table();
  const n = words.groups.trick_you.lines.length;
  for (let i = 0; i < n * 4; i++) {
    t.ev(took("you"));
    t.ev(took("them")); // so that no run of three comes between
  }
  const texts = t.said.filter((s) => s.group === "trick_you").map((s) => s.text);
  for (let i = 1; i < texts.length; i++) assert.notEqual(texts[i], texts[i - 1]);
  for (let k = 0; k < 4; k++) assert.equal(new Set(texts.slice(k * n, (k + 1) * n)).size, n);
});

test("a persona weights the same bank: concentrates is readier to speak of the stock", () => {
  const roll = () => 0.7; // above 0.6, below 0.6 * 1.5
  const a = table({ mode: "normal", rng: roll });
  const b = table({ mode: "normal", rng: roll, persona: "concentrates" });
  for (const t of [a, b]) {
    t.ev(dealt());
    for (let i = 0; i < 21; i++) t.ev({ kind: "drew", card: { id: i, card: "7S" } });
  }
  assert.ok(!a.groups().includes("stock_low"));
  assert.ok(b.groups().includes("stock_low"));
});

test("talk can be changed while playing, and refuses what it does not know", () => {
  const t = table({ mode: "quiet" });
  t.talk.setMode("chatty");
  t.ev(took("them"));
  assert.equal(t.said.length, 1);
  assert.throws(() => t.talk.setMode("loud"));
  assert.throws(() => t.talk.setPersona("nobody"));
});

test("whole sittings of the engine: every line said is in the bank, and none is empty", async () => {
  const { sitting, decide } = await import("./support/table.js");
  for (const seed of [1, 2, 3]) {
    const { state, send } = await sitting({ seed, level: 2 });
    const t = table();
    let seen = 0;
    let rng = seed;
    const random = () => ((rng = (rng * 1103515245 + 12345) >>> 0) / 2 ** 32);
    const totals = { you: 0, them: 0 };
    for (let i = 0; i < 500; i++) {
      const s = state();
      for (const e of s.events.slice(seen)) {
        if (e.kind === "took_trick" || e.kind === "declared") totals[e.who] += e.points;
        t.ev(e, { totals: { ...totals } });
      }
      seen = s.events.length;
      const c = decide(s, random);
      if (!c) break;
      send(c);
    }
    assert.ok(t.said.length > 20, `seed ${seed}: ${t.said.length}`);
    for (const l of t.said) {
      assert.ok(l.text.length > 0 && !/[{}]/.test(l.text), l.text);
    }
  }
});
