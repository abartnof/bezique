// Drives a whole game through the WebAssembly exports, as a page would.
//
//   cargo build --release -p bezique-wasm --lib --target wasm32-unknown-unknown
//   node crates/bezique-wasm/ffi-check.mjs target/wasm32-unknown-unknown/release/bezique_wasm.wasm
import { readFileSync } from 'node:fs';

const bytes = readFileSync(process.argv[2]);
const { instance } = await WebAssembly.instantiate(bytes, {});
const ex = instance.exports;
ex.bezique_new(4, 42, 0, 0);
// Read memory.buffer afresh after every call: the module may grow its memory.
const state = () =>
  JSON.parse(new TextDecoder().decode(new Uint8Array(ex.memory.buffer, ex.bezique_state(), ex.bezique_state_len())));
const send = (command) => {
  const b = new TextEncoder().encode(command);
  const at = ex.bezique_alloc(b.length);
  new Uint8Array(ex.memory.buffer, at, b.length).set(b);
  return ex.bezique_send(b.length) === 1;
};
let s = state();
let moves = 0;
const start = Date.now();
while (s.prompt.kind !== 'over') {
  const p = s.prompt;
  const command =
    p.kind === 'choose_dealer' ? 'dealer you' : p.kind === 'play' ? `play ${p.legal[0]}` : `declare ${p.options.length - 1}`;
  if (!send(command)) throw new Error(`${command}: ${state().error}`);
  s = state();
  moves += 1;
}
console.log(`${moves} commands, ${s.events.length} events, winner ${s.outcome.winner}, ${Date.now() - start} ms`);
