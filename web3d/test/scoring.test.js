import assert from "node:assert/strict";
import test from "node:test";
import { combinationName, fold, itemsOf, apply, initial, itemsFor, doubleWatch } from "../src/scoring.js";
import { decide, sitting } from "./support/table.js";

function rngOf(seed) {
  let x = seed >>> 0 || 1;
  return () => ((x = (Math.imul(x, 1664525) + 1013904223) >>> 0) / 2 ** 32);
}

// Plays a game; after every command the events it brought are folded one by
// one, and the running figures must reach the engine's at the end of the batch.
async function reconcile({ level, seed, persona = 0 }) {
  const t = await sitting({ level, seed, persona });
  const rng = rngOf(seed * 7 + 1);
  let s = t.state();
  let run = initial();
  let seen = 0;
  const kinds = new Set();
  for (let n = 0; n < 800 && s.prompt.kind !== "over"; n++) {
    t.send(decide(s, rng));
    s = t.state();
    for (let i = seen; i < s.events.length; i++) {
      const items = itemsOf(s.events, i);
      for (const it of items) kinds.add(`${it.cat}`);
      run = apply(run, s.events[i], items);
    }
    seen = s.events.length;
    assert.deepEqual(run.totals, s.totals, `totals, seed ${seed} step ${n}`);
    // The hand's figures: reset to nothing once the hand has ended and no new one is dealt yet.
    if (s.hand_scores) assert.deepEqual(run.hand, s.hand_scores, `hand, seed ${seed} step ${n}`);
  }
  assert.equal(s.prompt.kind, "over");
  assert.equal(run.done, true);
  assert.deepEqual(run.outcome, s.outcome);
  return { s, run, kinds };
}

test("the running figures reconcile with the engine over whole games", async () => {
  const kinds = new Set();
  let doubles = 0;
  let games = 0;
  for (const level of [1, 3, 5]) {
    for (let seed = 1; seed <= 12; seed++) {
      const r = await reconcile({ level, seed });
      r.kinds.forEach((k) => kinds.add(k));
      games++;
      if (r.s.outcome.double) doubles++;
    }
  }
  for (const k of ["brisques", "declared", "seven", "last"]) assert.ok(kinds.has(k), `a game scored ${k}`);
  assert.ok(doubles > 0 && doubles < games, "both double and plain games occurred");
});

test("the ledger of hands adds up to the totals", async () => {
  const { run, s } = await reconcile({ level: 3, seed: 21 });
  const sum = { you: 0, them: 0 };
  for (const h of Object.values(run.hands)) for (const c of Object.values(h)) for (const w of ["you", "them"]) sum[w] += c[w];
  assert.deepEqual(sum, s.totals);
});

test("fold equals stepping", async () => {
  const { s, run } = await reconcile({ level: 2, seed: 5 });
  assert.deepEqual(fold(s.events), run);
  assert.equal(itemsFor(s.events).length, s.events.length);
});

test("the dealer's ten for a seven turned up, the dix, and the last trick are told apart", () => {
  const ev = [
    { kind: "dealt", hand: 1, dealer: "them", turn_up: { id: 1, card: "7H" } },
    { kind: "took_trick", hand: 1, who: "you", points: 20 },
    { kind: "dix", hand: 1, who: "you", taken: { id: 2, card: "7H" } },
    { kind: "took_trick", hand: 1, who: "you", points: 30 },
    { kind: "hand_ends", hand: 1, you: 60, them: 10 },
  ];
  assert.deepEqual(itemsOf(ev, 0).map((i) => [i.who, i.cat, i.pts]), [["them", "seven", 10]]);
  assert.deepEqual(itemsOf(ev, 1).map((i) => [i.cat, i.pts]), [["brisques", 20]]);
  assert.deepEqual(itemsOf(ev, 2).map((i) => [i.cat, i.label, i.pts]), [["seven", "dix", 10]]);
  assert.deepEqual(itemsOf(ev, 3).map((i) => [i.cat, i.pts]), [["brisques", 20], ["last", 10]]);
  const end = fold(ev);
  assert.deepEqual(end.totals, { you: 60, them: 10 });
  assert.deepEqual(end.hand, { you: 0, them: 0 });
});

test("combinations are named from the announcement", () => {
  const t = (said) => combinationName(`You declare: "${said}"`);
  assert.equal(t("Forty for bezique, and twenty to score."), "bezique");
  assert.equal(t("Five hundred for double bezique."), "double bezique");
  assert.equal(t("Royal marriage, forty."), "royal marriage");
  assert.equal(t("Twenty for the marriage in clubs."), "marriage");
  assert.equal(t("Two-fifty in trumps."), "sequence");
  assert.equal(t("Sixty queens, and forty, and twenty to score."), "four queens");
  assert.equal(t("A hundred aces."), "four aces");
});

test("the double line", () => {
  assert.equal(doubleWatch({ you: 900, them: 300 }).atRisk, true);
  assert.equal(doubleWatch({ you: 900, them: 600 }).atRisk, false);
  assert.equal(doubleWatch({ you: 100, them: 100 }).leader, null);
});
