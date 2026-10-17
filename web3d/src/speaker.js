// Who speaks when: the opponent has one voice, so lines wait their turn in
// the order of their moments, on the table's clock (director.at). Chatter
// yields to the calls by the kit's rules (kit/js/dialogue.js, after
// strohmandeln's speaker.js): a chatter line is left out if it would come
// more than LATE after its own moment, and is cut when a call arrives that it
// would make more than SLACK late. A talkative table plays at the pace of a
// quiet one.
//
// createSpeaker({ clock, later, show, hide, pace }) -> { offer, stop }
//   clock() -> ms; later(ms, fn, kind) runs fn after ms on the table's clock
//   (kind "talk": a line still to be said; "linger": a box coming down);
//   show(line) puts the line's box up, hide(line) takes it down;
//   pace() -> a factor on how long a line stays: 1 at normal speed, less when fast.
//
// createBubble({ root, edge, view }) -> { show, hide, clear }: the speech
// bubble in the page's overlay, by the opponent's hand.
import { GAP, LATE, saying, SLACK } from "../../kit/js/dialogue.js";

// How long a line stays up: a person's time to say it (kit), scaled by the
// pace, and never gone before it could be read.
export const MIN_HOLD = 450;
export const holdFor = (text, pace = 1) => Math.max(MIN_HOLD, saying(text) * pace);

export function createSpeaker({ clock, later, show, hide, pace = () => 1 }) {
  let slots = []; // { line, start, end, state: "queued" | "showing" | "done" }
  let epoch = 0;

  // A queued line whose time has passed without being shown was dropped by a skip.
  const live = () => (slots = slots.filter((s) => (s.state === "queued" && s.start >= clock()) || s.state === "showing"));

  function cut(slot, now) {
    if (slot.state === "showing") hide(slot.line);
    slot.state = "done";
    slot.end = Math.min(slot.end, now);
  }

  // Offer a line at its moment: { text, call }. Returns whether it will be said.
  function offer(line) {
    const now = clock();
    live();
    if (line.call) {
      for (const slot of slots) {
        if (slot.line.call) continue;
        if (slot.state === "queued" || slot.end + GAP > now + SLACK) cut(slot, now);
      }
      live();
    }
    const tail = slots.length ? Math.max(...slots.map((s) => s.end)) + GAP : now;
    const start = Math.max(now, tail);
    if (!line.call && start - now > LATE) return false;
    const slot = { line, start, end: start + holdFor(line.text, pace()), state: "queued" };
    slots.push(slot);
    const mine = epoch;
    later(start - now, () => {
      if (mine !== epoch || slot.state !== "queued") return;
      slot.state = "showing";
      show(line);
      later(slot.end - slot.start, () => {
        if (mine !== epoch || slot.state !== "showing") return;
        slot.state = "done";
        hide(line);
      }, "linger");
    }, "talk");
    return true;
  }

  // A new sitting, an undo, or a skip that dropped what was still to come.
  function stop() {
    epoch++;
    for (const slot of slots) if (slot.state === "showing") hide(slot.line);
    slots = [];
  }

  return { offer, stop };
}

// The speech bubble: a small box in the overlay below the opponent's hand,
// faded in and out by CSS (.talk-bubble in style.css). `edge()` is
// director.handEdge("them"); `view()` the window's size.
export function createBubble({ root, edge, view = () => ({ width: window.innerWidth, height: window.innerHeight }) }) {
  const box = document.createElement("div");
  box.className = "talk-bubble";
  box.setAttribute("role", "status");
  box.setAttribute("aria-live", "polite");
  root.append(box);
  let timer = null;

  function place() {
    const at = edge();
    const { width, height } = view();
    const w = box.offsetWidth;
    const h = box.offsetHeight;
    box.style.left = `${Math.max(8, Math.min(width - w - 8, at.x - w / 2))}px`;
    box.style.top = `${Math.max(8, Math.min(height - h - 8, at.y + 14))}px`;
  }

  return {
    show(line) {
      clearTimeout(timer);
      box.textContent = line.text;
      place();
      box.classList.add("talk-show");
    },
    hide() {
      box.classList.remove("talk-show");
    },
    clear() {
      clearTimeout(timer);
      box.classList.remove("talk-show");
    },
  };
}
