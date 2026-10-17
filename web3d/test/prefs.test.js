import assert from "node:assert/strict";
import test from "node:test";
import { DEFAULTS, KEY, PREF_SPECS, gameConfig, loadPrefs, migrate, savePrefs, speedFactor, specsOf, urlOverrides, withUrl } from "../src/prefs.js";

const mem = (init) => {
  const m = new Map(init ? [[KEY, init]] : []);
  return { getItem: (k) => m.get(k) ?? null, setItem: (k, v) => m.set(k, v), m };
};

test("defaults declare every spec; keys are unique", () => {
  assert.equal(new Set(PREF_SPECS.map((s) => s.key)).size, PREF_SPECS.length);
  for (const k of ["faces", "hintRules", "tutor", "talk", "tracker", "level", "persona", "erratic", "speed", "surface"]) assert.ok(k in DEFAULTS, k);
});

test("a round trip keeps the values; bad ones take the default", () => {
  const s = mem();
  assert.ok(savePrefs({ ...DEFAULTS, level: 5, persona: 3, erratic: 250, faces: "classic" }, s));
  assert.deepEqual(loadPrefs(s), { ...DEFAULTS, level: 5, persona: 3, erratic: 250, faces: "classic" });
  const bad = mem(JSON.stringify({ v: 1, level: 9, faces: "wide", erratic: 5000, talk: "chatty" }));
  assert.deepEqual(loadPrefs(bad), { ...DEFAULTS, talk: "chatty" });
});

test("the old {level,hints} shape migrates", () => {
  assert.deepEqual(migrate({ level: 4, hints: true }), { level: 4, hints: true });
  assert.deepEqual(loadPrefs(mem(JSON.stringify({ level: 4, hints: true }))), { ...DEFAULTS, level: 4, hints: true });
});

test("a newer record is neither read nor overwritten; broken storage is survived", () => {
  const s = mem(JSON.stringify({ v: 9, level: 5 }));
  assert.deepEqual(loadPrefs(s), DEFAULTS);
  assert.equal(savePrefs(DEFAULTS, s), false);
  const throwing = { getItem() { throw new Error("blocked"); }, setItem() { throw new Error("blocked"); } };
  assert.deepEqual(loadPrefs(throwing), DEFAULTS);
  assert.equal(savePrefs(DEFAULTS, throwing), false);
  assert.deepEqual(loadPrefs(mem("{not json")), DEFAULTS);
  assert.equal(savePrefs(DEFAULTS, null), false);
});

test("the URL overrides, within the allowed values", () => {
  const p = withUrl(DEFAULTS, new URLSearchParams("level=2&hints=1&speed=instant&persona=7&erratic=300&table=dot-grid&faces=x"));
  assert.equal(p.level, 2);
  assert.equal(p.hints, true);
  assert.equal(p.speed, "instant");
  assert.equal(p.persona, DEFAULTS.persona);
  assert.equal(p.erratic, 300);
  assert.equal(p.surface, "dot-grid");
  assert.equal(p.faces, DEFAULTS.faces);
  assert.equal(withUrl({ ...DEFAULTS, hints: true }, new URLSearchParams("hints=0")).hints, false);
  assert.deepEqual(urlOverrides(new URLSearchParams("seed=5&fresh=1")), { seed: 5, fresh: true });
  assert.equal(urlOverrides(new URLSearchParams("seed=-1")).seed, null);
});

test("gameConfig carries persona and erraticism", () => {
  assert.deepEqual(gameConfig({ ...DEFAULTS, level: 4, persona: 2, erratic: 100 }, { seed: 7 }), { level: 4, seed: 7, persona: 2, erratic: 100 });
});

test("the new-game specs are exactly level, persona, erratic", () => {
  assert.deepEqual(specsOf("new").map((s) => s.key), ["level", "persona", "erratic"]);
});

test("speed factor: reduced motion plays at once", () => {
  assert.equal(speedFactor({ ...DEFAULTS }, false), 1);
  assert.equal(speedFactor({ ...DEFAULTS, speed: "fast" }, false), 2.2);
  assert.equal(speedFactor({ ...DEFAULTS }, true) > 100, true);
  assert.equal(speedFactor({ ...DEFAULTS, reducedMotion: "off" }, true), 1);
});
