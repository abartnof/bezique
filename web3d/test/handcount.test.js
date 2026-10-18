import assert from "node:assert/strict";
import test from "node:test";
import { choreograph, initialPlacement, opening } from "../src/choreography.js";
import { countAnchorOf, countPlan, lineFor, plansSince, unitsOf } from "../src/handcount.js";
import { fromState } from "../src/model.js";
import { itemsFor } from "../src/scoring.js";
import { decide, sitting } from "./support/table.js";

const sum = (units, who) => units.filter((u) => u.who === who).reduce((n, u) => n + u.pts, 0);
const increasing = (a) => a.every((t, i) => i === 0 || t > a[i - 1]);

const hand = [
  { kind: "dealt", hand: 1, dealer: "you", turn_up: { id: 1, card: "7S" } },
  { kind: "took_trick", hand: 1, who: "them", points: 20 },
  { kind: "declared", hand: 1, who: "you", points: 40, text: 'You declare: "Forty for bezique."' },
  { kind: "dix", hand: 1, who: "them", taken: true },
  { kind: "took_trick", hand: 1, who: "you", points: 20 },
  { kind: "declared", hand: 1, who: "them", points: 20, text: 'Your opponent declares: "Twenty for the marriage."' },
  { kind: "took_trick", hand: 1, who: "them", points: 10 },
  { kind: "hand_ends", hand: 1, you: 70, them: 60 },
];

test("a hand's units come in the ledger's order, each with its place and the tally so far", () => {
  const u = unitsOf(hand, 1);
  assert.deepEqual(
    u.map((x) => [x.cat, x.who, x.pts]),
    [["brisques", "them", 20], ["brisques", "you", 20], ["declared", "you", 40], ["declared", "them", 20], ["seven", "you", 10], ["seven", "them", 10], ["last", "them", 10]],
  );
  assert.deepEqual(u.map((x) => x.anchor), ["wonThem", "wonYou", "yourTable", "theirTable", "stock", "stock", "wonThem"]);
  assert.deepEqual(u.at(-1).running, { you: 70, them: 60 });
  assert.deepEqual(u.map((x) => x.stack), [0, 0, 0, 0, 0, 1, 0]); // the second at the stock lifts clear of the first
  assert.equal(u[2].badge.text, "+40 bezique");
});

test("the plan: units, the total, then (game end) the doubling and the victory", () => {
  const p = countPlan(hand, 7);
  assert.deepEqual(p.steps.map((s) => s.kind), [...Array(7).fill("unit"), "total"]);
  assert.deepEqual([p.steps.at(-1).you, p.steps.at(-1).them], [70, 60]);
  assert.ok(increasing(p.times), "strictly in order");
  assert.ok(p.end > p.times.at(-1));
  const over = [...hand.slice(0, 7), { kind: "game_ends", hand: 1, winner: "them", double: true }];
  const g = countPlan(over, 7);
  assert.deepEqual(g.steps.map((s) => s.kind), [...Array(7).fill("unit"), "total", "double", "victory"]);
  assert.ok(increasing(g.times));
  assert.equal(g.steps.at(-1).winner, "them");
  const plain = countPlan([...hand.slice(0, 7), { kind: "game_ends", hand: 1, winner: "you", double: false }], 7);
  assert.deepEqual(plain.steps.slice(-2).map((s) => s.kind), ["total", "victory"]);
  assert.equal(countPlan(hand, 3), null);
});

test("a game that ends on a hand already counted tells only the doubling and the victory", () => {
  const ev = [...hand, { kind: "game_ends", hand: 1, winner: "them", double: true }];
  assert.deepEqual(countPlan(ev, 8).steps.map((s) => s.kind), ["double", "victory"]);
  assert.deepEqual(plansSince(ev, 0).map((p) => p.kind), ["hand_ends", "game_ends"]);
  assert.deepEqual(plansSince(ev, 8).map((p) => p.kind), ["game_ends"]);
});

test("a long hand's beat shortens, within bounds", () => {
  assert.equal(lineFor(3), 650);
  assert.ok(lineFor(20) < 650 && lineFor(20) >= 280);
  assert.equal(lineFor(200), 280);
  assert.equal(countAnchorOf({ cat: "last", who: "you" }), "wonYou");
});

function rngOf(seed) {
  let x = seed >>> 0 || 1;
  return () => ((x = (Math.imul(x, 1664525) + 1013904223) >>> 0) / 2 ** 32);
}

// Whole games: every counted hand's units reconcile with the engine's own
// hand figures, and the choreography holds the table for exactly the plans.
test("over played-out games the count reconciles with the engine and the choreography holds for it", async () => {
  let games = 0;
  let doubles = 0;
  let ends = 0;
  for (const level of [1, 3]) {
    for (let seed = 1; seed <= 8; seed++) {
      const t = await sitting({ level, seed });
      const rng = rngOf(seed * 13 + level);
      let s = t.state();
      let prev = opening();
      let placement = initialPlacement(prev);
      for (let n = 0; n < 800; n++) {
        const next = { model: fromState(s), events: s.events, raw: s };
        const r = choreograph(prev, next, placement);
        placement = r.placement;
        const plans = plansSince(s.events, prev.events.length);
        const lines = plans.flatMap((p) => p.steps).length;
        assert.equal(r.count ? r.count.lines.length : 0, lines, `the held lines are the plans' steps, seed ${seed}`);
        if (r.count) assert.ok(r.count.lines.every((x, i, a) => i === 0 || x >= a[i - 1]) && r.count.end >= r.count.lines.at(-1));
        for (const p of plans) {
          ends++;
          const e = s.events[p.at];
          if (e.kind === "hand_ends") {
            assert.equal(sum(p.units, "you"), e.you, `hand ${e.hand} you, seed ${seed}`);
            assert.equal(sum(p.units, "them"), e.them, `hand ${e.hand} them, seed ${seed}`);
          } else if (p.units.length) {
            // The game ended mid-hand: the units are the hand so far.
            const mine = itemsFor(s.events).flat().filter((it) => it.hand === e.hand);
            for (const who of ["you", "them"]) assert.equal(sum(p.units, who), mine.filter((i) => i.who === who).reduce((a, i) => a + i.pts, 0));
          }
          if (e.kind === "game_ends") {
            const v = p.steps.at(-1);
            assert.equal(v.kind, "victory");
            assert.deepEqual(v.totals, s.totals);
            assert.equal(p.steps.some((x) => x.kind === "double"), Boolean(e.double));
            if (e.double) doubles++;
          }
        }
        prev = next;
        if (s.prompt.kind === "over") break;
        t.send(decide(s, rng));
        s = t.state();
      }
      assert.equal(s.prompt.kind, "over");
      games++;
    }
  }
  assert.ok(games === 16 && ends > 30 && doubles > 0);
});
