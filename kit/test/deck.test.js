// The deck's faces: switching art and back, and dimming that a mark's line
// must not undo.
import { test } from "node:test";
import assert from "node:assert/strict";
import { Scene } from "three";
import { createDeck } from "../js/deck.js";

const tex = (name) => ({ name, isTexture: true });
function setup() {
  const textures = { back: tex("back"), faces: { AS: tex("classic AS"), KH: tex("classic KH") } };
  const stage = { scene: new Scene(), registerInk() {} };
  return { textures, deck: createDeck(stage, textures) };
}

test("classic faces come back after Jumbo, even when the caller holds the textures' own map as 'classic'", () => {
  const { textures, deck } = setup();
  const classic = textures.faces; // as web3d/src/main.js held it
  const jumbo = { AS: tex("jumbo AS"), KH: tex("jumbo KH") };
  const mesh = deck.card("AS");
  deck.setFaces(jumbo);
  assert.equal(mesh.material[0].map.name, "jumbo AS");
  deck.setFaces(classic);
  assert.equal(mesh.material[0].map.name, "classic AS", "classic restored");
  assert.equal(classic.AS.name, "classic AS", "the caller's map was not overwritten");
  const later = deck.card("KH");
  assert.equal(later.material[0].map.name, "classic KH", "a card shown later takes the faces now set");
});

test("decorate leaves the dimming alone unless told", () => {
  const { deck } = setup();
  const mesh = deck.card("AS");
  deck.decorate(mesh, { dim: true });
  assert.equal(mesh.material[0].color.r, 0.72);
  deck.decorate(mesh, { line: "hint" });
  assert.equal(mesh.material[0].color.r, 0.72, "a mark's line keeps the dim");
  deck.decorate(mesh, { line: "plain" });
  assert.equal(mesh.material[0].color.r, 0.72);
  deck.decorate(mesh, { dim: false });
  assert.equal(mesh.material[0].color.r, 1);
});
