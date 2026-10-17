# Working on this repository

Bezique, the two-pack, two-player card game, as a Rust engine and one offline
web page. `PLAN.md` says where the work stands; `docs/RULES.md` is the rule
authority.

Standing rules:

- **Simulations stop early when the signal is clear.** Before the first
  batch, fix the comparison, a **floor** on the number of resamples (mirrored
  pairs), a **ceiling**, the batch size and a stopping boundary
  (O'Brien–Fleming). Run in batches and look between them: stop as soon as the
  effect crosses the boundary, or is plainly too small to matter, and never
  before the floor. Never peek with an unadjusted p < 0.05. Always use mirrored
  pairs: each deal played twice, seats swapped. Report the floor, the ceiling,
  the number used and the interval at the last look. No runs of millions.
- **Review before a long run.** A full code review of what a run depends on,
  and its fixes, come before any run of more than a few minutes.
- **Documents are public-facing and terse**: decisions, evidence, what
  follows. No narrative of who said what; the person who commissioned the game
  is "the maintainer".
