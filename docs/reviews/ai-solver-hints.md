# Review: agents, solver, lookahead, hints

Scope: `crates/bezique-core/src/{agents,solver,lookahead,opponent,observation,hints,race,rng}.rs`.
Read-only, static reading; nothing was run. `tests/hints.rs` and
`tests/hidden_swap.rs` were not read (see Unverified).

## Findings, ranked

### 1. Bug: `draw_out_trump` does not check the card is the highest trump (hints.rs:298)
The condition is only `card.suit() == trump && trumps >= 3`. The rule shown is
"lead your highest". The solver may lead a lower trump (for example the 9 while
holding the ace) to keep the ace for a later trick. The hint then says "lead your
highest" while naming a lower card.
Fix: require `card` to be the highest trump held (`rank` max), else fall through
to `solved_endgame` / `last_eight_lead_low`.

### 2. Bug: `keep_*` can name a card that is part of the card being played (hints.rs:389-395, `work` at 110-167)
The spared-card search takes any other legal card `c` with `c.rank() <= card.rank()`
and `work(c)`. Nothing excludes `c` being the partner of the card played. Scenarios:
- Play K of a suit while holding its Q, both unmarried: `keep_marriage kept=Q partner=K card=K` (keep the pair while playing half of it).
- Play A while holding three aces: `keep_four kept=A(other suit)`, the played ace being one of the three.
- Play Q spades while holding J diamonds: `keep_for_bezique kept=JD`, card=QS.
- Same for `keep_sequence` with another trump of the five.
Rare (the rung rarely spends a combination piece) but the sentence is then false.
Fix: pass `card` into `work` and skip when `card` is the same combination's
piece (partner rank/suit, same four rank, bezique pair, sequence ranks of trumps).

### 3. Risk: `let_brisque_go` claim is not tested (hints.rs:381-386)
The rule says "when it costs more than 10 points". The code checks only that a
winner exists and the card is not one. Rung 5 (lookahead) can decline a brisque
that the cheapest winner would take cheaply (for example a low trump or a plain
king), so the hint tells the player not to spend a strong card when the cheapest
winner is not strong.
Fix: require the cheapest winner to be a trump, ace/ten or `work` card
(`valuable`), else `generic`.

### 4. Risk: `throw_lowest` / `lead_low_nonbrisque` can name a combination piece as "least useful" (hints.rs:399-417)
`cost` is (trump, brisque, rank) only. When every other card is a trump, ace or
ten, the cheapest by cost can be a Q of an unmarried K/Q pair, Q spades, or a
trump seven (the last is caught earlier by `keep_`; the others only when a
lower-rank piece is held). `nothing_decisive` guards with `!valuable(card)`; these two do not.
Fix: add `work(view, card).is_none()` to both branches, else `generic`.

### 5. Risk: availability counts ignore the opponent's table (hints.rs:112, agents.rs:337, 367, 387, 399)
`available(t) = 2 - played`. Copies lying on the opponent's table are not
"played" yet can never reach the viewer's hand. So `keep_for_bezique`,
`keep_sequence`, `keep_four` can assert "can still come" when every other copy is
on the opponent's table. Same weakness in the agents' `keep_value` (a play-quality
leak, not an information leak). Only unseen cards plus own cards can still be had.
Fix: available = copies in own hand/table + copies in `view.unseen()`.

### 6. Risk: `last_eight_lead_master` for `kind=plain` is not "nothing can beat it" (hints.rs:303-317)
The test is on higher cards of the suit only. A plain master can be trumped if
the opponent is void in the suit and holds a trump. The key's rule text
("if nothing can [beat it], lead it") is false for plain unless the wording
(hinttext.js, not read) says "only a trump can". Check wording; or test the
opponent's trump/void status (known exactly in the last eight).

### 7. Nit: doc/code mismatch on the `keep_*` row (docs/HINTS.md "A lower card is played")
Code keeps a card of rank <= the played card, so the card played is the higher or equal one.
Fix the table wording.

### 8. Nit: `take_to_declare` counts `carried()` (hints.rs:333-340)
"a combination to declare" is also true of a carried score; fine, but the
`points` param then names the carried score. Wording should not say "declare".

### 9. Nit: `solved_endgame` is vacuously true for rung 4/5 (hints.rs:247-255)
At the top rung the endgame choice is always the solver's, so the key only
distinguishes "no simpler case fit". The claim is true; the solver breaks ties
by bit order, so "the exact solution is the card chosen" holds but the card may
be one of several equal-valued ones.

## Checked and clean

- Information: `View::of` exposes only seen opponent cards; relabelling of twins
  is consistent (lone visible twin = copy 0, order by public position); the test
  `no_view_names_a_card_its_player_could_not_know` covers the invariants.
  `hints::classify`, `agents::solve`, `lookahead::world_with` and `fingerprint`
  read `View` only; none reads the stock order or the opponent's concealed cards.
- Sampling: `world_with` shuffles the multiset of unseen individual cards and
  splits into opponent hand / stock: uniform over arrangements, so twin types
  are weighted correctly. Same worlds for every candidate (common random
  numbers). Seed is a function of the view: reproducible, and a seat swap gives
  a different but fixed seed. Sampling ignores inference from the opponent's play
  (accepted bias, not a bug).
- Solver: alpha-beta with fail-soft bounds, stored bound merge, and root window
  `value-1` are sound; key `canonical()` encodes count 0/1/2 per type
  injectively (twins are adjacent bits) plus led type; follow-and-win comes from
  `tricks::endgame_replies` (all-or-none winning, as `last_eight_*` assumes);
  tests compare against unpruned minimax with and without a table, and a reused
  solver. Values are within +-170; `INFINITY` 1000, no overflow.
- Rng: xoshiro256** pinned to the reference; Lemire `below` correct; Fisher-Yates
  correct; streams per purpose. `Erratic` consumes one draw per decision,
  `replay` preserves the stream. Mirrored-pair harness in `agents.rs` tests is right.
- Race: `outcomes` handles totals at or past 1,000, double below 500; fixed
  paths (common random numbers). No panics found on legal states.
- Performance: lookahead is worlds(16) x candidates x rung-4 rollouts to the
  endgame plus one solver call; solver table shared across worlds. No cliff seen.

## Unverified

- `tests/hints.rs`, `tests/hidden_swap.rs`: not read; nothing said about their coverage.
- `web3d/src/hinttext.js` wording (affects findings 1, 3, 6).
- `Hand`, `Holdings::level/candidates/carried`, `known()` internals were taken as
  specified, not audited.
- `game_value_of` (rung 6) read only for structure; its race-feature path is not checked.
- Nothing was executed; findings 1 and 2 are from reading, with no reproducing test written.
