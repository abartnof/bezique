<!--
The tutorial's pages, in the order of PAGE_KEYS (web3d/src/tutor.js): table,
first-trick, brisques, declaring, to-score, marriage, bezique, dix, fours,
sequence, last-draw, last-eight, double, game-end. Each is shown once, at the
first moment its element appears, and is the minimum needed to play it. Every
claim is a clause of docs/RULES.md (Foster 1897, with Cady 1896 where Foster is
silent): table 1, 2, 7; first-trick 3; brisques 3; declaring 4, 5; to-score 4
(rules 1 and 4, carrying a score); marriage 4; bezique 4; dix 2, 4; fours 4;
sequence 4; last-draw 5; last-eight 6; double 7; game-end 6, 7.
Text before the first heading is ignored by the parser.
-->

# The table and the goal

Bezique is played with two packs of 32 cards shuffled together, so every card appears twice. In each suit the cards rank **A, 10, K, Q, J, 9, 8, 7**: the ten stands second.

You have eight cards. The card turned up beside the stock names the **trumps**. The first to reach **1,000** points wins.

# The first trick

The player who is not dealing leads. Tap a card to play it; the other player answers with any card. While the stock lasts there is no need to follow suit.

The higher card of the suit led wins, and so does a trump played to a plain-suit lead. If the two cards are identical, the leader wins.

Under **Menu**, top right, are Undo, Hint, Help (these pages again), New game and Settings. **Hint** names a good card and the rule behind it. **Cards out**, under the score at the top left, lists what has been played.

# Brisques

Aces and tens are **brisques**. Each is worth **10**, scored by whoever wins the trick that holds it: 20 for a trick with two.

# Your turn to declare

Winning a trick lets you declare before you draw. You are shown the choices, and the first is always *Nothing: draw*.

- You score **one** declaration for each trick you win.
- Cards you declare lie face up on the table. They are still yours, and you may play them whenever you like.
- Drawing without declaring gives up the chance until your next trick won.

Then the winner draws from the stock, and the other player draws after.

# "And N to score"

You may announce several declarations at once, but one scores per trick. "Forty for bezique, and twenty to score" scores the 40 now and carries the 20.

A carried score is taken when you win a later trick, as long as all its cards are still on the table. Play one of them first and it is lost. Each announcement must put at least one new card from your hand on the table.

# Marriage

A king and queen of one suit, laid together, make a **marriage**: **20**. In trumps it is a **royal marriage**: **40**.

# Bezique

The queen of spades and the jack of diamonds make **bezique**: **40**.

Two queens of spades and two jacks of diamonds, all four on the table together, make **double bezique**: **500**. Declared whole it scores 500 only, so score a single bezique first and add the other pair later.

# The seven of trumps

The seven of trumps is the **dix**. After winning a trick, before you draw, you may swap it for the card turned up: **10**. If the turned-up card is already a seven of trumps, showing your other seven scores 10 as well.

You cannot score a dix and declare in the same turn. A dealer who turns up a seven scores 10 at once.

# Fours

Four of a kind, in any suits, score: four aces **100**, four kings **80**, four queens **60**, four jacks **40**. Four tens score nothing.

# The sequence

Ace, ten, king, queen and jack of trumps make the **sequence**: **250**.

A royal marriage already declared may grow into it by adding the ace, ten and jack. The other way about is not allowed: once you hold the sequence, it cannot yield a marriage.

# The last of the stock

One face-down card is left beside the turned-up card, so the next trick is the last with a stock.

Its winner may still declare, then takes the last face-down card, and the other player takes the turned-up card. After that no one declares again, and every score still carried is lost.

# The last eight tricks

The stock is gone. Every card on the table goes back into its owner's hand. The winner of the last stock trick leads.

Now the player who answers must **follow suit and win the trick if able**. With no card of the suit, play a trump if you have one. If you hold the suit but cannot beat the card led, any card of that suit will do. With neither suit nor trump, play anything.

Brisques still score, and whoever wins the very last trick scores **10**.

# The double game

A player has now reached **500**. A game won while the loser is still short of 500 is a **double game**, which counts for two. So the lead is worth keeping.

# The game is over

The first player to reach **1,000** wins at that moment, even in the middle of a hand. If the loser had not reached 500, it is a double game.
