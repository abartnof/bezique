# Code review of the overnight work, 2026-10-02

A review of commits `a70f478..0f3e6ca`: the twin relabelling in
`observation.rs`, the tournament harness, the agents' style, skills and
leanings, the endgame solver, the opponent, the session, the wasm crate and
the terminal client, with the documents' claims about them. Four reviewers
took one area each (observation and the engine; the solver, agents and
measurements; the session; the protocol and the clients). Each reproduced
its findings with a scratch test or command before reporting them; nothing
was taken on trust.

The labels are those of the first review (`2026-10-01-engine.md`):
**defect**, **latent defect** (wrong, but not reachable through the moves
the engine offers today), **document**, **test**, **preference**.

## Observation and the engine (O)

| # | Finding | Label | Resolution |
|---|---|---|---|
| O1 | With both copies of a type on the opponent's table, the view numbered them by physical copy, so the record of which copy went into which announcement told the viewer what the opponent had held: after a bezique and a double bezique, whether both queens were held at the first. | defect | fixed (`cda1fa0`): the opponent's table is ordered by the announcements each card has been in, which are public. A property test now swaps a type's twins everywhere, in the deal and every move, and requires the view of a player who never held that type to be unchanged at every position (`tests/twin_symmetry.rs`); it failed 70 times before the fix. |
| O2 | `Hand::arranged` accepted a stock listing a card twice (the check compared sets). | latent defect | fixed (`395fa17`) |
| O3 | `Hand::arranged` accepted more than eight cards each once the stock was gone, and a turn-up not of the trump suit. | latent defect | fixed (`395fa17`) |
| O4 | `Hand::known` was documented as returning the lower twin; it returns the one seen first, which is better. | document | fixed (`395fa17`) |

## The solver, agents and measurements (A)

Checked and sound: the solver is exact for the hand's points; rung 3 reads
only its view, and after the stock every card really is known; pairs are
truly mirrored; the statistics are right; `ladder-tuned-style.txt`
reproduces exactly.

| # | Finding | Label | Resolution |
|---|---|---|---|
| A1 | Rung 3 panicked on a position arranged in the last tricks: with no record of earlier tricks, the view's count of the opponent's hand was 62 cards. | latent defect | fixed (`e01ac7b`): when the count does not come out, the maxims play instead |
| A2 | The endgame skill counted cards whatever the memory flag said. | latent defect | fixed (`e01ac7b`): the skill acts only with memory |
| A3 | The persona costs were measured at rung 2, under a comment claiming rung 2 plays the stock as the top rung does; they differ in about 1.5% of those decisions. | document | fixed (`07f2a6f`): the comment and DESIGN say where they were measured |
| A4 | "Every report can be reproduced exactly" holds only at the commit that made it; `skills.txt` predates the final style; one report's game count was unexplained. | document | fixed (`07f2a6f`): `skills-final-style.txt` added, figures in `agents.rs` updated |
| A5 | A leaning was called "free by construction"; it costs up to 1.24 points a hand. | document | fixed (`07f2a6f`) |
| A6 | "Exact" means exact for the hand's points, not for the race to 1,000. Over 4,294 endgame decisions near the end of games the difference never mattered. | document | fixed (`07f2a6f`): DESIGN §10.3 says so |
| A7 | The solver was described as alpha-beta; it was memoised minimax without pruning. | document | fixed (`0466060`): it is now alpha-beta with a table of bounds, choosing exactly as before and about four times faster in play |
| A8 | The solver's test compared it with a reference sharing its scoring step, so a scoring error would pass both. | test | fixed (`e01ac7b`): a test checks the counted hand against the true one and the solver's value against the engine's own scoring in real games |

## The session (S)

Checked and sound: replay reproduces the sitting, persona and erraticism
included; undo then redo is identical at every step; hints read only the
person's view; the cut follows the rules; the opponent's cards are named by
type.

| # | Finding | Label | Resolution |
|---|---|---|---|
| S1 | Undoing the choice of dealer, then choosing the other, gave the opponent the very hand the person had just seen: the first deal comes from the seed alone. | defect | fixed (`8b4d996`): the choice cannot be undone |
| S2 | Undo replays the same stock, so a person can learn the next card drawn or the opponent's answer to a lead, and then play differently. | design | documented (`8b9c113`) as a learning aid, not fair play; whether to limit undo is a question for the maintainer |
| S3 | A dix on the last stock trick was narrated with the old turn-up: the loser actually takes the seven, and the card taken by the exchange was told as a draw. | defect | fixed (`bc27dfe`) |
| S4 | The last trick of a hand, its scores and the end of the hand were tagged with the next hand's number. | defect | fixed (`bc27dfe`) |
| S5 | A turned seven that wins the game was never told. | defect | fixed (`bc27dfe`) |
| S6 | "dealer them", after the person cut higher, was told as the opponent's choice. | defect | fixed (`bc27dfe`) |
| S7 | With the tracker off, the narration still lists every trick, and the hint counts cards. | document | documented (`8b9c113`) |
| S8 | A double bezique laid by the opponent was told with duplicate ids. | defect | fixed (`bc27dfe`) |
| S9 | A level above the top built repeated top rungs, weakening erraticism; an unknown persona key was silently drawn. | latent defect | fixed (`ee520c0`): the level is clamped; the fallback is documented |

## The protocol and the clients (W)

Checked and sound: no panics under fuzzed commands, natively or in wasm;
every state and event field matches PROTOCOL; the terminal client shows no
hidden card outside `--watch` and `--reveal`.

| # | Finding | Label | Resolution |
|---|---|---|---|
| W1 | Undo shows the future (as S2). | design | as S2 |
| W2 | The state carried the seed, from which every hidden card follows. | defect | fixed (`8b9c113`): removed; PROTOCOL says hidden cards are only as hidden as the seed and the module's memory |
| W3 | The leak test skipped strings and the narration, and compared by code; planted leaks in an event, in the error, or in true copy ids all passed it. | test | fixed (`8b9c113`): a wholly hidden type must be named nowhere in the hand's state, and a test shows planted leaks caught. Copy ids are not checked by this test; the state takes every card from the view, whose ids the twin-symmetry property checks (O1) |
| W4 | The narration carries every trick with the tracker off (as S7). | document | as S7 |
| W5 | Events keep the true id of an opponent's card whose twin the person has held; PROTOCOL said copy 0 always. | document | fixed (`bc27dfe`) |
| W6 | The terminal client refused `t` at the declare prompt. | defect | fixed (`6f69df0`) |
| W7 | After switching `play_forced` on, undo could be accepted and change nothing. | latent defect | fixed (`20e567e`): undo goes back to the decision before |
| W8 | PROTOCOL did not say how bad arguments are treated, that the state is empty before `bezique_new`, or how to reload over the FFI. | document | fixed (`8b9c113`) |
| W9 | DESIGN §9 listed prompts and events that were never built. | document | fixed (`a3280e6`) |
| W10 | `--help` left three options undescribed. | document | fixed (`a3280e6`) |

**Status, 2026-10-02:** every finding is resolved except S2/W1, which is a
design question (should undo be limited to points where nothing new has been
revealed?) and is documented meanwhile. A note not counted above: a saved
record replays only while the agents and the option order are unchanged; a
version field beside records is worth adding before records are kept.
