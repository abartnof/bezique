// What the opponent says: the engine's events (docs/PROTOCOL.md "Events")
// mapped to the phrase bank's groups (docs/PHRASES.md, web3d/words.json), one
// line picked per moment. The page calls `onEvent(event, ctx)` when an event
// is *seen* (at its beat on the table's clock), so timing is the caller's
// (and the speaker's); what is kept here is what the events alone decide:
// which moment it is, whether it is spoken of (mode, persona), and which line
// (a shuffle-bag per group, so none comes round again before the rest).
//
//   createTalk({ words, rng, say, mode, persona }) -> { onEvent, setMode, setPersona, mode, persona }
//     words:   web3d/words.json
//     rng:     () => [0, 1), seeded for a repeatable sitting
//     say:     ({ text, group, call }) -> void; `call` lines carry the game
//     mode:    "quiet" | "normal" | "chatty" (the `talk` pref)
//     persona: an opponent::personas() key; the same bank, weighted a little
//   onEvent(event, ctx): `ctx.totals` is { you, them }, the game's scores as
//     shown at that beat. `{ kind: "undo" }` is sent by the page on a take-back.

import { createBags } from "../../kit/js/bag.js";

export const MODES = ["quiet", "normal", "chatty"];
// The cut ranks as in play: A, 10, K, Q, J, 9, 8, 7.
const CUT_RANK = { A: 7, T: 6, K: 5, Q: 4, J: 3, 9: 2, 8: 1, 7: 0 };
// The stock holds 47 cards after the deal and each trick draws two, so after
// this many of your draws about five remain: time to remark on it.
export const LOW_AFTER = 21;
export const STREAK = 3;
export const RICH = 20;
// A lead this big (or a double in the air) is worth a word at the hand's end.
export const LEVEL_WITHIN = 30;
export const DOUBLE_AT = 700;

const rankOf = (card) => card[0];
const suitOf = (card) => card[1];

// Which combination a declaring formula names.
export function meldOf(formula) {
  const f = formula.toLowerCase();
  if (f.includes("royal marriage")) return "royal";
  if (f.includes("double bezique")) return "double";
  if (f.includes("bezique")) return "bezique";
  if (f.includes("in trumps")) return "sequence";
  if (f.includes("marriage")) return "marriage";
  if (/\b(aces|kings|queens|jacks)\b/.test(f)) return "four";
  return null;
}

export function createTalk({ words, rng = Math.random, say, mode, persona = "balanced" }) {
  const pick = createBags(rng);
  mode = mode ?? words.default_mode;
  let trump = null;
  let draws = 0; // your draws this hand
  let run = { who: null, n: 0 }; // tricks running to one side
  let stockOut = false; // the stock has run out this hand
  let lastTricks = 0; // tricks taken since
  let threat = null; // "you" | "them": the side the double is in the air for

  // Whether a moment of this group is spoken of, by mode and persona.
  function wanted(group) {
    const g = words.groups[group];
    if (!g) return false;
    const p = words.modes[mode][g.kind] ?? 0;
    const w = g.kind === "call" ? 1 : words.personas[persona]?.[group] ?? 1;
    return rng() < Math.min(1, p * w);
  }

  // Say one line of a group, if the mode allows; returns whether it was said.
  function speak(group, slots = {}) {
    if (!wanted(group)) return false;
    const g = words.groups[group];
    const line = pick(group, g.lines);
    if (!line) return false;
    const text = line.t.replace(/\{(\w+)\}/g, (m, k) => slots[k] ?? m);
    say({ text, group, call: g.kind === "call" });
    return true;
  }

  const sideOf = (who) => (who === "you" ? "you" : "them");

  function played(e) {
    const code = e.card.card;
    const who = sideOf(e.who);
    let kind = "low";
    if (trump && suitOf(code) === trump) kind = "trump";
    else if (rankOf(code) === "A" || rankOf(code) === "T") kind = "brisque";
    else if (rankOf(code) === "K" || rankOf(code) === "Q") kind = "high";
    speak(`play_${who}_${kind}`);
  }

  function tookTrick(e) {
    const who = sideOf(e.who);
    run = run.who === who ? { who, n: run.n + 1 } : { who, n: 1 };
    let group = `trick_${who}`;
    if (run.n === STREAK) group = `streak_${who}`;
    if (e.points >= RICH) group = `trick_rich_${who}`;
    if (stockOut && ++lastTricks === 8) {
      group = `last_trick_${who}`;
      stockOut = false;
    }
    speak(group, { points: e.points });
  }

  function onEvent(e, ctx = {}) {
    switch (e.kind) {
      case "cut": {
        speak("start");
        const a = CUT_RANK[rankOf(e.yours.card)];
        const b = CUT_RANK[rankOf(e.theirs.card)];
        if (a > b) speak("cut_you_high");
        else if (b > a) speak("cut_them_high");
        break;
      }
      case "dealt": {
        trump = suitOf(e.turn_up.card);
        draws = 0;
        run = { who: null, n: 0 };
        stockOut = false;
        const dealer = sideOf(e.dealer);
        speak(`deal_${dealer}`);
        if (rankOf(e.turn_up.card) === "7") speak(`seven_${dealer}`);
        if (e.hand >= 3) speak("pace");
        break;
      }
      case "played":
        played(e);
        break;
      case "took_trick":
        tookTrick(e);
        break;
      case "declared": {
        const who = sideOf(e.who);
        const formula = /"([^"]+)"/.exec(e.text ?? "")?.[1] ?? "";
        const meld = meldOf(formula);
        if (who === "them") {
          speak("decl_them", { formula });
          if (meld === "sequence" || meld === "double" || (meld === "four" && /aces/i.test(formula))) speak("decl_them_lucky");
        } else if (meld) {
          speak(`decl_you_${meld}`);
        }
        break;
      }
      case "dix":
        speak(`dix_${sideOf(e.who)}`);
        break;
      case "drew":
        if (++draws === LOW_AFTER) speak("stock_low");
        break;
      case "stock_out":
        stockOut = true;
        lastTricks = 0;
        speak("stock_out");
        speak(`stock_last_${sideOf(e.last_card)}`);
        speak("last_eight");
        break;
      case "hand_ends":
        handEnds(ctx.totals);
        break;
      case "game_ends":
        speak(`win_${sideOf(e.winner)}${e.double ? "_double" : ""}`);
        threat = null;
        break;
      case "undo":
        speak("undo");
        break;
      default:
        break;
    }
  }

  function handEnds(totals) {
    if (!totals) return;
    const { you, them } = totals;
    // The double is in the air for the trailing side while the leader is near a
    // thousand and the trailer short of five hundred; it is off once passed.
    if (threat && totals[threat] >= 500) {
      const side = threat;
      threat = null;
      speak(`double_off_${side}`);
      return;
    }
    if (them >= DOUBLE_AT && you < 500) threat = "you";
    else if (you >= DOUBLE_AT && them < 500) threat = "them";
    if (threat) {
      speak(`double_${threat}_threatened`);
      return;
    }
    const gap = Math.abs(you - them);
    const group = gap < LEVEL_WITHIN ? "hand_level" : you > them ? "hand_ahead_you" : "hand_ahead_them";
    speak(group, { gap });
  }

  return {
    onEvent,
    setMode(m) {
      if (!MODES.includes(m)) throw new Error(`unknown talk mode: ${m}`);
      mode = m;
    },
    setPersona(p) {
      if (!words.personas[p]) throw new Error(`unknown persona: ${p}`);
      persona = p;
    },
    get mode() {
      return mode;
    },
    get persona() {
      return persona;
    },
  };
}
