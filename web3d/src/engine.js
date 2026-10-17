// The engine as the page calls it (docs/PROTOCOL.md): the module's four
// exports and one function a call. A fault in the module (a panic) is told
// once to `onTrap`, and the engine is still after it (kit/js/engine.js `guard`).
import { guard } from "../../kit/js/engine.js";

export async function loadEngine(bytes, onTrap) {
  const { instance } = await WebAssembly.instantiate(bytes, {});
  const ex = instance.exports;
  for (const name of ["bezique_new", "bezique_alloc", "bezique_send", "bezique_state", "bezique_state_len"]) {
    if (typeof ex[name] !== "function") throw new Error(`the module lacks ${name}`);
  }
  const enc = new TextEncoder();
  const dec = new TextDecoder();
  // Read memory.buffer afresh every time: the module may grow its memory.
  const state = () => JSON.parse(dec.decode(new Uint8Array(ex.memory.buffer, ex.bezique_state(), ex.bezique_state_len())));
  return guard(
    {
      new: ({ level, seed, persona = 0, erratic = 0 }) => {
        ex.bezique_new(level, seed >>> 0, persona, erratic);
        return state();
      },
      send: (command) => {
        const bytes = enc.encode(command);
        const at = ex.bezique_alloc(bytes.length);
        new Uint8Array(ex.memory.buffer, at, bytes.length).set(bytes);
        const ok = ex.bezique_send(bytes.length) === 1;
        return { ok, state: state() };
      },
      state,
    },
    onTrap,
  );
}
