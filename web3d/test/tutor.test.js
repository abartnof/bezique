// The tutorial's moments, over real games: every page fires at most once,
// the declaring pages fire at the first option of their kind, and the order
// is a sensible one.
import assert from "node:assert/strict";
import fs from "node:fs";
import test from "node:test";
import { parseTutorial } from "../../kit/js/tutorial.js";
import { PAGE_KEYS, dueIn, due, dueAll } from "../src/tutor.js";
import { createTutorStore } from "../src/tutorstore.js";
import { pageBody, helpBody, kicker, createTutorView } from "../src/tutorview.js";
import { sitting } from "./support/table.js";

// The options a page is about, by the engine's own words (words.rs).
const OPTION = {
  marriage: /marriage/i,
  bezique: /bezique/i,
  dix: /seven of trumps/i,
  fours: /\b(aces|kings|queens|jacks)\b/i,
  sequence: /in trumps/i,
  "to-score": /to score/i,
};

// A player that likes to declare: the dearest option most of the time.
function chooser(rng, greed) {
  return (s) => {
    const p = s.prompt;
    if (p.kind === "choose_dealer") return rng() < 0.5 ? "dealer you" : "dealer them";
    if (p.kind === "play") return `play ${p.legal[Math.floor(rng() * p.legal.length)]}`;
    if (p.kind === "declare") {
      const carry = p.options.findIndex((o) => /to score/i.test(o.text));
      if (carry > 0 && greed >= 0.9) return `declare ${carry}`;
      if (rng() < greed) {
        let best = 0;
        p.options.forEach((o, i) => { if (o.points > p.options[best].points) best = i; });
        return `declare ${best}`;
      }
      return `declare ${Math.floor(rng() * p.options.length)}`;
    }
    return null;
  };
}

// Plays a whole sitting, asking the tutorial at every state as the page does.
async function played(seed, { level = 1, greed = 0.5 } = {}) {
  const { state, send } = await sitting({ seed, level });
  let r = seed * 7919;
  const rng = () => ((r = (r * 1103515245 + 12345) >>> 0) / 2 ** 32);
  const choose = chooser(rng, greed);
  const seen = [];
  const fired = []; // { key, at, event }
  const firstOption = {};
  let prev = null;
  for (let i = 0; i < 700; i++) {
    const s = state();
    if (s.prompt.kind === "declare") for (const [k, re] of Object.entries(OPTION)) if (firstOption[k] === undefined && s.prompt.options.some((o) => re.test(o.text))) firstOption[k] = i;
    for (let step = dueIn(prev, s, seen); step; step = dueIn(prev, s, seen)) {
      fired.push({ ...step, at: i });
      seen.push(step.key);
    }
    prev = s;
    const c = choose(s);
    if (!c) break;
    assert.ok(send(c), c);
  }
  return { fired, firstOption, last: state() };
}

const SEEDS = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
const games = [];
for (const seed of SEEDS) games.push(await played(seed, { greed: seed % 2 ? 0.9 : 0.3 }));
games.push(await played(21, { level: 4, greed: 1 }));

test("every page fires at most once a sitting, and only pages of the tutorial", () => {
  for (const g of games) {
    const keys = g.fired.map((f) => f.key);
    assert.equal(new Set(keys).size, keys.length);
    for (const k of keys) assert.ok(PAGE_KEYS.includes(k), k);
  }
});

test("the first page is the table; the first trick follows before any other", () => {
  for (const g of games) {
    assert.equal(g.fired[0].key, "table");
    assert.equal(g.fired[1].key, "first-trick");
  }
});

test("pages come in the order of PAGE_KEYS within a moment, and the basics come before the declaring", () => {
  for (const g of games) {
    const at = (k) => g.fired.findIndex((f) => f.key === k);
    if (at("marriage") >= 0) assert.ok(at("declaring") >= 0 && at("declaring") < at("marriage"));
    if (at("last-eight") >= 0 && at("last-draw") >= 0) assert.ok(at("last-draw") < at("last-eight"));
    const moments = g.fired.map((f) => f.at);
    assert.deepEqual(moments, [...moments].sort((a, b) => a - b), "time only goes forward");
    g.fired.forEach((f, n) => {
      const same = g.fired.slice(0, n).filter((x) => x.at === f.at);
      for (const x of same) assert.ok(PAGE_KEYS.indexOf(x.key) < PAGE_KEYS.indexOf(f.key));
    });
  }
});

test("each declaring page fires at the first option of its kind, never before", () => {
  const seen = new Set();
  for (const g of games) {
    for (const [k, i] of Object.entries(g.firstOption)) {
      const f = g.fired.find((x) => x.key === k);
      assert.ok(f, `${k} fired in a game that offered it`);
      assert.equal(f.at, i, `${k} at the first option`);
      seen.add(k);
    }
    for (const f of g.fired) if (OPTION[f.key]) assert.equal(f.at, g.firstOption[f.key], `${f.key} not before its option`);
  }
  for (const k of ["marriage", "bezique", "dix", "fours"]) assert.ok(seen.has(k), `the games reached ${k}`);
});

test("the declaring page opens on the first declare prompt, with the trick as its moment", () => {
  for (const g of games) {
    const d = g.fired.find((f) => f.key === "declaring");
    assert.ok(d);
    assert.ok(Number.isInteger(d.event) || d.event === null);
  }
});

test("the stock pages and the endings are reached in a whole game", () => {
  const all = new Set(games.flatMap((g) => g.fired.map((f) => f.key)));
  for (const k of ["brisques", "last-draw", "last-eight", "game-end"]) assert.ok(all.has(k), k);
  for (const g of games) if (g.last.prompt.kind === "over") assert.ok(g.fired.some((f) => f.key === "game-end"));
});

test("a seen page never fires again, and an undo (events shorter) fires no event page", async () => {
  const { state, send } = await sitting({ seed: 3, level: 1 });
  const s0 = state();
  assert.equal(due(null, s0, []), "table");
  assert.notEqual(due(null, s0, ["table"]), "table");
  const every = new Set(PAGE_KEYS.filter((k) => k !== "brisques"));
  assert.equal(due(null, s0, [...PAGE_KEYS]), null);
  assert.equal(due(s0, s0, [...every]), null);
  send("dealer you");
});

test("the double game's page waits for 500 with the other side short", () => {
  const base = { events: [], prompt: { kind: "play", legal: [] }, stock: 20, turn_up: { card: "9H" } };
  const all = PAGE_KEYS.filter((k) => k !== "double");
  assert.equal(due(null, { ...base, totals: { you: 480, them: 100 } }, all), null);
  assert.equal(due(null, { ...base, totals: { you: 510, them: 100 } }, all), "double");
  assert.equal(due(null, { ...base, totals: { you: 510, them: 520 } }, all), null);
});

test("the store keeps what was seen, forgets on reset, and survives broken storage", () => {
  const mem = new Map();
  const storage = { getItem: (k) => mem.get(k) ?? null, setItem: (k, v) => mem.set(k, v), removeItem: (k) => mem.delete(k) };
  const a = createTutorStore(storage);
  assert.deepEqual(a.seen(), []);
  a.add("table");
  a.add("table");
  a.add("brisques");
  assert.deepEqual(createTutorStore(storage).seen(), ["table", "brisques"]);
  assert.ok(mem.has("bezique.tutor.v1"));
  a.reset();
  assert.deepEqual(createTutorStore(storage).seen(), []);
  mem.set("bezique.tutor.v1", "{not json");
  assert.deepEqual(createTutorStore(storage).seen(), []);
  const broken = createTutorStore({ getItem() { throw new Error("no"); }, setItem() { throw new Error("no"); }, removeItem() { throw new Error("no"); } });
  broken.add("table");
  assert.deepEqual(broken.seen(), ["table"]);
});

test("the tutorial file has one page for every key, each short", () => {
  const md = fs.readFileSync(new URL("../tutorial.md", import.meta.url), "utf8");
  const pages = parseTutorial(md, PAGE_KEYS);
  assert.equal(pages.length, PAGE_KEYS.length);
  for (const p of pages) {
    assert.ok(PAGE_KEYS.includes(p.key));
    const words = p.blocks.flatMap((b) => (b.items ? b.items.flat() : b.spans)).map((s) => s.text).join(" ").split(/\s+/).length;
    assert.ok(words <= 140, `${p.key} is ${words} words`);
  }
});

// A tiny document: just enough for the builders.
const doc = {
  createElement: (tag) => ({ tag, children: [], attrs: {}, className: "", append(...c) { this.children.push(...c); }, setAttribute(k, v) { this.attrs[k] = v; }, set textContent(t) { this.children = [String(t)]; } }),
  createTextNode: (t) => String(t),
};
const text = (n) => (typeof n === "string" ? n : n.children.map(text).join(""));

test("the page and help builders draw what they are given", () => {
  const md = "# One\n\nHello **bold** and [two](#b).\n\n- a\n- b\n\n# Two\n\nx\n";
  const pages = parseTutorial(md, ["a", "b"]);
  const body = pageBody(doc, pages[0]);
  assert.match(text(body), /Hello bold and two/);
  const help = helpBody(doc, pages, { seen: ["a"] });
  assert.match(text(help), /One ✓/);
  assert.doesNotMatch(text(help), /Two ✓/);
});

test("a carried score's page fires at the first option that says so (the engine's own wording)", () => {
  const options = (texts) => ({ events: [{ kind: "took_trick" }], stock: 20, turn_up: { card: "9H" }, totals: { you: 0, them: 0 }, prompt: { kind: "declare", options: texts.map((text) => ({ text })) } });
  const rest = PAGE_KEYS.filter((k) => !["to-score"].includes(k));
  assert.equal(due(null, options(["Nothing: draw.", "Forty for bezique, and twenty to score."]), rest), "to-score");
  assert.equal(due(null, options(["Nothing: draw.", "Score forty, for the royal marriage, and twenty, and twenty to score."]), rest), "to-score");
  assert.equal(due(null, options(["Nothing: draw.", "Twenty for the marriage in clubs."]), rest), null);
});

test("the first tips wait for the deal, and a moment's tips are few and in order", () => {
  const choosing = { events: [], prompt: { kind: "choose_dealer" }, stock: 64, totals: { you: 0, them: 0 } };
  assert.equal(due(null, choosing, []), null, "nothing over the cut");
  const dealt = { events: [], prompt: { kind: "play", legal: [] }, stock: 47, turn_up: { card: "9H" }, totals: { you: 0, them: 0 } };
  assert.deepEqual(dueAll(null, dealt, []).map((p) => p.key), ["table", "first-trick"]);
  assert.deepEqual(dueAll(null, dealt, ["table"]).map((p) => p.key), ["first-trick"]);
  assert.deepEqual(dueAll(null, dealt, ["table", "first-trick"]), []);
  assert.equal(dueAll(null, dealt, [], 1).length, 1);
  const many = { events: [{ kind: "took_trick", points: 10 }], stock: 20, turn_up: { card: "9H" }, totals: { you: 0, them: 0 }, prompt: { kind: "declare", options: ["Nothing: draw.", "Forty for bezique, and twenty to score.", "Twenty for the marriage", "Four aces", "Seven of trumps"].map((text) => ({ text })) } };
  const keys = dueAll(null, many, ["table", "first-trick"]).map((p) => p.key);
  assert.ok(keys.length <= 3 && keys.length >= 2, keys.join());
  assert.deepEqual(keys, [...keys].sort((a, b) => PAGE_KEYS.indexOf(a) - PAGE_KEYS.indexOf(b)));
});

test("the tip says how many there are in a row", () => {
  assert.equal(kicker(1, 1), "Tip");
  assert.equal(kicker(2, 3), "Tip 2 of 3");
});

// A fake document with a keyboard and a focus, for the modal.
function fake() {
  const listeners = [];
  const mk = (tag) => ({
    tag, children: [], attrs: {}, tagName: tag.toUpperCase(), isConnected: true, parent: null,
    append(...c) { for (const k of c) { this.children.push(k); if (typeof k === "object") k.parent = this; } },
    setAttribute(k, v) { this.attrs[k] = v; },
    remove() { this.isConnected = false; },
    focus() { d.activeElement = this; },
    querySelector(sel) { return this.querySelectorAll(sel)[0]; },
    querySelectorAll(sel) {
      const out = [];
      const walk = (n) => { for (const k of n.children) if (typeof k === "object") { if (sel === "[data-focus]" ? "data-focus" in k.attrs : sel === "button, input, a[href]" ? ["button", "input"].includes(k.tag) : false) out.push(k); walk(k); } };
      walk(this);
      return out;
    },
  });
  const d = {
    body: mk("body"), activeElement: null,
    createElement: mk, createTextNode: (t) => String(t),
    addEventListener: (t, f) => listeners.push([t, f]),
    removeEventListener: (t, f) => { const i = listeners.findIndex(([, g]) => g === f); if (i >= 0) listeners.splice(i, 1); },
    key: (key, target = d.activeElement) => { let stopped = false; for (const [t, f] of [...listeners]) if (t === "keydown") f({ key, target, preventDefault() { stopped = true; } }); return stopped; },
    listeners: () => listeners.length,
  };
  d.activeElement = d.body;
  return d;
}

test("a tip's dialog: default button focused; Enter, Space, Esc and the backdrop all continue; focus returns", () => {
  const md = "# One\n\nHello.\n\n# Two\n\nx\n";
  const pages = parseTutorial(md, ["a", "b"]);
  const closed = [];
  for (const how of ["Enter", " ", "Escape", "backdrop"]) {
    const d = fake();
    const opener = d.createElement("button");
    d.activeElement = opener;
    const root = d.createElement("div");
    const view = createTutorView(root, { close: (r) => closed.push(r), pick() {}, tips() {}, reset() {} }, d);
    view.show(pages[0], { at: 1, of: 2 });
    assert.ok(view.isOpen());
    assert.equal(d.activeElement.attrs["data-focus"], "1", "the default button has the focus");
    assert.match(text(root.children[0]), /Tip 1 of 2/);
    assert.match(text(root.children[0]), /Next/);
    assert.match(text(root.children[0]), /paused/);
    const before = closed.length;
    if (how === "backdrop") { const m = root.children[0]; m.onclick({ target: m }); }
    else d.key(how, d.body); // not on a button: the key still continues
    assert.equal(closed.length, before + 1, how);
    view.close();
    assert.equal(d.activeElement, opener, "the focus returns to where it was");
    assert.equal(d.listeners(), 0, "no key handler is left");
    assert.equal(view.isOpen(), false);
  }
});

test("the last of a row says Got it, and Tab stays in the dialog", () => {
  const pages = parseTutorial("# One\n\nHello.\n", ["a"]);
  const d = fake();
  const root = d.createElement("div");
  const view = createTutorView(root, { close() {}, pick() {}, tips() {}, reset() {} }, d);
  view.show(pages[0], { at: 2, of: 2 });
  assert.match(text(root.children[0]), /Got it/);
  const stops = root.children[0].querySelectorAll("button, input, a[href]");
  assert.equal(stops.length, 2);
  d.activeElement = stops[1];
  assert.ok(d.key("Tab", stops[1]), "Tab is taken");
  assert.equal(d.activeElement, stops[0], "and wraps to the first stop");
});
