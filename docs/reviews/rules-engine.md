# Review: rules engine (core)

Scope: `crates/bezique-core/src/{cards,deal,melds,tricks,hand,game,words}.rs`,
tests `hand_invariants`, `twin_symmetry`, `deal_odds`, against `docs/RULES.md`.
Read-only; nothing was run. Static reading only.

## Findings

No correctness bug against RULES.md was found. Ranked:

1. **Risk: `Holdings::options` never offers "score a carried combination while
   announcing new ones"** (`melds.rs:446-448`, and the `offer` calls at 422-444
   always score a member of the new set). `check` (`melds.rs:333-352`) and
   RULES.md "Interpretations" allow it. Scenario: 40 carried (royal marriage),
   player lays a fresh bezique; the engine offers only "score bezique, carry
   the marriage", never "score the marriage, carry the bezique". Same action
   space feeds `Hand::legal` (`hand.rs:367-377`), so hints and any AI search
   cannot choose it; a human UI built on `legal()` cannot either. Fix: for each
   offered announcement with melds, also offer it scored on each carried
   combination not covered by a higher meld in it. Not verified whether the web
   page builds announcements outside `legal()`.
2. **Risk: free riders are always announced with their main combination**
   (`melds.rs:420-424`); the only alternative set is `wider`. Declining a rider
   is never offered. Announcing marks cards (rule 3), so a rider can bar a
   later equal-value combination. Not shown to be harmful in play (a rider adds
   a carried score and costs nothing but that bar); unverified for exotic
   cases. Fix if wanted: offer main alone when riders exist.
3. **Nit: cut compares rank, not card** (`game.rs:55`). RULES.md says "equal
   cards cut again"; 10S and 10H recut. Matches Foster's "higher card" reading;
   suggest the doc say "equal ranks".
4. **Nit: precedence** `game.rs:53`: `(...) as u8 % 64` casts before the
   modulo. Correct (max 126 fits u8) but fragile; write `((a + 1 + r) % 64) as u8`.
5. **Nit: wording** `words.rs:56-61`: "Score forty, for the royal marriage, and
   twenty to score" has a stray comma; period form is "Score forty."
6. **Nit: out-of-range ids.** `Suit::letter/name`, `Rank::letter/name` index
   arrays and panic on invalid values. Reachable only if an unchecked value
   passes; `Meld::is_valid` guards `check`, and `Hand::apply` guards `Card`
   ids. Clean today; a debug assert in the accessors would keep it so.

## Test strength

- `hand_invariants.rs:116-132`: endgame check asserts follow-suit only. Win-if-
  able and must-trump are covered in `tricks.rs` (reference test, 20,000
  cases) but not through `Hand::apply` on random hands. Add the same
  reference check on `legal()` in the endgame.
- Random play with uniform choice over `legal()` rarely produces: a dix at the
  last stock trick, a carried score alive at stock exhaustion, a double
  bezique after a scored single. The `melds.rs` reference tests cover the
  declaration logic well, but a hand-level test of "carried scores are lost
  and shown cards return to the hand at exhaustion" was not found in the
  parts read. `hand.rs` tests from line 760 on were not read: unverified.
- `melds.rs` `reference_candidates`/`reference_check` encode the same reading
  of RULES.md as the implementation; they catch implementation slips, not
  misreadings. Foster's worked examples (tests at `melds.rs:766-1058`) are the
  independent check, and they pass by inspection.
- `deal_odds.rs` tests uniformity of what each seat receives, not the 3-2-3
  order or the pone-first packet; `deal.rs:174` covers the order.
- `twin_symmetry.rs`: strong; the `checked > 40_000` floor guards against an
  empty loop.

## Checked and clean (by reading)

- Meld table, classes and levels; four tens score nothing; no plain sequence;
  double bezique needs all four cards; score values.
- Announcing marks cards (`announced` is appended whether scored or not);
  per-copy history; rules 2 and 3 via level comparison (`melds.rs:311-320`);
  rule 4 (a concealed card in every announced meld); no shared card within a
  class in one announcement; cross-class reuse allowed.
- Carried scores: lost on card played, on a higher same-class meld, and on
  stock exhaustion (`take_up` empties `shown`); scoring a carried score cannot
  repeat (`scored` flag, `carried()` filter).
- Dix: only after winning a trick, before drawing, not with a declaration;
  second seven shown and kept; turned seven scores the dealer 10 and cannot be
  re-scored (`sevens_scored` per card); exchange at the last stock trick gives
  the seven to the loser (`Stock::draw` order).
- Trick winner: higher of suit led or trump; identical cards go to the leader;
  ten above king (`Rank` order, `above()` shift incl. the 64-bit edge).
- Brisques scored as taken; last trick 10 on trick 32; both to the same seat.
- Draw: winner first; hands return table cards when the stock empties; the
  winner of the last stock trick leads the last eight; 24 stock tricks.
- Endgame replies: follow and beat, else follow, else trump, else any.
- Deal: 3-2-3 starting with the pone, 17th card turns up, 47 face down.
- Game: 1,000 checked after every action, including a turned seven on the next
  deal; double game uses the loser's total including the current hand; banking
  uses the old seat mapping before the dealer flips; failed actions change
  nothing (check before mutate in `Holdings::apply` and `Hand::apply`).
- No panics found on legal input: the `expect`s in `draw`, `exchange` and
  `take_trick` rest on invariants `Hand::arranged` enforces (even stock, turn-up
  present while face-down cards remain). Overflow: `u32` scores, `u8` seen
  counts capped at 2.
- Determinism: no hash-ordered iteration; `best_riders` ties break by first
  mask. Move order from `legal()` is deterministic but is not stable across
  rule changes, so persisted action indices (if any) would be fragile.

## Not verified

- Undo/replay (not in these files; `Game` keeps no action log).
- `rng.rs`, `observation.rs`, the web layer, and any caller of `options`.
- `hand.rs` tests after line 760 and `melds.rs` greedy rider branch beyond
  its test. Nothing was executed, including `cargo test`.
