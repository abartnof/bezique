import assert from "node:assert/strict";
import fs from "node:fs";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { hintView, KEYS, REASONS, sourceLine } from "../src/hinttext.js";
import { decide, sitting } from "./support/table.js";

const hint = (key, extra = {}) => ({ command: "play 3", text: "Play the 7D.", reason: { key, ...extra } });

test("the keys here are exactly the keys the engine can send", () => {
  const rs = fs.readFileSync(fileURLToPath(new URL("../../crates/bezique-core/src/hints.rs", import.meta.url)), "utf8");
  const block = /pub const KEYS: \[&str; \d+\] = \[([^\]]*)\]/.exec(rs)[1];
  const rust = [...block.matchAll(/"([a-z_]+)"/g)].map((m) => m[1]).sort();
  assert.deepEqual([...KEYS].sort(), rust);
});

test("a hint names the cards and gives a rule with a source", () => {
  const v = hintView(hint("take_brisque", { card: "AS", led: "TD" }));
  assert.match(v.do, /ten of diamonds/);
  assert.match(v.do, /ace of spades/);
  assert.match(v.rule, /^Win a trick/);
  assert.match(v.why, /Foster/);
  assert.equal(v.interp, false);
});

test("every reason words itself from its parameters, with no holes", () => {
  const params = { card: "7D", led: "TC", kept: "QS", partner: "KH", spared: "AH", winner: "AC", turn_up: "KD", points: 40, trumps: 3, count: 4, meld: "royal_marriage", suit: "hearts" };
  for (const key of Object.keys(REASONS)) {
    const v = hintView(hint(key, params));
    assert.ok(v.do && v.rule && v.why, key);
    for (const field of [v.do, v.rule, v.why]) assert.doesNotMatch(field, /undefined|NaN|\[object/, key);
    assert.ok(v.do.length < 160 && v.rule.length < 190, `${key} is too long to be read`);
    assert.ok(REASONS[key].src.who && REASONS[key].src.page, key);
  }
});

test("the project's own readings are marked as such", () => {
  const v = hintView(hint("take_with_lowest_winner", { card: "9S", led: "8S" }));
  assert.equal(v.interp, true);
  assert.match(v.why, /project's reading/);
  assert.match(sourceLine(REASONS.throw_lowest.src), /^Cady/);
});

test("a guard hint names what to keep, and says plainly that it is common wisdom", () => {
  const v = hintView(hint("nothing_decisive", { card: "9H", kept: "AS" }));
  assert.match(v.do, /nine of hearts/);
  assert.match(v.do, /keep your ace of spades/);
  assert.match(v.rule, /never a trump, an ace, a ten/);
  assert.equal(v.interp, true);
  assert.match(v.why, /Common wisdom/);
});

test("the situations that had no rule now have one that names their facts", () => {
  const master = hintView(hint("lead_master_brisque", { card: "TC", kept: "AC" }));
  assert.match(master.do, /ten of clubs/);
  assert.match(master.do, /ace of clubs stays/);
  assert.match(hintView(hint("lead_master_brisque", { card: "TC" })).do, /only a trump can take it\.$/);
  const spare = hintView(hint("take_sparing_pieces", { card: "TC", led: "9C", kept: "QC" }));
  assert.match(spare.do, /your queen of clubs, is part of a combination/);
  const solved = hintView(hint("solved_endgame", { card: "8C", count: 3 }));
  assert.match(solved.do, /exactly the 3 cards/);
  assert.match(solved.rule, /last eight tricks/);
  assert.equal(REASONS.lead_cheapest, undefined);
});

test("fact and rule layers switch separately", () => {
  const h = hint("throw_lowest", { card: "7D" });
  const only = (layers) => hintView(h, layers);
  assert.equal(only({ rules: false }).rule, "");
  assert.equal(only({ rules: false }).why, "");
  assert.ok(only({ rules: false }).do);
  assert.equal(only({ facts: false }).do, "");
  assert.ok(only({ facts: false }).rule);
});

test("an unknown or missing reason falls back to the engine's text", () => {
  const v = hintView(hint("something_new"));
  assert.equal(v.do, "Play the 7D.");
  assert.equal(v.rule, "");
  assert.equal(hintView({ command: "play 3", text: "Play the 7D." }).do, "Play the 7D.");
  assert.equal(hintView(null), null);
  const v2 = hintView(hint("take_brisque", {}));
  assert.ok(v2.do.length > 0);
});

test("real hints from played sittings all word themselves", async () => {
  let seen = 0;
  for (const seed of [1, 2, 3]) {
    const { state, send } = await sitting({ level: 2, seed });
    send("set hints on");
    let rng = seed;
    const random = () => ((rng = (rng * 1103515245 + 12345) >>> 0) / 2 ** 32);
    for (let i = 0; i < 400; i++) {
      const s = state();
      if (s.hint) {
        const v = hintView(s.hint);
        assert.ok(KEYS.includes(s.hint.reason.key), s.hint.reason.key);
        assert.ok(v.do, JSON.stringify(s.hint));
        assert.doesNotMatch(v.do + v.rule + v.why, /undefined|NaN/, JSON.stringify(s.hint));
        seen++;
      }
      const c = decide(s, random);
      if (!c) break;
      send(c);
    }
  }
  assert.ok(seen > 30);
});
