// The pack: codes like "AS" and "TD" (rank, suit), as the engine writes them.
// Ranks run high to low in bezique order: the ten ranks above the king.
export const RANKS = ["A", "T", "K", "Q", "J", "9", "8", "7"];
export const SUITS = ["S", "H", "D", "C"];
export const PACK = 64;

const TRUMP_LETTER = { spades: "S", hearts: "H", diamonds: "D", clubs: "C" };
export const trumpLetter = (name) => TRUMP_LETTER[name] ?? null;

export const SUIT_NAME = { S: "spades", H: "hearts", D: "diamonds", C: "clubs" };
export const SUIT_SYMBOL = { S: "♠", H: "♥", D: "♦", C: "♣" };
export const RANK_NAME = { A: "ace", T: "ten", K: "king", Q: "queen", J: "jack", 9: "nine", 8: "eight", 7: "seven" };

export const cardName = (code) => `${RANK_NAME[code[0]]} of ${SUIT_NAME[code[1]]}`;

// A hand reads suit by suit, the trump suit first, each from the ace down.
export function byHand(trump) {
  const suits = trump ? [trump, ...SUITS.filter((s) => s !== trump)] : SUITS;
  const key = (c) => suits.indexOf(c.code[1]) * 8 + RANKS.indexOf(c.code[0]);
  return (a, b) => key(a) - key(b) || (a.id ?? 0) - (b.id ?? 0);
}
