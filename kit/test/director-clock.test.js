// The director's clock off the manual mode: the render loop on a stubbed
// requestAnimationFrame, and the gates that hold it.
import { test } from "node:test";
import assert from "node:assert/strict";
import { PerspectiveCamera, Quaternion, Vector3 } from "three";
import { createDirector } from "../js/director.js";

function setup(rested) {
  const frames = [];
  globalThis.requestAnimationFrame = (cb) => frames.push(cb);
  const camera = new PerspectiveCamera(30, 1, 1, 1000);
  camera.position.set(0, 100, 50);
  camera.lookAt(0, 0, 0);
  camera.updateMatrixWorld();
  const log = { renders: 0 };
  const stage = { render: () => log.renders++, camera, renderer: { domElement: { getBoundingClientRect: () => ({ left: 0, top: 0, width: 200, height: 100 }) } } };
  const deck = { card: () => ({ position: new Vector3(), quaternion: new Quaternion(), userData: {} }), reveal() {}, decorate() {} };
  const placed = (s) => s.cards.map((code, id) => ({ id, code, zone: "your-hand", pose: { position: new Vector3(id, 0, 0), quaternion: new Quaternion() } }));
  const choreography = {
    opening: () => ({ cards: [] }),
    initialPlacement: placed,
    choreograph: (p, next) => ({
      placement: placed(next),
      motions: next.cards.map((code, id) => ({ id, delay: 0, duration: 20, path: () => ({ position: new Vector3(9, 9, 9), quaternion: new Quaternion() }) })),
      beats: {},
    }),
  };
  const director = createDirector({ stage, deck, view: () => ({}), choreography, cards: 2, rested });
  // Run the stubbed frames until none is asked for (or a generous bound).
  const pump = (bound = 200000) => {
    let n = 0;
    while (frames.length && n++ < bound) frames.shift()();
    return n;
  };
  return { director, frames, pump, log };
}

test("a gate that releases at once does not stall the render loop", () => {
  const { director, pump } = setup();
  director.restart({ cards: ["AS", "KH"] });
  director.advance({ cards: ["AS", "KH"] });
  let opened = 0;
  director.gate(0, (release) => {
    opened++;
    release();
  });
  pump();
  assert.equal(opened, 1);
  assert.equal(director.busy(), false, "the cards landed");
  assert.equal(director.meshes[0].position.x, 9);
  assert.equal(director.held(), false);
});

test("a stale release does not release a later gate", () => {
  const { director, pump } = setup();
  director.restart({ cards: ["AS", "KH"] });
  let first = null;
  let second = null;
  director.gate(0, (release) => (first = release));
  pump();
  assert.equal(director.held(), true);
  director.cancelTimed(); // drops the hold; `first` is now stale
  assert.equal(director.held(), false);
  director.gate(0, (release) => (second = release));
  pump();
  assert.equal(director.held(), true, "the second gate holds");
  first();
  assert.equal(director.held(), true, "the stale release is ignored");
  second();
  assert.equal(director.held(), false);
});

test("a release called twice releases once", () => {
  const { director, pump } = setup();
  director.restart({ cards: ["AS"] });
  let release = null;
  director.gate(0, (r) => (release = r));
  pump();
  release();
  director.gate(0, () => {});
  pump();
  assert.equal(director.held(), true);
  release();
  assert.equal(director.held(), true, "the spent release does not release the next gate");
});

test("a restart without the deal gives the shape of an animation's result", () => {
  const { director } = setup();
  const timing = director.restart({ cards: ["AS"] });
  assert.deepEqual(timing, { beats: {}, landed: {}, count: null, duration: 0 });
});

test("a settle does not let the superseded animation's rest run again", async () => {
  let rests = 0;
  const { director, pump } = setup(() => rests++);
  director.restart({ cards: ["AS", "KH"] });
  director.advance({ cards: ["AS", "KH"] });
  director.relayout(); // settle: skips the timeline, resolving the idle
  await Promise.resolve();
  await Promise.resolve();
  pump();
  assert.equal(rests, 0, "no rest for the animation the settle replaced");
});

test("twins: the code forms answer for the first copy, the id forms for the card asked", () => {
  const { director } = setup();
  director.restart({ cards: ["AS", "AS"] });
  assert.equal(director.meshOf("AS"), director.meshes[0]);
  assert.equal(director.meshOfId(1), director.meshes[1]);
  assert.equal(director.meshOfId(7), null);
  const first = director.screenPoint("AS");
  assert.deepEqual(director.screenPointOfId(0), first);
  assert.notDeepEqual(director.screenPointOfId(1), first);
  assert.equal(director.screenPointOfId(7), null);
});
