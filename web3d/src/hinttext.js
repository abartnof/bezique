// The hint in two layers (docs/HINTS.md): `do`, the concrete instruction for
// this position with the cards named, and `rule`, one sentence the player can
// carry to another hand, with its source. The engine sends only a reason
// (`hint.reason`: a key and the cards and numbers it needs, docs/PROTOCOL.md);
// the words are here. Pure functions, no DOM.

import { cardName } from "./cards.js";

const FOSTER = "Foster, Complete Hoyle (1897)";
const CADY = "Cady, Bézique (1896)";

// A source: `cite` is what the book says, `page` its place; `interp` marks a
// rule that is the project's own reading of the maxims, not a book's.
const book = (who, page, note) => ({ who, page, note, interp: false });
const mine = (note, who, page) => ({ who, page, note, interp: true });

const MELD = {
  royal_marriage: (p) => `royal marriage in ${p.suit}`,
  marriage: (p) => `marriage in ${p.suit}`,
  sequence: () => "sequence",
  bezique: () => "bezique",
  double_bezique: () => "double bezique",
  four_aces: () => "four aces",
  four_kings: () => "four kings",
  four_queens: () => "four queens",
  four_jacks: () => "four jacks",
};
const meldName = (p) => (MELD[p.meld] ?? (() => "combination"))(p);

// key -> { do(p, n), rule, src }. `n` names a card code; `p` is the params.
export const REASONS = {
  take_brisque: {
    do: (p, n) => `Take the ${n(p.led)} with your ${n(p.card)}: a trick holding an ace or ten is worth 10, and you can win it.`,
    rule: "Win a trick that holds an ace or a ten whenever you can: it is 10 for you and 10 less for your opponent.",
    src: book(FOSTER, "p. 340", "Brisques score as soon as the trick is taken; “there is no object in winning any trick which does not contain a brisque” but for the lead.") ,
    also: book(CADY, "p. 23", "If you do not win a brisque, your opponent does: a difference of twenty."),
  },
  take_to_declare: {
    do: (p, n) => `Win the ${n(p.led)} with your ${n(p.card)}: you hold a combination worth ${p.points}, and only a trick's winner may declare.`,
    rule: "When you hold a combination to declare, win the trick: only its winner can score it.",
    src: book(CADY, "p. 22", "Tricks are of no value unless you have cards to declare, or for the brisques in them."),
    also: book(FOSTER, "p. 340", "Only the winner of a trick may declare, before drawing."),
  },
  win_last_stock_trick: {
    do: (p, n) => `Win the ${n(p.led)} with your ${n(p.card)}: it is the last trick of the stock, so you declare once more and lead the last eight.`,
    rule: "Fight for the last trick of the stock: it brings a final declaration and the lead into the last eight.",
    src: mine("Measured in this project: a rung that fights for it gains about 2.5 points a hand (measurements/skills-final-style.txt).", FOSTER, "p. 343"),
  },
  take_with_lowest_winner: {
    do: (p, n) => `Take the ${n(p.led)} with your ${n(p.card)}: it is the cheapest card you hold that wins, so you spend no more than you must.`,
    rule: "When you mean to win a trick, win it with the lowest card that does, and keep the stronger cards for later tricks.",
    src: mine("A reading of the maxims; the books give no rule for it.", CADY, "p. 22"),
  },
  let_brisque_go: {
    do: (p, n) => `Let the ${n(p.led)} go and play your ${n(p.card)}: winning it would cost your ${n(p.winner)}, worth more than the 10 points.`,
    rule: "Do not spend a trump or a strong card on a brisque unless the 10 points repay it.",
    src: mine("A reading of the maxims: Foster values a brisque at ten (p. 340) but does not say what it is worth paying.", FOSTER, "p. 340"),
  },
  throw_lowest: {
    do: (p, n) => `You will not win this trick, so give up the ${n(p.card)}${p.spared ? `, and keep your ${n(p.spared)}` : ""}: it is the card you can best spare.`,
    rule: "When you will not win a trick, give up your least useful card and keep the aces and tens to win with.",
    src: book(CADY, "p. 22", "Throw away worthless cards, unless there is a sound reason for winning the trick; p. 23: keep a ten to win tricks with."),
  },
  lead_low_nonbrisque: {
    do: (p, n) => `Lead the ${n(p.card)}${p.spared ? `, not your ${n(p.spared)}` : ""}: it is low and scores nothing, so losing the trick costs little.`,
    rule: "Lead a low card you will not miss, and keep the aces and tens to win tricks with.",
    src: book(CADY, "p. 22", "Lead the lowest card in the hand, a seven instead of an eight; p. 23: do not lead a ten if you can avoid it."),
  },
  play_declared_card: {
    do: (p, n) => `Play the ${n(p.card)}: it has already scored and has no combination left to make.`,
    rule: "Play out cards you have already declared: they are spent, and they show your opponent nothing new.",
    src: book(CADY, "p. 22", "As a rule play your declared cards, which also lets you form the combination again."),
  },
  lead_cheapest: {
    do: (p, n) => `Lead the ${n(p.card)}: of the cards you hold, it is the one you will miss least.`,
    rule: "When no card is clearly worthless, lead the one least likely to score and keep the cards that still can.",
    src: mine("A reading of the maxims for a hand with no clearly worthless card.", CADY, "p. 22"),
  },
  keep_for_bezique: {
    do: (p, n) => `Play the ${n(p.card)} and keep the ${n(p.kept)}: it is half of a bezique, which scores 40.`,
    rule: "Play away cards that cannot score and hold the queen of spades and jack of diamonds: bezique is 40, two pairs 500.",
    src: mine("The scores are Foster's; that they are worth keeping over a spare card is this project's reading.", FOSTER, "p. 341"),
  },
  keep_marriage: {
    do: (p, n) => `Play the ${n(p.card)} and keep the ${n(p.kept)}: with your ${n(p.partner)} it is a marriage worth ${p.points}.`,
    rule: "Keep a king and queen of one suit together, and declare them after you win a trick.",
    src: mine("The scores are Foster's; keeping the pair is this project's reading.", FOSTER, "p. 341"),
  },
  keep_sequence: {
    do: (p, n) => `Play the ${n(p.card)} and keep the ${n(p.kept)}: you hold several of the trump sequence, worth 250.`,
    rule: "With three or more of the ace, ten, king, queen and jack of trumps, keep them all for the 250 sequence.",
    src: book(CADY, "p. 29", "Retain as many sequence cards as possible (the 250 is Foster, p. 341)."),
  },
  keep_four: {
    do: (p, n) => `Play the ${n(p.card)} and keep the ${n(p.kept)}: you hold three of a kind, and one more makes four.`,
    rule: "Keep three aces, kings, queens or jacks while the fourth can still come: four of a kind scores 40 to 100.",
    src: mine("The scores are Foster's (p. 341); keeping three for a fourth is this project's reading.", FOSTER, "p. 341"),
  },
  save_seven_for_dix: {
    do: (p, n) => `Play the ${n(p.card)} and keep your ${n(p.kept)}: swapped for the turn-up after a trick you win, it scores 10.`,
    rule: "Keep the seven of trumps rather than play it away: after any trick you win it scores 10 in exchange for the turn-up.",
    src: book(FOSTER, "p. 340", "The seven of trumps may be exchanged for the turn-up, scoring 10, right after winning a trick."),
  },
  declare_now: {
    do: (p) => `Declare the ${meldName(p)} now: it scores ${p.points}, and you may declare only after winning a trick, before you draw.`,
    rule: "After every trick you win, declare your best combination before you draw.",
    src: book(FOSTER, "pp. 340-341", "The winner of a trick may declare before drawing; declaring later waives the right until the next trick."),
  },
  score_carried: {
    do: (p) => `Score the ${meldName(p)} you announced earlier: ${p.points}, because you won a trick and its cards are still on the table.`,
    rule: "A combination you announced earlier scores on a later trick, so leave its cards on the table until it does.",
    src: book(FOSTER, "pp. 341-343", "Several may be announced at once, one scored per trick won."),
  },
  single_before_double: {
    do: (p) => `Declare the single bezique now (${p.points}): the other pair can follow later for the double, 500.`,
    rule: "Declare a single bezique first and the second pair after: the double needs a new card on the table.",
    src: book(FOSTER, "p. 342", "A second single bezique can be scored, but then the double is out of reach (docs/RULES.md)."),
  },
  royal_before_sequence: {
    do: (p) => `Declare the ${meldName(p)} (${p.points}) now: the sequence can follow on a later trick for 250.`,
    rule: "Declare the royal marriage first and the sequence on the next trick: that is two scores, not one.",
    src: book(FOSTER, "pp. 341-343", "A royal marriage may later be extended to the sequence by adding the ace, ten and jack."),
  },
  exchange_dix: {
    do: (p, n) => `Exchange your ${n(p.card)} for the turn-up ${n(p.turn_up)}: it scores 10, and the exchange comes right after a trick you win.`,
    rule: "Exchange the seven of trumps for the turn-up as soon as you win a trick, before you draw: it is 10 points.",
    src: book(FOSTER, "p. 340", "Made right after winning a trick, before drawing; it cannot be scored together with a declaration."),
  },
  show_dix: {
    do: () => "Show the seven of trumps: it scores 10 right after a trick you win.",
    rule: "A seven of trumps scores 10 when you show it after winning a trick, even if the turn-up is a seven.",
    src: book(FOSTER, "p. 340", "If the turn-up is a seven, or an exchange has been made, the exchange can still be made and scored."),
  },
  pass: {
    do: () => "Draw without declaring: you have nothing worth declaring.",
    rule: "Declaring is optional: with nothing to score, draw, and declare after the next trick you win.",
    src: book(FOSTER, "p. 340", "Drawing without declaring waives the right until the next trick won."),
  },
  last_eight_must_win: {
    do: (p, n) => `Play the ${n(p.card)}: it wins the ${n(p.led)}, and in the last eight tricks you must follow suit and win if you can.`,
    rule: "In the last eight tricks, follow suit and win the trick if you can, with a higher card of the suit or a trump.",
    src: book(FOSTER, "p. 343", "The second player must follow suit, and must win the trick if he can."),
  },
  last_eight_give_up: {
    do: (p, n) => `You cannot win the ${n(p.led)}: give up the ${n(p.card)}${p.spared ? `, and keep your ${n(p.spared)}` : ""}.`,
    rule: "When you cannot win a trick, give up your least valuable card: trumps, aces and tens are for tricks you can win.",
    src: mine("A reading of the maxims; Cady says to throw away worthless cards unless there is a reason to win the trick.", CADY, "p. 22"),
  },
  last_eight_lead_master: {
    do: (p, n) => `Lead the ${n(p.card)}: no higher card of that suit is still out${p.kind === "trump" ? ", so nothing can take it" : ", so only a trump can take it"}.`,
    rule: "Before you lead, ask what could beat the card. If nothing can, lead it: the trick is yours.",
    src: mine("A reading of the maxims: Cady (p. 23) has the last eight tricks decided by playing the cards that must win.", CADY, "p. 23"),
  },
  last_eight_lead_low: {
    do: (p, n) => `Lead the ${n(p.card)}, your cheapest card${p.spared ? `, and keep your ${n(p.spared)} for a trick it can win` : ""}.`,
    rule: "When no card of yours is sure to win, lead your least valuable one and keep the strong cards until they can win.",
    src: mine("A reading of the maxims; Cady (p. 22) leads the lowest card when there is no reason to lead another.", CADY, "p. 22"),
  },
  last_eight_lead_ace: {
    do: (p, n) => `Lead the ${n(p.card)}: only a trump can beat an ace of a plain suit, so it wins or draws a trump out.`,
    rule: "In the last eight tricks lead a high plain-suit card: it wins, or it draws a trump from your opponent.",
    src: book(CADY, "p. 23", "Play a high card of a plain suit to force a trump; p. 29: keep the aces for the last eight tricks."),
  },
  draw_out_trump: {
    do: (p, n) => `Lead the ${n(p.card)}: you hold ${p.trumps} trumps, so lead one to take theirs out.`,
    rule: "With three or more trumps in the last eight tricks, lead your highest to draw your opponent's, so they cannot trump your other winners.",
    src: mine("A reading of the maxims; Cady (p. 23) says to lead the lowest trump when reduced to trumps.", CADY, "p. 23"),
  },
};

// The ordered list of keys the engine can send (crates/bezique-core/src/hints.rs `KEYS`).
export const KEYS = [...Object.keys(REASONS), "generic"];

const SUIT_WORD = { spades: "spades", hearts: "hearts", diamonds: "diamonds", clubs: "clubs" };

// A card code as it is spoken: "7D" -> "seven of diamonds".
const spoken = (code) => (typeof code === "string" && code.length === 2 ? cardName(code) : String(code));

// How a source is told: "Foster, Complete Hoyle (1897), p. 340: ..." or, for the
// project's own reading, "This is the project's reading of the maxims. ...".
export function sourceLine(src, also) {
  if (!src) return "";
  const cite = `${src.who}, ${src.page}`;
  const lines = [src.interp ? `The project's reading, not a book's rule. ${src.note} (See ${cite}.)` : `${cite}: ${src.note}`];
  if (also) lines.push(`${also.who}, ${also.page}: ${also.note}`);
  return lines.join(" ");
}

// What the table shows for a hint: { do, rule, why, interp } or null.
//  - `do`: the instruction naming the cards (falls back to the engine's `text`);
//  - `rule`: the one-sentence principle, or "" for an unknown reason;
//  - `why`: the source line for the "Why?";
//  - `interp`: true when the rule is the project's own reading.
// `layers` = { facts, rules } switch the two layers (default both on); a
// layer that is off comes back as "". `why` goes with the rule.
export function hintView(hint, layers = {}) {
  if (!hint || !hint.command) return null;
  const { facts = true, rules = true } = layers;
  const r = hint.reason ?? {};
  const params = { ...r };
  if (params.suit) params.suit = SUIT_WORD[params.suit] ?? params.suit;
  const entry = REASONS[r.key];
  let doText = hint.text ?? "";
  try {
    if (entry) doText = entry.do(params, spoken);
  } catch {
    doText = hint.text ?? "";
  }
  doText = capitalise(doText);
  return {
    do: facts ? doText : "",
    rule: rules && entry ? entry.rule : "",
    why: rules && entry ? sourceLine(entry.src, entry.also) : "",
    interp: !!entry?.src?.interp,
  };
}

const capitalise = (s) => (s ? s[0].toUpperCase() + s.slice(1) : s);
