// Where everything rests, in centimetres on the table: the person at +z, the
// opponent at -z, x to the person's right, y up. Two arrangements share one
// shape: ZONES across a computer or a tablet held sideways, ZONES_PORTRAIT
// for a phone held upright. The eyes are the kit's camera numbers with their
// own `reach` (the extent of the table the eye must keep in view, as
// tangents of its axis); the arrangement of strohmandeln's, to be measured
// again for this table.
import { CAMERA, CAMERA_PORTRAIT, CARD, PITCH, facingEye } from "../../kit/js/units.js";

const DEG = Math.PI / 180;
const over = (target, distance) => Object.freeze([target[0], target[1] + distance * Math.sin(PITCH * DEG), target[2] + distance * Math.cos(PITCH * DEG)]);

const TARGET = Object.freeze([0, 0, -19]);
export const EYE = Object.freeze({
  ...CAMERA,
  position: over(TARGET, 118),
  target: TARGET,
  fov: 34,
  widthTan: 0.42,
  reach: Object.freeze({ up: 0.31, foot: 0.315 }),
});
const TARGET_PORTRAIT = Object.freeze([0, 0, -19]);
export const EYE_PORTRAIT = Object.freeze({
  ...CAMERA_PORTRAIT,
  position: over(TARGET_PORTRAIT, 96),
  target: TARGET_PORTRAIT,
  reach: Object.freeze({ up: 0.32, down: -0.34, across: 0.24 }),
  hand: Object.freeze({ across: 0.21, still: 0.21, perFill: 0.21 }),
});

const hand = (centre, eye, extra) => Object.freeze({ centre: Object.freeze(centre), lean: facingEye(eye.position, centre), ...extra });
const row = (z, width, pitch) => Object.freeze({ x: 0, z, width, pitch });

function arrangement({ eye, yours, theirs, yourTable, theirTable, theirKnown, trick, stock, won, cut }) {
  return Object.freeze({
    eye: eye.position,
    camera: eye,
    yourHand: yours,
    theirHand: theirs,
    yourTable,
    theirTable,
    theirKnown,
    trick: Object.freeze(trick),
    stock: Object.freeze(stock),
    won: Object.freeze(won),
    cut: Object.freeze(cut),
    stack: 0.05, // a card's thickness plus the hair between
  });
}

export const ZONES = arrangement({
  eye: EYE,
  yours: hand([0, 2.5, 13], EYE, { fan: true, radius: 40, spacing: 4.4, width: 62 }),
  theirs: hand([0, 7, -42], EYE, { fan: true, radius: 40, spacing: 1.4, width: 30 }),
  yourTable: row(-1, 64, 7),
  theirTable: row(-27, 64, 7),
  theirKnown: row(-36, 64, 7),
  trick: { x: 0, z: -15, gap: 1.2 },
  stock: { x: -42, z: -14 },
  won: { x: 42, you: 4, them: -32 },
  cut: { you: { x: -28, z: -14 }, them: { x: -28, z: -24 } },
});

export const ZONES_PORTRAIT = arrangement({
  eye: EYE_PORTRAIT,
  yours: hand([0, 2, 13], EYE_PORTRAIT, { fan: false, pitch: 3.9, width: 34 }),
  theirs: hand([0, 6, -36], EYE_PORTRAIT, { fan: true, radius: 30, spacing: 1.1, width: 20 }),
  yourTable: row(-1, 34, 4.2),
  theirTable: row(-25, 34, 4.2),
  theirKnown: row(-33, 34, 4.2),
  trick: { x: 0, z: -14, gap: 1 },
  stock: { x: -17, z: -14 },
  won: { x: 17, you: 6, them: -28 },
  cut: { you: { x: -9, z: -14 }, them: { x: -9, z: -24 } },
});

export { CARD };
