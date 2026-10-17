// DOM overlay for the bezique table: scores, info chips, ticker, control bar.
// Plain DOM, no libraries. createUi(root, { onAct }) -> { setScores, setInfo,
// say, setPrompt, hudBottom, barTop, onLayout }.

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
  const panel = el("div", "ui-panel", null, hud);
  const rows = {};
  for (const [key, label] of [["you", "You"], ["them", "Opponent"]]) {
    const r = el("div", "ui-score-row", null, panel);
    el("div", "ui-name", label, r);
    const total = el("div", "ui-total", "0", r);
    const hand = el("div", "ui-hand", "this hand 0", r);
    const bar = el("div", "ui-progress", null, r);
    const fill = el("div", "ui-progress-fill ui-" + key, null, bar);
    el("div", "ui-notch", null, bar);
    rows[key] = { total, hand, fill };
  }
  const chips = el("div", "ui-chips", null, hud);
  const chipTrump = el("span", "ui-chip", null, chips);
  const chipStock = el("span", "ui-chip", null, chips);
  const chipHand = el("span", "ui-chip", null, chips);
  const chipLevel = el("span", "ui-chip", null, chips);
  const ticker = el("div", "ui-ticker", "", hud);

  // ---- bottom bar
  const bottom = el("div", "ui-bottom", null, root);
  const hintLine = el("div", "ui-hintline", "", bottom);
  hintLine.hidden = true;
  const bar = el("div", "ui-bar", null, bottom);
  const main = el("div", "ui-main", null, bar);
  const tools = el("div", "ui-tools", null, bar);

  const undoBtn = el("button", "ui-btn ui-small", "Undo", tools);
  const hintBtn = el("button", "ui-btn ui-small", "Hint: off", tools);
  const newBtn = el("button", "ui-btn ui-small", "New game", tools);
  const levelWrap = el("label", "ui-level", "Level ", tools);
  const levelSel = el("select", "ui-select", null, levelWrap);
  for (let i = 1; i <= 5; i++) {
    const o = el("option", null, String(i), levelSel);
    o.value = String(i);
  }
  for (const b of [undoBtn, hintBtn, newBtn]) b.type = "button";
  hintBtn.setAttribute("aria-pressed", "false");

  let isOver = false;
  let hintOn = false;
  let sayTimer = 0;

  undoBtn.onclick = () => act("undo");
  hintBtn.onclick = () => act("hints");
  newBtn.onclick = () => {
    if (isOver || confirm("Start a new game?")) act("new");
  };
  levelSel.onchange = () => act("level", Number(levelSel.value));

  function setScores({ totals, hand } = {}) {
    for (const k of ["you", "them"]) {
      const t = totals ? totals[k] | 0 : 0;
      rows[k].total.textContent = String(t);
      rows[k].hand.textContent = "this hand " + (hand ? hand[k] | 0 : 0);
      rows[k].fill.style.width = Math.max(0, Math.min(100, t / 10)) + "%";
    }
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

  function say(text) {
    clearTimeout(sayTimer);
    ticker.textContent = text || "";
    ticker.classList.add("ui-show");
    sayTimer = setTimeout(() => ticker.classList.remove("ui-show"), 4000);
  }

  function setPrompt(raw, { busy, hint } = {}) {
    raw = raw || {};
    const prompt = raw.prompt || {};
    const kind = prompt.kind;
    isOver = kind === "over";
    main.textContent = "";
    root.querySelectorAll(".ui-modal").forEach((m) => m.remove());

    hintOn = !!hint;
    hintLine.hidden = !(hint && hint.text);
    hintLine.textContent = hint && hint.text ? hint.text : "";

    if (raw.error) say(raw.error);

    if (raw.level != null) levelSel.value = String(raw.level);

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
      const modal = el("div", "ui-modal", null, root);
      const card = el("div", "ui-modal-card", null, modal);
      el("h2", "ui-modal-title", o.winner === "you" ? "You win!" : "Your opponent wins", card);
      if (o.double) el("div", "ui-modal-sub", "(double game)", card);
      const t = raw.totals || {};
      el("div", "ui-modal-sub", "You " + (t.you | 0) + " – Opponent " + (t.them | 0), card);
      const b = el("button", "ui-btn ui-primary", "Play again", card);
      b.type = "button";
      b.onclick = () => act("new");
      btns.push(b);
      el("div", "ui-text", "Game over", main);
    }

    undoBtn.disabled = !!busy || !raw.can_undo;
    hintBtn.disabled = !!busy;
    newBtn.disabled = !!busy;
    levelSel.disabled = !!busy;
    for (const b of btns) b.disabled = !!busy;
    hintBtn.textContent = "Hint: " + (hintOn ? "on" : "off");
    hintBtn.setAttribute("aria-pressed", String(hintOn));
  }

  // The hint toggle reflects hint state passed to setPrompt; callers that
  // want it "on" with no text can pass hint: {} (truthy).

  function hudBottom() {
    return Math.max(hud.getBoundingClientRect().bottom, ticker.getBoundingClientRect().bottom);
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

  setScores({ totals: { you: 0, them: 0 }, hand: { you: 0, them: 0 } });
  setInfo({});
  return { setScores, setInfo, say, setPrompt, hudBottom, barTop, onLayout };
}
