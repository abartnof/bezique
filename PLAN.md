# Bezique: plan

> Living document: status, decisions, milestones, and what comes next. It is
> updated as each milestone lands, so that work can resume from here. The
> reasoning behind the decisions is in `docs/DESIGN.md`; the rules are in
> `docs/RULES.md`.

## Where we are

**2026-10-01: milestones 1–5 done: the rules engine is complete.** The pack,
the generator, the deal, the trick rules, declaring, a whole hand and a whole
game, in 95 tests that run in about a second. Planning was completed the same
day:

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
| Repository and licence | **Public** at `github.com/abartnof/bezique`; **MIT**, revisable later (2026-10-01) | `LICENSE` |
| Unit of play | **One game to 1,000**, doubled if the loser is under 500. Foster's partie (best of five) is possible later as a longer format | `RULES.md` §7 |
| Generator | xoshiro256\*\* seeded by SplitMix64, pinned to the authors' C (`tools/prng-reference/`) | `crates/bezique-core/src/rng.rs` |

## Milestones

- [x] **0. Planning**: literature, sources, rules, design, plan.
- [x] **1. The pack**: card types and physical cards, bitboards, codes, the
      seeded generator and shuffle, the deal (3-2-3) and the turn-up. A
      statistical test of the deal and draw against the exact odds, shown
      to catch an unshuffled pack and a naive shuffle.
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
- [ ] **6. Observation and the first rungs**: the view; rungs 1–2; the
      mirrored-pair harness and the measurement binaries.
- [ ] **7. Playable**: the session (undo, replay, aids, hints), the protocol
      and its document, the wasm crate, the terminal client. Play-test the
      declaring interface here, before the 3D table is built around it.
- [ ] **8. Rungs 3–5**: memory, watching the table, the exact endgame.
- [ ] **9. The 3D table**, adapted from piquet's (`DESIGN.md` §11).
- [ ] **10. Rungs 6–7, and the finish**: search, the game-level objective,
      hints, the tutorial, celebrations.

## Next action

**Milestone 6: observation and the first rungs.**

- `observation.rs`, test first: a seat's view of a hand (`DESIGN.md` §8).
  Its own concealed cards; both players' table cards with their histories
  and carried scores; the turn-up and the stock's size; cards known to be in
  the opponent's hand (taken by the dix, shown as the second seven, the
  turn-up taken at the last stock trick); every card played. Each field is
  asked *when was this shown or said, and to whom?*, and a test asserts that
  no view ever holds a card its seat could not know.
- Agents read only views. Rung 1 (legal, lowest lead, declare the most) and
  rung 2 (the period maxims, `DESIGN.md` §10.1).
- The mirrored-pair harness and a first measurement binary, then the first
  questions from "To measure" that rung 2 can answer: hands per game, where
  the points come from, the first lead.

## To measure, once the engine plays

Questions the sources raise or leave open. Each is settled by mirrored
measurement at the smallest size that answers it.

- Is the first lead an advantage or a burden, in a hand and in a game?
  Foster says the lead is a disadvantage unless one can declare; the
  literature review's crude bot found no edge at game level (50.2%).
- Mean hands per game; how often a game is doubled; how often it ends
  mid-hand.
- What the carried-score mechanic is worth against the simpler
  one-meld-per-trick rule.
- What winning the last stock trick is worth.
- Where the points come from (the crude bot: melds 60%, brisques 36%).
- Whether declaring at once or holding back to hide the hand matters.

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
