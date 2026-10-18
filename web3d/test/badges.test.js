import assert from "node:assert/strict";
import test from "node:test";
import { anchorOf, badgeFor, victoryOf } from "../src/badges.js";

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

test("the game's end is worded for a win, a double and a loss", () => {
  assert.match(victoryOf({ winner: "you", double: true }, { you: 1000, them: 300 }).title, /Double game/);
  assert.equal(victoryOf({ winner: "you", double: false }, { you: 1000, them: 700 }).won, true);
  assert.equal(victoryOf({ winner: "them", double: false }, { you: 700, them: 1000 }).won, false);
});
