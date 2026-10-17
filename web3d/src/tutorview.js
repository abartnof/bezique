// The tutorial's DOM: a page as a modal card over the table, and the Help
// list. The builders are pure over a document (tested over a tiny one); the
// view holds the one modal and reports what is pressed.

function node(doc, tag, attrs = {}, ...kids) {
  const e = doc.createElement(tag);
  for (const [k, v] of Object.entries(attrs)) if (v != null) e.setAttribute(k, v);
  for (const k of kids.flat()) if (k != null && k !== false) e.append(typeof k === "string" ? doc.createTextNode(k) : k);
  return e;
}

// A link span (`[text](#key)`) is a button reporting its page.
const spans = (doc, list) =>
  list.map((s) =>
    s.page ? node(doc, "button", { class: "tutor-link", type: "button", "data-page": s.page }, s.text) : s.bold ? node(doc, "b", {}, s.text) : s.italic ? node(doc, "i", {}, s.text) : s.text,
  );

// A parsed page (kit/js/tutorial.js parseTutorial): headings, paragraphs, lists.
export function pageBody(doc, page) {
  return node(
    doc,
    "div",
    { class: "tutor-page", "data-page": page.key },
    page.blocks.map((b) => {
      if (b.type === "h") return node(doc, "h3", {}, spans(doc, b.spans));
      if (b.type === "p") return node(doc, "p", {}, spans(doc, b.spans));
      return node(doc, b.type, {}, b.items.map((item) => node(doc, "li", {}, spans(doc, item))));
    }),
  );
}

// The Help list: a button for each page, the ones read marked.
export function helpBody(doc, pages, { seen = [], mark = "✓" } = {}) {
  return node(
    doc,
    "ul",
    { class: "tutor-pages" },
    pages.map((p) => node(doc, "li", {}, node(doc, "button", { class: "tutor-pick", type: "button", "data-page": p.key }, seen.includes(p.key) ? `${p.title} ${mark}` : p.title))),
  );
}

// The modal. `on` reports: close({ key, dontShow }), pick(key), tips(boolean), reset().
export function createTutorView(root, on, doc = document) {
  let modal = null;
  let onKey = null;
  const close = () => {
    if (onKey) doc.removeEventListener("keydown", onKey, true);
    onKey = null;
    modal?.remove();
    modal = null;
  };
  const open = (card, escape) => {
    close();
    modal = node(doc, "div", { class: "tutor-modal" }, card);
    root.append(modal);
    onKey = (e) => {
      if (e.key === "Escape") {
        e.preventDefault();
        escape();
      }
    };
    doc.addEventListener("keydown", onKey, true);
    modal.querySelector("[data-focus]")?.focus();
  };
  const button = (label, extra, click) => {
    const b = node(doc, "button", { class: "tutor-btn", type: "button", ...extra }, label);
    b.onclick = click;
    return b;
  };
  const wireLinks = (body) => body.querySelectorAll?.(".tutor-link").forEach((b) => (b.onclick = () => on.pick(b.dataset.page)));

  return {
    isOpen: () => modal !== null,
    close,
    // A page that fell due (or a page read from Help, with `back`).
    show(page, { back = false } = {}) {
      const body = pageBody(doc, page);
      wireLinks(body);
      const tick = node(doc, "input", { type: "checkbox", id: "tutor-off" });
      const done = () => on.close({ key: page.key, dontShow: tick.checked, back });
      const foot = back
        ? [button("Back to the list", { "data-focus": "1" }, done)]
        : [node(doc, "label", { class: "tutor-off", for: "tutor-off" }, tick, " Don't show tips"), button("Got it", { "data-focus": "1" }, done)];
      open(
        node(doc, "div", { class: "tutor-card", role: "dialog", "aria-modal": "true", "aria-label": page.title }, node(doc, "h2", {}, page.title), body, node(doc, "div", { class: "tutor-foot" }, foot)),
        done,
      );
    },
    // The Help list: any page again, the tips on or off, and a reset.
    help(pages, { seen, tips }) {
      const list = helpBody(doc, pages, { seen });
      list.querySelectorAll?.(".tutor-pick").forEach((b) => (b.onclick = () => on.pick(b.dataset.page)));
      const shut = () => on.close({ key: null, help: true });
      open(
        node(
          doc,
          "div",
          { class: "tutor-card", role: "dialog", "aria-modal": "true", "aria-label": "Help" },
          node(doc, "h2", {}, "How to play"),
          node(doc, "p", {}, "Read any page again."),
          list,
          node(doc, "div", { class: "tutor-foot" }, [
            button(tips ? "Tips: on" : "Tips: off", {}, () => on.tips(!tips)),
            button("Reset the tutorial", {}, () => on.reset()),
            button("Close", { "data-focus": "1" }, shut),
          ]),
        ),
        shut,
      );
    },
  };
}
