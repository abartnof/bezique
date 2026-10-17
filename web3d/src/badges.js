// Points shown where they are earned: a badge ("+40 bezique") that rises from
// the cards that scored, a short count when a hand ends, and a celebration
// when the game is won. The descriptors are pure (scoring.js items in, plain
// objects out); createBadges draws them in a layer over the canvas and keeps
// to the director's clock (`later`), so they hold when the table is held and
// quicken with the speed. Nothing here takes the pointer or sits over the
// person's hand: badges rise from the table's own rows and the stock.
import { CATEGORIES, TARGET } from "./scoring.js";

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

// The hand-end count from the running state: a line a category that scored,
// then the hand's total. `names` words the two sides.
export function settlementOf(state, hand, { totals = state.totals } = {}) {
  const h = state.hands[hand];
  if (!h) return null;
  const rows = CATEGORIES.filter(([k]) => h[k].you + h[k].them > 0).map(([k, label]) => ({ cat: k, label, you: h[k].you, them: h[k].them }));
  const sum = (w) => CATEGORIES.reduce((n, [k]) => n + h[k][w], 0);
  return { hand, rows, total: { you: sum("you"), them: sum("them") }, totals: { ...totals } };
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

  function place(node, { x, y }) {
    node.style.left = `${x}px`;
    node.style.top = `${y}px`;
  }

  return {
    // One badge, risen from its anchor and gone after about two seconds.
    show(item, { nth = 0 } = {}) {
      if (instant()) return null;
      const b = badgeFor(item);
      const at = project(b.anchor, b.who);
      if (!at) return null;
      const node = el("div", `bd-badge bd-${b.who} bd-${b.kind}${b.big ? " bd-big" : ""}`, b.text, layer);
      node.style.setProperty("--life", `${ms(2000)}ms`);
      place(node, { x: at.x + nth * 6, y: at.y - nth * 30 });
      remove(node, ms(2100));
      return node;
    },

    // The count after a hand: each scoring line told, the numbers popping out of it.
    settle(summary, { names = { you: "You", them: "Opponent" } } = {}) {
      if (instant() || !summary) return null;
      const alive = live();
      const card = el("div", "bd-settle", null, layer);
      card.style.setProperty("--life", `${ms(4200)}ms`);
      el("div", "bd-settle-title", `Hand ${summary.hand} counted`, card);
      const head = el("div", "bd-settle-row bd-settle-head", null, card);
      el("span", null, names.you, head);
      el("span", null, "", head);
      el("span", null, names.them, head);
      const rows = [...summary.rows, { label: "Hand", you: summary.total.you, them: summary.total.them, sum: true }];
      rows.forEach((r, i) => {
        later(ms(350 + i * 450), () => {
          if (!alive()) return;
          const row = el("div", `bd-settle-row${r.sum ? " bd-sum" : ""}`, null, card);
          const you = el("span", `bd-you${r.you ? "" : " bd-none"}`, r.you ? String(r.you) : "–", row);
          el("span", "bd-label", r.label, row);
          const them = el("span", `bd-them${r.them ? "" : " bd-none"}`, r.them ? String(r.them) : "–", row);
          if (!reduced()) {
            if (r.you && !r.sum) this.sparks(you, r.you, "you");
            if (r.them && !r.sum) this.sparks(them, r.them, "them");
          }
        });
      });
      remove(card, ms(4300));
      return card;
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
        const colours = ["#e0b13a", "#c8442f", "#2f6fd0", "#f4efe2", "#3f9a58"];
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
            const s = el("div", `bd-spark bd-big bd-you`, v.double && i % 2 ? "x2" : "+" + [10, 20, 40, 100][i % 4], layer);
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
