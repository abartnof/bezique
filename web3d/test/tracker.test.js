import assert from "node:assert/strict";
import test from "node:test";
import { trackerModel, seenCodes } from "../src/tracker.js";
import { decide, sitting } from "./support/table.js";

function rngOf(seed) {
  let x = seed >>> 0 || 1;
  return () => ((x = (Math.imul(x, 1664525) + 1013904223) >>> 0) / 2 ** 32);
}

test("the unseen cards are exactly the stock and the opponent's hidden hand, at every state", async () => {
  let states = 0;
  for (const [level, seed] of [[1, 3], [3, 4], [5, 9], [2, 12]]) {
    const t = await sitting({ level, seed });
    const rng = rngOf(seed);
    let s = t.state();
    t.send("set tracker on");
    for (let n = 0; n < 800 && s.prompt.kind !== "over"; n++) {
      t.send(decide(s, rng));
      s = t.state();
      if (!s.hand || s.prompt.kind === "over") continue;
      const m = trackerModel(s);
      assert.ok(m, "model with the tracker on");
      assert.equal(m.unseen, s.stock + s.their_hidden, `seed ${seed} step ${n}: ${m.unseen} unseen, stock ${s.stock} hidden ${s.their_hidden}`);
      for (const k of Object.values(m.out)) assert.ok(k >= 0 && k <= 2);
      states++;
    }
  }
  assert.ok(states > 100);
});

test("with the aid off there is no count", async () => {
  const t = await sitting({ level: 1, seed: 2 });
  t.send("dealer you");
  assert.equal(trackerModel(t.state()), null);
  assert.equal(seenCodes(t.state()), null);
});

test("brisques, trumps and the bezique cards are told", () => {
  const view = {
    hand: [{ card: "AS" }, { card: "QS" }],
    table: [],
    their_table: [{ card: "JD" }],
    their_known: [],
    turn_up: { card: "9H" },
    trump: "hearts",
    stock: 20,
    their_hidden: 8,
    trick: null,
    tricks_played: [{ led: { card: "TH" }, second: { card: "AS" } }],
  };
  const m = trackerModel(view);
  assert.equal(m.out.AS, 0);
  assert.equal(m.out.QS, 1);
  assert.equal(m.out.JD, 1);
  assert.equal(m.out["9H"], 1);
  assert.match(m.lines.find((l) => l.id === "brisques").text, /13 of 16 \(6 aces, 7 tens\)/);
  assert.match(m.lines.find((l) => l.id === "trumps").text, /Trumps out: 14 of 16/);
  assert.ok(m.rows.find((r) => r.suit === "H").trump);
});
