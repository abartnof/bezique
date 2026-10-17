// Whole sittings played by the engine, every change of state choreographed:
// the cards must be conserved, each mesh must end on its own slot, and what
// the person can click (their cards, by id) must be where the state says.
import assert from "node:assert/strict";
import test from "node:test";
import { choreograph, initialPlacement, opening } from "../src/choreography.js";
import { fromState } from "../src/model.js";
import { layout } from "../src/layout.js";
import { decide, sitting } from "./support/table.js";

const present = (raw) => ({ model: fromState(raw), events: raw.events, raw });

function play(seed, level = 1) {
  return async () => {
    const { state, send } = await sitting({ seed, level });
    let rng = seed;
    const random = () => ((rng = (rng * 1103515245 + 12345) >>> 0) / 2 ** 32);
    let prev = opening();
    let placement = initialPlacement(prev);
    let moves = 0;
    for (let i = 0; i < 600; i++) {
      const raw = state();
      const next = present(raw);
      const result = choreograph(prev, next, placement);
      placement = result.placement;
      assert.equal(placement.length, 64);
      assert.equal(new Set(placement.map((p) => p.id)).size, 64, "every mesh used once");
      const truth = layout(next.model);
      placement.forEach((p) => {
        assert.equal(p.code, truth.find((t) => t.zone === p.zone && t.index === p.index).code, `${p.zone}[${p.index}] shows the state's card`);
      });
      for (const c of raw.hand ?? []) assert.ok(placement.some((p) => p.zone === "your-hand" && p.cardId === c.id), `card ${c.id} is placed in the hand`);
      assert.ok(result.motions.every((m) => m.delay >= 0 && m.duration > 0));
      prev = next;
      const command = decide(raw, random);
      if (!command) break;
      assert.ok(send(command), command);
      moves++;
    }
    assert.ok(moves > 20);
  };
}

for (const seed of [1, 2, 3, 4, 5, 6]) test(`a whole sitting, seed ${seed}`, play(seed));
test("a level 4 sitting", play(9, 4));
