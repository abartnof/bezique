// The choreography: how the cards get from one state to the next.
//
// A "state" is the page's own: { model, events, raw } -- the model of the
// table (model.js) and every event of the sitting so far. The events since
// the last state are replayed into intermediate models, each laid out by
// `layout`, so every stage runs between two true layouts; a final settle
// lands everything on layout(next.model) whatever the steps got wrong. A new
// sitting is played out from the pack; an undo is one direct transition.
//
// choreograph(prev, next, placement, view) -> { motions, placement, beats, count, marks, landed }
//
//   count      { at, lines: [ms], end } when a hand or the game is counted (handcount.js):
//              the table is held for it before the cards are collected
//
//   placement  where each of the 64 meshes is: [{ id, zone, index, code, cardId, faceUp, pose }]
//   motions    [{ id, path, delay, duration, reveal }]: `reveal` is the face a
//              mesh shows from the start of its motion, or { code: null,
//              atEnd: true }, the face it stops showing when it lands
//   beats      { eventIndex: ms }: when each new event is seen to happen
//
// The shape follows strohmandeln's choreography.js; the stages are this game's.
import { Vector3 } from "three";
import { layDown, pull, slide, toss, transfer } from "../../kit/js/kinematics.js";
import { countPlan } from "./handcount.js";
import { layout } from "./layout.js";
import { packModel, stagesBetween } from "./model.js";
import { ZONES } from "./zones.js";

// Starting points at normal speed, tuned by eye; the speed setting scales them all.
export const TIMING = Object.freeze({
  think: 420, // the opponent's pause before a move that follows another
  play: 560,
  resort: 280, // a hand closing up
  gather: 520, // a trick swept into its winner's pile
  gatherStep: 140,
  draw: 440,
  drawStep: 300,
  declare: 480,
  declareStep: 140,
  dealCard: 300,
  dealStep: 70,
  collect: 420,
  collectStep: 12,
  cut: 420,
  cutStep: 180,
  dix: 620,
  merge: 480,
  direct: 420,
  beat: 320,
  gap: 220, // between one stage and the next
});

const GAP = { cut: 900, deal: 500, hand_ends: 600, collect: 400 };
const BEAT = { first_dealer: 700, game_ends: 600, hand_ends: 900, drew: 0 };

const onTable = (pose) => {
  const n = new Vector3(0, 0, 1).applyQuaternion(pose.quaternion);
  return Math.abs(n.y) > 0.9 && pose.position.y < 3;
};
const ride = (to) => 0.3 + 4 * Math.max(0, to.position.y);
const samePose = (a, b) => a.position.distanceToSquared(b.position) < 1e-8 && a.quaternion.angleTo(b.quaternion) < 1e-4;

// ---- which mesh goes where --------------------------------------------------

// Where a card that has no face to follow comes from, by where it goes.
const SOURCES = {
  trick: ["their-hand", "their-known", "their-table", "your-hand", "your-table", "stock"],
  "their-table": ["their-hand", "their-known", "stock"],
  "their-known": ["their-hand", "turn-up", "stock"],
  "their-hand": ["stock", "turn-up", "their-table", "their-known"],
  "your-hand": ["stock", "turn-up", "your-table"],
  "your-table": ["your-hand"],
  "won-you": ["trick"],
  "won-them": ["trick"],
  "turn-up": ["stock", "your-hand", "their-hand"],
  "cut-you": ["stock"],
  "cut-them": ["stock"],
  stock: ["won-you", "won-them", "trick", "their-hand", "your-hand", "your-table", "their-table", "their-known", "turn-up", "cut-you", "cut-them"],
};

// Of the free meshes in a zone, the one to take: the middle of the
// opponent's fan, the top of anything else.
function pickFrom(meshes, zone) {
  if (zone === "their-hand") return meshes[Math.floor((meshes.length - 1) / 2)];
  return meshes.reduce((a, b) => (b.index > a.index ? b : a));
}

export function assign(prev, slots) {
  const free = new Map(prev.map((p) => [p.id, p]));
  const out = new Array(slots.length);
  const claim = (i, mesh) => {
    free.delete(mesh.id);
    out[i] = { ...slots[i], id: mesh.id };
  };
  // A face follows the mesh already showing it: in its own zone first, then anywhere.
  for (const sameZone of [true, false]) {
    slots.forEach((slot, i) => {
      if (out[i] || !slot.code) return;
      const options = [...free.values()].filter((m) => m.code === slot.code && (!sameZone || m.zone === slot.zone));
      if (options.length) claim(i, options.reduce((a, b) => (Math.abs(b.index - slot.index) < Math.abs(a.index - slot.index) ? b : a)));
    });
  }
  // Backs stay in their zone, in order, the surplus left over.
  const zones = new Set(slots.map((s) => s.zone));
  for (const zone of zones) {
    const want = slots.map((s, i) => [s, i]).filter(([s, i]) => !out[i] && s.zone === zone && !s.code);
    const have = [...free.values()].filter((m) => m.zone === zone && !m.code).sort((a, b) => a.index - b.index);
    const k = Math.min(want.length, have.length);
    // The surplus leaves from the middle of the opponent's fan, the top of a pile.
    const gone = have.length - k;
    const from = zone === "their-hand" ? Math.floor((have.length - gone) / 2) : k;
    const keep = [...have.slice(0, from), ...have.slice(from + gone)];
    for (let j = 0; j < k; j++) claim(want[j][1], keep[j]);
  }
  // Whatever is left comes from where such cards come from.
  slots.forEach((slot, i) => {
    if (out[i]) return;
    const rest = [...free.values()];
    for (const zone of SOURCES[slot.zone] ?? []) {
      const options = rest.filter((m) => m.zone === zone);
      if (options.length) return claim(i, pickFrom(options, zone));
    }
    claim(i, rest[0]);
  });
  return out;
}

// ---- the stages -------------------------------------------------------------

function motionFor(kind, from, to) {
  const reveal = to.code && to.code !== from.code ? { code: to.code } : !to.code && from.code ? { code: null, atEnd: true } : undefined;
  const a = from.pose;
  const b = to.pose;
  let path;
  let duration;
  if (from.zone === to.zone && kind !== "deal" && kind !== "collect") {
    path = slide(a, b, { lift: ride(b) });
    duration = TIMING.resort;
  } else if (kind === "play") {
    path = pull(a, b, { clearance: 3 });
    duration = TIMING.play;
  } else if (kind === "gather") {
    path = toss(a, b, { clearance: 2 });
    duration = TIMING.gather;
  } else if (kind === "draw" || kind === "merge") {
    path = transfer(a, b, { clearance: 2 });
    duration = kind === "draw" ? TIMING.draw : TIMING.merge;
  } else if (kind === "declare") {
    path = layDown(a, b, { clearance: 2.5 });
    duration = TIMING.declare;
  } else if (kind === "deal" || kind === "cut") {
    path = toss(a, b, { clearance: 3 });
    duration = kind === "deal" ? TIMING.dealCard : TIMING.cut;
  } else if (kind === "collect") {
    path = transfer(a, b, { clearance: 1.5 });
    duration = TIMING.collect;
  } else if (kind === "dix") {
    path = transfer(a, b, { clearance: 3 });
    duration = TIMING.dix;
  } else {
    path = onTable(b) ? toss(a, b) : transfer(a, b, { clearance: 1.5 });
    duration = TIMING.direct;
  }
  return { path, duration, reveal };
}

// When each mover starts, within its stage.
function delays(kind, movers, stage) {
  const out = new Map();
  const main = movers.filter((m) => m.from.zone !== m.to.zone || kind === "deal" || kind === "collect");
  movers.filter((m) => !main.includes(m)).forEach((m) => out.set(m.to.id, 0));
  if (kind === "deal") {
    const dealer = stage.model.dealer ?? "them";
    const lists = { you: main.filter((m) => m.to.zone === "your-hand").sort((a, b) => a.to.index - b.to.index), them: main.filter((m) => m.to.zone === "their-hand").sort((a, b) => a.to.index - b.to.index) };
    const pone = dealer === "you" ? "them" : "you";
    let n = 0;
    for (const round of [3, 2, 3]) {
      for (const who of [pone, dealer]) {
        for (const m of lists[who].splice(0, round)) out.set(m.to.id, n++ * TIMING.dealStep);
      }
    }
    for (const m of main) if (!out.has(m.to.id)) out.set(m.to.id, n++ * TIMING.dealStep + TIMING.dealStep);
    return out;
  }
  const step = { gather: TIMING.gatherStep, draw: TIMING.drawStep, declare: TIMING.declareStep, collect: TIMING.collectStep, cut: TIMING.cutStep, merge: 60, dix: 200 }[kind] ?? 0;
  let ordered = main;
  if (kind === "draw") ordered = [...main].sort((a, b) => Number(b.to.zone.includes(stage.winner === "you" ? "your" : "their")) - Number(a.to.zone.includes(stage.winner === "you" ? "your" : "their")));
  ordered.forEach((m, i) => out.set(m.to.id, i * step));
  return out;
}

class Run {
  constructor(placement) {
    this.now = placement;
    this.motions = [];
    this.clock = 0;
    this.beats = {};
    this.count = null;
  }

  // The table held while a hand's (or the game's) count is told: its steps
  // at their times, and the stage that follows starts when it is over.
  hold(plan) {
    const start = this.clock;
    const lines = plan.times.map((t) => start + t);
    this.count = this.count ? { ...this.count, lines: [...this.count.lines, ...lines], end: start + plan.end } : { at: plan.at, lines, end: start + plan.end };
    this.clock = start + plan.end;
  }

  step(kind, slots, stage = {}) {
    const next = assign(this.now, slots);
    const before = new Map(this.now.map((p) => [p.id, p]));
    const think = kind === "play" && stage.event?.who === "them" ? TIMING.think : 0;
    const start = this.clock + think;
    const movers = next.filter((to) => !samePose(before.get(to.id).pose, to.pose) || before.get(to.id).code !== to.code).map((to) => ({ from: before.get(to.id), to }));
    const when = delays(kind, movers, stage);
    let end = start;
    for (const { from, to } of movers) {
      const { path, duration, reveal } = motionFor(kind, from, to);
      const delay = start + (when.get(to.id) ?? 0);
      this.motions.push({ id: to.id, path, delay, duration, reveal });
      end = Math.max(end, delay + duration);
    }
    if (stage.at != null) this.beats[stage.at] = start;
    this.now = next;
    if (movers.length) this.clock = end + (GAP[kind] ?? TIMING.gap);
    else if (kind !== "settle") this.clock = start + (BEAT[kind] ?? TIMING.beat);
  }

  result() {
    return { motions: this.motions, placement: this.now, duration: this.clock, beats: this.beats, count: this.count, marks: [], landed: {} };
  }
}

const layoutOf = (model, view = {}) => layout(model, { zones: view.zones ?? ZONES, chosen: view.chosen ?? null });

// The meshes placed straight onto a layout, the first time a state is seen.
export function initialPlacement(state, view = {}) {
  return layoutOf(state.model, view).map((slot, id) => ({ id, ...slot }));
}

// Before the first deal of a sitting: the whole pack squared. Choreographed
// from this to a new sitting's first state, the opening is played out.
export function opening() {
  return { model: packModel(), events: [] };
}

export function choreograph(prev, next, placement, view = {}) {
  const run = new Run(placement);
  const stages = stagesBetween(prev, next);
  if (stages && stages.length) {
    for (const stage of stages) {
      const plan = stage.kind === "hand_ends" || stage.kind === "game_ends" ? countPlan(next.events, stage.at) : null;
      if (plan && plan.steps.length) run.hold(plan);
      run.step(stage.kind, layoutOf(stage.model), stage);
    }
    run.step("settle", layoutOf(next.model, view));
  } else {
    run.step(stages ? "settle" : "direct", layoutOf(next.model, view));
  }
  // The final placement is the state's own, with its ids.
  return run.result();
}
