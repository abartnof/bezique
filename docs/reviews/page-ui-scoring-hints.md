# Review: page UI, scoring ledger, hand count, hints, tracker, tutorial

Scope: web3d/src/{ui,hinttext,tutor,tutorview,tracker,trackerview,scoreboard,scoring,handcount}.js, tutorial.md, tests. Read-only, time-boxed; nothing was run. badges.js, talk.js, speaker.js, the CSS files and most tests were not read (unverified). Hint params were read against crates/bezique-core/src/hints.rs and docs/RULES.md.

## Findings, ranked

### Bugs (hint wording false for the position)

1. **throw_lowest says "You will not win this trick" when the player can.** hinttext.js:376 with hints.rs:380-411. In the stock phase `classify_play` falls through to `throw_lowest` whenever a card is led, the played card is not a winner and the led card is not a brisque, even if `winners` is non-empty (nothing forces a win before the last eight). Scenario: opponent leads a king, you hold the ace of the suit, and the rung plays a nine: the hint says you will not win. The rule line ("When you will not win a trick...") is then also false. Fix: send a param (`can_win`) from hints.rs, or reword the first sentence to "You are not taking this trick, so give up...".
2. **let_brisque_go names an arbitrary winner and asserts a value that was not computed.** hinttext.js:371 with hints.rs:381-386. `winner` is `winners[0]`, the first in legal order, not the cheapest or the dearest. "Worth more than the 10 points" is never checked. Also ignores that the trick may hold 20 (follower's own brisque). Fix: pass the cheapest winner and reword to "may cost your X".
3. **keep_marriage / keep_four can name the played card as the kept card's partner or support.** hints.rs:271 `work()` is searched over other legal cards of rank <= the played card's. Scenario: hold K♥, Q♥ (undeclared), rung plays K♥ (forced follow). The hint reads "Play the king of hearts and keep the queen of hearts: with your king of hearts it is a marriage". For keep_four, playing one of three kings and "keeping" another gives "you hold three of a kind" when the play breaks it. Fix: in `work`, count the hand without the played card, and skip `kept` candidates that share the played card's partner type or rank.
4. **last_eight_lead_master: the rule contradicts the do-line for plain suits.** hinttext.js:471-474. Do-line says "only a trump can take it" (correct); rule says "If nothing can, lead it: the trick is yours". Fix: rule "...if only a trump can beat it, weigh whether they hold one."
5. **draw_out_trump: rule says "lead your highest", engine does not check it, source note says Cady leads the lowest.** hinttext.js:492-496, hints.rs:296-301. Scenario: three trumps, rung leads the lowest: do-line fine, rule false, the cited source contradicts the rule. Fix: reword the rule to match what the rung does and the book.

### Risks

6. **lead_master_brisque overclaims.** hinttext.js:393 "the 10 it carries is yours": a trump from the opponent takes it and the 10 with it. For an ace "no higher card of its suit is left out" is vacuous. Fix: "...and the 10 it carries is yours unless they trump it."
7. **take_sparing_pieces says "part of a combination you hold".** hinttext.js:365. `work()` also fires for a half bezique, the trump seven (dix) and a prospective four, not only held combinations. Also names one `kept` when several cheaper winners exist. Fix: "worth keeping for a combination".
8. **lead_low_nonbrisque claims "it is low".** hinttext.js:381. The test is only "no legal card costs less"; with a hand of kings the card is a king. Fix: "the cheapest you can play".
9. **solved_endgame count may be wrong with twins** (hints.rs:248 `theirs.cards().union(view.unseen())`). If union is a set union rather than a multiset sum, a twin type present in both is counted once. Unverified; check the type of `union`.
10. **Tracker silently clamps with `Math.max(0, ...)`** (tracker.js:51). A double count of any card, for example a table card also present in `tricks_played` or `trick`, shows as a plausible lower number instead of failing. Fix: have the model return a `conflict` flag when `seen > 2`, and test it over whole games (the unseen total must equal stock + their_hidden at every state; the line at tracker.js:64 prints both, so a test can assert it).
11. **Scoreboard `event()` returns early when `instant()`** (scoreboard.js:296), skipping `hud.dealt`/`endHand`. `rest()` repairs only when totals differ (scoreboard.js:331). A fast-forwarded span that scores nothing (a deal with no turned seven) leaves the HUD without the new hand's block. Also a mid-hand `game_ends` never calls `hud.endHand`; reset covers only the rest-time case. Unverified against kit/js/hud.js. Fix: compare hand count as well as totals in `rest`.
12. **Count timing coupling.** scoreboard.js:312 flattens `plansSince(events, first)` and indexes `timing.lines[k]`; choreography must compute the same plans from the same `first`. A mismatch shifts every badge by a step with no error (a skipped `undefined` is ignored at scoreboard.js:315). Add a test that the step count equals `lines.length` for a played game.
13. **Tutor keydown capture does not stop propagation** (tutorview.js:72-88). While a tip is open, game hotkeys (if main.js has any; unverified) still receive letters and digits. Fix: `stopPropagation` for all keys other than the handled ones, or mark the table `inert`.
14. **Dialogs: background not inert/aria-hidden; the "over" modal (ui.js:184) has no role=dialog, no focus move, and is rebuilt on every `setPrompt`.** The ticker (ui.js:34) and hint line have no `aria-live`, so errors and hints are silent for screen readers. Fix: `role="status"` on the ticker, `role="dialog"` plus focus on Play again.
15. **Menu: document-level `pointerdown`/`keydown` listeners** (ui.js:56-57) are added in `createUi`. Clean as long as `createUi` is called once; main.js:143 is the only call. They are never removed, so a second call (hot reload, tests) would stack. Escape closes the menu but does not return focus to the button.
16. **Tutorial "Cards out ... lists what has been played"** (tutorial.md:25). The panel lists what is still unseen. Reword.
17. **Tutorial double-game page: "counts for two"** (tutorial.md:89). docs/RULES.md says only "a double game"; no scoring multiple is stated. Say only what the rules say, or add the rule. The page also fires when anyone reaches 500, which is early but harmless.

### Nits

- hinttext.js:356 take_brisque rule "10 for you and 10 less for your opponent" is loose; "10 to you rather than to them".
- hinttext.js:481 last_eight_lead_ace rule says "a high plain-suit card"; the engine only fires for aces.
- hinttext.js:358 cites measurements/skills-final-style.txt for "about 2.5 points a hand"; I could not find that file in measurements/ listing (unverified, listing was truncated). Check it exists and says 2.5.
- tutor.js:196-197 `table` and `first-trick` share one condition; fine, ordered by PAGE_KEYS.
- ui.js:101 phase 2 line "else trump if able" omits "hold the suit but cannot beat it: any card of the suit".

## Tests that assert too little

- scoring.test.js:31 compares `run.hand` to `hand_scores` only when present, and the totals check runs after each batch, so offsetting errors between brisques, last trick and seven cannot be seen. The ledger test (line 55) sums all categories, same blindness. Add per-category assertions: for each hand, `last` is 10 for exactly the player who won the 32nd trick, `seven` only 10 per dealt-seven/dix event, and brisque points are multiples of 10. A hand-built fixture covers the split once (line 68) but not a game ending on the last trick, nor a seven turned up that ends the game.
- No test of `hinttext.js` REASONS against the position (the tests check strings exist). Suggested: for throw_lowest, assert the engine marks `can_win` false.

## Checked and clean

- scoring.js attribution: `took_trick.points` minus 10 only when `hand_ends` follows before any play/deal/game_ends; engine emits `hand_ends` before `game_ends` (session.rs:947-985), so a last-trick win that ends the game is still attributed correctly, and a mid-hand game end correctly gets no last-trick ten.
- Dealer's seven and dix at 10 each match RULES section 2/4.
- handcount.js: order is category then event order; `counted` guard prevents counting a hand twice when `game_ends` follows `hand_ends`; HUD ticks come only from `event()` (scoreboard.js:299), the count calls `badges.*` only, so no double ticks.
- Tutorial text against RULES: rank order, 3 sections of tie-break, brisques, one declaration per trick, carrying, dix, fours values, sequence extension, last-stock-trick, last eight, 1000 mid-hand: all agree.
- tutorview.js: its document keydown handler is removed in `close` and `open` closes the prior modal first; no leak. Focus return and checkbox Space/Enter handling are sound.
- Tracker arithmetic: 16 brisques, 16 trumps, per-type max 2 are correct for two packs; bezique line counts QS/JD twins correctly.
- Hint wording checked and correct against the rules: take_to_declare, win_last_stock_trick (stock==1 matches "one face-down card and the turn-up"), declare_now, score_carried, single_before_double, royal_before_sequence, exchange_dix, show_dix, last_eight_must_win, last_eight_give_up (legal set is only winners when any can win, so "cannot win" is true).

## Unverified

pass wording when declarations were legal; the CSS layering (z-index/pointer-events) and white-on-black badge contrast (badges.css not read); badges.js, talk.js, speaker.js, words.json/personas.json usage and their tests; engine `union` semantics; kit hud `endHand` behaviour.

## Resolution (UI)

- **10 fixed.** `trackerModel` returns `conflict` when a card is seen more than twice and the panel adds a "cannot be trusted" line. The whole-game test (unseen total equals stock + their_hidden at every state) already existed; it now also asserts `conflict` is false, and a unit test covers the flag.
- **11 fixed.** `rest()` also compares the HUD's hand count (blocks and counted hands) with the events, not only the totals, and resets when they differ; `reset()` marks a hand live only until its `hand_ends`, so a state between hands does not reset repeatedly. Checked against kit/js/hud.js: `dealt` adds the live block and `endHand` marks it counted; instant play calls neither. Mid-hand `game_ends` is covered by the same comparison (done: all hands counted). Not exercised in a browser (DOM and three needed).
- **12 partly already covered, hardened.** handcount.test.js already asserts, over played games, that the choreography's held lines equal `plansSince(events, prev.events.length)` steps. `count()` now warns when the steps and timed lines differ, rather than shifting badges silently.
- **Tests that assert too little: fixed.** scoring.test.js checks every category of every hand against figures worked out from the raw events (brisques, declared, seven, last), one last-trick ten per counted hand, and adds cases for a game ending on the last trick, a game ending mid-hand, and a seven turned up that ends the game. The REASONS-versus-position test belongs with the hint wording.
- **13 not a bug.** main.js, ui.js and the pages register no game hotkeys; the only document key listeners are the menu's Escape and the tutor's own capture. Nothing receives letters or digits behind a tip. No change.
- **14 fixed, partly.** The game-over dialog has `role=dialog`, `aria-modal`, a labelled title, takes focus on Play again, is built once per outcome (not on each `setPrompt`), and makes the HUD, bar and menu `inert` while shown. The ticker has `role=status` and the hint line `aria-live=polite`. Tutorial and New game dialogs already trap focus. Not checked with a screen reader.
- **15 fixed.** The menu's document listeners are removed if `createUi` runs again; Escape returns focus to the Menu button when the menu was open.
- **ui.js phase-2 line fixed:** now states the same-suit, any-card rule from RULES section 6.

## Resolution

Hint items (engine tests in `crates/bezique-core/src/hints_tests.rs`, wording tests in `web3d/test/hinttext.test.js`). Items 10-15 and the UI nits belong to other owners and are not addressed here.

1. Fixed by rewording: "You are not taking this trick" (the engine does not force a win in the stock phase); the rule and `docs/HINTS.md` say the same.
2. Fixed. The winner named is the cheapest winner; the value claim is removed.
3. Fixed. The card played is excluded from the hand count and from the combination the kept card serves.
4. Fixed. The rule no longer says the trick is yours; the engine fires the plain master lead only when the opponent cannot trump it.
5. Fixed. `draw_out_trump` leads the highest trump, as the rule says; the book's different advice stays noted as such in the source line.
6. Fixed ("unless they trump it").
7. Fixed ("worth keeping for a combination").
8. Fixed ("the cheapest card you can lead").
9. Not a bug: `CardSet::union` is a bitwise or with one bit per physical copy; a test records it.
16. Fixed in `web3d/tutorial.md`: Cards out counts the cards not yet seen.
17. Fixed: the page says only "double game", as `docs/RULES.md` does.
Nits: take_brisque and last_eight_lead_ace rules reworded. `measurements/skills-final-style.txt` exists but holds no 2.5 figure (+1.79 and +18.59 for other comparisons), so the figure is removed from the hint and `docs/HINTS.md`; the rule is marked as not measured on its own.
Test gap: a `can_win` assertion is unnecessary after the rewording; the wording test asserts the claim is gone.
