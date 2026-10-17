# The computer's two levers: skill and personality

Two independent dials, fixed for a sitting. Neither reads the other.

- **Skill** (levels 1 to 5): what the opponent can do. The ladder of
  capabilities in `agents.rs` and `docs/DESIGN.md` §10.1, unchanged by this
  document. Deterministic: a function of seed and view.
- **Personality** (six personas): which of several about-equally-good things
  it prefers. `opponent::personas()`. A persona is a taste, not a handicap.

A player who picks "Gambler" at level 2 and a player who picks "Gambler" at
level 5 meet the same temperament at two strengths; a player who picks level 3
meets the same strength under six temperaments. The new-game dialog offers
both choices separately (`web3d/personas.json`, below).

## 1. How personality is bounded

A persona is a [`Style`] (`agents.rs`): a few preference weights on the
keep-value scale (a seven is 0, an ace 11, a trump 8 more) and one
tie-breaking `Leaning`. Rules, enforced by tests in `opponent.rs`:

1. A persona moves at most **two** weights, none by more than 20 points, and
   never changes which rung's skills run (memory, the exact endgame, the
   last-stock-trick fight, the lookahead). The rungs are selected by level
   alone.
2. A `Leaning` acts only among cards within one point of the cheapest to
   throw (`pick` in `agents.rs`): it chooses *among near-equals*.
3. **Measured tolerance** `opponent::TOLERANCE` = 2 points a hand against
   `balanced` at the same level (about 1.4% of a hand's 140-odd points; one
   ladder step from rung 3 to 4 is +2.3). The weight shifts are not equal-value
   choices by construction (a larger marriage weight changes what it keeps),
   so the bound is established by measurement (section 4), and a persona that
   measured better than balanced was a better player, not a style, and was
   pulled back (the first gambler and hunter, section 4).

## 2. Audit of the personas as they stood

Before this change: `concentrates`, `close`, `bezique`, plus `balanced`.

| Old persona | What it changed | Orthogonal to skill? | Period advice encoded |
|---|---|---|---|
| `concentrates` | `Leaning::Concentrate`: among exactly equal cards, throw from the shortest suit | Yes by construction (ties only); measured -0.36 at rung 2, -0.36 at rung 4 (`personas-rung4.txt`) | Foster, "pick one or two suits or melds and discard the rest" (hints, Complete Hoyle pp. 366-368; LITERATURE S47). Cady p. 22: decide "which suit or suits he will go for and discard from the others" |
| `close` | `Leaning::Close`: among near-equals, play a card already on the table | Yes (ties within a point) | Cady p. 22: "the player should play his declared cards and thus avoid showing his opponent more than is actually necessary"; Pardon (LITERATURE S56) says the same |
| `bezique` | weight `bezique` 20 -> 40 | No: a weight, not a tie-break. Measured -1.35 (se 0.89) at rung 4, near the old limit; a first redesign (`bezique` 40 with `sequence` +3) measured -4.0 to -5.7 a hand at rung 2 | Pardon: keep the bezique cards "until near the end, even after declaring bezique" (S56); Cady p. 29 hint III keeps the aces of the bezique suits for the last eight |

Gaps found:

- Four of the six temperaments a reader of the period books would expect were
  missing: nothing hoarded marriages and fours (Foster: marriages and fours
  are the chief source of score; the old books' "keep kings and queens to
  marry rather than aces", 1864 and 1868 Hoyles, S45 and S46); nothing declared
  early or leaned on brisques; the personas were named by mechanism
  ("concentrates") not as archetypes, with no text for the dialog.
- The `bezique` persona's weight shift was visible but not safe: at the
  larger shift it was a worse player.
- No persona touched declaring. The balanced player holds back the larger
  combination for a later trick (single bezique before the double, marriage
  before the sequence; Foster, Rubicon hints). A cautious temperament that
  declares at once was not expressible.
- Brisques resist style: a first Hunter that led aces and tens (Cady p. 23,
  "he should not lead a ten if he can avoid doing so", argues against it) lost
  2 to 4 points a hand and won *fewer* brisques. The lever that works is
  readiness to spend a card to win one on the follow (`take_brisque`).

Advice not encoded, deliberately: trumping a brisque trick and caution in the
last eight tricks (Cady p. 23, Pardon "in the last eight tricks save your aces
and tens and win his") are *skill*: from level 3 the last eight tricks are
played exactly (`solver.rs`), so there is nothing left for a temperament to
choose. Playing for the lead versus for declarations ("the lead is a
disadvantage unless you can declare", Foster; Cady p. 29 hint I) is a tuned
weight set (`take_to_declare`, `urgency`), shared by all personas.

## 3. The personas

Each has a key, a name, a description for the dialog (kind, short, no real
names; the opponent is "your opponent") and what it encodes.

| Key | Name | Dialog description | Encoding | Period advice |
|---|---|---|---|---|
| `balanced` | The Steady Hand | Plays the old books' maxims evenly, with no marked taste. | the tuned default | Pole, Pardon, Foster, Hoffmann (LITERATURE §7) |
| `collector` | The Collector | Keeps its kings and queens back and loves to declare a marriage. | marriage +2, four +2; `Leaning::Collect` (of near-equals, throw anything but a king or queen) | Foster; Hoyle 1864/1868: keep kings and queens to marry rather than aces (S45, S46) |
| `hunter` | The Brisque Hunter | Chases the aces and tens, and will spend a good card to win one. | `take_brisque` 20 -> 40 | Cady p. 23: a brisque lost is "a difference of twenty"; 1864/1868 Hoyles: take aces and tens as brisques when you can (S45, S46) |
| `cautious` | The Careful Hand | Declares what it can at once, and shows you as little of its hand as it can. | `prompt` (declare the best announcement now, no holding back); `Leaning::Close` | Pardon, Cady p. 22: play declared cards, show nothing |
| `gambler` | The Gambler | Plays for the big prizes: double bezique and the sequence. | `bezique` 20 -> 26 | Pardon: keep the bezique cards to the end, even after declaring bezique (S56) |
| `specialist` | The Specialist | Picks a suit or two to build on and sheds the rest. | `Leaning::Concentrate` | Foster; Cady p. 22 |

The persona is part of the sitting: same seed and settings give the same
game, and `Session::replay` replays a record under the settings it was made
with. `RECORD_VERSION` is 4 (the persona list and play changed; the golden
fingerprint in `session.rs` is re-pinned). Older saved sittings are refused,
not misplayed.

## 4. Measurement: is personality orthogonal to skill?

Design, fixed before the first batch (`measure personality`,
`crates/bezique-core/src/bin/measure.rs`). Comparison: each persona against
`balanced` at the same level, mirrored pairs (each deal played twice with the
seats swapped), first hand of a game. Batch 1,000 pairs; floor 2,000 pairs
(no look before); ceiling 16,000; 16 equally spaced looks; stopping boundary
O'Brien-Fleming, z_k = 2.09 sqrt(16/k). A run stops when (a) the persona's
headline style statistic has crossed the boundary and the 95% interval of
the strength difference lies inside +-2 points a hand, or (b) the strength
difference's boundary interval lies wholly outside +-2 (the persona changes
skill), or at the ceiling. Compute: under 4 minutes at both levels.

Full numbers: `measurements/personality-level2.txt` (rung 2, where the
personas were calibrated) and `measurements/personality-level4.txt` (rung 4;
the personas change play only while the stock lasts, where rung 4 and rung 5
play alike). Points a hand, persona minus balanced, 95% interval at the last
look:

| Persona | Level | Pairs used | Strength (margin) | Headline style statistic | Stopped |
|---|---|---|---|---|---|
| collector | 2 | 14,000 | -0.55 [-1.35, +0.26] | combination points +0.92 [+0.17, +1.67] | style clear, strength inside |
| collector | 4 | 12,000 | -0.37 [-1.23, +0.50] | combination points +1.08 [+0.27, +1.89] | same |
| hunter | 2 | 4,000 | +0.83 [-0.10, +1.76] | brisque points +0.92 [+0.68, +1.15] | same |
| hunter | 4 | 3,000 | +0.30 [-0.75, +1.35] | brisque points +0.57 [+0.34, +0.80] | same |
| cautious | 2 | 7,000 | +0.32 [-1.10, +1.75] | tricks taken when scoring -0.07 [-0.11, -0.03] (earlier); combinations scored -0.04 a hand | same |
| cautious | 4 | 9,000 | +0.60 [-0.63, +1.84] | tricks when scoring (see file) | same |
| gambler | 2 | 7,000 | -0.11 [-0.51, +0.29] | double beziques +0.0010 a hand (z +3.6) | same |
| gambler | 4 | 7,000 | -0.09 [-0.48, +0.30] | double beziques (see file) | same |
| specialist | 2 | 16,000 | -1.18 [-2.26, -0.10] | combinations scored -0.013 [-0.023, -0.003] | ceiling |
| specialist | 4 | 16,000 | -1.70 [-2.78, -0.62] | combinations scored -0.011 [-0.021, -0.002] | ceiling |

Reading:

- Orthogonality holds for collector, hunter, cautious and gambler: each
  strength interval lies inside +-2 at both levels, and each differs from
  balanced in its own statistic. The strength intervals are inside the
  tolerance as stated, not zero.
- The style effects are real but small, as a tie-break should be: the gambler
  declares double bezique about 10% more often than balanced (+0.001 on a base
  of about 0.01 a hand); the hunter wins about 0.6 to 0.9 more brisque points
  a hand; the collector scores about 1 more point a hand in combinations and
  1.2 fewer in brisques; the careful hand declares about half a trick
  sooner on average (-0.07 tricks) and makes fewer, larger announcements.
  They are visible over a session, not a hand. **Gap**: if the maintainer
  wants a more theatrical opponent, widening the effect costs strength; the
  gambler's `bezique` at 28 measured +3.3 a hand at rung 2 (+3.6 at rung 4),
  which means the default is slightly undervaluing bezique cards and that
  weight should be reviewed as a skill change, not hidden in a persona.
- **The specialist is the weak point**: -1.2 (level 2) and -1.7 (level 4)
  points a hand, its interval reaching past -2 at the upper end. The point
  estimate is inside the tolerance and the cost is about 1% of a hand, but
  it is the only persona that is measurably a little weaker. It is kept
  because it encodes the clearest period advice and its cost is under the
  step from rung 3 to 4; reopen if a stricter bound is wanted.
- Rejected on measurement: gambler with `sequence` raised (cost 5 a hand),
  gambler with `bezique` 40 and `sequence` +3 (-4.0 to -5.7), `bezique` 14 (-1.7), `bezique` 26 with `sequence` 6 (-1.4), hunter that leads brisques (-2 to
  -4 and fewer brisques won), hunter with the lead pull plus `take_brisque`
  +10 (-1.75 to -1.9). These runs used the same harness
  (`MEASURE_VARIANT`) but not the full rule; they are history.

Reproduce: `cargo run --release -p bezique-core --bin measure -- personality`
(`MEASURE_LEVEL=4` for rung 4; a persona key restricts it).

## 5. For clients

`web3d/personas.json`: `[{key, name, description}]` in the engine's order.
The engine's `opponent::personas_json()` generates it
(`measure personas-json > web3d/personas.json`), and a test
(`the_clients_list_of_personas_is_the_engines`) fails if the file drifts.
The wasm module exports the same list (`bezique_personas`,
`bezique_personas_len`), and the state carries the sitting's persona key
(`docs/PROTOCOL.md`). `bezique_new`'s `persona` argument is the 1-based place
in the list, 0 to draw one from the seed.
