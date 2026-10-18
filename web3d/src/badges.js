// Points shown where they are earned: a badge ("+40 bezique") that rises from
// the cards that scored; when a hand ends, the same units told again, one
// badge each at its place over a tally that ticks up (handcount.js); and a
// celebration when the game is won. The descriptors are pure (scoring.js items
// in, plain objects out); createBadges draws them in a layer over the canvas
// and keeps to the director's clock (`later`), so they hold when the table is
// held and quicken with the speed. Nothing here takes the pointer or sits over
// the person's hand: badges rise from the table's own rows, piles and stock.
import { TARGET } from "./scoring.js";

// Where on the table a scoring item is earned: a declaration over its player's
// laid row, brisques and the last trick over the trick, the sevens at the stock
// (where the turn-up lies).
export function anchorOf(item) {
  if (item.cat === "declared") return item.who === "you" ? "yourTable" : "theirTable";
  if (item.cat === "seven") return "stock";
  return "trick";
}

// The badge for one item: { who, kind, text, pts, anchor, big }.
export function badgeFor(item) {
  const label = item.label === "brisque" || item.label === "brisques" ? (item.pts > 10 ? "brisques" : "brisque") : item.label;
  return {
    who: item.who,
    kind: item.cat,
    pts: item.pts,
    text: `+${item.pts} ${label}`,
    anchor: anchorOf(item),
    big: item.pts >= 100,
  };
}

// The words of the game's end, in the person's terms.
export function victoryOf({ winner, double }, totals) {
  const won = winner === "you";
  return {
    won,
    double: Boolean(double),
    title: won ? (double ? "Double game: you win!" : "Game: you win!") : "Your opponent wins the game",
    sub: `${totals.you} to ${totals.them}${double ? ". The loser had under 500: it counts double" : ""}`,
    target: TARGET,
  };
}

function el(tag, cls, text, parent) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text != null) e.textContent = text;
  if (parent) parent.appendChild(e);
  return e;
}

// `opts`: later(ms, fn) the director's clock; speed() the table's speed (1000 is Instant);
// project(anchor, who) -> { x, y } the screen point of a table anchor; reduced() calm motion.
export function createBadges(root, { later, speed = () => 1, project, reduced = () => false }) {
  const layer = el("div", "bd-layer", null, root);
  layer.setAttribute("aria-hidden", "true");
  const instant = () => speed() > 10;
  const ms = (n) => n / Math.max(0.25, Math.min(speed(), 4)); // wall time of a nominal duration
  const remove = (node, after) => later(after, () => node.remove());
  let epoch = 0;
  const live = () => {
    const mine = epoch;
    return () => mine === epoch;
  };

  // The tally of the hand being counted, high in the middle: "Hand 3   85 - 40".
  let tally = null;
  function tallyOf(hand) {
    if (tally && tally.hand === hand && tally.epoch === epoch) return tally;
    const node = el("div", "bd-tally", null, layer);
    const mid = project("trick", "you");
    if (mid) node.style.top = `${mid.y}px`;
    el("span", "bd-tally-hand", `Hand ${hand}`, node);
    el("span", "bd-tally-who", "You", node);
    el("span", "bd-tally-you", "0", node);
    el("span", "bd-tally-sep", "–", node);
    el("span", "bd-tally-them", "0", node);
    el("span", "bd-tally-who", "Opp", node);
    const set = ({ you, them }, { final = false, names } = {}) => {
      node.querySelector(".bd-tally-you").textContent = String(you);
      node.querySelector(".bd-tally-them").textContent = String(them);
      node.classList.toggle("bd-final", final);
      if (names) node.setAttribute("aria-label", `Hand ${hand}: ${names.you} ${you}, ${names.them} ${them}`);
    };
    tally = { hand, epoch, node, set, fade: (after) => remove(node, after) };
    return tally;
  }

  // A badge's words, with the side that scored told in words, not in colour.
  function sided(node, who, text) {
    el("span", "bd-side", who === "you" ? "You" : "Opp", node);
    node.append(text);
  }

  // Placed at a screen point, kept inside the window (a pile at the edge of the table).
  function place(node, { x, y }) {
    const half = node.offsetWidth / 2 + 6;
    const width = layer.clientWidth || window.innerWidth;
    node.style.left = `${Math.max(half, Math.min(width - half, x))}px`;
    node.style.top = `${y}px`;
  }

  return {
    // One badge, risen from its anchor and gone after about two seconds.
    show(item, { nth = 0 } = {}) {
      if (instant()) return null;
      const b = badgeFor(item);
      const at = project(b.anchor, b.who);
      if (!at) return null;
      const node = el("div", `bd-badge bd-${b.who} bd-${b.kind}${b.big ? " bd-big" : ""}`, null, layer);
      sided(node, b.who, b.text);
      node.style.setProperty("--life", `${ms(2000)}ms`);
      place(node, { x: at.x + nth * 6, y: at.y - nth * 30 });
      remove(node, ms(2100));
      return node;
    },

    // One unit of the hand's count: its badge, risen at its place on the table
    // (lifted clear of the last ones there), and the tally ticked up.
    unit(u) {
      if (instant()) return null;
      tallyOf(u.hand).set(u.running);
      const at = project(u.anchor, u.who);
      if (!at) return null;
      const node = el("div", `bd-badge bd-count bd-${u.who} bd-${u.cat}${u.badge.big ? " bd-big" : ""}`, null, layer);
      sided(node, u.who, u.badge.text);
      node.style.setProperty("--life", `${ms(1500)}ms`);
      place(node, { x: at.x, y: at.y - u.stack * 34 });
      remove(node, ms(1600));
      return node;
    },

    // The hand's total: the tally made large, the figures thrown out as sparks.
    total(step, { names = { you: "You", them: "Opponent" } } = {}) {
      if (instant()) return null;
      const t = tallyOf(step.hand);
      t.set({ you: step.you, them: step.them }, { final: true, names });
      if (!reduced()) {
        if (step.you) this.sparks(t.node.querySelector(".bd-tally-you"), step.you, "you");
        if (step.them) this.sparks(t.node.querySelector(".bd-tally-them"), step.them, "them");
      }
      t.fade(ms(1200));
      return t.node;
    },

    // The double game, the count's last figure before the banner.
    double() {
      if (instant()) return null;
      const node = el("div", "bd-badge bd-double-badge bd-big", "Double game: it counts twice", layer);
      node.style.setProperty("--life", `${ms(1900)}ms`);
      const mid = project("trick", "you");
      if (mid) place(node, { x: mid.x, y: mid.y + 74 });
      remove(node, ms(2000));
      return node;
    },

    // Small numbers thrown out of a node, as a firework's sparks.
    sparks(from, pts, who, count = pts >= 100 ? 9 : 5) {
      const box = from.getBoundingClientRect();
      const vals = pts >= 100 ? [pts] : [10, 20, 40].filter((v) => v <= pts);
      for (let i = 0; i < count; i++) {
        const s = el("div", `bd-spark bd-${who}`, `+${vals[i % vals.length]}`, layer);
        const angle = (Math.PI * 2 * i) / count + Math.random() * 0.5;
        const dist = 40 + Math.random() * 50;
        s.style.left = `${box.left + box.width / 2}px`;
        s.style.top = `${box.top + box.height / 2}px`;
        s.style.setProperty("--dx", `${Math.cos(angle) * dist}px`);
        s.style.setProperty("--dy", `${Math.sin(angle) * dist - 20}px`);
        s.style.setProperty("--life", `${ms(1300)}ms`);
        remove(s, ms(1400));
      }
    },

    // The game won (or lost): a banner, and for a win a fall of confetti and numbers, bigger for a double.
    celebrate(outcome, totals) {
      if (instant()) return null;
      const v = victoryOf(outcome, totals);
      const node = el("div", `bd-victory${v.won ? " bd-won" : ""}${v.double ? " bd-double" : ""}`, null, layer);
      node.style.setProperty("--life", `${ms(v.double ? 6500 : 5000)}ms`);
      el("div", "bd-victory-title", v.title, node);
      el("div", "bd-victory-sub", v.sub, node);
      if (v.won && !reduced()) {
        const n = v.double ? 70 : 36;
        const colours = ["#ffffff", "#d9d9d9", "#9a9a9a"];
        for (let i = 0; i < n; i++) {
          const c = el("div", "bd-confetti", null, layer);
          c.style.left = `${Math.random() * 100}%`;
          c.style.background = colours[i % colours.length];
          c.style.setProperty("--dx", `${(Math.random() - 0.5) * 160}px`);
          c.style.setProperty("--delay", `${ms(Math.random() * 1200)}ms`);
          c.style.setProperty("--life", `${ms(2600 + Math.random() * 1400)}ms`);
          remove(c, ms(5200));
        }
        const nums = v.double ? 14 : 7;
        for (let i = 0; i < nums; i++) {
          later(ms(300 + i * 280), () => {
            const s = el("div", "bd-spark bd-big", v.double && i % 2 ? "x2" : "+" + [10, 20, 40, 100][i % 4], layer);
            const a = (Math.PI * 2 * i) / nums;
            s.style.left = `${35 + (i % 4) * 10}%`;
            s.style.top = `${30 + (i % 3) * 8}%`;
            s.style.setProperty("--dx", `${Math.cos(a) * 90}px`);
            s.style.setProperty("--dy", `${Math.sin(a) * 90 - 30}px`);
            s.style.setProperty("--life", `${ms(1600)}ms`);
            remove(s, ms(1700));
          });
        }
      }
      remove(node, ms(v.double ? 6600 : 5100));
      return node;
    },

    clear() {
      epoch++;
      layer.textContent = "";
    },
    layer,
  };
}
