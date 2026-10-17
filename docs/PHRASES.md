# What the table says

The opponent's phrase bank, by game moment. Generated from `tools/phrases.py` into `web3d/words.json`;
edit the bank, not this file. `python3 tools/phrases.py --check` fails if either is stale.

## Design

- **Voice.** First person, plain, brief and kind: a card-room companion who explains as it plays. Luck, speed, the score and the run of the cards are fair game; looks, intelligence, age, gender, class, religion and nationality never are (`web3d/test/words.test.js` checks a word list).
- **What the sources give.** Foster (1897) and Cady (1896) record the declaring formulae ("Forty for bezique, and twenty to score", "Royal Marriage, forty", "Two-fifty in trumps") and the rules of the game, but no banter. So the declaring is said in the period formulae (the engine's own wording, `words.rs`), and the remarks are paraphrases of the rules, each with its page, so that they teach as they speak. 25 of 106 lines are the table's own interpretation (source T): luck, pace, congratulation and consolation, kept few and plain.
- **Modes** (pref `talk`, URL `?talk=`): `quiet` says only calls (the cut, the deal, the declaring formulae, the end of the game, undo); `normal` (the default) adds a remark at most moments with probability 0.6 and the play-by-play at 0.12; `chatty` says every remark and the play-by-play at 0.45.
- **Kinds.** A call carries the game and is always said. A remark follows an event. Chatter is a comment on a single card played; it yields to calls and is dropped if late.
- **Personas.** The same bank, weighted: a weight multiplies a remark's probability. `concentrates` remarks on small cards, trumps and the stock; `close` is reserved about brisques and marriages and readier to speak of pace and the score; `bezique` dwells on bezique, fours and the seven of trumps. Calls are never weighted.
- **Repeats.** Each group deals its lines from a shuffled bag (`kit/js/bag.js`): none comes round again before the rest, and never the same twice running. One line at a time: lines wait their turn on the table's clock and appear in a bubble by the opponent's hand, for a time scaled by the game speed and the text's length.
- **Slots.** `{formula}` is the engine's declaring formula; `{points}` a trick's points; `{gap}` the points between the scores.

## Sources

- **F97**: Foster, Foster's Complete Hoyle (1897), Bezique, pp. 337-345 (`docs/sources/foster-1897-bezique.txt`)
- **C96**: Cady, Bezique (1896), the rules of play (no page recorded in the text copy) (`docs/sources/cady-1896-bezique.txt`)
- **WP70**: Westminster Papers (1870), a correspondent calling bezique the slowest of slow games; second hand, docs/LITERATURE.md section 3.4 [S16] (`docs/LITERATURE.md`)
- **T**: Interpretation: table talk and card-room banter of the period are not recorded in the sources read, so these remarks are the table's own, kept few and plain

## Moments

### `start` (call)

the sitting begins.

| Line | Source |
|---|---|
| Good evening. Two packs, sixty-four cards, a thousand up. | F97 p. 339 |
| Ace, ten, king, queen, jack, nine, eight, seven: that is the rank. | F97 p. 338 |
| Cut for the deal: the higher card chooses. | F97 p. 339 |
| Bezique: queen of spades, jack of diamonds, and some patience. | F97 p. 341 |
| A game for two. I shall explain as we go. | T |

### `cut_you_high` (call)

the cut: yours is higher.

| Line | Source |
|---|---|
| You cut higher: the choice of dealing is yours. | F97 p. 339 |
| Your cut is the higher. Will you deal, or leave it to me? | F97 p. 339 |

### `cut_them_high` (call)

the cut: mine is higher.

| Line | Source |
|---|---|
| The higher cut is mine, so I choose to deal. | F97 p. 339 |
| I cut higher: the first deal, then. | F97 p. 339 |

### `deal_them` (call)

your opponent deals.

| Line | Source |
|---|---|
| My deal: eight cards each, the seventeenth turned for trumps. | F97 p. 339 |
| Three, two, three, and the trump turned up. You lead. | F97 pp. 339-340 |
| I deal; you, not dealing, lead any card you like. | F97 p. 339 |

### `deal_you` (call)

you deal.

| Line | Source |
|---|---|
| Your deal: the dealing alternates, and I lead first. | C96 |
| Deal away: three, two, three. I begin. | F97 p. 339 |
| The non-dealer leads, so I shall begin. | F97 pp. 339-340 |

### `seven_them` (remark)

a seven is turned up and the dealer (your opponent) scores ten.

| Line | Source |
|---|---|
| A seven turned up: ten for the dealer, and that is I. | F97 p. 339 |
| Ten for the seven turned up. A fortunate start. | C96 |

### `seven_you` (remark)

a seven is turned up and the dealer (you) scores ten.

| Line | Source |
|---|---|
| A seven turned up: ten to you as dealer. | F97 p. 339 |
| Ten to you for the seven, and trumps are made. Lucky. | C96 |

### `play_them_brisque` (chatter)

your opponent plays an ace or a ten.

| Line | Source |
|---|---|
| A brisque: ten for whoever takes it. | F97 p. 340 |
| Aces and tens are the brisques, ten points apiece. | F97 p. 340 |
| Only a trick with a brisque in it is worth the winning, bar the lead. | F97 p. 340 |

### `play_them_trump` (chatter)

your opponent plays a trump.

| Line | Source |
|---|---|
| A trump: it beats any plain suit. | F97 p. 340 |
| Trumps win plain suits, and I need not follow suit while the stock lasts. | F97 p. 340 |

### `play_them_high` (chatter)

your opponent plays a king or queen.

| Line | Source |
|---|---|
| A high card, to keep the lead. | F97 p. 340 |
| The lead is worth keeping: it carries the right to declare. | F97 p. 340 |

### `play_them_low` (chatter)

your opponent plays a seven, eight, nine or jack.

| Line | Source |
|---|---|
| A small card: nothing here worth winning. | F97 p. 340 |
| A small one. While the stock lasts, nobody need follow suit. | F97 p. 340 |
| Just a seven, to see what you hold. | T |

### `play_you_brisque` (chatter)

you play an ace or a ten.

| Line | Source |
|---|---|
| A brisque on the table: ten to whoever wins it. | F97 p. 340 |
| You lead a brisque. I shall give that some thought. | T |

### `play_you_trump` (chatter)

you play a trump.

| Line | Source |
|---|---|
| A trump: that wins unless I hold a higher one. | F97 p. 340 |
| Trumps already? Quite in order while the stock lasts. | F97 p. 340 |

### `play_you_high` (chatter)

you play a king or queen.

| Line | Source |
|---|---|
| A high card: if it holds, the lead stays yours. | F97 p. 340 |
| Playing to keep the lead, I think. | T |

### `play_you_low` (chatter)

you play a small card.

| Line | Source |
|---|---|
| A small card thrown: there is no need to follow suit yet. | F97 p. 340 |
| Prudent: nothing in this one worth giving away. | T |

### `trick_them` (remark)

your opponent takes a trick.

| Line | Source |
|---|---|
| That trick is mine, and with it the right to declare. | F97 p. 340 |
| I take it and draw first: the winner always does. | F97 p. 340 |
| Mine, and the lead with it. | F97 p. 340 |

### `trick_you` (remark)

you take a trick.

| Line | Source |
|---|---|
| Yours: declare before you draw, if you hold anything. | F97 p. 341 |
| That one is yours; the winner draws first. | F97 p. 340 |
| Well taken. | T |

### `trick_rich_them` (remark)

your opponent takes a trick worth twenty or more.

| Line | Source |
|---|---|
| {points} in one trick: brisques are worth the taking. | F97 p. 340 |
| {points} to me there. The cards fell kindly. | T |

### `trick_rich_you` (remark)

you take a trick worth twenty or more.

| Line | Source |
|---|---|
| {points} in that trick to you: brisques tell. | F97 p. 340 |
| {points} to you, and well earned. | T |

### `streak_you` (chatter)

you take three tricks running.

| Line | Source |
|---|---|
| Three tricks running to you: the cards are with you. | T |
| You have the run of the cards just now. | T |

### `streak_them` (chatter)

your opponent takes three tricks running.

| Line | Source |
|---|---|
| The run of the cards is mine for the moment. It will turn. | T |
| Three running: I cannot claim it as skill, only luck. | T |

### `decl_them` (call)

your opponent declares: the formula as the engine words it.

| Line | Source |
|---|---|
| {formula} | F97 pp. 342-343 |

### `decl_them_lucky` (remark)

your opponent declares a sequence, double bezique or four aces.

| Line | Source |
|---|---|
| The cards were kind to me there. | T |
| I could not have asked for a better draw. | T |

### `decl_you_marriage` (remark)

you declare a marriage in a plain suit.

| Line | Source |
|---|---|
| A marriage: king and queen of one suit, twenty. | F97 p. 341 |
| Twenty for the marriage; it would be forty in trumps. | F97 p. 341 |

### `decl_you_royal` (remark)

you declare the royal marriage.

| Line | Source |
|---|---|
| The royal marriage: king and queen of trumps, forty. | F97 p. 341 |
| Forty for the trump marriage, and the sequence may follow. | F97 p. 342 |

### `decl_you_sequence` (remark)

you declare the sequence in trumps.

| Line | Source |
|---|---|
| Ace, ten, king, queen, jack of trumps: two-fifty! | F97 p. 341 |
| A sequence in trumps: two hundred and fifty. Splendid. | F97 p. 341 |

### `decl_you_bezique` (remark)

you declare bezique.

| Line | Source |
|---|---|
| Queen of spades and jack of diamonds: forty for bezique. | F97 p. 341 |
| Bezique! A second pair would make it five hundred. | F97 p. 341 |

### `decl_you_double` (remark)

you declare double bezique.

| Line | Source |
|---|---|
| Double bezique: five hundred, the great prize of the game. | F97 p. 341 |
| Both queens and both jacks: five hundred. My congratulations. | F97 p. 341 |

### `decl_you_four` (remark)

you declare four aces, kings, queens or jacks.

| Line | Source |
|---|---|
| Four of a kind: aces a hundred, kings eighty, queens sixty, jacks forty. | F97 p. 341 |
| A fine four. Only one declaration to a trick, remember. | F97 p. 341 |

### `dix_them` (remark)

your opponent exchanges or shows the seven of trumps.

| Line | Source |
|---|---|
| The seven of trumps, ten points, exchanged for the turn-up. | F97 p. 340 |
| Ten for the seven of trumps, once the trick is won. | F97 p. 340 |

### `dix_you` (remark)

you exchange or show the seven of trumps.

| Line | Source |
|---|---|
| Ten for the seven of trumps: a fair exchange. | F97 p. 340 |
| The seven goes for the turn-up, and ten to you. Well spotted. | F97 p. 340 |

### `stock_low` (remark)

the stock is running low.

| Line | Source |
|---|---|
| The stock runs low; only a few draws are left. | F97 p. 343 |
| Count the stock if you wish: a few tricks yet before it is spent. | F97 p. 343 |

### `stock_out` (call)

the stock is exhausted.

| Line | Source |
|---|---|
| The stock is spent: the cards laid down go back into the hands. | F97 p. 343 |
| No more declarations now that the stock is gone. | F97 p. 341 |

### `stock_last_you` (remark)

you take the last card of the stock.

| Line | Source |
|---|---|
| You take the last card, and the turn-up comes to me. | F97 p. 343 |

### `stock_last_them` (remark)

your opponent takes the last card of the stock.

| Line | Source |
|---|---|
| I take the last card, and the turn-up is yours. | F97 p. 343 |

### `last_eight` (remark)

the last eight tricks begin.

| Line | Source |
|---|---|
| Eight tricks left, and each must follow suit and win if it can. | F97 p. 343 |
| The last eight tricks: brisques still count ten each. | C96 |
| The last trick of all is worth ten. | F97 p. 343 |

### `last_trick_you` (remark)

you take the last trick of all.

| Line | Source |
|---|---|
| The last trick of all, and ten for it, to you. | F97 p. 343 |

### `last_trick_them` (remark)

your opponent takes the last trick of all.

| Line | Source |
|---|---|
| The last trick of all is mine: ten for it. | F97 p. 343 |

### `hand_ahead_them` (remark)

the hand ends, your opponent ahead in the game.

| Line | Source |
|---|---|
| That is the hand. I lead by {gap}, but a thousand is a long road. | C96 |
| The cards have run my way this hand, and that is all it is. | T |

### `hand_ahead_you` (remark)

the hand ends, you ahead in the game.

| Line | Source |
|---|---|
| That is the hand. You lead by {gap}, and the thousand is near. | C96 |
| The cards are with you; I am trying to turn them. | T |

### `hand_level` (remark)

the hand ends, the game level.

| Line | Source |
|---|---|
| Level at the end of the hand. A thousand is the game. | C96 |
| Nothing between us yet. | T |

### `pace` (chatter)

a new hand, some way into the game.

| Line | Source |
|---|---|
| They called bezique the slowest of slow games. We shall see. | WP70 |
| No hurry: the cards will wait. | T |

### `double_you_threatened` (remark)

the hand ends, and you could still lose a double game.

| Line | Source |
|---|---|
| Mind the double: a thousand before you reach five hundred counts twice. | F97 p. 339 |
| Five hundred is your safe harbour: reach it and the game cannot count double. | F97 p. 339 |

### `double_them_threatened` (remark)

the hand ends, and your opponent could still lose a double game.

| Line | Source |
|---|---|
| I must reach five hundred, or the game counts double against me. | F97 p. 339 |
| The double is in the air: I need five hundred to be safe. | F97 p. 339 |

### `double_off_you` (remark)

you pass five hundred and the double is off.

| Line | Source |
|---|---|
| Past five hundred: the double game is off the table for you. | F97 p. 339 |

### `double_off_them` (remark)

your opponent passes five hundred and the double is off.

| Line | Source |
|---|---|
| Five hundred reached at last: I cannot be doubled now. | F97 p. 339 |

### `win_you` (call)

you win the game.

| Line | Source |
|---|---|
| A thousand to you: congratulations, well played. | C96 |
| The game is yours. Shall we have another? | C96 |
| The cards ran for you tonight, and you used them well. | T |

### `win_you_double` (call)

you win a double game.

| Line | Source |
|---|---|
| A thousand before I reached five hundred: a double game to you. Well played. | F97 p. 339 |

### `win_them` (call)

your opponent wins the game.

| Line | Source |
|---|---|
| The game falls to me this time; the cards helped. Another? | T |
| A thousand to me, thanks to the run of the cards. Thank you for the game. | T |
| It was close in places. Shall we play again? | T |

### `win_them_double` (call)

your opponent wins a double game.

| Line | Source |
|---|---|
| A thousand while you were short of five hundred: a double game. The cards were unkind to you. | F97 p. 339 |

### `undo` (call)

you take back a move.

| Line | Source |
|---|---|
| Take it back, by all means. | T |
| Of course: try another card. | T |
| Nothing is lost between friends. | T |

