// Bezique at a three-dimensional table.
//
// A client of the wasm module (docs/PROTOCOL.md): it draws the person's view
// and sends back commands, and holds no rules. Every change of state is
// played out on the cards by the director (choreography.js), with the score
// and the narration at each event's moment.
import { createDeck } from "../../kit/js/deck.js";
import { createDirector } from "../../kit/js/director.js";
import { loadTextures, vectorWidth } from "../../kit/js/art.js";
import { jumboTextures } from "../../kit/js/faces.js";
import { decodeBase64 } from "../../kit/js/engine.js";
import { createScene } from "../../kit/js/scene.js";
import { chooseSurface } from "../../kit/js/surfaces.js";
import { choreograph, initialPlacement, opening } from "./choreography.js";
import { loadEngine } from "./engine.js";
import { createNewGame } from "./newgame.js";
import { gameConfig, loadPrefs, savePrefs, speedFactor, urlOverrides, withUrl } from "./prefs.js";
import { createSettings } from "./settings.js";
import { createScoreboard } from "./scoreboard.js";
import { fromState } from "./model.js";
import { PAGE_KEYS, dueIn } from "./tutor.js";
import { createTutorStore } from "./tutorstore.js";
import { createTutorView } from "./tutorview.js";
import TUTORIAL from "../tutorial.md";
import { parseTutorial } from "../../kit/js/tutorial.js";
import { createBubble, createSpeaker } from "./speaker.js";
import { createTalk } from "./talk.js";
import { createUi } from "./ui.js";
import { EYE, EYE_PORTRAIT, ZONES, ZONES_PORTRAIT } from "./zones.js";

/* global WASM_BASE64, ART, WORDS */

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
  // The tutorial is on for a new player and off in a test run (`?seed=`), unless `?tutor=1`.
  const tutorOn = () => (params.has("seed") ? params.get("tutor") === "1" : prefs.tutor !== false && prefs.tutor !== "off");
  // Table talk: quiet | normal | chatty (docs/PHRASES.md).
  const talkMode = params.get("talk") ?? prefs.talk ?? WORDS.default_mode;
  prefs.talk = Object.keys(WORDS.modes).includes(talkMode) ? talkMode : WORDS.default_mode;

  // ---- the scene, the cards -------------------------------------------------
  const canvas = document.getElementById("stage");
  const stage = createScene(canvas, {
    table: chooseSurface({ chosen: prefs.surface, saved: null }),
    ...(params.has("lowfx") ? { shadow: "pcf", shadowMap: 256 } : {}),
    eyes: { across: EYE, portrait: EYE_PORTRAIT },
  });
  const textures = await loadTextures(ART, { anisotropy: stage.renderer.capabilities.getMaxAnisotropy(), pixelRatio: stage.renderer.getPixelRatio() });
  const deck = createDeck(stage, textures);

  // The card faces: high-visibility Jumbo Index by default (every rank and
  // suit in the corner, the ten as 10), the classic art on request; drawn in
  // the browser, changeable at run time (`bezique3d.setFaces`).
  const faceSets = { classic: textures.faces };
  let facesShown = "classic";
  function setFaces(choice) {
    const want = choice === "classic" ? "classic" : "jumbo";
    prefs.faces = want;
    if (want === facesShown) return facesShown;
    faceSets[want] ??= jumboTextures(Object.keys(textures.faces), { width: textures.width ?? vectorWidth(stage.renderer.getPixelRatio()), anisotropy: stage.renderer.capabilities.getMaxAnisotropy() });
    deck.setFaces(faceSets[want]);
    facesShown = want;
    stage.render();
    return want;
  }
  setFaces(prefs.faces);
  const zonesNow = () => (stage.portrait ? ZONES_PORTRAIT : ZONES);

  // ---- state ------------------------------------------------------------------
  let S = null; // the director's state: { model, events, raw }
  let raw = null; // the engine's last state
  let seed = 0;
  let waiting = false; // cards moving: the person may not act
  let chosen = null; // a card picked on a touch screen, to be played by a second tap
  let hover = null;

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
    stage.setInset(window.innerWidth >= 900 ? 360 : 0); // the panels stand in the left margin (style.css)
    stage.setStrips({ top: ui.hudBottom() + 6, foot: window.innerHeight - ui.barTop() + 6, raised: 0 }, { instant: true });
  }
  const board = createScoreboard({ ui, director, zones: zonesNow, overlay: document.getElementById("overlay"), reduced: () => window.matchMedia?.("(prefers-reduced-motion: reduce)").matches });
  ui.onLayout(frame);
  frame();
  stage.onReframe = () => director.relayout();

  // ---- the tutorial (web3d/src/tutor.js): a page at the first moment its element appears --
  const pages = parseTutorial(TUTORIAL, PAGE_KEYS);
  const pageOf = (key) => pages.find((p) => p.key === key);
  const tutorStore = createTutorStore((() => { try { return params.has("seed") && !params.has("keep") ? null : window.localStorage; } catch { return null; } })());
  let afterPage = null; // what "Got it" does for a page that fell due
  const tutorView = createTutorView(document.getElementById("overlay"), {
    close: ({ dontShow, help, back }) => {
      if (dontShow) { prefs.tutor = false; savePrefs(prefs); }
      if (help) tutorView.close();
      else if (back) openHelp();
      else if (afterPage) afterPage();
    },
    pick: (key) => { const p = pageOf(key); if (p) tutorView.show(p, { back: true }); },
    tips: (on) => { prefs.tutor = on; savePrefs(prefs); openHelp(); },
    reset: () => { tutorStore.reset(); prefs.tutor = true; savePrefs(prefs); openHelp(); },
  });
  function openHelp() {
    tutorView.help(pages, { seen: tutorStore.seen(), tips: tutorOn() });
  }
  // Pages due at this change of state, one after another, holding the table's clock until the last is dismissed.
  function tutorial(prev, next, timing) {
    if (!tutorOn() || tutorView.isOpen()) return;
    const first = dueIn(prev, next, tutorStore.seen());
    if (!first) return;
    const run = (release) => {
      const go = (step) => {
        tutorStore.add(step.key);
        afterPage = () => {
          const more = tutorOn() ? dueIn(prev, next, tutorStore.seen()) : null;
          if (more) go(more);
          else { afterPage = null; tutorView.close(); release?.(); }
        };
        tutorView.show(pageOf(step.key));
      };
      go(first);
    };
    if (!timing) return run();
    const ms = first.event !== null && timing.beats[first.event] !== undefined ? timing.beats[first.event] : Math.max(0, ...Object.values(timing.beats));
    director.gate(ms, run);
  }

  // ---- showing a state --------------------------------------------------------
  function sync() {
    board.rest(raw);
    ui.setInfo({ trump: raw.trump_symbol ?? null, stock: raw.stock ?? 64, hand: raw.hand_number ?? null, level: raw.level });
  }

  function rested() {
    waiting = false;
    if (!raw) return;
    sync();
    ui.setPrompt(raw, { busy: false, hint: raw.aids.hints ? (raw.hint ?? {}) : null, layers: { facts: prefs.hintFacts !== false, rules: prefs.hintRules !== false } });
    director.redecorate();
  }

  // ---- table talk: the opponent's lines, one at a time, in a bubble by its hand
  const bubble = createBubble({ root: document.getElementById("overlay"), edge: () => director.handEdge("them") });
  const speaker = createSpeaker({
    clock: () => director.clock(),
    later: (ms, fn, kind) => director.at(ms, fn, kind),
    show: (line) => bubble.show(line),
    hide: () => bubble.hide(),
    pace: () => Math.min(1.5, Math.max(0.35, 1 / speedFactor(prefs, systemReduced()))),
  });
  const talk = createTalk({ words: WORDS, rng: Math.random, say: (line) => speaker.offer(line), mode: prefs.talk });
  const setTalk = (mode) => {
    talk.setMode(mode);
    prefs.talk = mode;
    savePrefs(prefs);
  };
  const hushTalk = () => {
    speaker.stop();
    director.drop("talk");
  };

  function applyEvent(e, index) {
    talk.onEvent(e, { totals: board.running().totals });
    ui.say(e.text);
    board.event(S.events, index);
  }

  // `how`: "deal" (a new sitting, dealt out from the pack), "direct" (an undo
  // or a sitting restored: the cards at once) or "advance" (the next move played out).
  function present(next, how) {
    const first = S && how !== "deal" ? S.events.length : 0;
    const state = { model: fromState(next), events: next.events, raw: next };
    const before = how === "advance" ? raw : null;
    raw = next;
    waiting = true;
    chosen = null;
    ui.setPrompt(raw, { busy: true });
    let timing = null;
    if (how === "deal") timing = director.restart(state, { dealt: true });
    else if (how === "direct" || !S) director.restart(state);
    else timing = director.advance(state);
    S = state;
    tutorial(before, next, how === "direct" ? null : timing);
    if (how === "direct" || !timing) {
      board.reset(next.events);
      rested();
      return;
    }
    if (how === "deal") board.reset(next.events, { empty: true });
    for (const [index, ms] of Object.entries(timing.beats)) {
      const e = next.events[Number(index)];
      if (e && Number(index) >= first) director.at(ms, () => applyEvent(e, Number(index)));
    }
  }

  function persist() {
    if (raw && raw.record) store.set(SITTING, { v: raw.record_version, level: raw.level, persona: prefs.persona, erratic: prefs.erratic, seed, hints: raw.aids.hints, tracker: raw.aids.tracker, record: raw.record });
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
    hushTalk();
    engine.new(gameConfig(prefs, { seed }));
    if (prefs.hints) engine.send("set hints on");
    if (prefs.tracker) engine.send("set tracker on");
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
    if (saved.tracker) engine.send("set tracker on");
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
    else if (name === "help") {
      if (!tutorView.isOpen()) openHelp();
    }
    else if (name === "undo") {
      if (waiting) return;
      const r = engine.send("undo");
      if (r.ok) {
        director.cancelTimed();
        hushTalk();
        present(r.state, "direct");
        talk.onEvent({ kind: "undo" });
        persist();
      }
    } else if (name === "tracker") {
      prefs.tracker = !raw.aids.tracker;
      savePrefs();
      const r = engine.send(`set tracker ${prefs.tracker ? "on" : "off"}`);
      raw = r.state;
      persist();
      rested();
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
  window.bezique3d = { talk, setTalk, speaker, board, setFaces: (c) => { const r = setFaces(c); savePrefs(prefs); return r; }, facesShown: () => facesShown, state: () => raw, act, onAct, director, stage, ui, engine, waiting: () => waiting, screenPoint: (id) => director.screenPoint(S.model && placementCode(id)) };
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
