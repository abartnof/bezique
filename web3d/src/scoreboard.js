// The score as the page shows it: the kit's HUD (ledger by hand), the notes
// under it, the badges on the table, the count after a hand, and the cards-out
// panel. main.js feeds it the engine's events at their beats and its state at
// rest; the figures come from scoring.js, which the tests hold to the engine's
// own totals.
import { Vector3 } from "three";
import { createHud } from "../../kit/js/hud.js";
import { createBadges } from "./badges.js";
import { plansSince } from "./handcount.js";
import { apply, carriedNote, doubleNote, fold, hudEvent, hudGame, initial, itemsOf, ledgerOf } from "./scoring.js";
import { trackerModel } from "./tracker.js";
import { createTrackerView } from "./trackerview.js";

// `ui`: createUi's; `director`: the kit's; `zones()`: the table's zones now; `overlay`: the page's overlay element.
export function createScoreboard({ ui, director, zones, overlay, reduced = () => false }) {
  let run = initial();
  let upto = 0; // events folded so far: a late beat for one already shown (after a reset) is ignored
  let trackerView = null;
  const speed = () => director.timeline.speed;
  const instant = () => speed() > 10;

  const hud = createHud(ui.hudSlot, hudGame, {
    later: (ms, fn) => director.at(ms, fn),
    reduced,
    room: () => Math.max(0, ui.barTop() - ui.hudSlot.getBoundingClientRect().top - 60),
  });

  const point = (anchor) => {
    const z = zones();
    const at = {
      trick: [z.trick.x, 0.5, z.trick.z],
      stock: [z.stock.x, 1, z.stock.z],
      yourTable: [0, 1, z.yourTable.z],
      theirTable: [0, 1, z.theirTable.z],
      wonYou: [z.won.x, 0.5, z.won.you],
      wonThem: [z.won.x, 0.5, z.won.them],
    }[anchor];
    return at ? director.toScreen(new Vector3(...at)) : null;
  };
  const badges = createBadges(overlay, { later: (ms, fn) => director.at(ms, fn), speed, project: (anchor) => point(anchor), reduced });

  // The HUD shown at once from the events so far: ended hands, and the live one.
  function reset(events, { empty = false } = {}) {
    run = empty ? initial() : fold(events);
    upto = empty ? 0 : events.length;
    const all = ledgerOf(run);
    const numbers = Object.keys(run.hands).map(Number).sort((a, b) => a - b);
    const live = !run.done && numbers.length > 0 && all.length > 0;
    hud.reset({ hands: live ? all.slice(0, -1) : all, live: live ? all.at(-1) : null });
    badges.clear();
  }

  // One event told, at its beat: the running figures, the HUD's popup, the
  // badge on the table, the ledger's hand closed when a hand ends.
  function event(events, i) {
    if (i < upto) return;
    upto = i + 1;
    const e = events[i];
    const items = itemsOf(events, i);
    run = apply(run, e, items);
    ui.setNotes({ double: doubleNote(run.totals) });
    if (instant()) return; // rest() brings the HUD to the engine's figures at once
    if (e.kind === "dealt") hud.dealt(e.hand);
    items.forEach((it, nth) => {
      hud.score(hudEvent(it));
      badges.show(it, { nth });
    });
    if (e.kind === "hand_ends") hud.endHand(e.hand);
  }

  // The count a hand's end or the game's brings (handcount.js), told unit by
  // unit on the director's clock at the times the choreography held the table
  // for (`timing.lines`, ms from now): a badge a unit, the hand's total, the
  // doubling, then the victory. `first` is the index of the advance's first
  // new event.
  function count(events, first, timing) {
    if (!timing || instant()) return;
    const steps = plansSince(events, first).flatMap((p) => p.steps);
    steps.forEach((step, k) => {
      const ms = timing.lines[k];
      if (ms === undefined) return;
      director.at(ms, () => {
        if (step.kind === "unit") badges.unit(step);
        else if (step.kind === "total") badges.total(step);
        else if (step.kind === "double") badges.double();
        else if (step.kind === "victory") badges.celebrate({ winner: step.winner, double: step.double }, step.totals);
      });
    });
  }

  // The table at rest: the HUD matches the engine's figures (a popup still
  // playing is left to finish), the notes and the cards-out panel are redrawn.
  function rest(raw) {
    const totals = raw.totals;
    if (totals) {
      const t = hud.totals();
      if (hud.idle() && (t.you !== totals.you || t.opp !== totals.them)) reset(raw.events);
      ui.setNotes({ double: doubleNote(totals), carried: carriedNote(raw) });
    }
    ui.setTracker(Boolean(raw.aids?.tracker));
    if (raw.aids?.tracker && raw.hand) {
      if (!trackerView) trackerView = createTrackerView(ui.trackerRoot);
      trackerView.set(trackerModel(raw));
    } else {
      trackerView = null;
      ui.trackerRoot.textContent = "";
    }
  }

  return { hud, badges, reset, event, count, rest, running: () => run };
}
