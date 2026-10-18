// The count at a hand's end, and at the game's, as data: every scorable unit
// the hand earned told in turn (a declared combination, the brisques of one
// trick, a seven or the dix, the last trick's ten), then the hand's total, and
// at the game's end the doubling and the victory. Bezique scores as it goes,
// so each unit was shown once when it was earned; the count tells them again,
// one badge each at the place it belongs, in the HUD ledger's order.
//
// Pure: events in, steps out. The times are the kit's count schedule
// (kit/js/count.js countSchedule) in the choreography's ms; choreography.js
// holds the table for them, scoreboard.js plays them on the director's clock
// and badges.js draws them. web3d/test/handcount.test.js holds the units to
// the engine's own hand figures.
import { countSchedule } from "../../kit/js/count.js";
import { badgeFor } from "./badges.js";
import { CATEGORIES, fold, hudGame, hudSide, itemsFor } from "./scoring.js";

export const LEAD = 250; // ms before the first unit
export const LINE_MIN = 280;
export const LINE_MAX = 650;
export const LINE_BUDGET = 7000; // a long hand's units share this
export const TOTAL_HOLD = 1300; // the hand's total, read
export const DOUBLE_HOLD = 1100; // the doubling, read
export const TAIL = 700;

// Where on the table a unit's badge rises in the count: a combination over its
// player's laid row, the sevens at the stock, brisques and the last trick at
// the winner's pile.
export function countAnchorOf(item) {
  if (item.cat === "declared") return item.who === "you" ? "yourTable" : "theirTable";
  if (item.cat === "seven") return "stock";
  return item.who === "you" ? "wonYou" : "wonThem";
}

// The beat of one unit, shorter for a long hand.
export const lineFor = (n) => Math.max(LINE_MIN, Math.min(LINE_MAX, Math.round(LINE_BUDGET / Math.max(1, n))));

const rank = Object.fromEntries(CATEGORIES.map(([k], i) => [k, i]));

// The units of hand `hand` among events[0..upto), in the count's order: by
// category as the ledger lists them, within it as they happened. `running` is
// the hand's tally after the unit; `stack` how many of the units just before
// it rose at the same place (the badge is lifted clear of them).
export function unitsOf(events, hand, upto = events.length) {
  const items = itemsFor(events.slice(0, upto))
    .flat()
    .filter((it) => it.hand === hand);
  const ordered = items
    .map((it, n) => [it, n])
    .sort((a, b) => rank[a[0].cat] - rank[b[0].cat] || a[1] - b[1])
    .map(([it]) => it);
  const running = { you: 0, them: 0 };
  const units = [];
  ordered.forEach((it, i) => {
    running[it.who] += it.pts;
    const anchor = countAnchorOf(it);
    let near = 0;
    for (let j = units.length - 1; j >= 0 && near < 3 && units[j].anchor === anchor; j--) near++;
    units.push({
      i,
      who: it.who,
      side: hudSide(it.who),
      cat: it.cat,
      pts: it.pts,
      label: it.label,
      hand,
      at: it.at,
      anchor,
      stack: near % 3,
      badge: { ...badgeFor(it), anchor },
      running: { ...running },
    });
  });
  return units;
}

// The count at the end event `at` (a hand_ends or a game_ends), or null:
// { at, hand, kind, units, steps, times, end }. `steps` are the units, then
// { kind: "total" }, then at a game's end { kind: "double" } (the loser under
// 500) and { kind: "victory" }; `times[k]` is when step k happens and `end`
// when the count is over, ms from its start.
export function countPlan(events, at) {
  const e = events[at];
  if (!e || (e.kind !== "hand_ends" && e.kind !== "game_ends")) return null;
  const hand = e.hand;
  // A game that ends on the hand's last trick has had its hand counted already.
  const counted = e.kind === "game_ends" && events.slice(0, at).some((x) => x.kind === "hand_ends" && x.hand === hand);
  const units = counted ? [] : unitsOf(events, hand, at + 1);
  const steps = units.map((u) => ({ kind: "unit", ...u }));
  const sched = countSchedule(hudGame, units, { start: LEAD, line: lineFor(units.length) });
  const times = [...sched.lines];
  let t = LEAD;
  if (units.length) {
    const sum = units.at(-1).running;
    steps.push({ kind: "total", hand, you: sum.you, them: sum.them });
    times.push(sched.end);
    t = sched.end + TOTAL_HOLD;
  }
  if (e.kind === "game_ends") {
    if (e.double) {
      steps.push({ kind: "double", hand });
      times.push(t);
      t += DOUBLE_HOLD;
    }
    steps.push({ kind: "victory", winner: e.winner, double: Boolean(e.double), totals: fold(events, at + 1).totals });
    times.push(t);
    t += TAIL;
  }
  return { at, hand, kind: e.kind, units, steps, times, end: t };
}

// The counts an advance brings (events from index `first` on), as the
// choreography holds the table for them: one plan a hand or game end that has
// anything to tell; their steps in order are the director's `count.lines`.
export function plansSince(events, first = 0) {
  const out = [];
  for (let at = first; at < events.length; at++) {
    const plan = countPlan(events, at);
    if (plan && plan.steps.length) out.push(plan);
  }
  return out;
}
