// From cassino web3d/src/deck.js @ 73f0cd7
// From piquet web3d/src/deck.js @ 254cb3c.
// The cards as scene objects: one mesh per card, its ink line, and which face
// it shows.
//
// A card's face is assigned only when the human may know it (docs/TABLE3D.md
// section 5.1): until then it shows the back on both sides, so nothing in the
// scene can reveal a card the human could not see at a real table.

import { Mesh } from "three";
import { cardGeometry } from "./cards.js";
import { cardMaterials, INK, inkMaterial, RAMPS } from "./materials.js";

// The ink line is also the selection language (docs/TABLE3D.md section
// 8.2), which keeps the table free of glows and badges: a card pointed at
// takes a heavier line, a hinted one a cyan line, one chosen to throw an
// amber one.
export const INKS = {
  plain: { color: INK },
  hover: { color: INK, heavier: 1.5 },
  hint: { color: "#009fb7", heavier: 1.6 },
  chosen: { color: "#e8900c", heavier: 1.6 },
};

export function createDeck(stage, textures, { inkWidth = 2.5 } = {}) {
  const geometry = cardGeometry();
  const ramp = RAMPS.card();
  const inks = {};
  for (const [name, { color, heavier = 1 }] of Object.entries(INKS)) {
    inks[name] = inkMaterial({ color, width: inkWidth * heavier });
    stage.registerInk(inks[name]);
  }
  const ink = inks.plain;
  const unknown = cardMaterials({ face: textures.back, back: textures.back, ramp });
  // Kit change for Bezique, whose pack holds every card twice: the face
  // materials are one per mesh (the texture still shared), so dimming one
  // twin leaves the other. `known` is keyed by the mesh and its code.
  const known = new WeakMap();
  // The deck's own map of the faces shown now: the caller's sets (classic,
  // Jumbo) are never written to, so either can be set again.
  const faceNow = { ...textures.faces };
  const cardMeshes = [];
  const materialsOf = (mesh, code) => {
    let byCode = known.get(mesh);
    if (!byCode) known.set(mesh, (byCode = {}));
    return (byCode[code] ??= cardMaterials({ face: faceNow[code], back: textures.back, ramp }));
  };

  function card(code = null) {
    const mesh = new Mesh(geometry, unknown);
    if (code) mesh.material = materialsOf(mesh, code);
    cardMeshes.push(mesh);
    mesh.castShadow = true;
    mesh.receiveShadow = true;
    mesh.add(new Mesh(geometry, ink));
    mesh.userData.code = code;
    stage.scene.add(mesh);
    return mesh;
  }

  // Show this card's face -- or, with null, hide it again.
  function reveal(mesh, code) {
    mesh.material = code ? materialsOf(mesh, code) : unknown;
    mesh.userData.code = code;
  }

  // Change every face's art -- to the Jumbo Index faces, or back to the
  // classic ones (faces.js) -- on the cards where they lie.
  function setFaces(faces) {
    Object.assign(faceNow, faces);
    for (const mesh of cardMeshes) {
      const byCode = known.get(mesh) ?? {};
      for (const [code, materials] of Object.entries(byCode)) {
        materials[0].map = faceNow[code];
        materials[0].needsUpdate = true;
      }
    }
  }

  // Which line a card is drawn with, and whether its face is dimmed (a card
  // that may not be played). Faces are one material per card, so dimming
  // one dims only that card. `dim` left out keeps the card as it is (a hint
  // mark's line must not undo the dimming).
  function decorate(mesh, { line = "plain", dim } = {}) {
    mesh.children[0].material = inks[line];
    const code = mesh.userData.code;
    if (code && dim !== undefined) materialsOf(mesh, code)[0].color.setScalar(dim ? 0.72 : 1);
  }

  return { card, reveal, decorate, setFaces, geometry, ink };
}

export function place(mesh, pose) {
  mesh.position.copy(pose.position);
  mesh.quaternion.copy(pose.quaternion);
}
