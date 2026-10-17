// Who speaks when (src/speaker.js): one line at a time, in order, on the table's clock.
import assert from "node:assert/strict";
import test from "node:test";
import { createSpeaker, holdFor } from "../src/speaker.js";

function rig(pace = 1) {
  let now = 0;
  const timers = [];
  const log = [];
  const speaker = createSpeaker({
    clock: () => now,
    later: (ms, fn) => timers.push({ at: now + ms, fn }),
    show: (l) => log.push(["show", l.text, now]),
    hide: (l) => log.push(["hide", l.text, now]),
    pace: () => pace,
  });
  const advance = (ms) => {
    const to = now + ms;
    for (;;) {
      timers.sort((a, b) => a.at - b.at);
      const next = timers.find((t) => t.at <= to);
      if (!next) break;
      timers.splice(timers.indexOf(next), 1);
      now = next.at;
      next.fn();
    }
    now = to;
  };
  return { speaker, advance, log, at: () => now };
}

test("a line is shown at once and taken down after a time that grows with its length", () => {
  const r = rig();
  assert.ok(r.speaker.offer({ text: "Hello.", call: true }));
  r.advance(0);
  assert.deepEqual(r.log[0].slice(0, 2), ["show", "Hello."]);
  assert.ok(holdFor("a much longer line of words than before", 1) > holdFor("Hi.", 1));
  r.advance(holdFor("Hello.", 1) + 1);
  assert.equal(r.log.at(-1)[0], "hide");
});

test("lines wait their turn: one at a time, in order", () => {
  const r = rig();
  r.speaker.offer({ text: "First one.", call: true });
  r.speaker.offer({ text: "Second one.", call: true });
  r.advance(0);
  assert.equal(r.log.filter((l) => l[0] === "show").length, 1);
  r.advance(holdFor("First one.", 1) + 200);
  const shows = r.log.filter((l) => l[0] === "show").map((l) => l[1]);
  assert.deepEqual(shows, ["First one.", "Second one."]);
  const hideFirst = r.log.find((l) => l[0] === "hide");
  const showSecond = r.log.filter((l) => l[0] === "show")[1];
  assert.ok(showSecond[2] >= hideFirst[2]);
});

test("a chatter line that would come too late is left out", () => {
  const r = rig();
  r.speaker.offer({ text: "Hi.", call: true });
  assert.equal(r.speaker.offer({ text: "Chatter.", call: false }), true);
  for (let i = 0; i < 6; i++) r.speaker.offer({ text: `Another call number ${i} of some length.`, call: true });
  assert.equal(r.speaker.offer({ text: "More chatter.", call: false }), false);
});

test("a call cuts chatter that has not begun", () => {
  const r = rig();
  r.speaker.offer({ text: "Busy.", call: true });
  r.speaker.offer({ text: "Chatter.", call: false });
  r.speaker.offer({ text: "A call.", call: true });
  r.advance(20000);
  const shows = r.log.filter((l) => l[0] === "show").map((l) => l[1]);
  assert.deepEqual(shows, ["Busy.", "A call."]);
});

test("pace scales how long a line stays: faster play, shorter stay", () => {
  assert.ok(holdFor("Same words here.", 0.4) < holdFor("Same words here.", 1));
  assert.ok(holdFor("Same words here.", 0.4) >= 400);
});

test("stop forgets what was still to come", () => {
  const r = rig();
  r.speaker.offer({ text: "One.", call: true });
  r.speaker.offer({ text: "Two.", call: true });
  r.advance(0);
  r.speaker.stop();
  r.advance(20000);
  assert.deepEqual(r.log.filter((l) => l[0] === "show").map((l) => l[1]), ["One."]);
});
