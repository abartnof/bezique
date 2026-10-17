// The gear panel: the settings that take effect at once. Described as data
// (settingsModel: pure, tested), then drawn. It holds NO new-game setting; the
// opponent's skill, personality and slips are the New game dialog's (newgame.js).
import { PREF_SPECS, SECTIONS } from "./prefs.js";

// [{ id, title, rows: [{ key, kind, label, value, options: [{value,label,selected}] }] }]
export function settingsModel(prefs, specs = PREF_SPECS) {
  return SECTIONS.map((s) => ({
    ...s,
    rows: specs
      .filter((spec) => spec.scope === "game" && spec.section === s.id)
      .map((spec) => ({
        key: spec.key,
        kind: spec.kind,
        label: spec.label,
        value: prefs[spec.key],
        options: spec.options.map((o) => ({ value: o.value, label: o.label, selected: o.value === prefs[spec.key] })),
      })),
  })).filter((s) => s.rows.length);
}

const el = (tag, cls, text, parent) => {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text != null) e.textContent = text;
  if (parent) parent.appendChild(e);
  return e;
};

// root: where the panel is drawn (hidden until opened). onChange(key, value).
export function createSettings(root, { prefs, onChange = () => {} }) {
  let current = { ...prefs };
  const panel = el("div", "st-panel", null, root);
  panel.hidden = true;
  panel.setAttribute("role", "dialog");
  panel.setAttribute("aria-label", "Settings");

  function choose(key, value) {
    if (current[key] === value) return;
    current = { ...current, [key]: value };
    onChange(key, value);
    render();
  }

  function render() {
    panel.textContent = "";
    const head = el("div", "st-head", null, panel);
    el("h2", "st-title", "Settings", head);
    const close = el("button", "ui-btn ui-small", "×", head);
    close.type = "button";
    close.setAttribute("aria-label", "Close settings");
    close.onclick = () => hide();
    for (const s of settingsModel(current)) {
      const sec = el("section", "st-section", null, panel);
      el("h3", null, s.title, sec);
      for (const row of s.rows) {
        const r = el("div", "st-row", null, sec);
        r.dataset.row = row.key;
        el("span", "st-label", row.label, r);
        if (row.kind === "select") {
          const sel = el("select", "ui-select", null, r);
          sel.dataset.key = row.key;
          for (const o of row.options) {
            const opt = el("option", null, o.label, sel);
            opt.value = String(o.value);
            opt.selected = o.selected;
          }
          sel.onchange = () => choose(row.key, row.options.find((o) => String(o.value) === sel.value).value);
        } else {
          const group = el("div", "st-options", null, r);
          group.setAttribute("role", "radiogroup");
          for (const o of row.options) {
            const b = el("button", "ui-btn ui-small", o.label, group);
            b.type = "button";
            b.dataset.key = row.key;
            b.setAttribute("role", "radio");
            b.setAttribute("aria-checked", String(o.selected));
            b.setAttribute("aria-pressed", String(o.selected));
            b.onclick = () => choose(row.key, o.value);
          }
        }
      }
    }
  }
  let opener = null;
  function show() {
    opener = document.activeElement;
    render();
    panel.hidden = false;
    panel.querySelector("button[aria-checked=true],select")?.focus();
  }
  function hide() {
    panel.hidden = true;
    opener?.focus?.();
  }
  root.addEventListener("keydown", (e) => {
    if (e.key === "Escape" && !panel.hidden) hide();
  });
  render();
  return {
    open: show,
    close: hide,
    toggle: () => (panel.hidden ? show() : hide()),
    isOpen: () => !panel.hidden,
    set(key, value) {
      current = { ...current, [key]: value };
      if (!panel.hidden) render();
    },
  };
}
