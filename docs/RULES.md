# Bezique: the rules as implemented

> Status: **adopted**, 2026-10-01. Foster as the authority and one rule set
> without switches were confirmed by the maintainer on that date. The engine is
> built to this document, and any change to it is a change to the tests first.

This is the two-player game with two packs: the "ordinary game" of the
period books. Each rule cites its source by the IDs of the literature review
(`docs/LITERATURE.md` §2). Page numbers for Foster refer to the text kept in
`docs/sources/foster-1897-bezique.txt`.

## Authority

**Foster (1897) [S47] is the rule authority for the ordinary game.** Where
Foster is silent, Cady (1896) [S4] decides; where both are silent, the
choice is made here and labelled an *interpretation*.

The reasons, in order of weight:

1. **By Foster's day the rules had settled.** He records the disputes of
   1868–69 (which trick is "the last trick", whether the highest or lowest
   cut deals, whether trumps count before the stock is gone, whether a double
   marriage equals a double bezique) and says that "time and experience have
   finally settled all these points, and the rules of the game are now
   practically uniform in all countries" (p. 338). The literature review
   reaches the same conclusion independently: the two-player game is stable
   only from the 1890s (§0, §1 item 7).
2. **He is systematic where others are loose.** His treatment of
   declarations (pp. 340–343) states the general rules and then works
   examples, and those examples become this project's test fixtures.
3. **Pagat's bezique rules are a third-party archive with visible slips.**
   pagat.com was the authority for this project's predecessor (piquet), but
   its bezique rules page is Howard Fosdick's write-up with John McLeod's
   corrections [S2]. It says both "may declare one or more melds" and "you
   can declare only one meld" two paragraphs apart, and requires following
   suit "for these last six tricks", a remnant of the six-card Marjolet
   section it shares a page with; bezique has eight. Pagat remains the modern
   cross-check, and every point where this document departs from it is
   listed in the decisions table below.
4. **Foster is a period primary source**, which this project prefers for
   depth and for the reasoning behind rules.

**One rule set, no switches.** Each contested point is settled here once, and
only that rule is implemented. Variants with genuinely different games behind
them (Rubicon, six-pack "Chinese" bezique, Polish bezique) are candidates
for later, as separate games rather than settings.

## The game

### 1. Cards

- Two piquet packs of 32 cards, shuffled together: 64 cards, every card
  present twice. Both packs should have the same back (p. 338).
- Rank in each suit, high to low: **A, 10, K, Q, J, 9, 8, 7**, in cutting and
  in play (p. 338). The ten ranks second, above the king.
- Two cards of the same suit and rank are identical in play: played to one
  trick, the leader's wins (p. 340). In declaring, each copy keeps its own
  history (rule 3 below), and a player cannot tell an opponent's twins apart.

### 2. Cutting and dealing

- The players cut; the higher card has the choice of seat and decides who
  deals first (p. 339). Equal cards cut again (*interpretation*: Foster
  does not say). The players then deal alternately (Cady, p. 15 of the
  treatise; [S4]).
- The dealer gives eight cards to each player, three, then two, then three,
  beginning with the non-dealer (the *pone*), and turns the seventeenth card
  face up beside the stock: its suit is **trumps** for the hand. If it is a
  seven, the dealer scores **10** at once (p. 339).
- The rest of the pack is the **stock** (talon), face down. Either player may
  count it (the English laws, p. 343).

### 3. The play of the tricks while the stock lasts

- The pone leads to the first trick. The other player may play **any** card:
  there is no obligation to follow suit or to trump (pp. 339–340).
- The trick goes to the higher card of the suit led, or to a trump played to
  a plain-suit lead. **If identical cards are played, the leader wins**
  (p. 340).
- The winner turns the trick face down. **A trick once turned and quitted
  cannot again be seen**, and players may not count the tricks they have won
  (p. 343).
- **Brisques** (aces and tens) score **10 each** to the winner of the trick
  that contains them, **as soon as the trick is taken**: 20 if the trick holds
  two (p. 340).

### 4. Declaring, after winning a trick

The winner of a trick may declare before drawing. Drawing without declaring
waives the right until the next trick won (p. 340).

**The combinations** fall into three classes (p. 341):

| Class | Combination | Cards | Score |
|---|---|---|---|
| A | Common marriage | K and Q of a plain suit | 20 |
| A | Royal marriage | K and Q of trumps | 40 |
| A | Sequence | A, 10, K, Q, J of trumps | 250 |
| B | Bezique | Q♠ and J♦ | 40 |
| B | Double bezique | both Q♠ and both J♦ | 500 |
| C | Four aces | any four aces | 100 |
| C | Four kings | any four kings | 80 |
| C | Four queens | any four queens | 60 |
| C | Four jacks | any four jacks | 40 |

- A four may be "all of different suits, or any two of them may be of the
  same suit" (p. 341). Four tens score nothing. There is no plain-suit
  sequence.
- **Declared cards are laid face up on the table.** They are still part of the
  hand and may be led or played at any time, but they do not go back into the
  hand until the stock is exhausted (p. 340).

**The four rules of declaring** (pp. 341–343):

1. **One score per trick won.** Only one combination can be *scored* at a
   time, and a trick must be won for each. A player may nonetheless
   *announce* several at once, scoring one and carrying the rest "to score":
   "Forty for bezique, and twenty to score."
2. **No inferior declaration within a class.** Cards that form part of a
   higher combination already declared cannot make a lower one of the same
   class: a sequence cannot afterwards yield a royal marriage, not even by
   adding a new queen to its king. The reverse is allowed: a royal marriage
   may later be extended to the sequence by adding A, 10, J.
3. **No repetition within a class.** Cards once used in a combination cannot
   be used again in one of equal value in the same class: after four kings,
   one of them played, a new king does not make four kings again; after a
   spade marriage, the king played, a new K♠ does not remarry the old Q♠;
   after a bezique, the jack played, a new J♦ does not make another bezique
   with the old Q♠.
4. **At least one new card.** Every announcement must add at least one card
   from the hand to the table, "even if it is not scored until later"
   (p. 342). Foster states this for announcements in the same class; Cavendish
   (1870) [S55] and the Danish booklet [S62] state it generally, and every
   example in Foster and Cady satisfies it. It is implemented generally.

Cards may serve in combinations of **different classes** freely: a Q♠ may be
in a marriage, a bezique, a double bezique and four queens.

**Carrying a score.** A combination announced but not yet scored is scored
later, one per trick won, **provided all its cards are still on the table**.
It is lost if any of its cards is played first, or used in a higher
combination of the same class first, e.g. a royal marriage "to score" is lost
by adding A, 10, J to make the sequence (Cady, p. 14 [S4]). All carried scores
are lost when the stock is exhausted (p. 343). A player carrying a score is not
obliged to score it before something else (p. 343).

**Double bezique** must have all four cards on the table at once (Cady, p. 20,
in his Rubicon table, which by p. 15 governs the ordinary game "in everything
else"; Foster, pp. 341–343, does not say so in terms). Declared as a whole it
scores 500 only. The good order is a single bezique (40), then
the other pair (500 more). A second single bezique *can* be scored for 40, but
the double is then out of reach, since it would add no new card (p. 342).

**The seven of trumps (the dix).** After winning a trick and before drawing, a
player holding a seven of trumps may exchange it for the turn-up card, scoring
**10**. If the turn-up is already a seven of trumps (turned up, or put there
by an earlier exchange), showing the other seven scores 10 in the same way. A
player cannot score a seven and declare in the same turn (p. 340). A seven of
trumps scores nothing for being played.

### 5. Drawing

- After any declaration, the winner of the trick draws the top card of the
  stock and the loser the next, so each again holds eight (p. 340).
- **The last stock trick.** When one face-down card and the turn-up remain, the
  winner of the trick may still declare, then takes the last face-down card,
  and the loser takes the turn-up (or the seven exchanged for it). All
  declaring then ends and all carried scores are lost (p. 343; Cady, p. 15).

### 6. The last eight tricks

- Each player takes back into the hand every card still on the table. The
  winner of the last stock trick leads (p. 343).
- The second player **must follow suit and must win the trick if able**: with
  a higher card of the suit led; or, holding none of that suit, with a trump.
  Holding the suit but unable to beat it, any card of the suit may be played;
  holding neither suit nor trump, any card (p. 343).
- Brisques still score as taken. **The winner of the last trick of all scores
  10** (p. 343).

### 7. The game

- **Game is 1,000 points** (p. 339). **The first player to reach 1,000 wins at
  that moment**, mid-hand if need be (*interpretation*, below).
- **A double game** if the loser has not reached 500 (p. 339; Cady).

## Decisions on contested points

The rows of the literature review's conflict matrix (§6), each settled once.

| # | Point | Decision | Source | Note |
|---|---|---|---|---|
| 1 | Endgame duties | Follow suit and win if able; void, trump if able | Foster p. 343 | Pagat agrees, though it says "six tricks" for eight |
| 2 | Declare before or after drawing | Before; drawing waives it | Foster p. 340 | Unanimous in period sources; after-drawing is a pinochle rule |
| 3 | The dix | Exchange or show after a trick won, before drawing, for 10; not in the same turn as a declaration; nothing for playing it | Foster p. 340 | Pagat also scores a seven when played |
| 4 | Melds per trick | Several may be announced; one scores per trick; the rest are carried | Foster pp. 341–343, Cady | Pagat: one meld per trick |
| 5 | Fours | Any suits, duplicates allowed | Foster p. 341 | Panaché (one of each suit) not implemented |
| 6 | Plain-suit sequence | None | Foster | 150 in Rubicon only |
| 7 | The bezique pair | Always Q♠ and J♦ | Foster | Cavendish and Pardon change it with trumps; not implemented |
| 8 | Brisques | Scored as taken | Foster pp. 340, 345 | Pagat counts them after the hand; Foster calls that practice "not to be recommended" |
| 9 | "The last trick" | The thirty-second trick, 10 | Foster p. 343 | Cavendish's first guide used the last trick before the stock ran out |
| 10 | Game | 1,000 up; double if the loser is under 500 | Foster p. 339 | |
| 11 | The ten in the endgame | Keeps its rank | Foster | |
| 12 | Dealer | Higher cut chooses; deal then alternates | Foster p. 339; Cady | Lowest-deals was early (1860s) practice |
| 13 | Penalties | None: the engine refuses illegal moves | | Foster's irregularities (p. 344) cannot arise |
| 14 | Rubicon settlement | Not applicable | | Rubicon is a different game |
| 15 | Deal packets | 3-2-3 | Foster p. 339 | Cosmetic: shapes the deal animation |
| 16 | Re-use of declared cards | Foster's three classes and the four rules above | Foster pp. 341–342 | The US 1864 bar on re-using bezique cards is not implemented |
| 17 | Who draws first | The winner of the trick | Foster p. 340 | France: the pone always draws first |
| 18 | Hand and packs | Eight cards, two packs | Foster p. 339 | |
| 19 | Who wins a trick | Higher card of the suit led, or a trump | Foster p. 340 | "Any higher card" (N&Q 1866) not implemented |
| 20 | Two single beziques, then a double | The second single may score 40, but the double then needs a new card it cannot have | Foster p. 342 | Cady allows 40 + 40 + 500 |

### Interpretations

Where no source settles a point, the choice is made here and labelled.

- **The game ends the moment a player reaches 1,000.** Foster gives the target
  but not the moment. The race reading follows from the period texts that do
  speak to it: the 1860s American texts and the French manuals say a player
  who reaches game by declarations wins even if the other leads on brisques
  [S45, S49, S50]; Foster suggests pegging the score on a cribbage board,
  "going twice round to the game hole" (p. 339), and a cribbage game ends
  when a peg reaches the hole; and Cady's double game is reached by scoring
  1,000 "prior to his opponent obtaining 500". Because every score belongs to
  one player at a time, the race cannot tie.
- **Equal cuts are cut again.** Foster does not say; it is universal practice.
- **A carried score is lost if one of its cards enters a higher combination
  of the same class before it is scored.** Stated by Cady for the royal
  marriage extended to a sequence and the single bezique extended to a double;
  generalised here to the same situation in any class.
- **Announcing marks the cards.** For rules 2 and 3, a combination counts as
  declared when it is announced, scored or not. Foster says as much for the
  new-card rule ("even if it is not scored until later"), and the two cannot
  diverge without contradiction.
- **A carried combination may be scored in a turn that announces new
  ones.** Foster lets a player "make two or more announcements at the same
  time" and score one of them (p. 341), and says a player carrying a score
  "is not obliged to score it if he has anything else to announce" (p. 343);
  neither forbids the reverse, scoring the old while announcing the new, and
  the engine allows it.
- **The second seven of trumps stays in the hand.** Showing it scores 10; it is
  not laid on the table (the 1892 *American Hoyle* [S58]: "only shown").

## Worked examples

From Foster (pp. 341–343) and Cady (pp. 14–15), to be encoded as test fixtures
before the declaration code is written. Hearts are trumps unless stated.

1. **Foster, the 600 that cannot be had.** Holding double bezique, four queens
   and a royal marriage, a player cannot lay everything down and claim 600, nor
   four queens and two jacks for 560, nor four queens and a king for 100. He
   should declare a single bezique first (40) and later the other pair (500):
   declaring the double first forfeits the single.
2. **Foster, forty and twenty to score.** Four kings declared, one of them the
   K♠. On winning a trick the player shows bezique: "Forty for bezique, and
   twenty to score" (the spade marriage), and repeats "twenty to score" until
   he wins a trick to score it.
3. **Foster, the carried score set aside.** With twenty to score, a player who
   picks up the trump sequence declares "Royal marriage, forty, and twenty to
   score"; next trick, "Two-fifty in trumps, and twenty to score."
4. **Foster, kings then queens.** Eighty kings, then sixty queens: the queens
   are scored and the marriages they make with the kings on the table are
   announced to score.
5. **Foster, double bezique with marriages.** Four kings announced, two played,
   leaving both K♠ on the table: double bezique scores 500, and the two spade
   marriages are claimed to score.
6. **Foster, inferior declarations barred.** After the sequence, no royal
   marriage, not even with a new queen beside its king.
7. **Foster, repetitions barred.** Four kings, one played, plus a new king: not
   four kings again. Spade marriage, king played, new K♠ with the old Q♠: no
   marriage. Bezique, jack played, new J♦ with the old Q♠: no bezique.
8. **Foster, the second single.** A second single bezique may be scored for 40,
   but double bezique is then impossible.
9. **Cady, three marriages to score.** Four kings declared, one played to win a
   trick; four queens declared, three matching the suits of the kings still on
   the table, one of those kings the king of trumps: "Sixty and forty, and
   twenty, and twenty to score." Next trick: "Score forty, and twenty, and
   twenty to score."
10. **Cady, carried scores lost upward.** A royal marriage to score is lost if
    A, 10, J are added for the sequence; a single bezique to score is lost if
    the other pair is added for the double.

## Not implemented

- **Irregularities and penalties** (misdeals, revokes, drawing out of turn,
  wrong announcements): the engine offers only legal moves.
- **Stakes, and the partie of five games** (best of five, a double counting
  two; Foster p. 339, Cady): possible later as a longer format.
- **Variants:** no turn-up (first marriage makes trumps), Panaché, the
  trump-dependent bezique pair, brisques counted at the end, three- and
  four-handed play, Polish, Rubicon, six- and eight-pack games. See the
  literature review §5 for all of them.
