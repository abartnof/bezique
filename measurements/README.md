# Measurements

Reports printed by `crates/bezique-core/src/bin/measure.rs`, kept so that a
claim made in the documents can be traced to the run that produced it. Each
file is the program's output, unedited. Seeds are fixed in the program, so
every report can be reproduced exactly by the code at the commit that added
it; a report made before a later change to the agents is kept as history,
not as a current figure:

```
cargo run --release -p bezique-core --bin measure -- <report> [n]
```

| File | Report | What it shows |
|---|---|---|
| `anatomy-rung2.txt` | `anatomy 2000` | Rung 2 against itself, as first built: game length, doubles, the seat advantage, where the points come from, how often each combination is scored. Shows the defect that rung 2 throws its bezique cards away (double bezique scored in 0.2% of player-hands). |
| `anatomy-rung2-bezique14.txt` | `anatomy 2000` | The same, with bezique cards valued above aces and the table discount kept off cards with work left: doubles 0.8% of player-hands. |
| `anatomy-rung2-tuned.txt` | `anatomy 2000` | Rung 2 as tuned on 2026-10-02 (bezique 14, urgency 16, take_brisque 20): doubles 1.7%, the sequence 7.8% of player-hands, 4.58 hands a game. |
| `ladder-rung2-tuned.txt` | `ladder 2000` (when games used as many pairs as hands; `ladder 2000` now plays 1,000 pairs of games) | Tuned rung 2 against rung 1: +80.5 points a hand (se 2.5); 88.0% of games (se 0.5). |
| `ladder-rungs-1-5.txt` | `ladder 2000` (as first built) | A first five-rung ladder: memory (+1.2, se 0.7) and watching the table (+0.2, se 0.5) did not beat the rung below; the exact endgame did (+18.9, se 0.3). Kept as the record of why the ladder was rebuilt. |
| `skills.txt` | `skills 2000` | Each skill added to rung 2 alone: memory +1.9 (se 0.6); memory with the exact endgame +19.9 (se 0.7). Made before the final style. |
| `skills-final-style.txt` | `skills 2000` | The same with the final style (2026-10-02): memory +1.8 (se 0.6); memory with the exact endgame +18.6 (se 0.6). |
| `ladder-tuned-style.txt` | `ladder 2000` | The ladder with the final style (2026-10-02): rung 2 over 1 +98.6 (se 2.5), 92.0% of games; rung 3 over 2 +18.7 (se 0.7), 58.2%; rung 4 over 3 +2.3 (se 0.4), 51.4%. |
| `anatomy-rung2-final.txt` | `anatomy 2000` | Rung 2 with the final style: 4.25 hands a game, 12.4% doubles, 95.2% ended mid-hand, the first dealer winning 50.0%; double bezique scored in 2.5% of player-hands (0.2% before the night's fixes). |
| `lookahead-sweep.txt` | `lookahead 500 <horizon> <worlds>` | Rung 5 (rung 4 looking ahead near the end of the stock) against rung 4, swept over the horizon (face-down cards left when it starts looking) and the worlds sampled. Gains seemed to grow with worlds (8: +27.2, 16: +32.8, 32: +35.3 at horizon 9) and with the horizon up to 9–13, then to fall (21: +19.5, se 4.5). Each point is one run of 500 pairs, so these are suggestions, not findings. Times a hand are rough: other work shared the machine. |
| `lookahead-confirm.txt` | `MEASURE_SEED=60000000 lookahead 2000 9 16` | The chosen setting, horizon 9 and 16 worlds (the knee, not the maximum), confirmed on fresh seeds: +30.05 points a hand over rung 4 (se 1.29). It became rung 5. |
| `personality-level2.txt`, `personality-level4.txt` | `personality` / `MEASURE_LEVEL=4 personality` (sequential, O'Brien-Fleming; `docs/OPPONENT.md` §4) | The six personas of 2026-10-17 against balanced at rungs 2 and 4: strength inside +-2 points a hand for four of five, the specialist -1.2 and -1.7; each persona's own style statistic. Supersedes `personas-rung4.txt` (the earlier four personas). |
| `personas-rung4.txt` | `MEASURE_LEVEL=4 personas 5000` | The personas at rung 4, where they had only been calibrated at rung 2: concentrates −0.36 (se 0.96), close −0.32 (se 0.84), bezique −1.35 (se 0.89) points a hand against balanced. All within the two-point limit. |
| `lookahead-declarations.txt` | `lookahead-declarations 1000` | Rung 5 looking ahead in the turn of declaring too (each option valued in the same worlds), against rung 5: −0.94 points a hand (se 0.41). Not kept: rung 4's declaring stands. Declaring lays cards open, and determinised rollouts cannot value what is hidden (DESIGN §10.4, strategy fusion). |
| `anatomy-rung2-counted.txt` | `anatomy 2000` (after the full review of 2026-10-02) | The final rung 2 again, combinations now counted from the finished hands tallied: double bezique 1.0% of player-hands (not 2.5%), the sequence 5.7% (not 7.8%); every other figure identical to `anatomy-rung2-final.txt`. |
| `anatomy-rung5.txt` | `MEASURE_LEVEL=5 anatomy 1000` (at `bba7edc`) | Rung 5 against itself: 4.19 hands a game, 13.1% doubles, 94.5% ended mid-hand, the first dealer winning 51.0% (se 1.6); the sequence 7.8% of points (6.4% at rung 2), the dix 3.1%; the winner of the last stock trick out-scores the loser by 8.7 points over the hand (17.5 at rung 2), both sides now fighting for it. |
| `lookahead-32-worlds.txt` | `MEASURE_SEED=70000000 lookahead-vs-five 1500 9 32` | Rung 5 with 32 worlds against rung 5 (16): +2.44 points a hand (se 1.27). Under two standard errors, at twice the cost a decision: not adopted. A run of about 6,000 pairs would settle it. |
| `rollout-memory.txt` | `rollout-memory` (sequential; at `dda972f`) | Rung 5 whose rollout players recall the cards played before the world (the full review's finding A1), against rung 5 whose do not: −0.22 points a hand (se 0.42) at the cap of 8,000 pairs. No measurable effect; the fix is kept because it makes the rollouts what they are documented to be. |
| `ladder-after-review.txt` | `ladder 2000` (at `fbd6d3c`, after the full review's play fixes) | The ladder on play as it now stands (`RECORD_VERSION` 2): rung 2 over 1 +98.5 (92.0% of games), rung 3 over 2 +18.7 (58.3%), rung 4 over 3 +2.3 (51.4%), rung 5 over 4 +30.7 (se 1.3; 63.0%, game points +0.32 a game); the fixes changed little. |
| `anatomy-rung5-counted.txt` | `MEASURE_LEVEL=5 anatomy 1000` (at `fbd6d3c`) | Rung 5 against itself on play as it now stands, combinations counted correctly: 4.23 hands a game, 13.4% doubles, 95.5% ended mid-hand, the first dealer winning 53.0% (se 1.6); double bezique 1.0% of player-hands, the sequence 6.9%. |
| `worlds-32-sequential.txt` | `worlds 32` (sequential; at `96893fc`, after the full review) | Rung 5 with 32 worlds against rung 5 (16): +1.30 points a hand (se 0.49) at the cap of 10,000 pairs, z = 2.65: short of the bound of 3 fixed in advance, so inconclusive by the rule. Pooled with `lookahead-32-worlds.txt` (+2.44, se 1.27, other seeds), about +1.4 (se 0.46): probably real but small, at twice the cost of a decision. Not adopted; to revisit when the page's speed can be measured. Under this rule with no effect at all, about 2.3% of runs would stop "clear" wrongly (simulated by the review); a run stopping early overstates the effect. |
| `rung6-sequential.txt` | `rung6` (sequential; at `4965999`, after the review of rung 6) | Rung 6 (rung 5 playing for the game in the last stock tricks, on the race model) against rung 5, in mirrored games by game points: +0.000 a game (se 0.009) after 1,100 pairs, of which 1,011 split; stopped as nothing worth having (within ±0.02 game points a game at two standard errors). Not kept as a level. |
| `ladder-rung5.txt` | `ladder 2000` | The five-rung ladder (2026-10-02, after the solver's speed-up): rungs 2–4 exactly as in `ladder-tuned-style.txt`, so the faster solver plays as before; rung 5 over rung 4 +30.5 points a hand (se 1.3), 62.4% of games (se 0.8). |
| `ladder.txt` | `ladder 2000` | The ladder as rebuilt: rung 2 over rung 1 +79.0 (se 2.4), 87.5% of games; rung 3 (memory and the exact endgame) over rung 2 +19.2 (se 0.8), 58.0% of games. |

**A correction (full review of 2026-10-02).** The anatomy reports made
before `anatomy-rung2-counted.txt` counted combinations announced in every
hand, the unfinished last hand of each game included, but divided by the
finished hands only. Their rates per player-hand are inflated (double
bezique about 2.5 times, the sequence about 1.35 times), so the figures
quoted for them above (0.2%, 0.8%, 1.7%, 2.5%; the sequence 7.8%) overstate
the level, though not the direction of the changes. Points per hand and
their shares come from the hands' logs and are unaffected.

## Tuning rung 2 (2026-10-02)

Each weight was swept with `tune` against the then default (3,000 mirrored
pairs), a value chosen at the knee of the curve or on principle rather than
at the maximum (to avoid the optimizer's curse), and confirmed on fresh
seeds (`MEASURE_SEED`, 10,000 pairs; the values used were not recorded, so
these rows cannot be reproduced exactly, and since 2026-10-02 every report
prints its seeds). Margins are points a hand for the
variant against the default, with standard errors.

| Weight | Swept | Chosen | Confirmed on fresh seeds |
|---|---|---|---|
| `bezique` | 6 → 10, 14, 18, 24: +1.9, +1.8, +1.8, +2.1 (se ≈1.7) | 14 (Pardon: keep the bezique cards; outranks an ace) | +0.57 (se 0.69): free in points, so a persona dial |
| `urgency` | 0 → 2, 4, 8, 16, 24, 32, 64, 200: +10.8, +15.7, +33.2, +37.1, +37.6, +38.8, +38.2, +38.3 | 16 (the knee) | +37.7 (se 1.0) |
| `take_brisque` (with urgency 16) | 12 → 15, 20, 30: +7.7, +8.4, +8.7 | 20 | +10.8 (se 0.6) |
| `take_to_declare` (with urgency 16) | 7 → 0, 12, 20: −17.4, +1.4, +0.5 | 7 (unchanged) | – |
| (run, not kept) `last-stock-trick 2000`; confirmed with `MEASURE_SEED=90000000 … 3000` | | Rung 3 winning the last stock trick whenever it can, against rung 3: +2.49 (se 0.43); confirmed on fresh seeds +1.82 (se 0.38). It became rung 4. |

## Tuning rung 2 further, and the personas (2026-10-02)

Calibrating personas against the default exposed weights the default had
never been tuned on: a persona that measured *better* than balanced was
improved play, not a style, and went into the default instead. Confirmed
each on fresh seeds (`MEASURE_SEED`, values not recorded), points a hand for the new value
against the old, at rung 2:

| Weight | Change | Confirmed |
|---|---|---|
| `four` | 4 → 8 | +7.14 (se 0.79), 10,000 pairs |
| `marriage` | 4 → 7 | +1.32 (se 0.49), 10,000 pairs |
| `trump` | 5 → 8 | +1.35 (se 0.61), 20,000 pairs |
| `take_to_declare` | 7 → 20 | +1.41 (se 0.43), 20,000 pairs |
| `bezique` | 14 → 20 | +1.33 (se 0.24), 20,000 pairs |

The bezique weight is lumpy above 20 (26: −0.1; 32: +3.7; 40: +0.4, each
5,000 pairs, se ≈ 0.3–0.9): a threshold effect, not a smooth optimum, so it
was left at 20 rather than chased.

**Personas** (`personas.txt`, `MEASURE_LEVEL=2 measure personas 20000`): a
persona must cost no more than about two points a hand against balanced and
play visibly differently. Weight shifts mostly failed one test or the other
on a tuned default: they either cost points or were simply better play. A
"brisque hunter" was dropped: no lever moved brisques (a player's share is
set by the cards), and leading brisques home lost them (74 against 80 a
hand). What passed:

| Persona | How | Cost a hand | Combinations / brisques a hand |
|---|---|---|---|
| balanced | the default | – | 139.1 / 80.0 |
| concentrates | of cards exactly equal in worth, throws from its shortest suit (Foster) | −1.24 (se 0.48) | 140.0 / 79.7 |
| close | of cards within a point, plays from the table first (Pardon) | −0.69 (se 0.43) | 138.9 / 80.2 |
| bezique | bezique cards valued at 40, not 20 (Pardon) | −0.45 (se 0.44) | 144.8 / 76.9 |

