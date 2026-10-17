import assert from "node:assert/strict";
import test from "node:test";
import { anchorOf, badgeFor, settlementOf, victoryOf } from "../src/badges.js";
import { fold, itemsOf } from "../src/scoring.js";

test("a badge says the points and what earned them, tinted by who scored", () => {
  const b = badgeFor({ who: "you", cat: "declared", pts: 40, label: "bezique" });
  assert.deepEqual([b.text, b.who, b.anchor, b.big], ["+40 bezique", "you", "yourTable", false]);
  assert.equal(badgeFor({ who: "them", cat: "declared", pts: 100, label: "four aces" }).big, true);
  assert.equal(badgeFor({ who: "them", cat: "declared", pts: 20, label: "marriage" }).anchor, "theirTable");
  assert.equal(badgeFor({ who: "you", cat: "brisques", pts: 20, label: "brisques" }).text, "+20 brisques");
  assert.equal(badgeFor({ who: "you", cat: "brisques", pts: 10, label: "brisques" }).text, "+10 brisque");
  assert.equal(anchorOf({ cat: "seven" }), "stock");
  assert.equal(anchorOf({ cat: "last", who: "you" }), "trick");
});

test("a hand's settlement lists the categories that scored and the hand's total", () => {
  const ev = [
    { kind: "dealt", hand: 1, dealer: "you", turn_up: { id: 1, card: "7S" } },
    { kind: "took_trick", hand: 1, who: "them", points: 20 },
    { kind: "declared", hand: 1, who: "you", points: 40, text: 'You declare: "Forty for bezique."' },
    { kind: "took_trick", hand: 1, who: "them", points: 10 },
    { kind: "hand_ends", hand: 1, you: 50, them: 30 },
  ];
  assert.deepEqual(itemsOf(ev, 3).map((i) => i.cat), ["last"]); // ten, all of it the last trick
  const s = settlementOf(fold(ev), 1);
  assert.deepEqual(s.rows.map((r) => [r.cat, r.you, r.them]), [["brisques", 0, 20], ["declared", 40, 0], ["seven", 10, 0], ["last", 0, 10]]);
  assert.deepEqual(s.total, { you: 50, them: 30 });
  assert.equal(settlementOf(fold(ev), 9), null);
});

test("the game's end is worded for a win, a double and a loss", () => {
  assert.match(victoryOf({ winner: "you", double: true }, { you: 1000, them: 300 }).title, /Double game/);
  assert.equal(victoryOf({ winner: "you", double: false }, { you: 1000, them: 700 }).won, true);
  assert.equal(victoryOf({ winner: "them", double: false }, { you: 700, them: 1000 }).won, false);
});
