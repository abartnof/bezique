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
| `take_brisque` | The hint wins a trick led with an ace or ten | Win a trick that holds an ace or a ten: 10 points to you, 10 less for your opponent | Foster p. 340; Cady p. 23 |
| `take_to_declare` | The hint wins, and you hold a combination to declare | Win a trick when you have a combination to declare; only the winner may declare | Cady p. 22; Foster p. 340 |
| `win_last_stock_trick` | The hint wins, and one face-down card remains | Win the last trick of the stock: you declare once more and lead into the last eight | **Interpretation**; measured, about +2.5 points a hand (`measurements/skills-final-style.txt`) |
| `take_with_lowest_winner` | The hint wins with the cheapest winning card (a plain suit before a trump) | If you mean to win, win with the least card that does | **Interpretation** |
| `let_brisque_go` | An ace or ten is led, a winning card is held, and the hint does not play it | Do not spend a strong card or a trump on a brisque when it costs more than 10 points | **Interpretation** (Foster p. 340 gives the value only) |
| `throw_lowest` | Following, not winning, a card is given up | Give up your least useful card, and spare aces and tens | Cady pp. 22, 23 |
| `lead_low_nonbrisque` | Leading a non-brisque while the stock lasts | Lead a low card you will not miss, never a ten or an ace if you can avoid it | Cady pp. 22, 23 |
| `play_declared_card` | The card played is already declared and has no work left | Play cards you have already declared | Cady p. 22 |
| `lead_cheapest` | A lead or discard with no clearer case | Lead what you can spare, so the cards that can still score stay in hand | **Interpretation** |
| `keep_for_bezique` | A lower card is played and the queen of spades or jack of diamonds is held | Keep the queen of spades and the jack of diamonds: bezique 40, two pairs 500 | Scores: Foster p. 341; keeping them is **interpretation** |
| `keep_marriage` | The same, with a king or queen whose partner is held | Keep a king and queen of one suit together; declare them after a trick | Scores: Foster p. 341; keeping is **interpretation** |
| `keep_sequence` | The same, with three or more of A, 10, K, Q, J of trumps | Keep them for the sequence (250) | Cady p. 29; Foster p. 341 |
| `keep_four` | The same, with three of a kind (A, K, Q, J) and the fourth not known gone | Keep three of a kind while the fourth can still come | Scores: Foster p. 341; keeping is **interpretation** |
| `save_seven_for_dix` | The same, with the seven of trumps | Do not play away the seven of trumps: exchange it for the turn-up after a trick won, for 10 | Foster p. 340 |
| `declare_now` | Declaring a combination | After a trick won, declare your best combination before you draw | Foster pp. 340-341 |
| `score_carried` | Scoring a combination announced earlier | An earlier announcement scores on a later trick while its cards are on the table | Foster pp. 341-343 |
| `single_before_double` | The hint declares a single bezique when a double was offered | Declare the single first, then the second pair | Foster p. 342 (`docs/RULES.md`) |
| `royal_before_sequence` | The hint declares the royal marriage when a sequence was offered | Royal marriage first, the sequence on the next trick | Foster pp. 341-343 |
| `exchange_dix` / `show_dix` | Exchanging the seven of trumps, or showing it when the turn-up is one | Exchange the seven of trumps for the turn-up as soon as you win a trick, before you draw | Foster p. 340 |
| `pass` | Drawing without declaring | Declaring is optional; passing waives it until the next trick won | Foster p. 340 |
| `last_eight_must_win` | Last eight tricks, following, every legal card wins | Follow suit and win if you can | Foster p. 343 |
| `last_eight_choice` | Last eight tricks, any other choice (the engine solves the position exactly) | Count the trumps and bezique cards out and choose the card that wins what is left | **Interpretation**; Cady p. 23 says only that this play matters most |
| `last_eight_lead_ace` | Last eight tricks, leading a plain-suit ace | Lead a high plain-suit card: it wins, or draws a trump | Cady pp. 23, 29 |
| `draw_out_trump` | Last eight tricks, leading a trump while holding three or more | With three or more trumps, lead your highest to draw your opponent's | **Interpretation**; Cady p. 23 differs (lowest trump when reduced to trumps) |
| `generic` | Anything else | (instruction only) | |

Parameters by key: cards `card` (the one to play) in every play, `led` for a
following play, `spared` for `throw_lowest` and `lead_low_nonbrisque`, `kept`
(and `partner`, `points` for marriages) for the `keep_*` keys, `turn_up` for
`exchange_dix`, `trumps` for `draw_out_trump`; declarations carry `meld`
(`royal_marriage`, `marriage`, `sequence`, `bezique`, `double_bezique`,
`four_aces` ...), `points` and, for marriages, `suit`.

## Evidence and limits

`crates/bezique-core/tests/hints.rs` plays whole hands with hints on over
varied play: every hint is classified into a listed key, none panics, and the
hinted command is always accepted by the engine. In the 10-seed run about 1 in
75 hints is `generic`; the most frequent keys are `lead_low_nonbrisque`,
`throw_lowest`, `last_eight_choice` and `save_seven_for_dix`. The node test
`web3d/test/hinttext.test.js` checks that the client's keys equal the engine's,
that every wording is complete and short, and that unknown reasons fall back to
the engine's text.

The classification is of the choice, not of the reasoning inside the rung: the
reason shown is the nearest honest account, and may not be the whole of why the
rung preferred a card. `save_seven_for_dix` is the most common reason because
a seven of trumps held is often the cheapest alternative to the card chosen.
