# Bezique: plan

> Living document: status, decisions, milestones, and what comes next. It is
> updated as each milestone lands, so that work can resume from here. The
> reasoning behind the decisions is in `docs/DESIGN.md`; the rules are in
> `docs/RULES.md`.

## Where we are

**2026-10-02: a playable game with a measured five-level opponent, reviewed,
ready for its first play-test.** The rules engine (milestones 1–5),
observation and the opponent's rungs 1–5 with personas and erraticism
(milestones 6 and 8), the session, the JSON protocol and the wasm module
(milestone 7) are done; the terminal client runs on the session. The
overnight work has been reviewed and every finding but one design question
resolved. 181 tests, all passing. What remains: the maintainer's play-test,
the question of undo, and the 3D table. Planning was completed on
2026-10-01:

- `docs/LITERATURE.md`: the literature review (68 sources, the conflict
  matrix, strategy, computed odds), as delivered.
- `docs/sources/`: Foster (1897) and Cady (1896) on bezique, as text.
- `docs/RULES.md`: the ordinary two-pack game as it will be implemented, with
  Foster as the authority, every contested point settled once, and ten
  worked examples ready to become fixtures.
- `docs/DESIGN.md`: architecture, method, the card and declaration models,
  observation, the opponent, the table, testing.
- `analysis/bezique_stats.py`: the literature review's exact-odds script,
  checked; its figures are the targets for the deal-and-draw tests.

## Unattended session, night of 2026-10-01

Approved by the maintainer: work autonomously on this VM (no other compute),
commit and push each change, leave design decisions (the 3D table, the
score display) for discussion. The queue, in order; each entry is marked as
it lands, with what was found.

1. ~~Triage the background code review of the engine and fix confirmed
   defects, test first.~~ **Done**: 20 findings, 18 fixed, 2 efficiencies
   deferred (`docs/reviews/2026-10-01-engine.md`). The weightiest: twins could
   tell a player where hidden cards lay (finding 1).
2. ~~The mirrored-pair harness and first measurements.~~ **Done**:
   `tournament.rs`, `src/bin/measure.rs`, `measurements/`. The anatomy of a
   game exposed that rung 2 threw its bezique cards away and would not fight
   for the lead to declare; tuned by mirrored measurement, rung 2 now wins
   88% of games against rung 1 (was 72%).
3. ~~Rungs 3–5.~~ **Done, as rungs 3–4**: an exact endgame solver
   (`solver.rs`); rung 3 = memory with the exact endgame (+19.2 points a hand
   over rung 2); rung 4 = rung 3 fighting for the last stock trick (+1.8 to
   +2.5). Memory alone (+1.9) and watching the table (nothing) did not earn
   rungs of their own; watching was removed. A rung 5 needs search in the
   stock phase (DESIGN §10.4), not built.
4. ~~Difficulty, erraticism and personas.~~ **Done** (`opponent.rs`;
   `measurements/README.md`): levels 1–4; erraticism slips a rung per
   decision from the agent's own stream; four personas (balanced,
   concentrates, close, bezique), each within 1.24 points a hand of
   balanced. Calibration also tuned five more weights into the default.
   (Commit `bc7fae8` also carries the terminal client's `--level`,
   `--persona` and `--erratic` options, not named in its message.)
   Originally: difficulty
   1–5 is the ladder; erraticism draws the effective rung per decision;
   personas are stable styles taken from the period writers' disagreements
   (brisques or combinations; boldness for double bezique; declaring at once
   or holding back; hoarding trumps), each measured and calibrated so that a
   persona is a style, not a handicap. Personas are described by how they
   play, not given a writer's name (piquet's rule for the opponent; to
   confirm with the maintainer).
5. ~~The session layer, the protocol and the wasm crate.~~ **Done**:
   `session.rs` (cut, dealer choice, record and replay, undo, aids, hints,
   events), `crates/bezique-wasm` (JSON state from the person's view; a
   leak test that catches a planted leak; 207 KB module playing a rung-4
   game from Node in half a second), `docs/PROTOCOL.md`.
6. ~~The terminal client: level, persona, erraticism.~~ **Done** (see 4).

Constraints: nothing large deleted, no history rewritten, nothing drastic.
Report due 08:45 Los Angeles time, 2026-10-02.

**Summary of the session** (stopped at about 02:40 Los Angeles time, when the
session's usage limit was reached; everything is committed and pushed):

- The code review's 20 findings: 18 fixed, 2 efficiencies deferred. The
  weightiest, a twin channel that told a player where hidden cards lay, is
  closed and tested (`docs/reviews/2026-10-01-engine.md`).
- Measuring found and fixed two large weaknesses in rung 2 (it threw its
  bezique cards away; it would not fight for the lead to declare) and tuned
  seven weights, each confirmed on fresh seeds. Rung 2 now beats rung 1 in
  92% of games.
- The ladder was rebuilt by measurement: rung 3 is memory with an exact
  endgame solver (+18.7 points a hand), rung 4 fights for the last stock
  trick (+2.3). Two proposed rungs failed and were dropped.
- Four personas, each within 1.24 points a hand of balanced, and
  erraticism. The lesson, recorded in DESIGN §10.1: on a tuned default a
  style must break near-ties, not shift weights.
- The session, the JSON protocol (with a leak test that catches a planted
  leak), a 207 KB wasm module, `docs/PROTOCOL.md`; the terminal client on
  the session, with undo, hints and a trick tracker.
- Two process slips, both recorded: commit `bf275e3` claimed rung 4 before
  the ladder had it (completed by `811f7c5`), and `bc7fae8` carried the
  client's new options without naming them.

## Session of 2026-10-02 (day)

Resumed after the usage limit, with the queue from "Next action": the
play-test waits for the maintainer, so the engine work went ahead.

- **The review of the overnight work** (`docs/reviews/2026-10-02-overnight.md`):
  four reviewers, 31 findings, each reproduced; all resolved but one design
  question. The weightiest: undoing the choice of dealer handed the
  opponent the hand just seen; two twins on the opponent's table told what
  it had held (now checked by a twin-symmetry property over whole hands);
  the state carried the seed; five errors in the narration.
- **The solver** is now alpha-beta with a table of bounds: the same cards
  chosen, about four times faster in play and nine times in search.
- **Rung 5** looks ahead once nine or fewer face-down cards remain: +30.1
  points a hand over rung 4 on fresh seeds (se 1.3), and 62.4% of games in
the full ladder run. Levels run 1 to 5.
- **The personas** hold at rung 4: each within 1.35 points a hand of
  balanced.
- **The "To measure" questions** that the anatomy already answered are
  recorded below.

## Decisions

### Settled

| Decision | Choice | Reasoning |
|---|---|---|
| The game | Two-player bezique with two packs, the "ordinary game" | `RULES.md` |
| Language | **Rust from the first line**; no Python oracle | `DESIGN.md` §4 |
| What replaces the oracle | Brute-force references in the tests, period fixtures, exact statistical targets, invariants | `DESIGN.md` §4, §12 |
| Reuse of piquet | Copy and adapt; no shared crate until a third game | `DESIGN.md` §3 |
| Card model | 64 physical cards in a `u64`, twins interleaved; moves and search deduplicated by type, location and history | `DESIGN.md` §5 |
| Declaration rules | One inequality on per-card class levels, plus the new-card rule | `DESIGN.md` §6.1 |
| Scoring | An ordered event log; totals derived | `DESIGN.md` §7 |
| Opponent | A measured capability ladder, with erraticism and style | `DESIGN.md` §10 |
| Measurement | Mirrored pairs, always; smallest run that can answer | `DESIGN.md` §10.5 |
| Client contract | The page holds no rules; JSON protocol; cards carry ids | `DESIGN.md` §9 |
| Working method | TDD; atomic commits to `main` | `DESIGN.md` §2 |
| Rule authority | **Foster (1897)**, Cady where he is silent; pagat as the modern cross-check (confirmed 2026-10-01) | `RULES.md` "Authority" |
| Variants | **One rule set, no switches**; Rubicon, six-pack and Polish bezique only ever as separate games (confirmed 2026-10-01) | `RULES.md` "Authority" |
| Repository and licence | **Private** at `github.com/abartnof/bezique` (made private 2026-10-01; to be opened later); every commit pushed. **MIT**, revisable later | `LICENSE` |
| Unit of play | **One game to 1,000**, doubled if the loser is under 500. Foster's partie (best of five) is possible later as a longer format | `RULES.md` §7 |
| Generator | xoshiro256\*\* seeded by SplitMix64, pinned to the authors' C (`tools/prng-reference/`) | `crates/bezique-core/src/rng.rs` |

## Milestones

- [x] **0. Planning**: literature, sources, rules, design, plan.
- [x] **1. The pack**: card types and physical cards, bitboards, codes, the
      seeded generator and shuffle, the deal (3-2-3) and the turn-up. A
      statistical test of the deal and draw against the exact odds, shown
      to catch an unshuffled pack. (It was also credited with catching a
      naive shuffle; the review showed its power there is slight. Shuffle
      bias is the permutation test's job, in `rng.rs`.)
- [x] **2. Tricks**: who wins a trick; the follow-and-win rule of the last
      eight tricks, checked against Foster's sentence on random endgames.
- [x] **3. Declarations**: combinations, class levels, the new-card rule,
      announcements and carried scores, the curated options. Every worked
      example in `RULES.md` as a test; references for candidates and for
      `check` on random positions; planted defects used to find untested
      rules (one was found, and covered).
- [x] **4. The hand**: the phase machine end to end (drawing, the dix, the
      last stock trick, the endgame), the score log, invariants over 4,500
      hands of random play. Positions are arranged in code; a text notation
      waits for the terminal client, which needs one (milestone 7).
- [x] **5. The game**: the cut, alternating deals, the race to 1,000, the
      double game.
- [x] **6. Observation and the first rungs**: the view (checked for leaks on
      every position of random games, and by the twin-symmetry property);
      rungs 1–2; the mirrored-pair harness and the measurement binary.
- [x] **7. Playable**: the terminal client (`cargo run --release -p
      bezique-cli`, with `--watch` for two computer players); the session
      (undo, replay, aids, hints), the protocol and its document, the wasm
      crate. The maintainer's play-test of the declaring interface comes
      before the 3D table is built around it.
- [x] **8. Rungs 3–5**: memory with the exact endgame; the last stock trick;
      looking ahead at the end of the stock. Watching the table was tried
      and dropped.
- [ ] **9. The 3D table**, adapted from piquet's (`DESIGN.md` §11).
- [ ] **10. Rungs 6–7, and the finish**: search, the game-level objective,
      hints, the tutorial, celebrations.

## Next action

**Start here next session.**

1. **The maintainer's first play-test** (`PLAYTEST.md`, now with
   `--level 5`), and acting on what it finds. This comes first: the
   declaring interface is the biggest unknown for fun.
2. **Undo** (to decide with the maintainer): undo replays the same stock,
   so it can show the next card drawn or the opponent's answer to a lead.
   Keep it as a documented learning aid (as now), or allow it only where
   nothing new has been revealed since the decision taken back.
3. **The 3D table** (milestone 9): first a conversation about the score
   display in homage to the period markers ("To discuss with the
   maintainer" below), then `web3d` adapted from piquet (DESIGN §11).
4. **Engine work that needs no decision**, when the above wait. Done on
   2026-10-02: the anatomy at rung 5; a record version guarded by a golden
   game; looking ahead in declarations (measured worse, not kept); the
   solver 2.5 times faster again. Still open: rung 6, play for the game
   rather than the hand (the race to 1,000 and the double at 500, DESIGN
   §10.4). It changes only a game's last hand, about one in four, so a gain
   in games won would be small: telling a 1–2% gain from noise needs on
   the order of 10,000 mirrored pairs of games, about three hours on this
   machine. To agree with the maintainer before running.

## To measure, once the engine plays

Questions the sources raise or leave open. Each is settled by mirrored
measurement at the smallest size that answers it. Answered so far, at rung
2 against itself (`measurements/anatomy-rung2-final.txt`, 2,000 games):

- **The first lead**: no measurable edge either way. The pone, who leads
  first, scores 224.2 a hand to the dealer's 222.2, and the first dealer
  wins 50.0% of games (standard error 1.1). Foster's "a disadvantage
  unless one can declare" is not borne out on average.
- **Game length**: 4.25 hands a game; 12.4% of games doubled; 95.2% end in
  the middle of a hand.
- **Where the points come from**: brisques 35.8%, combinations 58.7% (four
  aces alone 19.1%), the dix 2.8%, the last trick 2.2%. The literature
  review's crude bot: melds 60%, brisques 36%.
- **The last stock trick**: fighting for it is worth +1.8 to +2.5 points a
  hand (rung 4 over rung 3); its winner out-scores the loser over the hand
  52.1% of the time.

Still open:

- What the carried-score mechanic is worth against the simpler
  one-meld-per-trick rule (`measure riders` measures free riders only).
- Whether declaring at once or holding back to hide the hand matters.
- (Answered: the same anatomy at rung 5, `anatomy-rung5.txt`. Much the
  same game: 4.19 hands, 13.1% doubled, no seat advantage; stronger play
  makes more of the sequence and the dix, and the last stock trick, now
  fought for by both, is worth less to its winner.)

## To discuss with the maintainer, when the table is built

- **A score display in homage to the period's bezique markers** (requested
  2026-10-01). Tony Hall's survey, "Bezique Markers, 1860–1960" (World of
  Playing Cards, https://www.wopc.co.uk/games/bezique-markers-1860-1960;
  literature review S6), shows dials to 1,000, Goodall's "Register" and
  three-circle markers, and De La Rue's two-circle 0–90 / 0–900 dials. Foster
  adds counters (button 500, blue 100, red 50, white 10) and the pull-up
  cribbage board. The point is to find what those devices *afforded* (the
  score read at a glance, the 500 mark that decides a double, the turning
  dial as the score ticks) and make a digital piece that does the same. D3 is
  an acceptable fallback. This replaces, or reshapes, piquet's broadcast-style
  score bug (`DESIGN.md` §11).

## Ideas, not owed

- An "About bezique" page from the literature review: the 1869 craze,
  Churchill, Caillebotte's *La Partie de bésigue* (1880), the invented
  "Gustave Flaker" and how it was traced.
- Captions in the period formula: "Forty for bezique, and twenty to score."
- Later games: Rubicon (four packs), the six-pack game, Polish bezique.

## Notes for a future session

- Read `DESIGN.md` §5–§8 before touching cards, declarations or observation.
  Observation is where piquet's subtle defects lived.
- `~/piquet` is the working reference for structure: its
  `docs/PROTOCOL.md`, `docs/TABLE3D.md`, and the "Hard-won lessons" in its
  `PLAN.md`.
- `CREDITS.md` arrives with the first third-party asset (the card art, with
  the 3D table). Every asset is credited in the commit that brings it in.
- The development VM has two cores. Keep the fast test loop fast, and when it
  slows, find the one slow test.
