// The New game dialog: the choices that apply only when a game starts (the
// opponent's skill, its way of play, how often it slips, and optionally the
// deal's seed). Values are prefilled from the last game. If a game is in
// progress, starting abandons it, and the dialog says so before it deals.
import { LEVEL_LINES, PERSONAS, specsOf } from "./prefs.js";

export const NEW_SPECS = specsOf("new");

// How the slider's number reads.
export function erraticWord(permille) {
  if (permille <= 0) return "never";
  if (permille <= 150) return "rarely";
  if (permille <= 400) return "now and then";
  if (permille <= 700) return "often";
  return "very often";
}

// The line under the choices: the skill, then the way of play.
export function opponentLine(level, persona) {
  const p = PERSONAS.find((x) => x.value === persona) ?? PERSONAS[0];
  return `${LEVEL_LINES[level] ?? ""} ${p.line}`.trim();
}

// A seed field's text: blank is a fresh deal (null); else a whole number.
export function parseSeed(text) {
  const t = String(text ?? "").trim();
  if (t === "") return { seed: null };
  if (!/^\d{1,10}$/.test(t) || Number(t) > 0xffffffff) return { seed: null, error: "A seed is a whole number up to 4294967295." };
  return { seed: Number(t) };
}

// The chosen values as the page keeps them: each held to its spec.
export function normalise(values) {
  const out = {};
  for (const s of NEW_SPECS) {
    const v = s.kind === "toggle" ? null : Number(values[s.key]);
    if (s.kind === "toggle") out[s.key] = typeof values[s.key] === "boolean" ? values[s.key] : values[s.key] === "true" ? true : s.default;
    else if (s.kind === "range") out[s.key] = Math.min(s.max, Math.max(s.min, Math.round(v / s.step) * s.step || 0));
    else out[s.key] = s.options.some((o) => o.value === v) ? v : s.default;
  }
  return out;
}

const el = (tag, cls, text, parent) => {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text != null) e.textContent = text;
  if (parent) parent.appendChild(e);
  return e;
};

// onStart({ level, persona, erratic, hints, tracker, playForced }, seed|null). inProgress(): a game is on.
export function createNewGame(root, { prefs, inProgress = () => false, onStart }) {
  let last = normalise(prefs);
  const modal = el("div", "ng-modal", null, root);
  modal.hidden = true;
  const card = el("div", "ui-modal-card ng-card", null, modal);
  card.setAttribute("role", "dialog");
  card.setAttribute("aria-label", "New game");
  let chosen = {};
  let opener = null;

  function render() {
    card.textContent = "";
    el("h2", "ui-modal-title", "New game", card);
    if (inProgress()) el("p", "ng-warn", "A game is in progress. Starting a new one abandons it.", card);
    el("p", "ng-line", opponentLine(chosen.level, chosen.persona), card);
    for (const s of NEW_SPECS) {
      const row = el("div", "st-row", null, card);
      row.dataset.row = s.key;
      el("span", "st-label", s.label, row);
      if (s.kind === "toggle") out[s.key] = typeof values[s.key] === "boolean" ? values[s.key] : values[s.key] === "true" ? true : s.default;
    else if (s.kind === "range") {
        const wrap = el("div", "ng-range", null, row);
        const input = el("input", null, null, wrap);
        Object.assign(input, { type: "range", min: s.min, max: s.max, step: s.step, value: chosen[s.key] });
        input.setAttribute("aria-label", s.label);
        const word = el("span", "ng-word", erraticWord(chosen[s.key]), wrap);
        input.oninput = () => {
          chosen[s.key] = Number(input.value);
          word.textContent = erraticWord(chosen[s.key]);
        };
      } else {
        const group = el("div", "st-options", null, row);
        group.setAttribute("role", "radiogroup");
        for (const o of s.options) {
          const on = o.value === chosen[s.key];
          const b = el("button", "ui-btn ui-small", o.label, group);
          b.type = "button";
          b.dataset.key = s.key;
          b.setAttribute("role", "radio");
          b.setAttribute("aria-checked", String(on));
          b.setAttribute("aria-pressed", String(on));
          b.onclick = () => {
            chosen[s.key] = o.value;
            render();
          };
        }
      }
    }
    const seedRow = el("div", "st-row", null, card);
    el("span", "st-label", "Replay a deal (seed, optional)", seedRow);
    const seedInput = el("input", "ui-select ng-seed", null, seedRow);
    Object.assign(seedInput, { type: "text", inputMode: "numeric", placeholder: "random", value: chosen.seedText ?? "" });
    seedInput.oninput = () => (chosen.seedText = seedInput.value);
    const err = el("p", "ng-warn", "", card);
    const buttons = el("div", "ui-choices", null, card);
    const cancel = el("button", "ui-btn", "Cancel", buttons);
    cancel.type = "button";
    cancel.onclick = () => close();
    const start = el("button", "ui-btn ui-primary", inProgress() ? "Abandon and deal" : "Deal", buttons);
    start.type = "button";
    start.dataset.role = "start";
    start.onclick = () => {
      const { seed, error } = parseSeed(chosen.seedText);
      if (error) {
        err.textContent = error;
        return;
      }
      const values = normalise(chosen);
      last = values;
      close();
      onStart(values, seed);
    };
  }

  function open() {
    opener = document.activeElement;
    chosen = { ...last, seedText: "" };
    render();
    modal.hidden = false;
    card.querySelector("[data-role=start]")?.focus();
  }
  function close() {
    modal.hidden = true;
    opener?.focus?.();
  }
  root.addEventListener("keydown", (e) => {
    if (e.key === "Escape" && !modal.hidden) close();
  });
  return {
    open,
    close,
    isOpen: () => !modal.hidden,
    remember(values) {
      last = normalise({ ...last, ...values });
    },
  };
}
