// The layout: where all 64 cards rest for a model, and which show their
// faces. A pure function of the model and of how the person asks to see it
// (the card chosen).
//
// layout(model, view) -> 64 slots { zone, index, code, cardId, faceUp, pose }
//
// `code` is a card only where the person may know it AND the card is drawn
// face up; the opponent's hidden hand, the stock and the won piles are
// backs. `cardId` is the engine's id, for the person's own cards.
import { Vector3 } from "three";
import { fan, lying, row } from "../../kit/js/kinematics.js";
import { CARD } from "../../kit/js/units.js";
import { byHand, PACK } from "./cards.js";
import { ZONES } from "./zones.js";

const DEG = Math.PI / 180;
export const CHOSEN_LIFT = 1.6;
// A pile is drawn thicker than its cards are (the eye looks almost straight
// down), so a stock of fifty reads as a pile and not as one card.
export const PILE_STEP = 0.11;
const STEP = PILE_STEP;
// Between one suit and the next in a row of table cards, so a declared group
// (a marriage, a sequence) reads as a group.
export const SUIT_GAP = 1.4;

// A small fixed turn for each card, so piles look placed by a hand.
export function jitter(key, degrees = 3) {
  let h = 2166136261;
  for (const ch of String(key)) h = Math.imul(h ^ ch.charCodeAt(0), 16777619);
  return ((h >>> 0) / 4294967295 - 0.5) * 2 * degrees * DEG;
}

const toward = (centre, side) => centre.clone().add(new Vector3(0, 0, 300 * side));
export const spacingFor = (n, spacing, width) => (n < 2 ? spacing : Math.min(spacing, (width - CARD.width) / (n - 1)));

function pile(zone, count, at, { faceUp = false, from = 0, key }) {
  return Array.from({ length: count }, (_, i) => ({
    zone,
    index: i,
    pose: lying({ x: at.x, z: at.z, height: (from + i) * STEP, faceUp, yaw: jitter(`${key}${i}`) }),
  }));
}

// A row of cards lying face up, centred on the zone, a little apart suit from
// suit, closing up when there are many (each card's
// index strip stays clear of its neighbour: layout.test.js).
export function rowOffsets(codes, zone) {
  const n = codes.length;
  const breaks = codes.map((c, i) => (i > 0 && c[1] !== codes[i - 1][1] ? 1 : 0));
  const nBreaks = breaks.reduce((a, b) => a + b, 0);
  const room = zone.width - CARD.width;
  const gap = nBreaks ? Math.min(SUIT_GAP, Math.max(0, room - (n - 1) * zone.pitch) / nBreaks) : 0;
  const pitch = n < 2 ? zone.pitch : Math.min(zone.pitch, (room - gap * nBreaks) / (n - 1));
  let x = 0;
  const at = codes.map((_, i) => (x += (i ? pitch : 0) + breaks[i] * gap));
  const mid = (at[n - 1] ?? 0) / 2;
  return at.map((v) => v - mid);
}

function lyingRow(cards, zone, name) {
  const offsets = rowOffsets(cards.map((c) => c.code), zone);
  return cards.map((c, i) => ({
    zone: name,
    index: i,
    code: c.code,
    cardId: c.id,
    faceUp: true,
    pose: lying({ x: zone.x + offsets[i], z: zone.z, height: i * 0.004, faceUp: true }),
  }));
}

function yourHand(cards, view, Z) {
  const zone = Z.yourHand;
  const n = cards.length;
  if (!n) return [];
  const centre = new Vector3(...zone.centre);
  let poses;
  if (zone.fan) {
    const s = spacingFor(n, zone.spacing, zone.width);
    const step = s / (zone.radius + 3);
    const angles = Array.from({ length: n }, (_, i) => (i - (n - 1) / 2) * step);
    poses = fan({ count: n, angles, centre, facing: toward(centre, 1), radius: zone.radius, tilt: zone.lean * DEG });
  } else {
    const pitch = n > 1 ? Math.min(zone.pitch, (zone.width - CARD.width) / (n - 1)) : zone.pitch;
    poses = row({ count: n, centre, facing: toward(centre, 1), gap: pitch - CARD.width, tilt: zone.lean * DEG });
    poses.forEach((pose, i) => pose.position.addScaledVector(new Vector3(0, 0, 1).applyQuaternion(pose.quaternion), i * 0.06));
  }
  return cards.map((c, i) => {
    const pose = poses[i];
    if (view.chosen != null && c.id === view.chosen) pose.position.addScaledVector(new Vector3(0, 1, 0).applyQuaternion(pose.quaternion), CHOSEN_LIFT);
    return { zone: "your-hand", index: i, code: c.code, cardId: c.id, faceUp: true, pose };
  });
}

function theirHand(n, Z) {
  const zone = Z.theirHand;
  if (!n) return [];
  const centre = new Vector3(...zone.centre);
  const s = spacingFor(n, zone.spacing, zone.width);
  const step = s / (zone.radius + 3);
  const angles = Array.from({ length: n }, (_, i) => (i - (n - 1) / 2) * step);
  return fan({ count: n, angles, centre, facing: toward(centre, -1), radius: zone.radius, tilt: -zone.lean * DEG }).map((pose, i) => ({ zone: "their-hand", index: i, code: null, faceUp: false, pose }));
}

export function layout(model, view = {}) {
  const Z = view.zones ?? ZONES;
  const slots = [];
  const sorted = (cards) => [...cards].sort(byHand(model.trump));

  slots.push(...yourHand(sorted(model.hand), view, Z));
  slots.push(...lyingRow(sorted(model.table), Z.yourTable, "your-table"));
  slots.push(...theirHand(model.hidden, Z));
  slots.push(...lyingRow(sorted(model.known.map((code) => ({ code }))), Z.theirKnown, "their-known"));
  slots.push(...lyingRow(sorted(model.theirTable), Z.theirTable, "their-table"));

  // The open trick: the lead on the left, the answer beside it.
  model.trick.forEach((t, i) => {
    const x = Z.trick.x + (i === 0 ? -1 : 1) * (CARD.width / 2 + Z.trick.gap / 2);
    slots.push({ zone: "trick", index: i, code: t.code, faceUp: true, pose: lying({ x, z: Z.trick.z, height: 0, faceUp: true, yaw: jitter(`t${i}${t.code}`, 4) }) });
  });

  // The stock, with the turn-up lying crosswise beneath it.
  const at = Z.stock;
  if (model.turnUp) slots.push({ zone: "turn-up", index: 0, code: model.turnUp, faceUp: true, pose: lying({ x: at.x, z: at.z, height: 0, faceUp: true, yaw: Math.PI / 2 + jitter("up", 2) }) });
  slots.push(...pile("stock", model.stock, at, { from: model.turnUp ? 1 : 0, key: "s" }));

  if (model.cut) {
    for (const who of ["you", "them"]) slots.push({ zone: `cut-${who}`, index: 0, code: model.cut[who], faceUp: true, pose: lying({ ...Z.cut[who], height: 0, faceUp: true, yaw: jitter(`c${who}`, 4) }) });
  }

  slots.push(...pile("won-you", model.won.you, { x: Z.won.x, z: Z.won.you }, { key: "wy" }));
  slots.push(...pile("won-them", model.won.them, { x: Z.won.x, z: Z.won.them }, { key: "wt" }));

  return conserve(slots, Z);
}

// Every card is somewhere: a step that miscounts (a model the events could
// not follow exactly) is made to hold 64 cards, extras taken from the stock
// and missing ones put on it, and the last layout, the state's own, never needs it.
function conserve(slots, Z) {
  while (slots.length > PACK) {
    const i = slots.findLastIndex((s) => s.zone === "stock" || s.zone === "won-you" || s.zone === "won-them");
    slots.splice(i < 0 ? slots.length - 1 : i, 1);
  }
  const stock = slots.filter((s) => s.zone === "stock").length;
  for (let i = stock; slots.length < PACK; i++) {
    slots.push({ zone: "stock", index: i, code: null, faceUp: false, pose: lying({ x: Z.stock.x, z: Z.stock.z, height: i * STEP, faceUp: false }) });
  }
  return slots;
}

