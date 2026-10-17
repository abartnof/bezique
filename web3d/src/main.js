// Bezique at a three-dimensional table.
//
// A client of the wasm module (docs/PROTOCOL.md): it draws the person's view
// and sends back commands, and holds no rules. Every change of state is
// played out on the cards by the director (choreography.js), with the score
// and the narration at each event's moment.
import { createDeck } from "../../kit/js/deck.js";
import { createDirector } from "../../kit/js/director.js";
import { loadTextures } from "../../kit/js/art.js";
import { decodeBase64 } from "../../kit/js/engine.js";
import { createScene } from "../../kit/js/scene.js";
import { chooseSurface } from "../../kit/js/surfaces.js";
import { choreograph, initialPlacement, opening } from "./choreography.js";
import { loadEngine } from "./engine.js";
import { createNewGame } from "./newgame.js";
import { gameConfig, loadPrefs, savePrefs, speedFactor, urlOverrides, withUrl } from "./prefs.js";
import { createSettings } from "./settings.js";
import { fromState } from "./model.js";
import { createUi } from "./ui.js";
import { EYE, EYE_PORTRAIT, ZONES, ZONES_PORTRAIT } from "./zones.js";

/* global WASM_BASE64, ART */

const params = new URL(window.location.href).searchParams;
const SITTING = "bezique.sitting.v1";

const store = {
  get(key) {
    try {
      return JSON.parse(window.localStorage.getItem(key));
    } catch {
      return null;
    }
  },
  set(key, value) {
    try {
      window.localStorage.setItem(key, JSON.stringify(value));
    } catch {
      /* private window, storage full: the game still plays */
    }
  },
};

const systemReduced = () => !!window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;

async function main() {
  let stopped = false;
  const engine = await loadEngine(decodeBase64(WASM_BASE64), (error) => {
    console.error(error);
    stopped = true;
    ui.say("The table has stopped. Reload the page to carry on.");
  });

  const prefs = withUrl(loadPrefs(), params);
  const url = urlOverrides(params);

  // ---- the scene, the cards -------------------------------------------------
  const canvas = document.getElementById("stage");
  const stage = createScene(canvas, {
    table: chooseSurface({ chosen: prefs.surface, saved: null }),
    ...(params.has("lowfx") ? { shadow: "pcf", shadowMap: 256 } : {}),
    eyes: { across: EYE, portrait: EYE_PORTRAIT },
  });
  const textures = await loadTextures(ART, { anisotropy: stage.renderer.capabilities.getMaxAnisotropy(), pixelRatio: stage.renderer.getPixelRatio() });
  const deck = createDeck(stage, textures);
  const zonesNow = () => (stage.portrait ? ZONES_PORTRAIT : ZONES);

  // ---- state ------------------------------------------------------------------
  let S = null; // the director's state: { model, events, raw }
  let raw = null; // the engine's last state
  let seed = 0;
  let waiting = false; // cards moving: the person may not act
  let chosen = null; // a card picked on a touch screen, to be played by a second tap
  let hover = null;
  let shown = { totals: { you: 0, them: 0 }, hand: { you: 0, them: 0 } };

  const view = () => ({ chosen, zones: zonesNow() });
  const legal = () => (raw?.prompt?.kind === "play" ? raw.prompt.legal : []);
  const hintId = () => (raw?.hint?.command?.startsWith("play ") ? Number(raw.hint.command.slice(5)) : null);

  function decorate(placement, d, meshes) {
    const mine = !waiting && raw?.prompt?.kind === "play";
    for (const m of placement) {
      let line = "plain";
      let dim = false;
      if ((m.zone === "your-hand" || m.zone === "your-table") && m.cardId !== undefined) {
        const ok = legal().includes(m.cardId);
        if (m.cardId === chosen) line = "chosen";
        else if (m.cardId === hintId() && !waiting) line = "hint";
        else if (mine && m.cardId === hover && ok) line = "hover";
        dim = mine && !ok;
      }
      d.decorate(meshes[m.id], { line, dim });
    }
  }

  const director = createDirector({
    stage,
    deck,
    view,
    decorate,
    rested: () => rested(),
    choreography: { choreograph, initialPlacement, opening },
    zones: ZONES,
    cards: 64,
    manual: params.has("manual"),
    speed: speedFactor(prefs, systemReduced()),
  });

  // ---- the page ---------------------------------------------------------------
  const overlay = document.getElementById("overlay");
  const ui = createUi(overlay, { onAct: (act, data) => onAct(act, data) });
  const settings = createSettings(overlay, { prefs, onChange: (key, value) => changed(key, value) });
  const newGameDialog = createNewGame(overlay, {
    prefs,
    inProgress: () => !!raw && raw.prompt?.kind !== "over",
    onStart: (values, chosenSeed) => newGame(values, chosenSeed),
  });
  document.getElementById("loading").remove();
  canvas.setAttribute("aria-label", "The bezique table");

  function frame() {
    stage.setStrips({ top: ui.hudBottom() + 6, foot: window.innerHeight - ui.barTop() + 6, raised: 0 }, { instant: true });
  }
  ui.onLayout(frame);
  frame();
  stage.onReframe = () => director.relayout();

  // ---- showing a state --------------------------------------------------------
  function sync() {
    shown = { totals: { ...raw.totals }, hand: { ...raw.hand_scores } };
    ui.setScores(shown);
    ui.setInfo({ trump: raw.trump_symbol ?? null, stock: raw.stock ?? 64, hand: raw.hand_number ?? null, level: raw.level });
  }

  function rested() {
    waiting = false;
    if (!raw) return;
    sync();
    ui.setPrompt(raw, { busy: false, hint: raw.aids.hints ? (raw.hint ?? {}) : null });
    director.redecorate();
  }

  function applyEvent(e) {
    ui.say(e.text);
    if (e.kind === "took_trick" || e.kind === "declared") {
      shown.hand[e.who] += e.points;
      shown.totals[e.who] += e.points;
    } else if (e.kind === "hand_ends") {
      shown.hand = { you: 0, them: 0 };
    }
    ui.setScores(shown);
  }

  // `how`: "deal" (a new sitting, dealt out from the pack), "direct" (an undo
  // or a sitting restored: the cards at once) or "advance" (the next move played out).
  function present(next, how) {
    const first = S ? S.events.length : 0;
    const state = { model: fromState(next), events: next.events, raw: next };
    raw = next;
    waiting = true;
    chosen = null;
    ui.setPrompt(raw, { busy: true });
    let timing = null;
    if (how === "deal") timing = director.restart(state, { dealt: true });
    else if (how === "direct" || !S) director.restart(state);
    else timing = director.advance(state);
    S = state;
    if (how === "direct" || !timing) {
      rested();
      return;
    }
    shown = how === "deal" ? { totals: { you: 0, them: 0 }, hand: { you: 0, them: 0 } } : shown;
    for (const [index, ms] of Object.entries(timing.beats)) {
      const e = next.events[Number(index)];
      if (e && Number(index) >= first) director.at(ms, () => applyEvent(e));
    }
  }

  function persist() {
    if (raw && raw.record) store.set(SITTING, { v: raw.record_version, level: raw.level, persona: prefs.persona, erratic: prefs.erratic, seed, hints: raw.aids.hints, record: raw.record });
  }

  // A setting within the game, taking effect at once. The other aids' keys
  // (faces, hintRules, tutor, talk, tracker) are saved here and wired by their owners.
  function changed(key, value) {
    prefs[key] = value;
    savePrefs(prefs);
    if (key === "speed" || key === "reducedMotion") director.setSpeed(speedFactor(prefs, systemReduced()));
    else if (key === "surface") stage.setSurface(chooseSurface({ chosen: value, saved: null }));
    else if (key === "hints") setHints(value);
  }

  function setHints(on) {
    if (!raw || raw.aids.hints === on) return;
    raw = engine.send(`set hints ${on ? "on" : "off"}`).state;
    persist();
    rested();
  }

  // Starts a game from the New game dialog's values (level, persona, erratic).
  function newGame(values = prefs, chosenSeed = null) {
    Object.assign(prefs, { level: values.level, persona: values.persona, erratic: values.erratic });
    savePrefs(prefs);
    newGameDialog.remember(prefs);
    seed = Number.isInteger(chosenSeed) ? chosenSeed : url.seed !== null && !S ? url.seed : Math.floor(Math.random() * 2 ** 32);
    director.cancelTimed();
    engine.new(gameConfig(prefs, { seed }));
    if (prefs.hints) engine.send("set hints on");
    present(engine.state(), "deal");
    persist();
  }

  // The sitting kept from last time, replayed command by command.
  function restore() {
    const saved = store.get(SITTING);
    if (!saved || url.seed !== null || url.fresh) return false;
    seed = saved.seed;
    prefs.level = saved.level;
    prefs.persona = saved.persona ?? 0;
    prefs.erratic = saved.erratic ?? 0;
    let s = engine.new(gameConfig(prefs, { seed }));
    if (s.record_version !== saved.v) return false;
    if (saved.hints) engine.send("set hints on");
    for (const command of saved.record) {
      if (command.startsWith("*")) continue;
      const r = engine.send(command);
      if (!r.ok) return false;
    }
    s = engine.state();
    if (s.prompt.kind === "over") return false;
    present(s, "direct");
    return true;
  }

  // ---- what the person does ---------------------------------------------------
  function act(command) {
    if (waiting || stopped) return false;
    const r = engine.send(command);
    if (!r.ok) {
      ui.say(r.state?.error ?? "That is not allowed.");
      return false;
    }
    present(r.state, "advance");
    persist();
    return true;
  }

  function onAct(name, data) {
    if (name === "dealer") act(`dealer ${data}`);
    else if (name === "declare") act(`declare ${data}`);
    else if (name === "new") newGameDialog.open();
    else if (name === "settings") settings.toggle();
    else if (name === "undo") {
      if (waiting) return;
      const r = engine.send("undo");
      if (r.ok) {
        director.cancelTimed();
        present(r.state, "direct");
        persist();
      }
    } else if (name === "hints") {
      const on = !raw.aids.hints;
      settings.set("hints", on);
      changed("hints", on);
    }
  }

  const mouse = (e) => e.pointerType === "mouse" || e.pointerType === "pen";
  function cardAt(e) {
    const hit = director.pick(e.clientX, e.clientY);
    return hit && (hit.zone === "your-hand" || hit.zone === "your-table") ? hit.cardId : null;
  }
  canvas.addEventListener("pointermove", (e) => {
    if (!mouse(e) || waiting) return;
    const id = cardAt(e);
    if (id !== hover) {
      hover = id;
      director.redecorate();
      canvas.style.cursor = id !== null && legal().includes(id) ? "pointer" : "default";
    }
  });
  canvas.addEventListener("click", (e) => {
    if (waiting || raw?.prompt?.kind !== "play") return;
    const id = cardAt(e);
    if (id === null) {
      if (chosen !== null) {
        chosen = null;
        director.rearrange();
      }
      return;
    }
    if (!legal().includes(id)) {
      act(`play ${id}`); // refused, and the engine says why
      return;
    }
    if (e.pointerType === "touch" && chosen !== id) {
      chosen = id;
      director.rearrange();
      return;
    }
    act(`play ${id}`);
  });

  // The test hook: what the browser checks read and drive.
  window.bezique3d = { state: () => raw, act, onAct, director, stage, ui, engine, waiting: () => waiting, screenPoint: (id) => director.screenPoint(S.model && placementCode(id)) };
  function placementCode(id) {
    return director.placement().find((m) => m.cardId === id)?.code;
  }

  if (!restore()) newGame();
}

main().catch((error) => {
  console.error(error);
  const p = document.getElementById("loading");
  if (p) p.textContent = `Bezique could not start: ${error.message}`;
});
