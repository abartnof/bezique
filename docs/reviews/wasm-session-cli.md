# Review: wasm boundary, session, tournament harness, CLI

Scope read: `bezique-wasm/src/lib.rs`, `tests/protocol.rs`, `ffi-check.mjs`,
`session.rs`, `tournament.rs`, `measure.rs` (lines 1-200, 296-440, 800-855
only), `bezique-cli/src/main.rs`, `docs/PROTOCOL.md`, `web3d/src/engine.js`,
`kit/js/engine.js`, and the restore/persist code in `web3d/src/main.js`.
Not read: `table.rs`, the remainder of `measure.rs`, `examples/`. Nothing was
run.

## Ranked findings

1. **Risk: the sequential rule is not O'Brien-Fleming.**
   `tournament.rs:249-259` (`Sequential::stop`) uses one constant `z` (3.0 at
   `measure.rs:309,379,403`) at every look. CLAUDE.md asks for O'Brien-Fleming.
   `personas` (`measure.rs:~805`) does it right: `z = 2.09*sqrt(LOOKS/k)`.
   Constant 3.0 is conservative at late looks but costs power and is not the
   stated design.
   Fix: give `Sequential` a `looks` count and compute `z_k = c*sqrt(K/k)`,
   with `c` about 2.09 for K=5 and 2.04 for larger K, as `personas` does.
   Share one helper.

2. **Risk: the "nothing worth having" stop is an unadjusted repeated peek.**
   `tournament.rs:254` stops when `mean +/- 2se` lies inside `+/-worth`. That
   is a fixed 95% interval checked at every batch, so the chance of a false
   "no effect" rises with the number of looks. The same applies to `inside`
   (1.96) in `personas`, `measure.rs:~806`. Fix: use the same
   O'Brien-Fleming-scaled `z_k` for the futility interval (or a stated
   conditional-power rule). Also record the look count.

3. **Risk: the reports do not carry what CLAUDE.md requires.**
   `report_hands`, `report_games` and `sequential_hands`
   (`measure.rs:~96-135, 410-425`) print the mean, se, pairs used, rule and
   stop reason. They do not print the interval at the last look, the boundary
   `z` at that look, or the floor under that name (`min` is printed as "looks
   from"). Fix: print `mean +/- z_k*se` and the `[floor, ceiling, used]`
   triple in one line. `personas` is complete and is the model.

4. **Risk: the page restores a sitting without checking that the table's moves
   came out the same.** `web3d/src/main.js:337-352` checks only
   `record_version` and `r.ok`. The core's `Session::replay` has a second
   guard (`session.rs:315-323`, `session.record != record`) that the FFI does
   not export. If a change to play lands without a `RECORD_VERSION` bump, the
   saved sitting loads as a different game, silently, as long as every
   command is accepted. Fix: after the loop, compare `engine.state().record`
   with `saved.record` (array equality) and return false on a mismatch. It is
   about three lines, and it matches the guard the docs describe. Optionally
   also export a `bezique_replay`.

5. **Nit/risk: `engine.js` passes arguments unvalidated.**
   `web3d/src/engine.js:19`. The wasm side clamps ints (`lib.rs:37-51`), but
   JS coerces first. `erratic` 0.5 becomes 0, so a caller who thinks in 0-1
   gets 0 with no error. A negative or NaN `level` becomes a huge u32 and is
   clamped to 5 (NaN to 0 and so level 1). `persona` below 0 is the same.
   `prefs.js` limits the UI (`erratic` max 1000), but a corrupted
   localStorage value is not rejected here. Fix: `Math.trunc`/range-check in
   `new`, or throw. Low impact.

6. **Nit: a `set` entry is appended to `record` on every toggle**
   (`session.rs:514`). It is unbounded, but only by user toggles, and every
   record is replayed on each reload. Cheap fix: collapse a consecutive
   identical `set` of the same aid.

7. **Nit/test gap: `tournament.rs:386-409`, `hand_margins_match_an_independent_count`.**
   It recounts with `game.total` after hand 1 is over. `total` is banked
   points plus the *new* hand's `score`. If hand 2's turn-up is the seven of
   trumps (about 1 deal in 32), the dealer's 10 is included in `total` but not
   in `previous_hand().score` that `hands()` uses (`tournament.rs:197-207`).
   The test then fails for that seed, or passes only because none of the seeds
   hit one. `hands()` itself is right: hand 1 only. Unverified, because I did
   not run it. Fix: compare against `game.previous_hand()` scores, or exclude
   such seeds in the test.

8. **Nit: the FFI tests are thin.**
   - `ffi-check.mjs` and `tests/protocol.rs` do not exercise `undo`, a refused
     command, a command sent before `bezique_new` (expect `null` state and
     return 0), a `len` larger than the buffer (clamped; fine), or invalid
     UTF-8 (lossy, becomes "not a command"; fine).
   - `no_state_shows_a_hidden_card` runs at level 3 with the tracker on and
     hints off. The `hint.reason` parameters (`card`, `led`, `kept`,
     `spared`) are never leak-tested with hints on. They come from the
     person's own `View`, so I expect them clean (unverified).
   - Add a hints-on run to that test.

9. **Nit: the CLI exit codes are consistent.** Usage errors give 2 and play
   errors give 1. `--erratic` NaN or inf is rejected because
   `(0.0..=1.0).contains` is false for NaN. The `measure` arg parsers panic
   with a message (exit 101) on a bad number (`measure.rs:~36-42`), which is
   acceptable for a dev tool, though `exit(2)` would match.

## Checked and clean

- **Command parsing** (`session.rs:498-560`): arity is exact (`split_whitespace`
  patterns). Ids outside 0-255 fail to parse; 64-255 are caught by
  `Card::is_valid` in `Hand::apply` (`hand.rs:387`) before any shift, so no
  aliasing or overflow. Not-held, not-your-turn, `declare` out of range or
  outside the declare phase, `dealer` twice, `set` with a bad aid or value
  and `undo` with nothing all return an error and leave the state unchanged.
  A leading `*` is not a command, so replaying a record that still contains
  starred entries is refused rather than corrupting it. Records are
  normalised (`play 5`, not `play +5`).
- **Phase handling**: `play` and `declare` check the prompt kind first.
  `set` before the deal is fine (`run` returns with no game). No path I saw
  panics on bad input. The `expect`s sit behind prompt checks.
- **Undo** (`session.rs:637-671`):
  - It is a rebuild, so the live session is untouched on failure.
  - `script` is cleared afterwards.
  - `Erratic::replay` consumes the same RNG draws as `choose`
    (`opponent.rs:179`; the rungs hold no state), so a post-undo game
    reloads identically from its record.
  - The "aid made the decision forced" recursion terminates, and
    `record.len() - 1` cannot underflow because `last` exists.
  - The dealer choice and `set` are not undoable, as documented.
  - `play_forced` is only settable at game start in the page (`prefs`
    scope `new`), so `startAids`-then-replay ordering in `restore` is safe.
    If that scope ever changes, a mid-game `play_forced` would make
    `restore` fail ("not your turn") and lose the sitting.
- **State leaks between games**: `bezique_new` replaces the whole `Session`
  and re-renders. `OUT`, `IN` and `PERSONAS` are the only other state and are
  overwritten each call. `IN` is cleared in `alloc`.
- **FFI memory**:
  - The `state` and `personas` pointers are valid until the next call, as
    documented, and the JS re-reads `memory.buffer` each time.
  - `send` clamps `len`, so there is no overread.
  - `alloc` with a huge `len` traps. The guard stops the engine and offers a
    reload.
  - There is no `unsafe`.
  - After a panic (abort) the instance is dead and the guard says so.
- **JSON writer** (`lib.rs:57-71`): it escapes quote, backslash and control
  characters. Other non-ASCII goes through as UTF-8, which is valid JSON.
  Numbers are integers or booleans, never NaN.
- **Protocol vs docs**: every field in `PROTOCOL.md` is emitted, with the
  documented types. The ids are numbers; `points`, `stock` and the like are
  numbers; `legal` is a list of numbers; `double` is a bool; `tricks_played`
  is null or a list. The table-absent fields before the dealer is chosen
  match. `error_code: "stopped"` is added by the page guard only and is not
  in the protocol (fine; say so in the doc if the page reads it).
- **`tournament.rs` mirroring**: both seatings use the same `Game::new(seed,
  Side::A)` deal and the same per-slot agent seed. `swap` flips the agents,
  and the credit logic `usize::from((winner==A)==swap)` is correct for all
  four cases. Hand margin sign handling is correct. Pair-level SE (sample
  variance, n-1) is the right unit. Batches cover consecutive seeds with no
  gaps, and results do not depend on the thread count (both tested).
  `points_a_game` is the pair sum over 2.
- **`personas` look in `measure.rs`**: the floor (`FLOOR`), the ceiling
  (`LOOKS`) and the OBF z are implemented, and it reports the pairs used, the
  boundary z and the 95% interval.

## Unverified

I did not build or run anything, and I did not review `table.rs`, the rest of
`measure.rs` (`tune`, `ladder`, `anatomy`, `lookahead*`, `worlds`) or
`examples/`. Whether the other reports pass an explicit floor/ceiling is
unchecked.

## Resolution

Findings 4 and 5 are the web3d restore and `engine.js` and are not covered
here. Nothing was measured; tests only, each short.

1. **Fixed, with a correction.** `Sequential` loses its constant `z`. Looks
   are `max / batch`, and the boundary at look `k` is `z_k = c sqrt(K/k)` from
   one shared helper (`tournament::obf_z`, `obf_constant`) that `personality`
   now also uses. The finding's constants were wrong: `c` is 2.04 for K=5
   (matching the published 2.040) and rises with K: 2.083 for 10, 2.116 for
   16, 2.146 for 32, 2.171 for 60. `personality` used 2.09 at K=16, slightly
   loose (about 1%). `c` is computed by integrating the random walk's
   crossing chance and checked against the published values and by a
   null simulation (type I error within 3.5%-6.2% at K=5, 16, 40).
2. **Fixed.** The "nothing worth having" stop uses the same `z_k` as the
   half-width of the interval; `personality`'s `inside` test too (was 1.96).
   The look count is recorded and printed. A simulation at an effect equal
   to the smallest worth having shows the false-stop rate under 3.5% for K=5,
   16, 40.
3. **Fixed.** Sequential reports print `mean ± z_k se` as an interval, the
   boundary `z`, the look number and `[floor, ceiling, used]` on one line
   (`Sequential::describe`). Fixed-n reports print a 95% interval at a
   single look and `[n, n, n]`. `personality` prints the triple and its
   intervals at `z_k`.
6. **Fixed, old records stay valid.** A `set` equal to the entry just before
   it is not recorded. `replay` compares records with consecutive repeated
   `set` entries collapsed on both sides, so a record saved with repeats
   still loads (a page that restores a sitting sends its aids again, so
   such records exist). `RECORD_VERSION` is unchanged: play is not altered
   (the golden game is unaffected). Consequence for the restore fix of
   finding 4: a comparison of the replayed record with `saved.record` must
   collapse consecutive identical `set` entries on both sides, or it will
   refuse old sittings.
7. **Confirmed, fixed.** The test counted with `Game::total`, which includes
   the new hand's turn-up (10 for the seven of trumps); seeds 40-69 happened
   to miss it. It now takes each side's banked points (total less the new
   hand's score), and a second test finds seeds that turn up the seven and
   checks the naive count differs and `hands()` is right. `hands()` itself
   was right.
8. **Fixed.** New protocol tests: undo (state and record restored, refused
   when nothing to undo), refused commands leave the state unchanged apart
   from the error, a command before `bezique_new` (returns 0, state `null`),
   an oversized length, invalid UTF-8, a new table replacing the old, and the
   leak test now also runs with hints on (clean).
9. **Fixed.** The `measure` argument and environment parsers print a message
   and exit 2. A bad `MEASURE_LEVEL` was silently replaced by the default in
   three places and is now refused, as is a level outside 1..=5. The CLI
   needed no change.

### Measurements affected

- `measurements/rung6-sequential.txt` stopped "NothingWorthHaving" at 1,100
  pairs on the unadjusted 2 se interval (se 0.009). Under the corrected rule
  (K=60, look 11, z = 5.1) that interval is about ±0.046, so the rule would
  not have stopped there. The point estimate is +0.000 and the interval at 2
  se is within ±0.02, but the claim is not certified at the stated error.
  Not rerun; a rerun under the new rule is the way to close it.
- `measurements/personality-level{2,4}.txt`: the "strength inside the
  tolerance" half of each stop used an unadjusted 95% interval, and the
  boundary 2.09 was about 1% low at K=16. Style-clear stops are barely
  affected. Not rerun.
- Runs that stopped at the cap (`worlds-32-sequential`, `rollout-memory`) or
  declared a clear effect at constant z = 3.0 are unaffected: 3.0 was more
  conservative than the corrected boundary at every look past the first few.
