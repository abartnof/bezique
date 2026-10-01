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

