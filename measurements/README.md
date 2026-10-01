# Measurements

Reports printed by `crates/bezique-core/src/bin/measure.rs`, kept so that a
claim made in the documents can be traced to the run that produced it. Each
file is the program's output, unedited. Seeds are fixed in the program, so
every report can be reproduced exactly:

```
cargo run --release -p bezique-core --bin measure -- <report> [n]
```

| File | Report | What it shows |
|---|---|---|
| `anatomy-rung2.txt` | `anatomy 2000` | Rung 2 against itself, as first built: game length, doubles, the seat advantage, where the points come from, how often each combination is scored. Shows the defect that rung 2 throws its bezique cards away (double bezique scored in 0.2% of player-hands). |
| `anatomy-rung2-bezique14.txt` | `anatomy 2000` | The same, with bezique cards valued above aces and the table discount kept off cards with work left: doubles 0.8% of player-hands. |
| `anatomy-rung2-tuned.txt` | `anatomy 2000` | Rung 2 as tuned on 2026-10-02 (bezique 14, urgency 16, take_brisque 20): doubles 1.7%, the sequence 7.8% of player-hands, 4.58 hands a game. |
| `ladder-rung2-tuned.txt` | `ladder 2000` | Tuned rung 2 against rung 1: +80.5 points a hand (se 2.5); 88.0% of games (se 0.5). |
| `ladder-rungs-1-5.txt` | `ladder 2000` (as first built) | A first five-rung ladder: memory (+1.2, se 0.7) and watching the table (+0.2, se 0.5) did not beat the rung below; the exact endgame did (+18.9, se 0.3). Kept as the record of why the ladder was rebuilt. |
| `skills.txt` | `skills 2000` | Each skill added to rung 2 alone: memory +1.9 (se 0.6); memory with the exact endgame +19.9 (se 0.7). |
| `ladder-tuned-style.txt` | `ladder 2000` | The ladder with the final style (2026-10-02): rung 2 over 1 +98.6 (se 2.5), 92.0% of games; rung 3 over 2 +18.7 (se 0.7), 58.2%; rung 4 over 3 +2.3 (se 0.4), 51.4%. |
| `anatomy-rung2-final.txt` | `anatomy 2000` | Rung 2 with the final style: 4.25 hands a game, 12.4% doubles, 95.2% ended mid-hand, the first dealer winning 50.0%; double bezique scored in 2.5% of player-hands (0.2% before the night's fixes). |
| `ladder.txt` | `ladder 2000` | The ladder as rebuilt: rung 2 over rung 1 +79.0 (se 2.4), 87.5% of games; rung 3 (memory and the exact endgame) over rung 2 +19.2 (se 0.8), 58.0% of games. |

## Tuning rung 2 (2026-10-02)

Each weight was swept with `tune` against the then default (3,000 mirrored
pairs), a value chosen at the knee of the curve or on principle rather than
at the maximum (to avoid the optimizer's curse), and confirmed on fresh
seeds (`MEASURE_SEED`, 10,000 pairs). Margins are points a hand for the
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
each on fresh seeds (`MEASURE_SEED`), points a hand for the new value
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

