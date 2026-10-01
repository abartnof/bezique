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
