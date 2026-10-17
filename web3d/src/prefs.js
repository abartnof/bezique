// The person's settings, kept in the browser under `bezique.prefs.v1`.
//
// Two kinds, declared once in PREF_SPECS:
//   scope "game": within the game, in the gear panel (settings.js), each taking
//     effect at once and reported to the page through onChange(key, value);
//   scope "new":  only when a new game starts, in the New game dialog
//     (newgame.js) and nowhere else: the opponent's skill, personality and
//     slips. What is kept for them is the last game's choice, prefilled next time.
// Storage that is missing, full or blocked (a private window) is survived: the
// page then plays from the defaults. The stored record carries `v`, the version
// of the defaults, so a default changed later is not taken for a choice.
import { PATTERNS } from "../../kit/js/surfaces.js";

export const KEY = "bezique.prefs.v1";
export const VERSION = 1;

const opts = (list) => list.map(([value, label, line]) => ({ value, label, ...(line ? { line } : {}) }));
const onOff = opts([[true, "On"], [false, "Off"]]);

export const LEVELS = opts([[1, "1"], [2, "2"], [3, "3"], [4, "4"], [5, "5"]]);

// The engine's personas (crates/bezique-core/src/opponent.rs `personas()`),
// 1 to 4 in that order; 0 draws one from the seed.
export const PERSONAS = opts([
  [0, "Surprise me", "Its way of playing is drawn from the deal."],
  [1, "Balanced", "Plays a balanced game."],
  [2, "Concentrates", "Throws from its shortest suits, to be void of them."],
  [3, "Close", "Plays from the table first, showing you as little as it can."],
  [4, "Bezique", "Will give almost anything for double bezique."],
]);

export const LEVEL_LINES = Object.freeze({
  1: "Plays any card it may.",
  2: "Takes what it can.",
  3: "Counts what has been played.",
  4: "Remembers every card.",
  5: "The strongest the engine plays.",
});

// kind: "choice" (a row of buttons), "toggle", "select" (a dropdown), "range".
export const PREF_SPECS = Object.freeze([
  // ---- within the game
  { key: "surface", scope: "game", section: "table", kind: "select", label: "Table top", default: "random", options: opts([["random", "Random"], ...PATTERNS.map((p) => [p.id, p.name])]) },
  { key: "faces", scope: "game", section: "table", kind: "choice", label: "Card faces", default: "jumbo", options: opts([["jumbo", "Jumbo"], ["classic", "Classic"]]) },
  { key: "speed", scope: "game", section: "table", kind: "choice", label: "Dealing speed", default: "normal", options: opts([["normal", "Normal"], ["fast", "Fast"], ["instant", "Instant"]]) },
  { key: "reducedMotion", scope: "game", section: "table", kind: "choice", label: "Reduced motion", default: "auto", options: opts([["auto", "Auto"], ["on", "On"], ["off", "Off"]]) },
  { key: "hints", scope: "game", section: "help", kind: "toggle", label: "Hints", default: false, options: onOff },
  { key: "hintRules", scope: "game", section: "help", kind: "toggle", label: "Hints name the rule", default: true, options: onOff },
  { key: "tracker", scope: "game", section: "help", kind: "toggle", label: "Card tracker", default: true, options: onOff },
  { key: "tutor", scope: "game", section: "help", kind: "toggle", label: "Tips and tutorial", default: true, options: onOff },
  { key: "talk", scope: "game", section: "help", kind: "choice", label: "Table talk", default: "normal", options: opts([["quiet", "Quiet"], ["normal", "Normal"], ["chatty", "Chatty"]]) },
  // ---- when a new game starts
  { key: "level", scope: "new", kind: "choice", label: "Your opponent's skill", default: 3, options: LEVELS },
  { key: "persona", scope: "new", kind: "choice", label: "Your opponent's way of play", default: 0, options: PERSONAS },
  { key: "erratic", scope: "new", kind: "range", label: "Slips sometimes", default: 0, min: 0, max: 1000, step: 50 },
]);

export const SECTIONS = Object.freeze([
  { id: "table", title: "The table" },
  { id: "help", title: "Help at the table" },
]);

export const specsOf = (scope) => PREF_SPECS.filter((s) => s.scope === scope);

export const DEFAULTS = Object.freeze(Object.fromEntries(PREF_SPECS.map((s) => [s.key, s.default])));

// Whether `value` is one the spec allows.
export function valid(spec, value) {
  if (spec.kind === "range") return Number.isInteger(value) && value >= spec.min && value <= spec.max;
  return spec.options.some((o) => o.value === value);
}

// Kept fields are checked one by one; a bad one takes its default.
function sanitise(kept) {
  const out = { ...DEFAULTS };
  for (const spec of PREF_SPECS) if (valid(spec, kept[spec.key])) out[spec.key] = kept[spec.key];
  return out;
}

// The first shape was { level, hints } with no version: its fields carry over
// and every other field starts from its default.
export function migrate(kept) {
  if (!kept || typeof kept !== "object" || Array.isArray(kept)) return null;
  if (kept.v === undefined) return { level: kept.level, hints: kept.hints };
  if (kept.v > VERSION) return null; // written by a newer page: not understood
  return kept;
}

function defaultStore() {
  try {
    return globalThis.localStorage ?? null;
  } catch {
    return null;
  }
}

export function loadPrefs(store = defaultStore()) {
  try {
    const raw = store?.getItem(KEY);
    if (!raw) return { ...DEFAULTS };
    const kept = migrate(JSON.parse(raw));
    return kept ? sanitise(kept) : { ...DEFAULTS };
  } catch {
    return { ...DEFAULTS };
  }
}

// Whether it was kept. A record written by a newer page is not overwritten.
export function savePrefs(prefs, store = defaultStore()) {
  try {
    if (!store) return false;
    const old = store.getItem(KEY);
    if (old) {
      try {
        if (JSON.parse(old)?.v > VERSION) return false;
      } catch {
        /* unparsable: replaced */
      }
    }
    store.setItem(KEY, JSON.stringify({ ...prefs, v: VERSION }));
    return true;
  } catch {
    return false;
  }
}

// The URL's say over what was kept (for tests and shared links); a value the
// spec does not allow is ignored. `level`, `persona`, `erratic` count too.
export function withUrl(prefs, params) {
  const out = { ...prefs };
  for (const spec of PREF_SPECS) {
    const name = spec.key === "surface" ? "table" : spec.key;
    if (!params.has(name)) continue;
    const text = params.get(name);
    let value = text;
    if (spec.kind === "toggle") value = text !== "0" && text !== "off" && text !== "false";
    else if (spec.kind === "range" || typeof spec.default === "number") value = Number(text);
    if (valid(spec, value)) out[spec.key] = value;
  }
  return out;
}

// The URL's other overrides, which are not preferences.
export function urlOverrides(params) {
  const n = params.has("seed") ? Number(params.get("seed")) : NaN;
  return { seed: Number.isInteger(n) && n >= 0 ? n : null, fresh: params.has("fresh") };
}

// The arguments of engine.new for these prefs (docs/PROTOCOL.md).
export function gameConfig(prefs, { seed } = {}) {
  return {
    level: prefs.level,
    seed: Number.isInteger(seed) ? seed : Math.floor(Math.random() * 2 ** 32),
    persona: prefs.persona,
    erratic: prefs.erratic,
  };
}

// The director's speed factor (kit: bigger is faster). Reduced motion, asked
// for or the system's, plays everything at once.
const SPEED = { normal: 1, fast: 2.2, instant: 1000 };
export function speedFactor(prefs, systemReduced = false) {
  const calm = prefs.reducedMotion === "on" || (prefs.reducedMotion === "auto" && systemReduced);
  return calm ? SPEED.instant : SPEED[prefs.speed] ?? 1;
}
