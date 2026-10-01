# Bezique: design

> Status: living document, first written 2026-10-01, before any code. With
> `PLAN.md` it should be enough to resume the work if all other context is
> lost. The rules themselves are in `docs/RULES.md`, and the sources in
> `docs/LITERATURE.md`.

## 1. What we are building

A playable, teachable version of two-player **bezique**, the 64-card game that
was a craze in Britain from 1869, a fixture of club and country-house play into
the 1930s, and Winston Churchill's favourite game, and that is now played by
few. The audience is new players, so teaching is a first-class goal.

Deliverables, in priority order:

1. A correct, fast, well-tested **rules engine**.
2. An **opponent** whose skill, consistency and style can each be adjusted.
3. A **table**: one self-contained, offline HTML page with a three.js scene
   and a Material Design 3 overlay, with a tutorial, hints and explanations,
   every aid a toggle.

The project is a sibling of `piquet` (same maintainer, same machine). It
borrows that project's architecture, front end and card art, and its
hard-won lessons. Section 3 says exactly what carries over.

Non-goals for now: networked play; three- and four-handed bezique; the
multi-pack games (Rubicon, six-pack), which are candidates for later as
separate games; sound of any kind.

## 2. Engineering conventions

- **Rust**, from the first line (§4 gives the reasoning).
- **Test-driven development.** Tests precede implementation, always.
- **Atomic commits** straight to `main`, often: one logical change each.
- **Modular**: trick-taking, declarations, scoring, observation, agents, the
  session and every client are separate. Meld logic never leaks into trick
  logic.
- **Plain data.** Game state is plain, serialisable values. Every action
  returns a new state or mutates one owned value; there is no shared mutable
  state.
- **The engine holds the rules; clients hold none.** The browser table and the
  terminal are clients of one protocol (§9).
- **No runtime dependencies** in the core crate. Test-only dependencies are
  allowed when they earn their place.
- **Measure rather than argue**, wherever measuring is cheap, and always with
  mirrored pairs (§10.5).
- **Documents are public-facing**: neutral, dated, and free of session diary.

## 3. What carries over from piquet

`piquet` is a finished two-player trick-and-meld game with the same kind of
engine, protocol and table. Britannica files the two games together as
trick-and-meld games. The question for each part is whether it is
game-agnostic.

| Part | piquet | For bezique |
|---|---|---|
| Workspace shape | `piquet-core`, `piquet-wasm`, `piquet-cli` | **Copy**: `bezique-core`, `bezique-wasm`, `bezique-cli` |
| Scoring as an ordered event log | `scoring.rs` | **Copy the idea**: bezique's scores are simpler, and the log still drives narration and the score tab |
| `observation`, the information sets | `observation.rs` | **Copy the discipline**; the content is new (§8) |
| The table session: undo, replay, aids, hints | `table.rs` | **Adapt**: same loop, new prompts |
| JSON protocol, wasm exports | `docs/PROTOCOL.md`, `piquet-wasm` | **Adapt**: same loop; cards gain physical identities (§5) |
| Capability ladder, erraticism, style | `agents.rs`, `style.rs` | **Copy the design**; the rungs are new (§10) |
| Mirrored-pair tournaments | `tournament.rs` | **Copy** |
| Exact endgame solver | `solver.rs` | **Rewrite**: bezique's endgame is perfect information and smaller |
| Seeded generator | Mersenne Twister, for parity with the Python oracle | **Replace** with a small modern generator; there is no oracle to match (§4) |
| 3D scene, cards, ink lines, motion | `web3d/src`: scene, cards, materials, kinematics, easing, timeline, framing, surfaces, art | **Copy verbatim** |
| Layout, choreography, overlay, celebrations | `web3d/src` | **Adapt**: every place a card code is a unique key must change (§11) |
| Card art | 33 WebP images (32 faces and a back), 1.6 MB embedded | **Copy**: bezique uses the piquet pack, twice |
| Python oracle and golden vectors | `python/`, `vectors/` | **Not repeated** (§4) |

**Copy, do not share, for now.** A common crate or package would couple the
two projects' lifecycles for a small shared surface. The rank orders differ
(piquet's ten sits between the jack and the nine, bezique's above the king),
the packs differ (32 unique cards against 64 with duplicates), and the hand
types differ (`u32` against `u64`). The front end has the larger generic
core, and is the strongest candidate for extraction if a third card game
arrives. Copied files keep a one-line header naming their origin.

## 4. Language and method

Piquet began in Python and moved to Rust after two days. The Python phase
bought three things: a quick loop for discovering the rules and the AI; an
oracle, whose golden vectors caught three defects in the Rust port; and time
to make the language decision with evidence (piquet `DESIGN.md` §13). It cost
a second implementation, a vector format, and a list of things "the port left
behind".

**Bezique starts in Rust**, because each of the three reasons has lapsed or has
a cheaper substitute:

1. **The language decision is made and its pipeline proven.** Rust → wasm →
   JSON protocol → three.js page shipped in piquet. The toolchain (Rust 1.98
   with the `wasm32-unknown-unknown` target, Node, Chromium) is installed.
2. **Bezique's AI is search-bound from the start.** A hand has 24 tricks of
   hidden information before the endgame. The stronger opponents and every
   measurement of them will run Monte Carlo search, thousands of mirrored hands
   at a time. In piquet, the interpreter cost 20–54 times the speed of compiled
   code on exactly this kind of work (piquet `DESIGN.md` §13.7). A Python
   bezique would have to be ported before its AI became interesting.
3. **The rules are already researched.** The literature review, with Foster's
   and Cady's texts on disk, leaves rule *choices*, not rule *discovery*, and
   the choices are made in `RULES.md`.

What replaces the oracle, which was always the real value of a second
implementation:

- **Brute-force references inside the test suite.** The fast meld-legality
  code is checked against an exhaustive search over card assignments, and the
  endgame solver against plain minimax, on thousands of random positions.
  This is the oracle pattern without a second language.
- **Worked examples from the period texts** (`RULES.md`, "Worked examples")
  as fixtures, written before the code they test.
- **Exact targets for statistical tests.** `analysis/bezique_stats.py` gives
  exact odds for what the deal and the draw deliver (56.45% for the two
  bezique cards, 5.66% for all four, 22.64% for the trump sequence), and these
  are independent of how anyone plays.
- **Invariants over random play**: 160 brisque points per hand, 32 tricks, 64
  cards conserved, scores reconciling with the event log.

**Keeping experiments cheap.** Piquet's discoveries came from thirty-line
scripts run once, and a compiled language makes those more ceremonious. The
mitigation is structural: a `src/bin/` measurement harness with a shared
mirrored-pair driver and TSV output, so a new experiment is a short file;
Python only for analysing what those emit.

**Not TypeScript.** It would remove the wasm build, but the search speed matters
more here than in piquet, and piquet's audit of JavaScript integer hazards (its
`DESIGN.md` §2.1) still applies.

## 5. The card model

Bezique has two physically distinct cards of every kind, which the rules treat
as identical in play but not in declaring: a K♠ that has been married carries
that history, and its twin does not. The model has to represent both
identities without the twins costing the search anything.

**Card types and physical cards.**

- A **card type** is a suit and a rank: 32 of them. Ranks are indexed low to
  high in bezique order, 7 8 9 J Q K 10 A; the type index is
  `suit × 8 + rank`.
- A **card** (physical) is `type × 2 + copy`, 0 to 63. The copies are
  interleaved, so a suit occupies sixteen consecutive bits, the two twins are
  adjacent, and a higher bit is a higher card.
- A set of cards is a `u64` bitboard. "Holds a card of this suit" is a masked
  test; "the highest card of a suit" is a leading-zeros count; "beats" compares
  type indices. Identical types played to one trick go to the leader.
- A **code** is two characters, rank then suit (`AS`, `TD`, `7C`), exactly as
  in piquet, and names a type, not a card.

**Twins are equivalent unless their histories differ.** Two concealed copies of
a type are the same move: the list of moves offers one (`Hand::legal`), and
the engine accepts either. A concealed copy and one on the table are
different moves, because the table copy carries a history; two table copies
can differ in their histories *and in which carried score each guards*
(review finding 9: two kings with equal levels, one guarding a carried
marriage, are 20 points apart). So the list of moves offers every table card
separately, and a search that hashes positions must key table cards by type,
history and carried-score membership, never by type and history alone.

**Twins are told apart by the engine, never by a player.** The engine keeps
physical ids (a mesh per card for the 3D table, histories per card for
declaring), but real twins are indistinguishable, so physical ids must not
reach a player as information. Review finding 1 showed two channels: the
engine always played the lower concealed twin, so a copy number said where
its twin lay; and seen cards were tracked by copy, so a player who had seen a
K♥ taken and a K♥ played was told whether another remained. Both are closed
(`hand.rs`, `observation.rs`):

- What a player knows of the other's concealed cards is a *count per type*,
  reduced whenever a card of that type leaves the concealed hand.
- In a player's view, the twins of a type that player has never held are
  renumbered from what is visible now: a lone visible copy is always copy 0,
  and two are numbered by where they lie, which is public (on the opponent's
  table, by the announcements each card has been in). Once the stock is
  gone, the record of the other player's declarations keeps types only,
  because the cards may be back in hand.
- Tested on every position of random games: a lone twin the viewer never
  held is always shown as copy 0, and the view agrees with the truth type for
  type. And the stronger property (`tests/twin_symmetry.rs`): swapping the
  two copies of a type everywhere, in the deal and in every move, leaves the
  view of a player who never held that type unchanged at every position.

**More packs later.** The layout generalises to `type × packs + copy` (`u128`
for Rubicon's four packs, three words for six). Nothing is built for it now,
but the set type stays behind its own interface so that the representation can
change.

## 6. Declarations

### 6.1 One rule instead of two

Foster's two reuse rules (`RULES.md` §4, rules 2 and 3) are one rule seen
twice. Give each class an ordered set of **levels**:

| Class | Level 1 | Level 2 |
|---|---|---|
| A, marriages and sequence | marriage (common or royal) | sequence |
| B, bezique | single bezique | double bezique |
| C, fours | four of a kind | n/a |

Each physical card on the table carries a **history**: the highest level at
which it has been declared, per class (0 if never). Then:

> **A card may join a combination only if its history in that combination's
> class is strictly below the combination's level.**

That one inequality forbids an inferior combination from a superior one (a
marriage from a sequence) and a repetition of an equal one (four kings again,
a second marriage for a married queen). It allows the superior one (a sequence
from a royal marriage, a double from a single) and leaves the other classes
untouched (a Q♠ in a marriage, a bezique and four queens). Together with rule
4 (each announcement adds at least one new card from the hand), every worked
example in `RULES.md` follows without a special case. That includes Foster's
trap of the second single bezique: its four cards then sit on the table, so a
double could add no new card.

History needs five bits per card (A: 0–2, B: 0–2, C: 0–1) and matters only while
the card lies on the table, which it leaves only by being played or when the
stock runs out (and declaring ends).

### 6.2 Announcements and carried scores

After winning a trick, a player makes one **turn of declaring**, which is one
of:

- **Nothing.** The player draws, and the right lapses until the next trick won.
- **The dix.** Exchange or show a seven of trumps for 10, with no announcement
  in the same turn (Foster p. 340).
- **Announce and score.** Lay new cards on the table, announcing a set S of
  combinations, every one including at least one of the new cards and no card
  in two combinations of the same class; then score exactly one combination
  from S or from those already **carried**.

A carried combination can be scored only while every card in it is still on the
table *and* still at the level it was announced at. A card played away, or
raised into a higher combination of the same class, voids it (Cady). All carried
scores lapse when the stock is exhausted.

The full choice set is the set of legal subsets of announceable combinations,
times the choice of which one to score. With at most eight cards in
possession this is small, but not one-line small. Two observations shrink it
for people:

1. **Free riders cost nothing.** A combination that uses only cards already on
   the table plus the cards being laid anyway reveals nothing new. Announcing
   it carries a score that would otherwise be lost for good, because the same
   cards could never again supply a new card. Foster's and Cady's
   "and twenty to score" examples are all free riders.
2. **Laying extra cards early is dominated.** A combination whose new cards
   could wait for a later trick won loses nothing by waiting, and laying it
   early only shows the cards.

So the engine *checks* every legal announcement (`Holdings::check`, which
`Hand::apply` uses) but *offers* a curated list (`Holdings::options`, which
`Hand::legal` returns, for people and agents alike). Each option lays one
combination with the best consistent set of free riders, once for each
choice of which of them to score now, plus each carried combination scored
alone; phrased the period way: *"Sixty queens, and forty, and twenty, and
twenty to score."* Free riders can conflict within a class: with one bezique
already declared, laying a second Q♠ may complete either a second single or
a double, never both; the curation picks by value. Nothing yet enumerates
every legal announcement; a search that wants the full set must build it
(review finding 7). The review's fuzzing found no omitted announcement that
scores more than an offered one; whether any plays better is not measured.

### 6.3 Reference implementation

Legality is tested against a deliberately naive reference. It enumerates every
assignment of physical cards in the player's possession to each combination
type, filters by the history rule and the new-card rule, and compares the
resulting set of options with the fast generator's, on random histories. The
reference is too slow for play and fast enough for ten thousand test positions.

## 7. The hand, the game and the score

**The hand is a phase machine.** The phases, per trick:

```
stock phase:  lead → follow → (resolve: brisques scored) → winner's turn of
              declaring → winner draws → loser draws → lead …
last stock trick: … → declaring → winner takes the last card, loser the turn-up
              → carried scores lapse, table cards return to hand
endgame:      lead → follow (must follow, must win) → resolve … → last trick +10
```

**Legal actions are generated, never validated after the fact.** Every prompt
comes with its legal set, so an illegal move cannot be represented. The engine
still refuses a malformed command with a reason fit to show a player.

**Scores are an ordered event log**: `(who, points, source, trick)`, where the
source is a combination, the dix, a brisque or the last trick. Totals are
derived. The log drives the narration, the score tab and the celebrations, and
it is the per-decision record that measurements read.

**The game** is a sequence of hands, dealers alternating, ended by the first
event that takes a score to 1,000 (`RULES.md` §7). It is a double game if the
loser stands below 500 at that moment.

**Seeds.** A game is determined by `(level, seed)`. The generator is small and
modern (SplitMix64 seeding xoshiro256\*\*, or PCG32), and is checked
against its published reference outputs. It draws bounded integers without
bias (Lemire's method) and shuffles by Fisher–Yates. It behaves identically
native and in wasm. Agents draw from their own streams, so the opponent's
thinking never changes the cards.

## 8. Observation: what each player knows

This is the module most likely to be subtly wrong. Piquet's leaked four times,
each a field that told a player something no one had said aloud. Every field
of a player's **view** must answer the question *when was this shown or said,
and to whom?*

| Information | Who knows it | Why |
|---|---|---|
| Own concealed cards | owner | held |
| Every card on the table (both players), with its history | both | laid face up |
| Carried scores ("twenty to score") | both | said aloud, repeated each trick |
| The turn-up, and the stock's size | both | face up; the English laws allow counting the stock |
| A card taken by the dix exchange | both, until played | the exchange is made in view |
| The second seven of trumps, shown for 10 | both, until played | shown, then kept in hand |
| The turn-up taken by the loser of the last stock trick | both | it was face up |
| Each card played to a trick | both, as it is played | trick play is in view |
| A quitted trick | nobody may look again | Foster p. 343; memory is a skill |
| The opponent's concealed cards and the stock order | nobody | |

Two consequences:

- **Memory is a capability, not a right.** The view records every card that has
  been played, because the player saw it. A rung of the ladder that "remembers
  what has been played" uses that record; lower rungs ignore it. The human gets
  the same record through an aid (a *card tracker*, switchable).
- **The endgame is perfect information** for anyone who has kept track: once
  the stock is exhausted, the opponent's eight cards are exactly the cards not
  yet seen. That makes the last eight tricks exactly solvable (§10.3).

Inference from what was *not* declared (a player who wins a trick and declares
nothing has no declarable combination, or is hiding one) is real but weak,
because declining is a legitimate strategy. It belongs to a later rung, and it
must be measured before it is trusted.

## 9. The protocol

The table protocol follows piquet's (`piquet/docs/PROTOCOL.md`): `start(level,
seed)`, then a loop of reading the state's prompt, drawing the state, and
sending a one-line command. The engine runs the opponent between human
decisions, keeps a replayable record, and supports undo and aids. Differences:

- **Cards carry ids.** Every card the human may see is `{ "id", "card" }`, and
  commands that name a card use the id (`play 23`). The terminal client
  accepts codes and asks when two of the player's cards differ only in where
  they lie or what they have been declared in.
- **Prompts**: `cut`, `choose_dealer`, `play` (lead or follow, with the legal
  ids), `declare` (the curated options of §6.2, with the dix and "nothing"),
  `next_hand`, `over`.
- **Events**, enough to animate every intermediate state: `dealt`, `turned_up`,
  `played`, `took_trick`, `scored`, `announced` (laid on the table, with
  what is carried), `dix`, `drew` (the human's own card; the opponent's as a
  count), `last_card` (who took the last card and who the turn-up), `stock_out`
  (table cards return to hand), `hand_ends`, `game_ends`.
- **Nothing in the state names a card the human could not know.** The
  protocol test walks every string in every state of many whole games, the
  narration included, and fails on any concealed card.

The protocol document is written when the session is built (milestone 7),
from the engine that exists rather than ahead of it.

## 10. The opponent

### 10.1 A ladder of capabilities

As in piquet, skill is a **ladder of named capabilities**, each a concept a
human learns, so that the ladder is also the curriculum. It is not a noise
dial. Each rung must beat the one below in mirrored play, or it is removed;
piquet deleted two proposed rungs that way.

The ladder as built and measured (2026-10-02; `agents.rs`,
`measurements/README.md`). Margins are points a hand over the rung below, in
mirrored hands; games are the share won in mirrored games.

| Rung | Capability | Over the rung below |
|---|---|---|
| 1 | Plays legally: its lowest card, and whatever declaration scores most | – |
| 2 | The period maxims (Pole, Pardon, Foster, Hoffmann), weighed by a tuned style: values each card for keeping (rank, trumps, bezique cards above aces while the double lives, marriage partners, sequence cards, three of a rank, the dix); throws the cheapest; takes brisques cheaply; wins the lead when there is something to declare, paying more the more it would score; single bezique before the double, royal marriage before the sequence | +98.6 (92% of games) |
| 3 | Remembers every card played, and so plays the last eight tricks exactly (`solver.rs`): in bezique, memory pays in the endgame, where the cards played tell the opponent's hand | +18.7 (58%) |
| 4 | Fights for the last trick before the stock runs out: its winner makes the last declaration and leads into the endgame | +2.3 (51%) |

Two proposed rungs failed measurement and were dropped, as piquet dropped
two: memory used only to notice combinations that can no longer be made
(+1.9 over rung 2: real, but small and now part of rung 3), and watching the
opponent's table (counting its cards as gone, taking tricks to stop it
declaring: nothing measurable). Search in the stock phase (§10.4) and play
for the game rather than the hand remain proposals for rungs 5 and up.

**Erraticism** (`opponent.rs`): on each decision the opponent may slip a
rung, and another, with the set probability, drawn from its own stream, so
a sitting replays exactly. Real players are inconsistent, not uniformly bad.

**Personas** (`opponent.rs`): a style must be close to EV-neutral, or it is
a lower skill level wearing a hat. Calibrating that turned out to be the
most instructive measurement of the night:

- Shifting weights (keep sequences longer, chase brisques, hoard trumps) on
  a well-tuned default mostly fails one way or the other: it costs points,
  or it is simply better play and belongs in the default (five weights moved
  into the default that way).
- Brisques resist style altogether: a player's share of the 160 is set by
  the cards and the trick rules, and no lever moved it.
- What works is piquet's own definition: a style chooses among options of
  nearly equal worth. A `Leaning` breaks near-ties only, so it costs
  little; how near, and what it costs, are measured.

The personas, each within 1.24 points a hand of balanced (measured at rung
2, which plays the stock as the top rung does in all but about 1.5% of
decisions): **balanced**;
**concentrates** (of cards exactly equal in worth, throws from its shortest
suit: Foster's "pick one or two suits and discard the rest"); **close**
(plays from the table first, after Pardon: show the opponent as little as
possible); **bezique** (values bezique cards at twice the default, after
Pardon; the one weight shift that is both free and visible, scoring 145 in
combinations a hand against 139). An opponent is described by how it plays,
never named.

### 10.2 Decisions per trick

While the stock lasts: which card to lead (from hand or table); which card to
play second (anything); and, after winning, the turn of declaring. In the
endgame, only the card, from a set that the follow-and-win rule usually cuts
to one or two.

### 10.3 The endgame solver

Once the stock is exhausted both hands are known (§8), the follower's choices
are forced down hard by follow-and-win, and only brisques and the last trick
remain to score. An exact alpha-beta search over at most eight tricks is
small, and a table of bounds on the canonical position (twins merged) makes
it fast: about a millisecond for a full endgame. It is checked against
unmemoised minimax on random short endgames, against unpruned minimax on
full ones (value and card chosen), and against the engine's own scoring in
real games. "Exact" is for the hand's points: the solver maximises the
margin over the rest of the hand, which is not always the same as winning
the game when a player is close to 1,000 (§10.4). The review of 2026-10-02
checked 4,294 endgame decisions taken within 170 points of the game's end
against a search for the race and found none where it mattered. The search extends
naturally backwards into the last stock tricks: there, the hidden cards are
the opponent's concealed few plus a couple of stock cards, so few enough worlds
remain to enumerate exactly.

### 10.4 Search in the stock phase

Determinised Monte Carlo: sample the opponent's concealed cards and the stock
order consistently with the view, play each candidate move out with a fast
rung-5 policy, and average. Three cautions carry over from piquet, all
measured there:

- **The optimizer's curse.** Choosing the argmax of noisy rollout estimates
  overstates the gain. Piquet's naive discard search believed it was gaining
  6–8 points where the true gain was nil. Use common random numbers across
  candidates, enough samples, and treat a small apparent edge as noise until
  mirrored play confirms it.
- **Strategy fusion.** Determinisation assumes the hidden cards will be
  revealed, and it overvalues moves that rely on that. Information-set MCTS
  is the known remedy. Try the simple method first and measure the gap.
- **The objective.** Points in a hand are a first approximation. The game is
  a race to 1,000 with a double-game threshold at 500. Piquet found that
  linearising its partie objective measured *worse* than ignoring it, so the
  game value should be settled at the leaves from a measured table, not
  approximated on the way down.

### 10.5 Measurement

**Mirrored pairs, always**: each deal is played twice with the seats swapped,
and only the difference counts. Piquet's ad-hoc unmirrored harness disagreed
with itself by five sigma. Runs are sized by the smallest design that can
answer the question, with the stopping rule fixed in advance; any long or
paid compute run is agreed with the maintainer first, with a runtime and cost
estimate.

## 11. The table

The front end is piquet's `web3d`, copied and adapted. A survey of it
(2026-10-01) sorted every module, and this section records the result.

**Reused as is**: the renderer and scene, card geometry, toon materials and
the in-plane ink hull, kinematics and easing, the timeline, framing for phones,
the procedural table surfaces, the art loader, the render-on-demand director
loop, the build script's inlining (renamed), the offline browser test harness.

**Twins break the card-code-as-key assumption.** It appears in about fifteen
places. All of them move to physical ids, and three would fail outright:

- The choreography's `match()` (meshes found by code: two slots claim one
  mesh).
- The event reducer (removing a played card by code removes both twins).
- The hand's "combos" sort (a `Set` of codes drops the second twin).

Materials are shared per code, so dimming one twin dims the other; faces get
a material per mesh, with the texture still shared. Selection, hint marks,
"just drawn" lifts and the celebration runner are keyed by code too.

**New zones.**

- **The stock**: a squared pile of up to 47, with the turn-up lying crosswise
  beneath it.
- **The table cards** in front of each player: face up, still part of the hand,
  playable by click. One card can belong to several combinations, so the
  layout groups by card, not by combination, and shows each card's history
  quietly.
- **Won tricks**: face down in a pile (Foster), not shingled face up as in
  piquet.

**The deal**: 3-2-3 and the turn-up, not piquet's twos and talon.

**The score.** Piquet's broadcast-style score bug (two numbers and a ticking
update) and the collapsible running tab both carry over. Bezique's numbers run
to 1,000, so each side's progress could show as a filling ring with the 500
mark visible. A nod to the period's dial markers is an idea to try, not a
decision. The big moments that earn a celebration are double bezique, the
sequence and the winning of a game (doubled games especially); routine scores
only tick.

**Teaching.** A tutorial page at the first occurrence of each phase: the first
trick; the first turn of declaring; the dix; the last stock trick; the last
eight tricks. Explanations give the actual numbers in front of the player
("You hold Q♠ and J♦: win a trick and declare bezique for 40"). Hints come
from the top rung. Fact, interpretation and hint are separate layers, the last
two each switchable.

**Budget.** Piquet's page is 3.1 MB: art 1.6, JavaScript 1.0, engine 0.4. The
art is the same here, so a similar page is expected; 5 MB is a guideline for
modesty, not a wall.

## 12. Testing

1. **Unit tests**, test first, in every module.
2. **Period fixtures**: the worked examples in `RULES.md`, written as tests
   before the declaration code exists, in a compact text notation for
   positions (designed with the hand module, milestone 4).
3. **Brute-force references**: meld legality (§6.3) and the endgame solver
   (§10.3) against naive versions on thousands of random positions.
4. **Statistical invariants** over random play: brisques total 160 per hand;
   32 tricks; 64 cards conserved; the event log reconciles with the totals;
   and the deal and draw reproduce the exact odds of
   `analysis/bezique_stats.py` within sampling error.
5. **Protocol tests**: no state ever names a card its reader could not know;
   replay reproduces a game byte for byte, native and wasm.
6. **Front-end tests**: piquet's node tests (layout invariants, choreography
   lands exactly on layout, no face shown that is not known) and its offline
   browser test, adapted.

**A test that cannot fail is not a test**: deterministic agents in mirrored
pairs tie exactly, so a tolerance must be checked for whether it does any
work. **The slow statistical tests stay in the default run**, because in
piquet they caught more real defects than the unit tests. When the suite
feels slow, look for the one slow test.

## 13. Risks and open questions

- **The announcement interface** (§6.2) is where bezique could become
  persnickety. The curated options must make the common case one click (or
  none, with an aid), and the rare case possible. It should be play-tested
  early, in the terminal, before the 3D table is built around it.
- **Game length.** The literature review's crude self-play bot needed 4.45 hands
  on average to reach 1,000. At 32 tricks a hand, of which the human decides
  half, that is a sitting of perhaps 70 card choices. It needs play-testing
  for fun, not just correctness.
- **The race-to-1,000 interpretation** (`RULES.md`, "Interpretations")
  shapes the end of every game. It is well supported, but not stated outright
  by any source read.
- **Strategy fusion** may cap what determinised search can do (§10.4). It is
  unmeasured.
