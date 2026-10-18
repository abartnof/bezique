// The "cards out" panel's DOM, from tracker.js's model: a card on the right
// margin, a head that folds it, the counts as lines, and a grid of the
// thirty-two kinds with the copies of each still unseen.
function el(tag, cls, text, parent) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text != null) e.textContent = text;
  if (parent) parent.appendChild(e);
  return e;
}

const CHEV = '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M6 9l6 6 6-6"></path></svg>';

// `root`: the element to fill. `onFold(open)` is told when the head is pressed.
export function createTrackerView(root, { onFold = () => {} } = {}) {
  root.textContent = "";
  root.classList.add("tracker-root");
  const card = el("section", "tracker-card", null, root);
  card.setAttribute("aria-label", "Cards out");
  const head = el("button", "tracker-head", null, card);
  head.type = "button";
  el("span", "tracker-title", "Cards out", head);
  const chev = el("span", "tracker-chev", null, head);
  chev.innerHTML = CHEV;
  const body = el("div", "tracker-body", null, card);
  let open = false; // closed until the person opens it
  let model = null;

  function draw() {
    card.classList.toggle("folded", !open);
    head.setAttribute("aria-expanded", String(open));
    body.textContent = "";
    if (!open) return;
    if (!model) {
      el("p", "tracker-note", "The count begins once the hand is dealt.", body);
      return;
    }
    const ul = el("ul", "tracker-lines", null, body);
    for (const l of model.lines) el("li", null, l.text, ul).dataset.line = l.id;
    const grid = el("div", "tracker-grid", null, body);
    grid.setAttribute("aria-hidden", "true");
    const first = el("div", "tracker-row tracker-heads", null, grid);
    el("span", "tracker-suit", "", first);
    for (const r of model.rows[0].cells) el("span", "tracker-rank", r.rank === "T" ? "10" : r.rank, first);
    for (const row of model.rows) {
      const r = el("div", `tracker-row${row.trump ? " tracker-trump" : ""}`, null, grid);
      el("span", `tracker-suit tracker-suit-${row.suit}`, row.glyph, r);
      for (const c of row.cells) {
        const cell = el("span", `tracker-cell tracker-out-${c.out}${c.kind === "brisque" ? " tracker-brisque" : ""}${c.bezique ? " tracker-bez" : ""}`, c.out ? String(c.out) : "·", r);
        cell.title = `${c.rank === "T" ? "10" : c.rank}${row.glyph}: ${c.out} unseen`;
      }
    }
    el("p", "tracker-key", "Number of each card still unseen. Gold: brisques (aces, tens). Ringed: bezique cards.", body);
  }

  head.addEventListener("click", () => {
    open = !open;
    draw();
    onFold(open);
  });

  return {
    // `m`: trackerModel's, or null before a hand is dealt.
    set(m) {
      model = m;
      draw();
    },
    setOpen(v) {
      open = v;
      draw();
    },
    node: card,
  };
}
