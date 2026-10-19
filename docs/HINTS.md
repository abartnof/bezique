# Hints that teach

> Status: **built**, 17 Oct 2026. Engine: `crates/bezique-core/src/hints.rs`.
> Wording: `web3d/src/hinttext.js`. Wire format: `docs/PROTOCOL.md`, "The hint's reason".

## Decision

A hint answers two questions, in two layers:

- **do**: what to play in this position, with the cards named and the numbers
  from the position ("Win the ten of diamonds with your ace of spades: an ace
  or ten is worth 10").
- **rule**: one sentence the player can carry to another hand, with its
  source. The rule is shown quieter than the instruction; a "Why?" opens the
  source line.

Every hint is two sentences: **do** says why this card now (the position's
own facts), and **rule** states what generalises, so the same situation next
time needs no hint. Note-keeping ("count the cards") is not a reason.

The hint itself is the top rung's choice, unchanged. The engine does not
change what it advises; after the choice it classifies it against the other
legal moves with cheap deterministic tests (`hints::classify`) and sends a
reason key and its parameters. A choice that fits no case is `generic`, shown
as the plain instruction with no rule. Hints stay off by default.

The layers are switched separately by the client's preferences: `hintFacts`
(the instruction, default on) and `hintRules` (the rule and its source,
default on).

## Sources and what is claimed

Foster is *Foster's Complete Hoyle* (1897) and Cady is *Bézique* (1896); both
are kept in `docs/sources/`. Page numbers are those of `docs/RULES.md` for
Foster and the running heads of Cady's text. Where a rule is only the
project's reading of the maxims, the hint says so ("The project's reading, not
a book's rule") and the table below marks it **interpretation**. The period
books give few rules of play (Cady's "Hints and Suggestions", pp. 22-23, and
"A Few Hints", p. 29, are the main source), so many rules here are
interpretations. No rule is presented as a book's unless the book says it.

## The rules

| Key | When it fires | Rule shown | Source |
|---|---|---|---|
| `take_brisque` | The hint wins a trick led with an ace or ten | Win a trick that holds an ace or a ten: 10 to you rather than to them | Foster p. 340; Cady p. 23 |
| `take_to_declare` | The hint wins, and you hold a combination to declare or one announced and not yet scored | Win a trick when you have a combination to score; only the winner may declare | Cady p. 22; Foster p. 340 |
| `win_last_stock_trick` | The hint wins, and one face-down card remains | Win the last trick of the stock: you declare once more and lead into the last eight | **Interpretation**; not measured on its own (`measurements/skills-final-style.txt` holds the memory and endgame comparisons, not this) |
| `take_with_lowest_winner` | The hint wins with the cheapest winning card (a plain suit before a trump) | If you mean to win, win with the least card that does | **Interpretation** |
| `take_sparing_pieces` | The hint wins, a cheaper card also wins, and every such card is worth keeping for a combination, held or in prospect | To win a trick, spend the cheapest winner you do not need for a combination, and keep the kings and queens you mean to marry | **Interpretation** of a period maxim: win with the ace and save the king (American Hoyle 1864, via `docs/LITERATURE.md`) |
| `let_brisque_go` | An ace or ten is led, a winning card is held, and the hint does not play it; the card named is the cheapest winner (a plain suit before a trump) | Do not spend a strong card or a trump on a brisque when it costs more than 10 points | **Interpretation** (Foster p. 340 gives the value only) |
| `throw_lowest` | Following, the card played is not a winner (a winner may be held: the stock phase does not force a win), no legal card costs less, and it is no piece of a combination | When you are not taking a trick, give up your least useful card, and spare aces and tens | Cady pp. 22, 23 |
| `lead_low_nonbrisque` | Leading a non-brisque while the stock lasts, the cheapest legal card and no piece of a combination | Lead a low card you will not miss, never a ten or an ace if you can avoid it | Cady pp. 22, 23 |
| `play_declared_card` | The card played is already declared and has no work left | Play cards you have already declared | Cady p. 22 |
| `lead_master_brisque` | Leading an ace or ten of a plain suit while the stock lasts, with no higher card of that suit unseen | Lead an ace or ten once nothing higher of its suit is left: only a trump can take it, and its 10 is yours unless they trump it | Foster pp. 366-368 (bring home a plain-suit ten, via `docs/LITERATURE.md`); Cady p. 23 warns against leading a ten otherwise |
| `nothing_decisive` | A cheap plain card that is not the cheapest, no case above fits, and a trump, ace, ten or combination card is held | When no play stands out, spend a low card that scores nothing and never a trump, an ace, a ten or a card of a combination | **Common wisdom**; Cady p. 22 says no general rule for discarding can be laid down |
| `keep_for_bezique` | A card is played and a card of equal or lower rank is held that still serves a bezique (the queen of spades or jack of diamonds, the other half not the card played) | Keep the queen of spades and the jack of diamonds: bezique 40, two pairs 500 | Scores: Foster p. 341; keeping them is **interpretation** |
| `keep_marriage` | The same, with a king or queen whose partner is held, the partner not the card played | Keep a king and queen of one suit together; declare them after a trick | Scores: Foster p. 341; keeping is **interpretation** |
| `keep_sequence` | The same, with three or more of A, 10, K, Q, J of trumps, the card played not among them | Keep them for the sequence (250) | Cady p. 29; Foster p. 341 |
| `keep_four` | The same, with three of a kind (A, K, Q, J) held besides the card played, and a fourth still to come | Keep three of a kind while the fourth can still come | Scores: Foster p. 341; keeping is **interpretation** |
| `save_seven_for_dix` | The same, with the seven of trumps | Do not play away the seven of trumps: exchange it for the turn-up after a trick won, for 10 | Foster p. 340 |
| `declare_now` | Declaring a combination | After a trick won, declare your best combination before you draw | Foster pp. 340-341 |
| `score_carried` | Scoring a combination announced earlier | An earlier announcement scores on a later trick while its cards are on the table | Foster pp. 341-343 |
| `single_before_double` | The hint declares a single bezique when a double was offered | Declare the single first, then the second pair | Foster p. 342 (`docs/RULES.md`) |
| `royal_before_sequence` | The hint declares the royal marriage when a sequence was offered | Royal marriage first, the sequence on the next trick | Foster pp. 341-343 |
| `exchange_dix` / `show_dix` | Exchanging the seven of trumps, or showing it when the turn-up is one | Exchange the seven of trumps for the turn-up as soon as you win a trick, before you draw | Foster p. 340 |
| `pass` | Drawing without declaring | Declaring is optional; passing waives it until the next trick won | Foster p. 340 |
| `last_eight_must_win` | Last eight tricks, following, every legal card wins | Follow suit and win if you can | Foster p. 343 |
| `last_eight_give_up` | Last eight tricks, following, no legal card wins, and the hint gives up the cheapest | When you cannot win, give up your least valuable card; trumps, aces and tens are for tricks you can win | **Interpretation**; Cady p. 22 |
| `last_eight_lead_master` | Last eight tricks, leading a card that nothing in the opponent's hand can beat: no higher card of its suit, and for a plain suit either a card of that suit to follow with or no trump | Before you lead, ask what could beat the card; if nothing can, lead it | **Interpretation**; Cady p. 23 |
| `last_eight_lead_low` | Last eight tricks, leading the cheapest card when nothing is sure to win | Lead your least valuable card and keep the strong ones until they can win | **Interpretation**; Cady p. 22 |
| `last_eight_lead_ace` | Last eight tricks, leading a plain-suit ace | Lead an ace of a plain suit: it wins, or draws a trump | Cady pp. 23, 29 |
| `draw_out_trump` | Last eight tricks, leading the highest trump while holding three or more | With three or more trumps, lead your highest to draw your opponent's before they can trump your winners | **Interpretation**; Cady p. 23 differs (lowest trump when reduced to trumps) |
| `solved_endgame` | Last eight tricks, no case above fits, and the exact solution is the card chosen (there may be equal-valued others; the wording says only that it scores as well as any) | The opponent's hand is exactly the cards not played and not yours: lay them out and play the hand through before you lead | Cady p. 23; Foster pp. 366-368 (study the exposed cards) |
| `generic` | Anything else | (instruction only) | |

A combination is "still to come" only if a copy is in your hand or unseen (stock or the opponent's concealed hand), not on the opponent's table or in a trick; the card being played is not counted as held.

Parameters by key: `card` (the one to play) in every play, `led` for a
following play, `spared` for `throw_lowest`, `lead_low_nonbrisque`,
`last_eight_give_up` and `last_eight_lead_low`, `kind` (`trump` or `plain`)
for `last_eight_lead_master`, `kept` for the `keep_*` keys (with `partner`,
`points` for marriages), for `take_sparing_pieces` and `nothing_decisive`
(the card to hold on to), and for `lead_master_brisque` (a higher card of the
suit held, optional), `turn_up` for `exchange_dix`, `trumps` for
`draw_out_trump`, `count` (the cards the opponent holds) for `solved_endgame`;
declarations carry `meld` (`royal_marriage`, `marriage`, `sequence`,
`bezique`, `double_bezique`, `four_aces` ...), `points` and, for marriages,
`suit`.

## Evidence and limits

`crates/bezique-core/tests/hints.rs` plays whole hands with hints on over
varied play: every hint is classified into a listed key, none panics, and the
hinted command is always accepted by the engine. In the 10-seed run about 1 in
70 hints is `generic`; the most frequent keys are `lead_low_nonbrisque`,
`throw_lowest`, `save_seven_for_dix` and `last_eight_give_up`.

Residual `generic`, measured over 80 seeds (11,400 hints, levels 1-5, varied
play): 3.5% generic plus
2.3% in the old `lead_cheapest` catch-all before; 1.5% generic after, all in
the stock phase. The old catch-all's claim (the card is the cheapest) was false
by construction, so it is gone. Situations and what they got:

| Situation (share of hints before) | Now |
|---|---|
| Last eight, card is not the cheapest or the master lead (2.1%) | `solved_endgame`: the opponent's hand is exactly the unseen cards and the rung solves it exactly; the rule is to treat it as an open hand |
| Stock phase, a plain card that is neither lowest nor the dearest, with a trump, ace, ten or combination card held (2.3%) | `nothing_decisive`: a guard, naming the dearest card to keep (common wisdom) |
| Leading an ace or ten nothing higher of its suit can beat (0.3%) | `lead_master_brisque` |
| Winning with a card dearer than a cheaper winner that is a combination piece (0.1%) | `take_sparing_pieces` |
| Leading or giving up a ten or ace that a higher card out can beat; a low trump led or played to a trick a cheaper plain card would win (1.5%) | **No sound rule found**: the books say not to lead a ten and to win brisques, and the rung does both; left `generic` |

A low trump was also removed from `throw_lowest` and `lead_low_nonbrisque`
(a trump is never the cheapest card when a plain one is held). The node test
`web3d/test/hinttext.test.js` checks that the client's keys equal the engine's,
that every wording is complete and short, and that unknown reasons fall back to
the engine's text.

The classification is of the choice, not of the reasoning inside the rung: the
reason shown is the nearest honest account, and may not be the whole of why the
rung preferred a card. `save_seven_for_dix` is the most common reason because
a seven of trumps held is often the cheapest alternative to the card chosen.
