// The engine's module run in Node, for the tests: a sitting at a given level
// and seed, played by whatever policy the test brings.
import fs from "node:fs";
import { fileURLToPath } from "node:url";

const WASM = fileURLToPath(new URL("../../../target/wasm32-unknown-unknown/release/bezique_wasm.wasm", import.meta.url));

export async function sitting({ level = 1, seed = 1, persona = 0, erratic = 0 } = {}) {
  const { instance } = await WebAssembly.instantiate(fs.readFileSync(WASM), {});
  const ex = instance.exports;
  ex.bezique_new(level, seed, persona, erratic);
  const state = () => JSON.parse(new TextDecoder().decode(new Uint8Array(ex.memory.buffer, ex.bezique_state(), ex.bezique_state_len())));
  const send = (command) => {
    const b = new TextEncoder().encode(command);
    const at = ex.bezique_alloc(b.length);
    new Uint8Array(ex.memory.buffer, at, b.length).set(b);
    return ex.bezique_send(b.length) === 1;
  };
  return { state, send };
}

// One decision of a simple player: the first legal card, the dearest declaration.
export function decide(s, rng = Math.random) {
  const p = s.prompt;
  if (p.kind === "choose_dealer") return rng() < 0.5 ? "dealer you" : "dealer them";
  if (p.kind === "play") return `play ${p.legal[Math.floor(rng() * p.legal.length)]}`;
  if (p.kind === "declare") return `declare ${Math.floor(rng() * p.options.length)}`;
  return null;
}
