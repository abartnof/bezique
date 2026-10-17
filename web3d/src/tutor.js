// The tutorial's moments: which page falls due at a state of the table. It
// holds no rules of play; it reads the engine's state and events
// (docs/PROTOCOL.md) and names the page whose element has just appeared. A
// page is shown the first time its element appears and then never again
// (the page keeps the keys it has shown in `seen`); the words are the pages of
// web3d/tutorial.md, one for each key below, by position.
//
// The pattern is Strohmandeln's and Cassino's (pageDue); the pages here are
// bezique's own.

// The keys in the order they are due when more than one is due at a moment.
export const PAGE_KEYS = [
  "table",
  "first-trick",
  "brisques",
  "declaring",
  "to-score",
  "marriage",
  "bezique",
  "dix",
  "fours",
  "sequence",
  "last-draw",
  "last-eight",
  "double",
  "game-end",
];

// The engine's words for the options of a turn of declaring (words.rs).
const OPTION = {
  "to-score": /to score/i,
  marriage: /marriage/i,
  bezique: /bezique/i,
  dix: /seven of trumps/i,
  fours: /\b(aces|kings|queens|jacks)\b/i,
  sequence: /in trumps/i,
};

// Each check answers false (not due) or { event }: the index in the state's
// events the moment belongs to (the cards hold there), or null for the end
// of the move.
const CHECKS = {
  table: () => ({ event: null }),
  "first-trick": ({ s }) => s.prompt?.kind === "play" && { event: null },
  brisques: ({ fresh }) => {
    const i = fresh.findIndex(({ e }) => e.kind === "took_trick" && e.points > 0);
    return i >= 0 && { event: fresh[i].at };
  },
  declaring: ({ s, taken }) => s.prompt?.kind === "declare" && { event: taken },
  "to-score": ({ s, taken }) => offered(s, OPTION["to-score"]) && { event: taken },
  marriage: ({ s, taken }) => offered(s, OPTION.marriage) && { event: taken },
  bezique: ({ s, taken }) => offered(s, OPTION.bezique) && { event: taken },
  dix: ({ s, taken }) => offered(s, OPTION.dix) && { event: taken },
  fours: ({ s, taken }) => offered(s, OPTION.fours) && { event: taken },
  sequence: ({ s, taken }) => offered(s, OPTION.sequence) && { event: taken },
  // One face-down card and the turn-up remain: the next trick is the last of the stock.
  "last-draw": ({ s }) => s.stock === 1 && Boolean(s.turn_up) && s.prompt?.kind === "play" && { event: null },
  "last-eight": ({ fresh }) => {
    const i = fresh.findIndex(({ e }) => e.kind === "stock_out");
    return i >= 0 && { event: fresh[i].at };
  },
  double: ({ s }) => {
    const t = Object.values(s.totals ?? {});
    return t.length === 2 && Math.max(...t) >= 500 && Math.min(...t) < 500 && { event: null };
  },
  "game-end": ({ s }) => s.prompt?.kind === "over" && { event: null },
};

const offered = (s, re) => s.prompt?.kind === "declare" && s.prompt.options.some((o) => re.test(o.text));

// The page due now, if any, as { key, event }. `prev` is the state before
// (null for a first look, a restored sitting or an undo: no event is fresh
// then), `state` the engine's state, `seen` the keys already shown. Call it
// again after each page is closed until it answers null, so that two pages
// of one moment follow each other in order.
export function dueIn(prev, state, seen = []) {
  if (!state) return null;
  const from = prev && state.events.length >= prev.events.length ? prev.events.length : state.events.length;
  const fresh = state.events.slice(from).map((e, n) => ({ e, at: from + n }));
  const took = fresh.find(({ e }) => e.kind === "took_trick");
  const ctx = { s: state, fresh, taken: took ? took.at : null };
  for (const key of PAGE_KEYS) {
    if (seen.includes(key)) continue;
    const hit = CHECKS[key](ctx);
    if (hit) return { key, event: hit.event };
  }
  return null;
}

export const due = (prev, state, seen = []) => dueIn(prev, state, seen)?.key ?? null;
