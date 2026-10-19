// DOM overlay for the bezique table: scores, info chips, ticker, control bar.
// Plain DOM, no libraries. createUi(root, { onAct }) -> { hudSlot, trackerRoot, setNotes, setTracker, setInfo,
// say, setPrompt, hudBottom, barTop, onLayout }.

import { hintView } from "./hinttext.js";

function el(tag, cls, text, parent) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text != null) e.textContent = text;
  if (parent) parent.appendChild(e);
  return e;
}

export function createUi(root, { onAct }) {
  const act = (...a) => onAct && onAct(...a);
  root.textContent = "";

  // ---- top HUD
  const hud = el("div", "ui-hud", null, root);
  // The score itself is the kit's HUD (main.js mounts it here); under it, two quiet notes.
  const hudSlot = el("div", "ui-hudslot", null, hud);
  const noteDouble = el("div", "ui-note", "", hud);
  const noteCarried = el("div", "ui-note ui-carried", "", hud);
  const trackerRoot = el("div", null, null, hud); // "Cards out", under the score
  const chips = el("div", "ui-chips", null, hud);
  const chipTrump = el("span", "ui-chip", null, chips);
  const chipStock = el("span", "ui-chip", null, chips);
  const chipHand = el("span", "ui-chip", null, chips);
  const chipLevel = el("span", "ui-chip", null, chips);
  // With hints on, which of the game's two phases the hand is in, and what that changes.
  const phaseBox = el("div", "ui-phase", null, hud);
  phaseBox.hidden = true;
  const ticker = el("div", "ui-ticker", "", hud);
  ticker.setAttribute("role", "status"); // errors and moves are read out, politely

  // ---- bottom bar
  const bottom = el("div", "ui-bottom", null, root);
  const hintLine = el("div", "ui-hintline", "", bottom);
  hintLine.hidden = true;
  hintLine.setAttribute("aria-live", "polite");
  const bar = el("div", "ui-bar", null, bottom);
  const main = el("div", "ui-main", null, bar);

  // ---- the menu, top right: the tools, behind one button
  const menu = el("div", "ui-menu", null, root);
  const menuBtn = el("button", "ui-btn ui-small ui-menubtn", "\u2630 Menu", menu);
  menuBtn.type = "button";
  menuBtn.setAttribute("aria-haspopup", "true");
  menuBtn.setAttribute("aria-expanded", "false");
  const tools = el("div", "ui-tools", null, menu);
  tools.hidden = true;
  const setMenu = (open) => {
    tools.hidden = !open;
    menuBtn.setAttribute("aria-expanded", String(open));
  };
  menuBtn.onclick = () => setMenu(tools.hidden);
  // Document-level listeners: createUi is called once per page (main.js); a
  // second call removes the first one's, so a reload or a test does not stack them.
  const onPointer = (e) => { if (!menu.contains(e.target)) setMenu(false); };
  const onEscape = (e) => {
    if (e.key !== "Escape" || tools.hidden) return;
    setMenu(false);
    if (menu.contains(document.activeElement) || document.activeElement === document.body) menuBtn.focus();
  };
  window.__beziqueUiOff?.();
  document.addEventListener("pointerdown", onPointer);
  document.addEventListener("keydown", onEscape);
  window.__beziqueUiOff = () => {
    document.removeEventListener("pointerdown", onPointer);
    document.removeEventListener("keydown", onEscape);
  };

  const undoBtn = el("button", "ui-btn ui-small", "Undo", tools);
  const hintBtn = el("button", "ui-btn ui-small", "Hint: off", tools);
  const helpBtn = el("button", "ui-btn ui-small", "Help", tools);
  helpBtn.type = "button";
  helpBtn.onclick = () => { setMenu(false); act("help"); };
  const trackerBtn = el("button", "ui-btn ui-small", "Cards out: off", tools);
  const newBtn = el("button", "ui-btn ui-small", "New game", tools);
  const gearBtn = el("button", "ui-btn ui-small", "\u2699 Settings", tools);
  gearBtn.dataset.act = "settings";
  for (const b of [undoBtn, hintBtn, trackerBtn, newBtn, gearBtn]) b.type = "button";
  trackerBtn.setAttribute("aria-pressed", "false");
  hintBtn.setAttribute("aria-pressed", "false");

  // While the dialog is up the page behind it takes no focus and is hidden from assistive technology.
  const setBackground = (on) => {
    for (const n of [hud, bottom, menu]) n.toggleAttribute("inert", on);
  };

  let isOver = false;
  let hintOn = false;
  let sayTimer = 0;

  undoBtn.onclick = () => act("undo");
  hintBtn.onclick = () => act("hints");
  newBtn.onclick = () => { setMenu(false); act("new"); }; // the New game dialog asks, and confirms an abandoned game
  gearBtn.onclick = () => { setMenu(false); act("settings"); };
  trackerBtn.onclick = () => act("tracker");

  // The notes under the score: the double line, and what is carried "to score".
  function setNotes({ double = "", carried = "" } = {}) {
    noteDouble.textContent = double;
    noteCarried.textContent = carried;
    noteCarried.hidden = !carried;
  }

  function setInfo({ trump, stock, hand, level } = {}) {
    chipTrump.textContent = "Trumps " + (trump || "–");
    chipTrump.classList.toggle("ui-red", trump === "♥" || trump === "♦");
    chipStock.textContent = "Stock " + (stock == null ? "–" : stock);
    chipHand.textContent = "Hand " + (hand == null ? "–" : hand);
    chipHand.hidden = hand == null;
    chipLevel.textContent = "Level " + (level == null ? "–" : level);
    chipLevel.hidden = level == null;
  }

  const PHASES = [
    ["Phase 1 of 2: the stock", "Any card may be played. The trick's winner declares, then draws."],
    ["Phase 2 of 2: the last eight tricks", "No drawing or declaring. Follow suit and win if able. Holding the suit but unable to win, play any card of it; with none of it, trump if able, else any card."],
  ];
  function setPhase(raw, on) {
    const dealt = raw && raw.hand_number != null && raw.stock != null && !["choose_dealer", "over"].includes(raw.prompt?.kind);
    phaseBox.hidden = !(on && dealt);
    if (phaseBox.hidden) return;
    const [title, line] = PHASES[raw.stock === 0 ? 1 : 0];
    phaseBox.textContent = "";
    el("div", "ui-phase-title", title, phaseBox);
    el("div", "ui-phase-line", line, phaseBox);
  }

  function say(text) {
    clearTimeout(sayTimer);
    ticker.textContent = text || "";
    ticker.classList.add("ui-show");
    sayTimer = setTimeout(() => ticker.classList.remove("ui-show"), 4000);
  }

  // The hint line: what to do (the cards named), the rule behind it in a
  // quieter line, and a "Why?" with the period source (src/hinttext.js).
  function renderHint(hint, layers) {
    hintLine.textContent = "";
    const v = hintView(hint, layers);
    hintLine.hidden = !(v && (v.do || v.rule));
    if (hintLine.hidden) return;
    if (v.do) el("div", "ui-hint-do", v.do, hintLine);
    if (v.rule) {
      el("div", "ui-hint-rule", v.rule, hintLine);
      const btn = el("button", "ui-hint-whybtn", "Why?", hintLine);
      const why = el("div", "ui-hint-why", v.why, hintLine);
      why.hidden = true;
      btn.type = "button";
      btn.setAttribute("aria-expanded", "false");
      btn.onclick = () => {
        why.hidden = !why.hidden;
        btn.setAttribute("aria-expanded", String(!why.hidden));
      };
    }
  }

  function setPrompt(raw, { busy, hint, layers } = {}) {
    raw = raw || {};
    const prompt = raw.prompt || {};
    const kind = prompt.kind;
    isOver = kind === "over";
    main.textContent = "";
    // The game-over dialog is built once per outcome and left alone by later
    // setPrompt calls (a rebuild would drop the focus on its button).
    const overKey = kind === "over" ? JSON.stringify([raw.outcome, raw.totals]) : null;
    root.querySelectorAll(".ui-modal").forEach((m) => {
      if (overKey !== null && m.dataset.over === overKey) return;
      m.remove();
      setBackground(false);
    });

    hintOn = !!hint;
    setPhase(raw, hintOn);
    renderHint(hint, layers);

    if (raw.error) say(raw.error);

    const btns = [];
    if (busy) {
      el("div", "ui-text", "…", main);
    } else if (kind === "choose_dealer") {
      el("div", "ui-text", "You cut higher. Who deals first?", main);
      const row = el("div", "ui-choices", null, main);
      for (const [label, who] of [["I deal", "you"], ["Opponent deals", "them"]]) {
        const b = el("button", "ui-btn ui-primary", label, row);
        b.type = "button";
        b.onclick = () => act("dealer", who);
        btns.push(b);
      }
    } else if (kind === "play") {
      el("div", "ui-text", raw.phase === "lead" ? "Your lead: tap a card in your hand" : "Your turn: tap a card in your hand to play", main);
    } else if (kind === "declare") {
      el("div", "ui-text", "Declare?", main);
      const list = el("div", "ui-options", null, main);
      const hintIdx = hint && hint.command && /^declare (\d+)$/.exec(hint.command);
      (prompt.options || []).forEach((o, i) => {
        const label = o.text + (o.points ? " — +" + o.points : "");
        const b = el("button", "ui-btn ui-option" + (i === 0 ? " ui-quiet" : ""), label, list);
        b.type = "button";
        if (hintIdx && Number(hintIdx[1]) === i) b.classList.add("ui-hinted");
        b.onclick = () => act("declare", i);
        btns.push(b);
      });
    } else if (kind === "over") {
      const o = raw.outcome || {};
      const shown = [...root.querySelectorAll(".ui-modal")].find((m) => m.dataset.over === overKey);
      let b = shown?.querySelector("button");
      if (!shown) {
        const modal = el("div", "ui-modal", null, root);
        modal.dataset.over = overKey;
        const card = el("div", "ui-modal-card", null, modal);
        card.setAttribute("role", "dialog");
        card.setAttribute("aria-modal", "true");
        const title = el("h2", "ui-modal-title", o.winner === "you" ? "You win!" : "Your opponent wins", card);
        title.id = "ui-over-title";
        card.setAttribute("aria-labelledby", title.id);
        if (o.double) el("div", "ui-modal-sub", "(double game)", card);
        const t = raw.totals || {};
        el("div", "ui-modal-sub", "You " + (t.you | 0) + " – Opponent " + (t.them | 0), card);
        b = el("button", "ui-btn ui-primary", "Play again", card);
        b.type = "button";
        b.onclick = () => act("new");
        setBackground(true);
        b.focus(); // disabled while busy: the focus lands when it is enabled below
      }
      btns.push(b);
      el("div", "ui-text", "Game over", main);
    }

    undoBtn.disabled = !!busy || !raw.can_undo;
    hintBtn.disabled = !!busy;
    trackerBtn.disabled = !!busy;
    for (const b of btns) {
      const was = b.disabled;
      b.disabled = !!busy;
      if (was && !busy && b.closest?.(".ui-modal")) b.focus();
    }
    hintBtn.textContent = "Hint: " + (hintOn ? "on" : "off");
    hintBtn.setAttribute("aria-pressed", String(hintOn));
  }

  // The hint toggle reflects hint state passed to setPrompt; callers that
  // want it "on" with no text can pass hint: {} (truthy).

  function hudBottom() {
    // The "Cards out" panel stands in the margin like the score does: it does not push the table down.
    const panel = trackerRoot.getBoundingClientRect().height;
    return Math.max(hud.getBoundingClientRect().bottom, ticker.getBoundingClientRect().bottom) - (panel ? panel + 6 : 0);
  }
  function barTop() {
    return bottom.getBoundingClientRect().top;
  }
  function onLayout(fn) {
    if (typeof ResizeObserver !== "undefined") {
      const ro = new ResizeObserver(() => fn());
      ro.observe(hud);
      ro.observe(bottom);
    }
    window.addEventListener("resize", fn);
  }

  // The tracker button shows whether the counting aid is on.
  function setTracker(on) {
    trackerBtn.textContent = "Cards out: " + (on ? "on" : "off");
    trackerBtn.setAttribute("aria-pressed", String(!!on));
  }

  setNotes({});
  setInfo({});
  return { hudSlot, trackerRoot, setNotes, setTracker, setInfo, say, setPrompt, hudBottom, barTop, onLayout };
}
