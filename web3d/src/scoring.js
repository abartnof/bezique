// What the narration scores, as running figures and as badges.
//
// The engine's events (docs/PROTOCOL.md) carry most points but not all: a
// `took_trick` has its brisques (and the last trick's ten, Foster p. 343), a
// `declared` the one combination scored, but the dealer's ten for a seven
// turned up (p. 339) and the dix (the seven of trumps exchanged or shown,
// p. 340) are events with no points field: ten each, by rule. This module
// reads a list of events into scoring items, `{ who, cat, pts, label, at }`,
// and folds items into the running score the page shows. It holds no rule of
// play, only this accounting; web3d/test/scoring.test.js checks it against
// the engine's own `totals` and `hand_scores` over whole games.

import { defineGame } from "../../kit/js/scoring.js";

export const TARGET = 1000;
export const DOUBLE_LINE = 500;
export const SEVEN_POINTS = 10; // the dealer's seven and the dix
export const LAST_TRICK_POINTS = 10;

// The ledger's categories, in order (kit/js/scoring.js defineGame).
export const CATEGORIES = [
  ["brisques", "Brisques"],
  ["declared", "Declared"],
  ["seven", "Sevens"],
  ["last", "Last trick"],
];

const OTHER = { you: "them", them: "you" };
export const other = (who) => OTHER[who];

// The name of a combination from the words it was announced in
// ("Forty for bezique, and twenty to score.", "Royal marriage, forty.").
export function combinationName(text) {
  const said = (text.match(/"([^"]*)"/)?.[1] ?? text).split(/, and /)[0].toLowerCase();
  if (/double bezique/.test(said)) return "double bezique";
  if (/bezique/.test(said)) return "bezique";
  if (/in trumps|sequence/.test(said)) return "sequence";
  if (/royal marriage/.test(said)) return "royal marriage";
  if (/marriage/.test(said)) return "marriage";
  const four = said.match(/\b(aces|kings|queens|jacks)\b/);
  if (four) return `four ${four[1]}`;
  return "declared";
}

// Whether a declaration scored a combination announced earlier ("Score forty for ...").
export const scoredFromCarry = (text) => /"score /i.test(text);

// The last trick of a hand is the took_trick the hand's end follows.
function lastTrickAt(events, i) {
  for (let j = i + 1; j < events.length; j++) {
    const k = events[j].kind;
    if (k === "hand_ends") return true;
    if (k === "played" || k === "took_trick" || k === "dealt" || k === "game_ends") return false;
  }
  return false;
}

// What event `i` scores: [{ who, cat, pts, label, at, hand, carried? }], in the order it is told.
export function itemsOf(events, i) {
  const e = events[i];
  const out = [];
  const add = (item) => item.pts > 0 && out.push({ at: i, hand: e.hand, ...item });
  switch (e.kind) {
    case "dealt": {
      // A seven turned up scores the dealer ten at once.
      if (e.turn_up?.card?.startsWith("7")) add({ who: e.dealer, cat: "seven", pts: SEVEN_POINTS, label: "seven turned up" });
      break;
    }
    case "took_trick": {
      const last = lastTrickAt(events, i);
      const brisques = e.points - (last ? LAST_TRICK_POINTS : 0);
      add({ who: e.who, cat: "brisques", pts: brisques, label: brisques > 10 ? "brisques" : "brisque", brisques: brisques / 10 });
      if (last) add({ who: e.who, cat: "last", pts: LAST_TRICK_POINTS, label: "last trick" });
      break;
    }
    case "declared":
      add({ who: e.who, cat: "declared", pts: e.points, label: combinationName(e.text), carried: scoredFromCarry(e.text) });
      break;
    case "dix":
      add({ who: e.who, cat: "seven", pts: SEVEN_POINTS, label: e.taken ? "dix" : "seven shown" });
      break;
    default:
  }
  return out;
}

// Every event's items, by index.
export const itemsFor = (events) => events.map((_, i) => itemsOf(events, i));

// ---- the running score -------------------------------------------------------

const zero = () => ({ you: 0, them: 0 });
const blankHand = () => Object.fromEntries(CATEGORIES.map(([k]) => [k, zero()]));

export function initial() {
  return { totals: zero(), hand: zero(), hands: {}, handNo: 0, done: false, outcome: null };
}

// One event folded in: its items, then what the event itself does (a hand
// ending, the game ending). Returns a new state.
export function apply(state, event, items = []) {
  const s = { ...state, totals: { ...state.totals }, hand: { ...state.hand }, hands: { ...state.hands } };
  if (event.kind === "dealt") {
    s.handNo = event.hand;
    s.hand = zero();
    s.hands[event.hand] = blankHand();
  }
  for (const it of items) {
    s.totals[it.who] += it.pts;
    s.hand[it.who] += it.pts;
    const n = it.hand ?? s.handNo;
    const h = (s.hands[n] = s.hands[n] ? { ...s.hands[n] } : blankHand());
    h[it.cat] = { ...h[it.cat], [it.who]: h[it.cat][it.who] + it.pts };
  }
  if (event.kind === "hand_ends") {
    s.closed = { ...s.hand };
    s.hand = zero();
  }
  if (event.kind === "game_ends") {
    // The engine keeps the last hand's figures at the game's end.
    if (s.hand.you === 0 && s.hand.them === 0 && s.closed) s.hand = { ...s.closed };
    s.done = true;
    s.outcome = { winner: event.winner, double: Boolean(event.double) };
  }
  return s;
}

// The running score after the first `upto` events.
export function fold(events, upto = events.length) {
  let s = initial();
  for (let i = 0; i < upto; i++) s = apply(s, events[i], itemsOf(events, i));
  return s;
}

// The hands in the ledger's shape for the kit HUD (kit/js/hud.js reset): one
// block a hand, `opp` for the person's opponent.
export function ledgerOf(state) {
  const toHud = (h) => Object.fromEntries(CATEGORIES.map(([k]) => [k, { you: h[k].you, opp: h[k].them }]));
  const numbers = Object.keys(state.hands).map(Number).sort((a, b) => a - b);
  return numbers.map((n) => toHud(state.hands[n]));
}
export const hudSide = (who) => (who === "them" ? "opp" : "you");

// How far a score is from the double line, for the display: the loser's
// figure against 500 decides a double game (Foster p. 339).
export function doubleWatch(totals) {
  const lo = Math.min(totals.you, totals.them);
  const hi = Math.max(totals.you, totals.them);
  return { leader: totals.you === totals.them ? null : totals.you > totals.them ? "you" : "them", behind: lo < DOUBLE_LINE, atRisk: hi >= TARGET - 250 && lo < DOUBLE_LINE, hi, lo };
}

// ---- the kit's score HUD (kit/js/hud.js) --------------------------------------

// Twenty segments of fifty points: the quarter marks are 250, 500 (the double
// line), 750 and 1000.
export const hudGame = defineGame({
  categories: CATEGORIES,
  target: TARGET,
  segments: 20,
  lit: (units) => Math.floor(units / 50),
  goal: "1000 up",
  names: { you: "You", opp: "Opponent" },
  words: { hand: "Hand" },
  line: (l) => l,
});

// A scoring item as the HUD takes it.
export const hudEvent = (item) => ({ side: hudSide(item.who), cat: item.cat, label: item.label, pts: item.pts, hand: item.hand });

// ---- the quiet notes under the score ------------------------------------------

const said = (list) => list.map((c) => c.points).join(" + ");

// Melds announced and still to score (state.carried, their_carried): "To score: you 20 + 40".
export function carriedNote(s) {
  const mine = s?.carried ?? [];
  const theirs = s?.their_carried ?? [];
  const parts = [];
  if (mine.length) parts.push(`you ${said(mine)}`);
  if (theirs.length) parts.push(`your opponent ${said(theirs)}`);
  return parts.length ? `To score: ${parts.join("; ")}` : "";
}

// The double game (Foster p. 339): 1000 up, doubled if the loser has under 500.
export function doubleNote(totals) {
  const { lo, hi } = doubleWatch(totals);
  if (lo >= DOUBLE_LINE) return "Both past 500: no double game now";
  if (hi >= TARGET - 250) return "Under 500 when 1000 is reached: a double game";
  return "1000 wins; under 500 at the end is a double game";
}
