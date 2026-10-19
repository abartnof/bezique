// The counting aid: which cards are still unseen, worked out from the
// person's own view and nothing else (docs/PROTOCOL.md): their hand and
// table, the opponent's laid and known cards, the turn-up, the trick in
// hand and, with the tracker aid on, every trick of the hand. Whatever is
// not among these is in the stock or in the opponent's hand, and the model
// says no more than that.
//
// What a player counts, from the period books (Foster 1897 pp. 345-347;
// Cady 1896 pp. 16-17; docs/LITERATURE.md): the brisques, aces and tens,
// since each is ten to whoever takes it; the trumps, which win; the kings
// and queens, which make marriages; and the queens of spades and jacks of
// diamonds that make bezique. Foster adds that a player "may not count the
// tricks they have won" at the table (p. 343), which is why the page keeps
// this aid off until it is asked for (RULES.md section 3).

export const RANKS = ["A", "T", "K", "Q", "J", "9", "8", "7"];
export const SUITS = ["S", "H", "D", "C"];
export const SUIT_NAME = { S: "spades", H: "hearts", D: "diamonds", C: "clubs" };
export const SUIT_GLYPH = { S: "♠", H: "♥", D: "♦", C: "♣" };
export const RANK_NAME = { A: "ace", T: "ten", K: "king", Q: "queen", J: "jack", 9: "nine", 8: "eight", 7: "seven" };
const COPIES = 2;

// The cards of the person's view, as codes; null if the tracker aid is off
// (a hand's played tricks are not given, so the count would be a guess).
export function seenCodes(s) {
  if (!s || !s.tricks_played || !s.hand) return null;
  const codes = [];
  const add = (c) => c && codes.push(c.card);
  [s.hand, s.table, s.their_table, s.their_known].forEach((list) => (list ?? []).forEach(add));
  add(s.turn_up);
  for (const t of s.tricks_played) {
    add(t.led);
    add(t.second);
  }
  if (s.trick) {
    add(s.trick.led);
    add(s.trick.second);
  }
  return codes;
}

const plural = (n, word) => `${n} ${word}${n === 1 ? "" : "s"}`;

// { out: { code: copies unseen }, conflict, unseen, stock, hidden, trump, lines, rows } or null.
export function trackerModel(s) {
  const codes = seenCodes(s);
  if (!codes) return null;
  const seen = {};
  for (const c of codes) seen[c] = (seen[c] ?? 0) + 1;
  const out = {};
  // A card seen more often than it exists is a double count in the view, not a
  // small number to show: it is flagged, and the count is left as it falls.
  let conflict = false;
  for (const suit of SUITS)
    for (const rank of RANKS) {
      const left = COPIES - (seen[rank + suit] ?? 0);
      if (left < 0) conflict = true;
      out[rank + suit] = Math.max(0, left);
    }
  const count = (pred) => Object.entries(out).reduce((n, [code, k]) => (pred(code[0], code[1]) ? n + k : n), 0);
  const trump = s.trump ? s.trump[0].toUpperCase() : null;
  const stock = s.stock ?? 0;
  const hidden = s.their_hidden ?? 0;
  const unseen = count(() => true);

  const aces = count((r) => r === "A");
  const tens = count((r) => r === "T");
  const trumps = trump ? count((r, su) => su === trump) : 0;
  const qs = out.QS;
  const jd = out.JD;
  const lines = [
    { id: "unseen", text: `${plural(unseen, "card")} unseen: ${stock} in the stock, ${hidden} in your opponent's hand` },
    { id: "brisques", text: `Brisques out: ${aces + tens} of 16 (${plural(aces, "ace")}, ${plural(tens, "ten")}), ${(aces + tens) * 10} points` },
  ];
  if (trump) lines.push({ id: "trumps", text: `Trumps out: ${trumps} of 16` });
  lines.push({ id: "bezique", text: `Bezique: ${qs} queen${qs === 1 ? "" : "s"} of spades and ${jd} jack${jd === 1 ? "" : "s"} of diamonds out` });
  lines.push({ id: "marriages", text: `Kings out: ${count((r) => r === "K")}, queens out: ${count((r) => r === "Q")}` });

  if (conflict) lines.push({ id: "conflict", text: "This count cannot be trusted: a card was counted more than twice." });

  const rows = SUITS.map((suit) => ({
    suit,
    glyph: SUIT_GLYPH[suit],
    trump: suit === trump,
    cells: RANKS.map((rank) => ({ code: rank + suit, rank, out: out[rank + suit], kind: rank === "A" || rank === "T" ? "brisque" : "plain", bezique: rank + suit === "QS" || rank + suit === "JD" })),
  }));
  return { out, conflict, unseen, stock, hidden, trump, lines, rows };
}
