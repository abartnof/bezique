# Analysis scripts

Small, standalone scripts that compute or measure something about the game.
They are not part of the engine and nothing imports them.

| Script | What it computes | Provenance |
|---|---|---|
| `bezique_stats.py` | Exact odds of *receiving* each meld's cards over a hand (two-, four- and six-pack), carte blanche, the seven of trumps, distinguishable hands with duplicate cards; a seeded Monte Carlo cross-check | Written for the literature review (`docs/LITERATURE.md` §10b, Tables 1–3 and 5); added to the repository unchanged on 2026-10-01 |

`python3 analysis/bezique_stats.py` takes about thirty seconds (the Monte Carlo
dominates).

**Checked on 2026-10-01.** Every exact figure in `docs/LITERATURE.md` §10b
Tables 1–3 and 5 is reproduced. The single and double bezique odds also
agree with independent closed forms, 1 − 2·C(62,32)/C(64,32) + C(60,32)/C(64,32)
and C(60,28)/C(64,32). The Monte Carlo agrees with the exact values to
within about one standard error.

The model is stated in the script's docstring: a player receives exactly half
the pack, a uniformly random half, so every figure is an **upper bound** on
declaring the meld. The engine's statistical tests use these exact figures as
targets for the deal and the draw, which are independent of play.

The literature review also cites a companion self-play script,
`bezique_sim.py` (§10b Table 4). It is not in the repository.
