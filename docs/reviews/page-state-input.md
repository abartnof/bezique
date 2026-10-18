# Review: web3d page state, input, persistence

Scope: web3d/src/{main,model,layout,zones,choreography,cards,newgame,settings,prefs,tutorstore}.js. Read-only. Tests: `node --test ../kit/test/ test/` passes (296 of 296).

## Bugs

1. **Tracker toggle wipes every saved preference.** main.js:392 calls `savePrefs()` with no argument. `savePrefs(undefined)` writes `{v:1}` (prefs.js:127), so level, persona, surface, speed, talk, tutor and the rest revert to defaults on next load. Scenario: press the table's Tracker button, reload. Fix: `savePrefs(prefs)`. Add a test that `savePrefs()` without prefs does nothing or throws.

2. **Settings "Card faces" does nothing until reload.** `changed()` (main.js:299-305) saves `prefs.faces` but never calls `setFaces`; the only callers are the start-up line 97 and the test hook at 441. Fix: `else if (key === "faces") setFaces(value)`. Check `talk` and `tutor` too: `talk` is saved but `talk.setMode` is only reached through `setTalk`, so confirm the gear panel routes to it (unverified).

3. **Hints/tracker toggle during animation releases the "waiting" flag.** `setHints` (main.js:307-312) and the `tracker` branch (390-396) call `rested()`, which sets `waiting = false`, `setPrompt(busy:false)` and redecorates while cards are still moving or a tutorial gate holds the clock. Scenario: opponent's move animating, press Hints: the person can click a card and `act()` runs mid-animation, and `present()` then restarts the director over a half-played motion. Fix: refresh the prompt and decoration only when `!waiting`, or have both toggles update `raw`, set a `dirty` flag, and let the director's own `rested` apply it.

4. **Toggles replace `raw` with an unchecked reply.** main.js:309 and 393 assign `raw = engine.send(...).state` without testing `ok`. A refused command makes `raw` the error object, so `raw.prompt` is undefined and the table is dead. Fix: test `r.ok`, otherwise keep `raw`.

5. **`onAct` before the first deal crashes.** While the opening New game dialog is up (`raw === null`), `tracker` and `hints` dereference `raw.aids` (main.js:391, 398) and `undo` calls the engine with no game. Fix: `if (!raw) return` for these names (except `new`, `settings`, `help`). Also, `onAct("new")` during the opening dialog calls `open()` again with `opening=false` (newgame.js:128-129), which clears `first`; Cancel/Escape then closes without dealing and the page has no game. Fix: in `open`, keep `first ||= opening`, or return when already open.

6. **A corrupt saved sitting stops the page.** `restore()` (main.js:337-357) assumes `saved.record` is an array and `saved.level` is valid. A parsable but wrong value (`record` missing, `null` after `JSON.parse`) throws, `main()` catches it, and the page shows "could not start" every load, since nothing clears the key. Fix: wrap the body in try/catch, validate `Array.isArray(saved.record)`, and `store.set(SITTING, null)` on failure. Also a failed restore leaves `prefs.level/persona/erratic/playForced` and `seed` already overwritten (lines 340-347); restore them or assign after success.

## Risks

7. **Stale tutorial state after `cancelTimed`.** `cancelTimed()` clears the queue and calls `release()` (kit director.js:336). If a tutorial page is open and the person reaches New game (dialog opened via the Menu; whether the tutorial backdrop blocks it is unverified), `newGame` -> `present` skips `tutorial()` because `tutorView.isOpen()` (main.js:185), and "Got it" then runs the old `afterPage` and its stale `release`. Fix: on `newGame` and `undo`, close the tutor view and null `afterPage`, or make New game unavailable while a page is open.

8. **Game-over count wait is not cancelled by a restore or undo.** `countUntil` (main.js:216) is only reset in the non-direct path of `present` (line 290). It is harmless today (restore refuses an "over" state, undo is blocked while waiting), but a new path that presents an "over" state directly would wait on an old value. Fix: reset `countUntil = 0` at the top of `present`.

9. **Keyboard focus is lost in the New game dialog.** Each option click calls `render()` (newgame.js:94-97), which empties the card, so focus drops to `body`; the Escape handler is on `root` and no longer sees the key. Same in settings.js:63-69 and `choose`. Fix: re-focus the button with the same `data-key` and value after render, or update `aria-checked` in place.

10. **Hover stays set after the pointer leaves the canvas or after a play.** `hover` is only reset by `pointermove` (main.js:409-417), and the handler returns early when `waiting`. A card stays highlighted (and `cursor: pointer`) after the pointer leaves or after the card moves. Fix: clear `hover` in `present()` and on `pointerleave`.

11. **Click on an illegal card calls `act`** (main.js:428-430) and prints the engine's refusal. If the engine ever accepts (a rules change), `legal()` and the engine disagree silently; the design is deliberate but untested.

12. **Double `startAids` after `engine.new`.** `gameConfig` already passes hints, tracker and playForced, but `engine.new` in engine.js:15 ignores them (it destructures level, seed, persona, erratic only), so the `startAids` calls are what apply them. Fine, but `playForced`, `hints` and `tracker` in `gameConfig` are dead arguments; document or remove. Restore (lines 344-347) sets aids from the saved final value and then replays a record that may include `set hints` commands; order is consistent but unverified for a record that toggles hints.

13. **`changed("hints"/"tracker")` writes new-game prefs** (main.js:300, 391): toggling Hints mid-game changes the default for the next game. Probably intended ("table's own buttons"); state it in prefs.js.

## Nits

- main.js:392 `prefs.tracker` is set before the engine reply is known (see 4).
- main.js:412-415 `cursor` is only updated when the hovered id changes, so it is stale after legality changes (a turn passing) until the pointer moves.
- newgame.js:38 `Math.round(v/step)*step || 0`: a `v` of NaN gives 0, fine; a step of 0 would divide by zero, but no spec has one.
- prefs.js:91 `migrate` maps an unversioned record to `{level, hints}`; `hints` is now a new-game pref and carries over, which matches the intent.
- model.js:36-40 `won` is derived as the remainder, so any miscount in `elsewhere` shifts cards into the won piles silently; `conserve()` covers 64 total only.

## Checked and clean

- Layout holds exactly 64 cards: `conserve()` trims from stock/won piles and pads the stock (layout.js:140-150). Model events cannot cause a row with `c[1]` mis-split: codes are rank-suit two characters ("TD"), so the ten is not an issue (cards.js).
- Opening dialog paths: Deal sets `first=false` before closing, so no double deal; Cancel/Escape on the opening dialog deals the last choices once; Cancel on a mid-game dialog does nothing; `?seed` and `?fresh` skip the dialog and `url.seed` is used only for the first deal (main.js:327).
- Sitting save: `persist()` is called after each `act`, undo, new game and toggle; storage access is wrapped in try/catch in `store`, `loadPrefs`, `savePrefs`, `createTutorStore`; a newer-version prefs record is not overwritten (prefs.js:122).
- Prefs `sanitise` takes the default for any invalid field; `withUrl` validates against the spec.
- `act()` and the undo path both refuse while `waiting` or `stopped`; `present()` always sets `waiting = true` and every non-direct path ends in `rested()` via the director.
- Listeners: the `canvas` and `root` listeners are added once; the dialog and settings re-render by replacing `textContent`, with handlers as properties (no accumulation). No per-render timers in the scoped files.
- Choreography: `assign` claims every slot, falling back to `rest[0]`; event kinds without a case in `reduce` fall through to a copy, and the final `settle` step lands on the state's own layout.

## Unverified

- Whether the tutorial or settings backdrop blocks the Menu's New game button (item 7), and whether `ui.js` shortcuts can reach `onAct` with `raw === null` (item 5).
- Portrait/landscape switch: `director` receives `ZONES` at creation while `view()` supplies `zonesNow()`; correct if the kit's `relayout` uses `view().zones`, not checked.
- Row overlap at maximum counts in portrait (`yourTable` width 34, pitch 4.2) was not measured; layout.test.js covers rows, but not checked against the portrait arrangement.
