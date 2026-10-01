# Bezique: plan

> Living document: status, decisions, milestones, and what comes next. It is
> updated as each milestone lands, so that work can resume from here. The
> reasoning behind the decisions is in `docs/DESIGN.md`; the rules are in
> `docs/RULES.md`.

## Where we are

**2026-10-01: planning complete; no code yet.**

- `docs/LITERATURE.md`: the literature review (68 sources, the conflict
  matrix, strategy, computed odds), as delivered.
- `docs/sources/`: Foster (1897) and Cady (1896) on bezique, as text.
- `docs/RULES.md`: the ordinary two-pack game as it will be implemented, with
  Foster as the proposed authority, every contested point settled once, and ten
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

### Awaiting the maintainer

1. **Rule authority.** Foster (1897) is proposed in place of pagat, which was
   piquet's authority; `RULES.md` "Authority" gives the case.
2. **One rule set, no switches.** Piquet settled each conflict once rather
   than adding options. The literature review recommends named presets
   instead. Proposed: one rule set, as piquet, with Rubicon and the six-pack
   game as possible later *games*, not settings.
3. **A GitHub repository** (private, as piquet began) **and a licence**
   (piquet is MIT).
4. **Unit of play.** Proposed: one game to 1,000. Foster also describes a
   *partie*, the best of five games with a double counting two, which is
   possible later as a longer format.

## Milestones

- [x] **0. Planning**: literature, sources, rules, design, plan.
- [ ] **1. The pack**: card types and physical cards, bitboards, codes, the
      seeded generator and shuffle, the deal (3-2-3) and the turn-up. A
      statistical test of the deal and draw against the exact odds.
- [ ] **2. Tricks**: who wins a trick; the follow-and-win rule of the last
      eight tricks.
- [ ] **3. Declarations**: combinations, class levels, the new-card rule,
      announcements and carried scores, the curated options. Every worked
      example in `RULES.md` as a test first; the brute-force reference.
- [ ] **4. The hand**: the phase machine end to end (drawing, the dix, the
      last stock trick, the endgame), the event log, a text notation for
      positions, invariants over random self-play.
- [ ] **5. The game**: the cut, alternating deals, the race to 1,000, the
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

**Milestone 1, test first.** Create the Cargo workspace (`bezique-core`
first; `bezique-wasm` and `bezique-cli` when they have something to hold),
then `cards.rs`. The first tests:

- the pack is 64 cards, two of each of 32 types;
- codes round-trip (`AS`, `TD`, `7C`), and parse errors are reported;
- rank order within a suit is A, 10, K, Q, J, 9, 8, 7, and twins compare equal;
- suit masks partition the 64 bits; the highest card of a suit in a set.

Then the generator against its published reference outputs, the shuffle,
and the deal.

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

## Ideas, not owed

- A score display that nods to the period's dial markers, or Foster's
  counters (button 500, blue 100, red 50, white 10).
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
