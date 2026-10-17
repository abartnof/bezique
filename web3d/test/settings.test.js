import assert from "node:assert/strict";
import test from "node:test";
import { DEFAULTS, specsOf } from "../src/prefs.js";
import { settingsModel } from "../src/settings.js";
import { NEW_SPECS, erraticWord, normalise, opponentLine, parseSeed } from "../src/newgame.js";

test("the gear panel holds every in-game setting and no new-game one", () => {
  const rows = settingsModel(DEFAULTS).flatMap((s) => s.rows).map((r) => r.key);
  assert.deepEqual([...rows].sort(), specsOf("game").map((s) => s.key).sort());
  for (const k of ["level", "persona", "erratic", "seed"]) assert.ok(!rows.includes(k), k);
});

test("the model marks the selected option", () => {
  const row = settingsModel({ ...DEFAULTS, faces: "classic" }).flatMap((s) => s.rows).find((r) => r.key === "faces");
  assert.deepEqual(row.options.filter((o) => o.selected).map((o) => o.value), ["classic"]);
});

test("the New game dialog offers Surprise me and each persona", () => {
  const persona = NEW_SPECS.find((s) => s.key === "persona");
  assert.equal(persona.options[0].label, "Surprise me");
  assert.deepEqual(persona.options.map((o) => o.value), [0, 1, 2, 3, 4, 5, 6]);
  assert.match(opponentLine(3, 3), /aces and tens/);
});

test("seed parsing", () => {
  assert.deepEqual(parseSeed(""), { seed: null });
  assert.deepEqual(parseSeed(" 42 "), { seed: 42 });
  assert.ok(parseSeed("abc").error);
  assert.ok(parseSeed("99999999999").error);
});

test("normalise holds values to the specs", () => {
  assert.deepEqual(normalise({ level: "4", persona: "9", erratic: "5000" }), { level: 4, persona: 0, erratic: 1000 });
  assert.equal(normalise({ level: 3, persona: 1, erratic: 130 }).erratic, 150);
});

test("erraticism words", () => {
  assert.equal(erraticWord(0), "never");
  assert.equal(erraticWord(1000), "very often");
});
