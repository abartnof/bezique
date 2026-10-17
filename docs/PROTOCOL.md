# The table protocol

How a client plays bezique against the engine. The engine side is
`bezique_core::session` (a sitting, advanced one human decision at a time)
and `crates/bezique-wasm` (the same as JSON and one-line commands, compiled
to WebAssembly). A client holds no game logic: it reads the state, draws it,
and sends a command.

## The loop

```
bezique_new(level, seed, persona, erraticism)   -> state
loop:
    read state.prompt     -- what the person must decide now
    draw the state
    send(command)         -> accepted? ; the state again
```

The engine runs the opponent between the person's decisions, so after an
accepted command the state is already at the person's next decision (or the
end of the game).

The person is always side A. `level` is 1 to 5 (`agents::TOP`); `persona` is
0 to draw one from the seed, else its place in `opponent::personas()` from 1
(balanced, concentrates, close, bezique); `erraticism` is in thousandths. The
same arguments always give the same sitting. Out-of-range arguments are not
refused: the level is clamped to 1–5, erraticism to 0–1,000, and a persona
number past the list is drawn from the seed as 0 is. Before `bezique_new`
the state is empty (`bezique_state_len()` is 0), and a command sent then
leaves it as `null`. Until the dealer is chosen the state has no table: the
fields from `hand_number` to `tricks_played` are absent. The `cut` event
belongs to no hand (`hand: 0`).

## Commands

| Command | When | Meaning |
|---|---|---|
| `dealer you` / `dealer them` | `prompt.kind == "choose_dealer"` | Having cut higher, choose who deals first |
| `play <id>` | `prompt.kind == "play"` | Lead or answer with your card numbered `id` (one of `prompt.legal`, or its twin if you hold both) |
| `declare <n>` | `prompt.kind == "declare"` | Choose option `n` of `prompt.options`; option 0 is to draw without declaring |
| `undo` | `can_undo` | Take back the last decision, with everything that followed it |
| `set <aid> on` / `off` | any time | Switch an aid: `hints`, `play_forced`, `tracker` |

A command the rules forbid is refused: the state is left exactly as it was
and `error` says why, in words fit to show a player ("you must follow hearts
while you can"). Sending a move just to show the refusal is fine; the reason
is the lesson.

**Aids** change what the person is asked, never what happens. `hints` adds
`hint`, the top rung's choice in the person's place (it counts every card
played, whether or not the tracker is on); `play_forced` plays a card when
it is the only one allowed; `tracker` adds `tricks_played`, every trick of
the hand (off, only the last trick shows: Foster says a quitted trick may not
be looked at again). The narration (`events`) is the full log of the sitting
whatever the tracker says, so with the tracker off a client should show only
the last trick's `played` events. A turn of declaring with nothing to
declare is never put to the person.

**Reloading and undo.** `record` lists every command from the person's seat,
`*` marking what the table did for them. `Session::replay(seed, settings,
record)` rebuilds the sitting exactly; undo is a replay without the last
decision. The choice of dealer cannot be undone: the first deal comes from
the seed, and the choice decides only which of its two hands is whose.
A saved record replays only against the same play: the state's
`record_version` (`session::RECORD_VERSION`) changes whenever the deal, the
opponent's choices or the order of the declaring options change, so a page
should save it with the record and refuse a record of another version. There
is no replay export over the FFI: to reload, call `bezique_new` with
the same arguments and send each entry of the record that is not marked
`*`. Erraticism is not in the state, so the page keeps its own arguments.

Because undo is a replay, the stock and the opponent's cards are the same
after it: a person who undoes can learn the next card drawn, or the
opponent's answer to a lead, and then play differently. Undo is a learning
aid, not a part of fair play.

## The state

Every field comes from the person's view (`observation::View`), so a client
cannot show more than the person could know. The protocol tests play whole
games and check, type by type, that no state shows a card hidden from the
person, and that a type whose two copies are both hidden is named nowhere
in the hand's state: not in a card, a narration line, the error or any
other string. The seed is not in the state, since it fixes every card; but
hidden cards are only as hidden as the seed and the module's memory, which
an engine running in the page cannot keep from a person determined to look.

```jsonc
{
  "protocol": 1, "record_version": 3,
  "level": 4,
  "opponent": "Your opponent remembers every card, ... Its style: ...",
  "phase": "lead",            // choose_dealer, lead, follow, declare, over
  "hand_number": 2, "you_deal": false,
  "trump": "hearts", "trump_symbol": "♥",
  "turn_up": {"id": 23, "card": "9H"},   // null once taken
  "stock": 31,                // face-down cards left
  "totals": {"you": 340, "them": 410},       // the game, finished hands included
  "hand_scores": {"you": 60, "them": 20},    // this hand
  "hand": [ {"id": 1, "card": "AS"}, ... ],  // yours, concealed, as a hand is read
  "table": [ ... ],           // yours, laid on the table (still yours to play)
  "carried": [ {"points": 20, "said": "twenty", "cards": [...]} ],  // yours, to score
  "their_table": [ ... ], "their_carried": [ ... ],
  "their_known": [ ... ],     // their concealed cards you have seen
  "their_hidden": 6,          // how many of their concealed cards you have not
  "trick": {"leader": "them", "led": {"id": 40, "card": "QS"}},     // or null
  "last_trick": {"leader": "you", "led": ..., "second": ..., "winner": "you"},
  "tricks_played": null,      // with the tracker aid, every trick of the hand
  "prompt": { ... },          // below
  "aids": {"hints": false, "play_forced": false, "tracker": false},
  "can_undo": true,
  "record": ["dealer you", "play 12", "*declare 0", ...],
  "hint": null,               // with hints: {"command": "play 12", "text": "Play the KS.", "reason": {"key": "take_brisque", "card": "KS", "led": "TD"}}
  "events": [ ... ],          // below
  "outcome": null,            // at the end: {"winner": "you", "double": false}
  "error": null
}
```

### The hint's reason

`hint.reason` says why the top rung chose as it did, for a client to word
(`docs/HINTS.md`). `key` is one of the keys in `bezique_core::hints::KEYS`
(`generic` when the choice fits no case); the other members are the few
parameters that key needs: cards as codes (`card`, `led`, `kept`, `spared`,
`partner`, `turn_up`), numbers (`points`, `trumps`) and words (`meld`, `suit`).
Which members a key carries is listed in `docs/HINTS.md`. `command` and `text`
are unchanged. The member is additive, so `PROTOCOL` stays 1; a client that
ignores `reason` loses nothing, and one that meets a key it does not know
should show `text`.

### Card ids and twins

A card is `{"id", "card"}`: the code (`AS`, `TD`, `7C`) and a number from 0
to 63. Every card exists twice, and **real twins cannot be told apart**, so
ids carry no information beyond what the person may know (`docs/DESIGN.md`
§5):

- Your own cards have their true ids; `play <id>` takes them.
- Of a type you have never held, the copies you can see are numbered from
  what is visible now: a lone copy is always copy 0, and two are numbered by
  where they lie. The same physical card may therefore be numbered
  differently in a later state; key a client's meshes for the opponent's
  cards by code and position, not by id.
- In events, the opponent's cards are named by type only: copy 0, unless
  you have held a twin of that card, when its true id is kept. Within one
  event two copies of a type are numbered 0 and 1 (a double bezique lays
  four distinct ids), but across events the same id may name different
  physical cards; key by code.

### Prompts

```jsonc
{"kind": "choose_dealer"}
{"kind": "play", "legal": [12, 30, 41]}
{"kind": "declare", "options": [
   {"text": "Nothing: draw.", "cards": [], "points": 0},
   {"text": "Exchange the seven of trumps for the KH: ten.", "cards": [...], "points": 10},
   {"text": "Forty for bezique, and twenty to score.", "cards": [...], "points": 40} ]}
{"kind": "over"}
```

A declaration option's `cards` are those it lays on the table (or the seven
exchanged or shown). Options are worded in the period formulae (`words.rs`):
the combination scored, then what is carried "to score".

### Events

The narration, oldest first, for the whole sitting. Each has `kind`, `hand`
(the hand it belongs to) and `text` (one sentence, ready to show), plus:

| kind | fields |
|---|---|
| `cut` | `yours`, `theirs`: the cards shown in the cut for deal |
| `first_dealer` | `dealer`: `you` or `them` |
| `dealt` | `dealer`, `turn_up` |
| `played` | `who`, `card` |
| `took_trick` | `who`, `points` (brisques, and the last trick) |
| `declared` | `who`, `laid` (the cards laid), `points` (the one scored) |
| `dix` | `who`, `taken` (the turn-up taken, or null if the seven was only shown) |
| `drew` | `card` (yours only) |
| `stock_out` | `last_card` (who took it), `turn_up` (taken by the other) |
| `hand_ends` | `you`, `them`: the hand's points |
| `game_ends` | `winner`, `double` |

## WebAssembly

The module has no imports. It exports its memory and:

| Export | |
|---|---|
| `bezique_new(level: u32, seed: u32, persona: u32, erraticism_permille: u32)` | sit down at a new table |
| `bezique_alloc(len: usize) -> *mut u8` | a buffer to write a command into |
| `bezique_send(len: usize) -> u32` | carry out the command just written; 1 if accepted |
| `bezique_state() -> *const u8`, `bezique_state_len() -> usize` | the state, as UTF-8 JSON |

```js
const { instance } = await WebAssembly.instantiate(bytes, {});
const ex = instance.exports;
ex.bezique_new(4, 42, 0, 0);
const state = () => JSON.parse(new TextDecoder().decode(
  new Uint8Array(ex.memory.buffer, ex.bezique_state(), ex.bezique_state_len())));
const send = (command) => {
  const b = new TextEncoder().encode(command);
  const at = ex.bezique_alloc(b.length);
  new Uint8Array(ex.memory.buffer, at, b.length).set(b);
  return ex.bezique_send(b.length) === 1;
};
```

Read `memory.buffer` afresh after every call: the module may grow its
memory. `crates/bezique-wasm/ffi-check.mjs` plays a whole game this way
(at level 4 by default, or the level given after the module's path): in
Node, a game takes about 0.35 s at level 4 and 1.0 s at level 5. Built with
`cargo build --release -p bezique-wasm --lib --target
wasm32-unknown-unknown`, the module is about 248 KB (built without the
`race` feature of `bezique-core`, which only the measurements need).
