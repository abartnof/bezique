// The table as the page keeps it, and the engine's events as changes to it.
//
// A model holds only what the person may know: their hand and table cards
// with the engine's ids, the opponent's known cards, how many they hold
// unseen, the stock, the turn-up, the open trick, and how many cards each
// side has won. `fromState` reads one off the engine's state; `reduce` moves
// it by one event, so the choreography can lay out every step between two
// states. Whatever the steps get wrong, the last layout is the state's own.
import { PACK, trumpLetter } from "./cards.js";

export function packModel() {
  return { hand: [], table: [], known: [], hidden: 0, theirTable: [], stock: PACK, turnUp: null, trick: [], won: { you: 0, them: 0 }, cut: null, trump: null, dealer: null };
}

const own = (c) => ({ code: c.card, id: c.id });

export function fromState(s) {
  if (s.hand === undefined) {
    const cut = s.events.find((e) => e.kind === "cut");
    return { ...packModel(), stock: cut ? PACK - 2 : PACK, cut: cut ? { you: cut.yours.card, them: cut.theirs.card } : null };
  }
  const m = {
    hand: s.hand.map(own),
    table: s.table.map(own),
    known: s.their_known.map((c) => c.card),
    hidden: s.their_hidden,
    theirTable: s.their_table.map((c) => ({ code: c.card })),
    stock: s.stock,
    turnUp: s.turn_up ? s.turn_up.card : null,
    trick: s.trick ? [{ who: s.trick.leader, code: s.trick.led.card }] : [],
    won: { you: 0, them: 0 },
    cut: null,
    trump: trumpLetter(s.trump),
    dealer: s.you_deal ? "you" : "them",
  };
  const elsewhere = m.hand.length + m.table.length + m.known.length + m.hidden + m.theirTable.length + m.stock + (m.turnUp ? 1 : 0) + m.trick.length;
  const won = Math.max(0, PACK - elsewhere);
  const yours = 2 * s.events.filter((e) => e.kind === "took_trick" && e.who === "you" && e.hand === s.hand_number).length;
  m.won.you = Math.min(won, yours);
  m.won.them = won - m.won.you;
  return m;
}

const take = (list, code) => {
  const i = list.findIndex((c) => (c.code ?? c) === code);
  return i < 0 ? null : list.splice(i, 1)[0];
};

function lay(m, who, code) {
  if (who === "you") {
    const c = take(m.hand, code) ?? take(m.table, code);
    if (!c) return;
    m.table.push(c);
  } else {
    if (!take(m.known, code) && m.hidden > 0) m.hidden -= 1;
    m.theirTable.push({ code });
  }
}

function removePlayed(m, who, code) {
  if (who === "you") return take(m.hand, code) ?? take(m.table, code);
  return take(m.known, code) ?? take(m.theirTable, code) ?? (m.hidden > 0 ? (m.hidden -= 1) : null);
}

// One event applied to a model: a new model. `ctx.final` is the model the
// whole change ends at (a deal's cards are known only from there).
export function reduce(model, e, ctx = {}) {
  const m = structuredClone(model);
  switch (e.kind) {
    case "cut":
      m.cut = { you: e.yours.card, them: e.theirs.card };
      m.stock = PACK - 2;
      return m;
    case "dealt": {
      const f = ctx.final ?? m;
      return { ...packModel(), hand: structuredClone(f.hand), hidden: 8, stock: PACK - 17, turnUp: e.turn_up.card, trump: f.trump, dealer: e.dealer };
    }
    case "played":
      removePlayed(m, e.who, e.card.card);
      m.trick.push({ who: e.who, code: e.card.card });
      return m;
    case "took_trick":
      m.won[e.who] += m.trick.length;
      m.trick = [];
      return m;
    case "declared":
      for (const c of e.laid) lay(m, e.who, c.card);
      return m;
    case "dix": {
      if (!e.taken || !m.trump) return m;
      const seven = `7${m.trump}`;
      if (e.who === "you") {
        take(m.hand, seven) ?? take(m.table, seven);
        m.hand.push({ code: e.taken.card, id: e.taken.id });
      } else {
        take(m.known, seven);
        m.known.push(e.taken.card);
      }
      m.turnUp = seven;
      return m;
    }
    case "stock_out": {
      m.hand.push(...m.table.splice(0));
      m.known.push(...m.theirTable.splice(0).map((c) => c.code));
      if (m.turnUp) {
        if (e.last_card === "you") m.known.push(m.turnUp);
        else m.hand.push({ code: m.turnUp, id: e.turn_up.id });
        m.turnUp = null;
      }
      m.stock = 0;
      return m;
    }
    case "hand_ends":
      return { ...packModel(), trump: m.trump, dealer: m.dealer };
    default:
      return m;
  }
}

// The draw that follows a trick taken: the winner draws and then the other
// (a stock of one is the last card and the turn-up). The engine reports only
// the person's own draw, in the `drew` event.
export function draw(model, winner, drew) {
  const m = structuredClone(model);
  if (m.stock < 1) return null;
  const give = (who, code) => {
    if (who === "you") m.hand.push({ code: code ?? drew.card.card, id: code ? undefined : drew.card.id });
    else if (code) m.known.push(code);
    else m.hidden += 1;
  };
  const loser = winner === "you" ? "them" : "you";
  give(winner, undefined);
  if (m.stock === 1) {
    const up = m.turnUp;
    m.turnUp = null;
    m.stock = 0;
    give(loser, up);
  } else {
    m.stock -= 2;
    give(loser, undefined);
  }
  return m;
}

// The steps between two states: one per new event, the draw standing in for
// the person's `drew`. Null when the change is not a continuation (an undo,
// a new sitting).
export function stagesBetween(prev, next) {
  if (!prev || next.events.length < prev.events.length) return null;
  const fresh = next.events.slice(prev.events.length);
  const stages = [];
  let m = prev.model;
  let winner = null;
  fresh.forEach((e, n) => {
    const at = prev.events.length + n;
    if (e.kind === "took_trick") winner = e.who;
    const drawn = e.kind === "drew" && winner ? draw(m, winner, e) : null;
    const after = drawn ?? reduce(m, e, { final: next.model });
    stages.push({ kind: drawn ? "draw" : e.kind, event: e, at, before: m, model: after, winner });
    m = after;
  });
  return stages;
}
