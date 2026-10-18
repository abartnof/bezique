# Review: render kit, build and gate

Scope: `kit/js/*` (listed files), `kit/build.py`, `kit/test/`, `bin/gate`, `web3d/game.toml`, and the `main.js` call sites that use them. Read-only; nothing was run. Everything below is from reading, so "unverified" means not reproduced in a browser.

## Bugs

1. **Classic faces cannot be restored after switching to Jumbo.** `web3d/src/main.js:84-92` sets `faceSets.classic = textures.faces`. `kit/js/deck.js:442` `setFaces` does `Object.assign(textures.faces, faces)`, which overwrites that same object with the Jumbo textures. Scenario: faces default to jumbo, then Settings picks classic. `deck.setFaces(faceSets.classic)` assigns Jumbo onto Jumbo, so the cards stay Jumbo while `facesShown` says "classic". Fix: in `deck.js` keep `textures.faces` as a fresh object (`textures.faces = { ...faces }`, or assign from `{...faces}` into a private map), or in `main.js` copy: `faceSets = { classic: { ...textures.faces } }`.
2. **`screenPoint(id)` and `meshOf(code)` collapse twins.** `director.js:221, 256` use `placement.find(x => x.code === code)`, which returns the first copy. `main.js:441-444` maps cardId -> code -> first placement with that code, so asking for the second copy of a card can return the other copy's position (another zone, or the opponent's hand). Used by the test hook only today, but it makes any browser check or tutorial pointer wrong for twins. Fix: add `screenPointOfId(cardId or mesh id)` and `meshOfId`, and make the code forms documented as "first copy".
3. **Stale frame after an inset-only reframe.** `main.js:153-156` `frame()` calls `stage.setInset(...)`, which reframes the camera but does not render (`scene.js:215-222`). `setStrips(..., {instant:true})` renders only if the strips changed. Scenario: a tablet rotated across the 900 px width changes the inset only; the scene's own resize listener (registered earlier) renders first, then `ui.onLayout(frame)` moves the camera, and the table stays drawn with the old framing until the next hover or move. Fix: in `setInset`, `render()` when it returns true, or have `frame()` call `stage.render()` when `setInset` returned true.
4. **Render-loop stall if a gate releases synchronously.** `director.js:86-93` `due()` sets `heldAt`, then calls `entry.fn(release)`. If `fn` calls `release()` at once (a tutorial page that is disabled or already seen), `release()` clears `heldAt` and calls `wake()`, which returns because `running` is still true. `due` then returns true, so `frame()` takes the `held` branch, sets `running = false` and schedules nothing, although the queue may still hold entries and the timeline be busy. Cards freeze until something else wakes the loop. `skip()` (line 353) is safe because it ends with `wake()`. Fix: in `frame()`, after `due()`, if it returned true and `heldAt === null`, `requestAnimationFrame(frame)` and return; or defer the `fn(release)` call to a microtask.

## Risks

5. **Hint marks wipe dimming.** `director.js:164-168` `light()` calls `deck.decorate(mesh, { line })`, and `deck.decorate` defaults `dim = false` (`deck.js:455`). A mark's closing "plain", if it fires after the animation has come to rest (so after the final `dress()`), un-dims the unplayable cards with no later redecorate. Fix: have `decorate` leave `dim` untouched unless given (`dim = undefined`), or call `dress()` after each mark.
6. **Spurious `rested()` after `settle`.** `director.js:124` `settle()` calls `timeline.skip()`, which resolves the pending `idle()` promise. The `.then` at line 182 still matches the current token because `settle` does not bump `animations`, so `dress(); rested()` run once more after the direct restart. Harmless today because `rested()` is idempotent, but a stale `countUntil` makes it schedule the over-prompt wait twice. Fix: bump `animations` in `settle`.
7. **A stale gate `release` closure releases a later hold.** `director.js:78` `release` checks only `heldAt === null`. After `restart()` or `cancelTimed()` drops a queued gate, a tutorial close that still holds the old `release` will release a newer gate. Fix: give each gate its own token, and have `release` ignore a stale one.
8. **`restart()` without `dealt` returns `{ beats: {}, count: null }`.** `director.js:274` omits `landed` and `duration`, which `animate` always returns. A caller doing `timing.landed[k]` after a direct restart throws. `main.js` guards with `!timing`, so it is safe today. Fix: return the same shape.
9. **`screenPoint` reads the mesh's current pose, `handEdge` reads the placement's final pose.** Mid-animation `screenPoint` points at the card where it is now, not where it will rest. Document it, or use `m.pose`.
10. **Zero-size canvas gives a NaN camera.** `scene.js:136-141` `resize()` divides by `canvas.clientHeight` with no `|| 1` guard (the other paths have one). A hidden or collapsed canvas yields NaN or Infinity aspect, so the camera and `toScreen` break until the next resize. Fix: guard like `aspectNow`. (Unverified in a browser.)
11. **Upright framing with the strips overlapping.** `framing.js:186` `(up - down) / (top - foot)` is Infinity if `top == foot`, so fov goes to 180. Negative values are rescued by the `max` with the width term. Fix: `Math.max(top - foot, 0.05)`.
12. **Shadow plane and table gaps are comfortable only with 24-bit depth.** Near 20, far 400 gives a depth step of about 3e-5 cm at 100 cm, against the 0.005 shade gap and 0.012 table drop. A 16-bit depth buffer (some old mobile GPUs) gives about 0.008 cm and would bring the flicker back. Unverified; three requests 24-bit by default.
13. **The reveal tween starts from `performance.now()` but steps with the rAF timestamp** (`scene.js:175-179`, `250-253`). The first rAF timestamp can precede `start` slightly. `t` is clamped by the easing, so this is harmless; noted only.
14. **Timeline re-entrancy.** `timeline.js:39-49`: if an `onDone` calls `add()`, the array being iterated is pushed to and re-sorted mid-loop. Nothing does this now (the director's callbacks only reveal faces).

## Nits

- `units.js:7` still says "the table top is y = 0"; the table plane is now -0.012 (`scene.js:104`). `scene.js:108` says the shadow sheet is "below the lowest card", but a lying card's underside is at y = 0, below the sheet at 0.005. Visually harmless (the card is opaque above it); the comment is wrong.
- `deck.js:415-418`: one set of three materials per mesh per code ever shown, never disposed. Bounded (64 x 32 sets at worst, programs shared); fine for page life. Textures replaced by `setFaces` are kept in `faceSets` by design.
- `art.js:147-148`: a missing `back` leaves `undefined` and white backs, silently. `build.py` checks the count (33), so it is guarded at build time.
- `scene.js:208`: the resize listener is never removed (one scene per page; fine).
- `bin/gate` runs `cd web3d && node --test ../kit/test/ test/`; on Node versions where `--test` takes a directory differently this could run no kit tests, but the "ran no tests" check only guards a total of zero, not a missing kit half. Consider asserting the kit test count separately.

## Build and gate

- Stale-output detection is sound: `--check` builds in memory and compares bytes, wasm paths are remapped (`/bezique`, `/cargo`, `/rustup`) and RUSTFLAGS is replaced, not extended.
- Gaps: (a) `--check` rebuilds wasm, so the gate's cost and its result depend on the toolchain version; `rust-toolchain.toml` pinning is not visible in what I read (unverified). (b) `bin/gate` never checks the working tree is clean, so a page built from uncommitted sources passes `--check` and can still be committed alone with stale sources; run the gate on the committed tree or add `git diff --quiet -- web3d kit src crates` before the check. (c) The `fmt`/`clippy` outputs are discarded for fmt, so the failing file is not shown. (d) `game.toml` `count = 33` is a hard number; adding or removing art fails with a clear message, fine. (e) `build.py:269` sets `NODE_PATH` but esbuild receives no `--resolve-dir` guard if `kit/node_modules` is a dangling link; the README covers this.
- `kit/test/__pycache__` is present in the tree; confirm it is ignored by git.

## Checked and clean

- Picking: `pick()` uses `intersectObjects(meshes, false)` (ink child not hit) and resolves by `userData.id`, so twins are distinct; `main.js` uses `cardId`, so hover, dim, chosen and hint are per card, not per code.
- Per-mesh materials: dimming one twin leaves the other (`deck.js:415-458`); `reveal` and `decorate` use the same per-mesh lookup.
- Render-on-demand: `wake`/`frame`/`running` hold in the normal paths; the held clock resumes with the correct offset (`release`); `skip()` always ends with `wake()`; `manual` mode runs frames directly.
- Idle token (`animations`) correctly drops a superseded animation's rest.
- Ink hull: stays in the card plane, no longer coplanar with the table (table -0.012, hull at 0, shade at 0.005, wood top at -0.02); reveal far plane does not change the near plane, so precision at the pull-back is fine.
- `framing.js` desktop and upright maths: consistent with `scene.js` `placeRevealed` (same film-offset and view-offset formulas).
- `hud.js`: every deferred action goes through `later` and checks `epoch`; reset cancels the queue; no timers outside `later` by default except the `setTimeout` default, which the game overrides with `director.at`.
- `build.py`: placeholders replaced in one pass, `</script`/`</style` guarded, JSON `</` escaped, missing esbuild and cargo give one-line errors.

## Not checked

`kinematics.js` paths, `surfaces.js`, `faces.js` drawing, `dialogue.js`, `scoring.js` internals beyond the HUD contract, `kit/test/` contents (their coverage of items 1-4 was not read; none of 1-4 is likely covered), and anything in a real browser.
