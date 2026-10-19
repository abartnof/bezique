//! Measurements of the game and of its opponents.
//!
//! ```text
//! cargo run --release -p bezique-core --bin measure -- anatomy [games]
//! cargo run --release -p bezique-core --bin measure -- riders [pairs]
//! cargo run --release -p bezique-core --bin measure -- ladder [pairs]
//! cargo run --release -p bezique-core --bin measure -- tune <field> <pairs> <value>...
//! cargo run --release -p bezique-core --bin measure -- personality [persona]   (MEASURE_LEVEL)
//! cargo run --release -p bezique-core --bin measure -- personas-json
//! cargo run --release -p bezique-core --bin measure -- lookahead [pairs] [horizon] [worlds]
//! cargo run --release -p bezique-core --bin measure -- lookahead-declarations [pairs]
//! cargo run --release -p bezique-core --bin measure -- lookahead-vs-five [pairs] [horizon] [worlds]
//! cargo run --release -p bezique-core --bin measure -- rollout-memory
//! cargo run --release -p bezique-core --bin measure -- hand-logs [hands]
//! cargo run --release -p bezique-core --bin measure -- worlds [worlds]
//! cargo run --release -p bezique-core --bin measure -- rung6
//! ```
//!
//! Each prints a short report, with the seeds it used; the reports kept are
//! in `measurements/`. Each report deals from its own default seeds;
//! `MEASURE_SEED=<n>` moves them all on by n, for confirming a choice on fresh
//! deals. `MEASURE_THREADS` sets the threads (every core by default); the
//! results do not depend on it.

use std::collections::BTreeMap;

use bezique_core::agents::{rung, rung_with, Maxims, Skills, Style};
use bezique_core::cards::Suit;
use bezique_core::deal::Seat;
use bezique_core::game::{Game, Side};
use bezique_core::hand::{Action, Hand, Phase, Source};
use bezique_core::lookahead::Lookahead;
use bezique_core::melds::{self, Announcement, Meld};
use bezique_core::observation::{Agent, View};
use bezique_core::opponent;
use bezique_core::tournament;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let n = |default: u64| match args.get(1) {
        None => default,
        Some(s) => parsed(s, "a number of pairs or games"),
    };
    match args.first().map(String::as_str) {
        Some("anatomy") => anatomy(n(2000)),
        Some("riders") => riders(n(2000)),
        Some("ladder") => ladder(n(1000)),
        Some("tune") => tune(&args[1..]),
        Some("skills") => skills(n(2000)),
        Some("last-stock-trick") => last_stock_trick(n(2000)),
        Some("personas") => personas(n(1000)),
        Some("personality") => personality(level_from_env(2), args.get(1).map(String::as_str)),
        Some("personas-json") => print!("{}", opponent::personas_json()),
        Some("lookahead") => lookahead(n(200), &args[2..]),
        Some("lookahead-declarations") => lookahead_declarations(n(500)),
        Some("lookahead-vs-five") => lookahead_vs_five(n(500), &args[2..]),
        Some("rollout-memory") => rollout_memory(),
        Some("hand-logs") => hand_logs(n(20_000)),
        Some("worlds") => worlds(n(32)),
        Some("rung6") => rung6(),
        _ => {
            eprintln!("measure: no such report; see the list at the top of measure.rs");
            std::process::exit(2);
        }
    }
}

fn maxims(_: u64) -> Box<dyn Agent> {
    Box::new(Maxims::default())
}

/// Rung 2 announcing only the combination it scores: what free riders are
/// worth.
struct NoRiders(Maxims);

impl Agent for NoRiders {
    fn choose(&mut self, view: &View, legal: &[Action]) -> Action {
        match self.0.choose(view, legal) {
            Action::Declare(a) if a.melds.contains(&a.score) && a.melds.len() > 1 => {
                Action::Declare(Announcement {
                    melds: vec![a.score],
                    score: a.score,
                })
            }
            Action::Declare(a) if !a.melds.contains(&a.score) && !a.melds.is_empty() => {
                Action::Declare(Announcement {
                    melds: vec![],
                    score: a.score,
                })
            }
            other => other,
        }
    }
}

/// Refuses the command line: a message on stderr and exit code 2, as a usage
/// error is everywhere in this repository.
fn usage(message: &str) -> ! {
    eprintln!("measure: {message}");
    std::process::exit(2);
}

/// `text` as a `T`, or a usage error saying what was wanted.
fn parsed<T: std::str::FromStr>(text: &str, wanted: &str) -> T {
    text.parse()
        .unwrap_or_else(|_| usage(&format!("{text:?} is not {wanted}")))
}

/// `MEASURE_LEVEL`, or `default`; a value that is not a level is refused,
/// not ignored (ignoring it would measure the wrong rung).
fn level_from_env(default: u8) -> u8 {
    let level = std::env::var("MEASURE_LEVEL").map_or(default, |s| parsed(&s, "a level in MEASURE_LEVEL"));
    if !(1..=bezique_core::agents::TOP).contains(&level) {
        usage(&format!("MEASURE_LEVEL={level} is outside 1..={}", bezique_core::agents::TOP));
    }
    level
}

fn no_riders(_: u64) -> Box<dyn Agent> {
    Box::new(NoRiders(Maxims::default()))
}

fn report_hands(name: &str, h: &tournament::Hands) {
    println!(
        "{name}: {:+.2} points a hand for the first ({} mirrored pairs, standard error {:.2}; 95% interval [{:+.2}, {:+.2}] at a single look, z = 1.96; [floor, ceiling, used] = [{}, {}, {}]; seeds {}-{})",
        h.mean(),
        h.pairs,
        h.standard_error(),
        h.mean() - 1.96 * h.standard_error(),
        h.mean() + 1.96 * h.standard_error(),
        h.pairs,
        h.pairs,
        h.pairs,
        h.first_seed,
        h.first_seed + u64::from(h.pairs) - 1
    );
}

/// `fixed`: a run of a set number of pairs, one look, so a 95% interval; a
/// sequential run prints its own interval line after this one.
fn report_games(name: &str, g: &tournament::Games, fixed: bool) {
    if fixed {
        println!(
            "{name}: game points {:+.3} a game, 95% interval [{:+.3}, {:+.3}] at a single look, z = 1.96; [floor, ceiling, used] = [{}, {}, {}] pairs",
            g.points_a_game(),
            g.points_a_game() - 1.96 * g.points_standard_error(),
            g.points_a_game() + 1.96 * g.points_standard_error(),
            g.pairs(),
            g.pairs(),
            g.pairs()
        );
    }
    println!(
        "{name}: the first won {} of {} games, {:.1}% (standard error {:.1}); game points {:+.3} a game (se {:.3}, a double counting two); pairs swept {}-{}, split {}; doubles {}-{}; {:.2} hands a game; seeds {}-{}",
        g.wins[0],
        g.wins[0] + g.wins[1],
        100.0 * g.share(),
        100.0 * g.standard_error(),
        g.points_a_game(),
        g.points_standard_error(),
        g.swept[0],
        g.swept[1],
        g.split,
        g.doubles[0],
        g.doubles[1],
        f64::from(g.hands) / f64::from(g.wins[0] + g.wins[1]),
        g.first_seed,
        g.first_seed + u64::from(g.pairs()) - 1
    );
}

/// The first seed of a report: its own default, moved on by `MEASURE_SEED`
/// if set, so that a choice made on one run can be confirmed on fresh deals.
/// Every report prints the seeds it used. A value that is not a plain
/// number is refused, not ignored: ignoring it would rerun the same deals.
fn seeds(default: u64) -> u64 {
    match std::env::var("MEASURE_SEED") {
        Err(_) => default,
        Ok(s) => {
            let shift: u64 = parsed(&s, "a plain number in MEASURE_SEED");
            default + shift
        }
    }
}

/// Rung 2 with one weight of its style changed, against the default, in
/// mirrored hands: a positive margin means the change plays better.
fn tune(args: &[String]) {
    let Some(field) = args.first() else {
        usage("tune <field> <pairs> <value>...: a field of Style is wanted")
    };
    let Some(pairs) = args.get(1) else {
        usage("tune <field> <pairs> <value>...: a number of pairs is wanted")
    };
    let pairs: u64 = parsed(pairs, "a number of pairs");
    for value in &args[2..] {
        let value: i32 = parsed(value, "a whole number");
        let mut style = Style::default();
        let slot = match field.as_str() {
            "trump" => &mut style.trump,
            "bezique" => &mut style.bezique,
            "marriage" => &mut style.marriage,
            "royal" => &mut style.royal,
            "sequence" => &mut style.sequence,
            "four" => &mut style.four,
            "dix" => &mut style.dix,
            "carried" => &mut style.carried,
            "shown" => &mut style.shown,
            "take_brisque" => &mut style.take_brisque,
            "take_to_declare" => &mut style.take_to_declare,
            "lead_brisques" => &mut style.lead_brisques,
            "urgency" => &mut style.urgency,
            other => usage(&format!("no field {other} in Style")),
        };
        *slot = value;
        let profile_style = style.clone();
        let variant = move |_: u64| -> Box<dyn Agent> {
            Box::new(Maxims {
                style: style.clone(),
                ..Maxims::default()
            })
        };
        report_hands(
            &format!("{field} = {value}"),
            &tournament::hands(seeds(4_000_000), pairs, &variant, &maxims),
        );
        let (melds, brisques) = profile(&profile_style, &Style::default(), 2, pairs.min(4000));
        println!("    scores {melds:.1} in combinations and {brisques:.1} in brisques a hand (balanced: about 140 and 80)");
    }
}

/// Each skill alone, added to rung 2, against rung 2.
fn skills(pairs: u64) {
    let only = |memory, endgame, last_stock_trick| {
        move |_: u64| -> Box<dyn Agent> {
            Box::new(Maxims {
                skills: Skills {
                    memory,
                    endgame,
                    last_stock_trick,
                },
                ..Maxims::default()
            })
        }
    };
    for (name, make) in [
        ("memory", only(true, false, false)),
        ("memory and the exact endgame", only(true, true, false)),
    ] {
        report_hands(
            &format!("rung 2 with {name}, against rung 2"),
            &tournament::hands(seeds(7_000_000), pairs, &make, &maxims),
        );
    }
}

/// Rung 3 fighting for the last stock trick, against rung 3.
fn last_stock_trick(pairs: u64) {
    let fighting = |_: u64| -> Box<dyn Agent> {
        Box::new(Maxims {
            skills: Skills {
                memory: true,
                endgame: true,
                last_stock_trick: true,
            },
            ..Maxims::default()
        })
    };
    let three = |_: u64| rung(3);
    report_hands(
        "rung 3 fighting for the last stock trick, against rung 3",
        &tournament::hands(seeds(8_000_000), pairs, &fighting, &three),
    );
}

/// Rung 4 looking ahead near the end of the stock, against rung 4:
/// `lookahead <pairs> [horizon] [worlds]`.
fn lookahead(pairs: u64, rest: &[String]) {
    let horizon = rest.first().map_or(9, |s| parsed(s, "a horizon"));
    let worlds = rest
        .get(1)
        .map_or(16, |s| parsed(s, "a number of worlds"));
    let ahead = move |_: u64| -> Box<dyn Agent> {
        Box::new(Lookahead {
            horizon,
            worlds,
            ..Lookahead::default()
        })
    };
    let four = |_: u64| rung(4);
    let start = std::time::Instant::now();
    report_hands(
        &format!("rung 4 looking ahead (horizon {horizon}, {worlds} worlds), against rung 4"),
        &tournament::hands(seeds(11_000_000), pairs, &ahead, &four),
    );
    println!(
        "    {:.2} s a hand",
        start.elapsed().as_secs_f64() / (2 * pairs) as f64
    );
}

/// Rung 5 with another horizon and number of worlds, against rung 5 as it
/// stands: `lookahead-vs-five <pairs> <horizon> <worlds>`.
fn lookahead_vs_five(pairs: u64, rest: &[String]) {
    let horizon = rest.first().map_or(9, |s| parsed(s, "a horizon"));
    let worlds = rest
        .get(1)
        .map_or(32, |s| parsed(s, "a number of worlds"));
    let variant = move |_: u64| -> Box<dyn Agent> {
        Box::new(Lookahead {
            horizon,
            worlds,
            ..Lookahead::default()
        })
    };
    let five = |_: u64| rung(5);
    report_hands(
        &format!("rung 5 with horizon {horizon} and {worlds} worlds, against rung 5"),
        &tournament::hands(seeds(13_000_000), pairs, &variant, &five),
    );
}

/// Rung 5 with rollouts that recall the cards played before the world
/// against rung 5 with rollouts that do not, in mirrored hands, sequentially
/// (DESIGN §10.5): stop when the difference is clear or clearly smaller than
/// a point a hand.
fn rollout_memory() {
    let with = |_: u64| rung(5);
    let without = |_: u64| -> Box<dyn Agent> {
        Box::new(Lookahead {
            rollout_memory: false,
            ..Lookahead::default()
        })
    };
    sequential_hands(
        "rung 5 with rollout memory, against rung 5 without",
        seeds(14_000_000),
        tournament::Sequential {
            batch: 250,
            min: 500,
            max: 8000,
            worth: 1.0,
        },
        &with,
        &without,
    );
}

/// The library of hands' score logs that `race` replays: the first hand of
/// each game between two rung-4 players, which no total has touched (a
/// library of every finished hand would leave out the hands that end games,
/// which are bigger than most: the review of 2026-10-02 found it
/// undervalued doubles by a quarter). Written to
/// `crates/bezique-core/data/hand-logs.bin`.
fn hand_logs(hands: u64) {
    let first = seeds(20_000_000);
    let mut logs: Vec<Vec<i16>> = Vec::new();
    for seed in first..first + hands {
        let mut game = Game::new(seed, Side::A);
        let mut agents = [rung(4), rung(4)];
        tournament::play(&mut game, &mut agents, |g| g.hand_number() > 1);
        let hand = game.previous_hand().unwrap_or(game.hand());
        logs.push(
            hand.log()
                .iter()
                .map(|s| {
                    let points = s.points as i16;
                    if s.seat == Seat::Dealer {
                        points
                    } else {
                        -points
                    }
                })
                .collect(),
        );
    }
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/data/hand-logs.bin");
    std::fs::write(path, bezique_core::race::pack(&logs)).expect("the library written");
    let points: f64 = logs
        .iter()
        .map(|h| h.iter().map(|&s| f64::from(s.abs())).sum::<f64>())
        .sum::<f64>()
        / logs.len() as f64;
    println!(
        "{} first hands from rung 4 against rung 4 written to {path} ({:.1} points a hand, both players; seeds {}-{})",
        logs.len(),
        points,
        first,
        first + hands - 1
    );
}

/// Rung 5 with more worlds against rung 5 (16), in mirrored hands,
/// sequentially: stop when the difference is clear, or clearly under a
/// point a hand (not worth doubling the cost of a decision for).
fn worlds(worlds: u64) {
    let more = move |_: u64| -> Box<dyn Agent> {
        Box::new(Lookahead {
            worlds: worlds as usize,
            ..Lookahead::default()
        })
    };
    let five = |_: u64| rung(5);
    sequential_hands(
        &format!("rung 5 with {worlds} worlds, against rung 5 (16)"),
        seeds(15_000_000),
        tournament::Sequential {
            batch: 250,
            min: 500,
            max: 10_000,
            worth: 1.0,
        },
        &more,
        &five,
    );
}

/// Rung 6 (rung 5 playing for the game near its end) against rung 5, in
/// mirrored games judged by game points (a double counting two),
/// sequentially: stop when the difference is clear, or clearly under 0.02
/// game points a game.
fn rung6() {
    let six = |_: u64| -> Box<dyn Agent> {
        Box::new(Lookahead {
            objective: bezique_core::lookahead::Objective::Game,
            ..Lookahead::default()
        })
    };
    let five = |_: u64| rung(5);
    let rule = tournament::Sequential {
        batch: 100,
        min: 300,
        max: 6000,
        worth: 0.02,
    };
    let (g, stop) = tournament::games_until(seeds(16_000_000), rule, &six, &five);
    report_games("rung 6 against rung 5, games", &g, false);
    println!(
        "    game points a game, {}",
        rule.describe(
            u64::from(g.pairs()),
            g.points_a_game(),
            g.points_standard_error(),
            stop
        )
    );
}

/// A sequential run in mirrored hands, reported with its rule and why it
/// stopped.
fn sequential_hands(
    name: &str,
    first_seed: u64,
    rule: tournament::Sequential,
    a: tournament::Make,
    b: tournament::Make,
) {
    let (h, stop) = tournament::hands_until(first_seed, rule, a, b);
    println!(
        "{name}: {:+.2} points a hand for the first ({} mirrored pairs, standard error {:.2}; seeds {}-{})",
        h.mean(),
        h.pairs,
        h.standard_error(),
        h.first_seed,
        h.first_seed + u64::from(h.pairs) - 1
    );
    println!(
        "    {}",
        rule.describe(u64::from(h.pairs), h.mean(), h.standard_error(), stop)
    );
}

/// Rung 5 looking ahead in the turn of declaring too, against rung 5.
fn lookahead_declarations(pairs: u64) {
    let declaring = |_: u64| -> Box<dyn Agent> {
        Box::new(Lookahead {
            declarations: true,
            ..Lookahead::default()
        })
    };
    let five = |_: u64| rung(5);
    report_hands(
        "rung 5 looking ahead in declaring too, against rung 5",
        &tournament::hands(seeds(12_000_000), pairs, &declaring, &five),
    );
}

/// Each persona at the top rung against the balanced one: what it costs in
/// mirrored hands, and how it plays (where its points come from, per hand).
fn personas(pairs: u64) {
    // Personas change only play before the stock runs out. Rung 2 plays
    // that part much as the top rung does (they differ in about 1.5% of
    // those decisions: memory, and the last stock trick) and far faster, so
    // MEASURE_LEVEL=2 calibrates; the costs at the top are checked after.
    let top = level_from_env(bezique_core::agents::TOP);
    let balanced = opponent::persona("balanced").unwrap();
    for p in opponent::personas() {
        let style = p.style.clone();
        let mine = move |_: u64| rung_with(top, &style);
        let base_style = balanced.style.clone();
        let theirs = move |_: u64| rung_with(top, &base_style);
        let h = tournament::hands(seeds(9_000_000), pairs, &mine, &theirs);
        let (melds, brisques) = profile(&p.style, &balanced.style, top, pairs.min(4000));
        println!(
            "{:<10} {:+6.2} a hand (se {:.2}); scores {:.1} in combinations and {:.1} in brisques a hand",
            p.key,
            h.mean(),
            h.standard_error(),
            melds,
            brisques
        );
    }
}

/// The points a hand that `style` scores in combinations and in brisques,
/// playing first hands against `against`, both seatings.
fn profile(style: &Style, against: &Style, level: u8, pairs: u64) -> (f64, f64) {
    let (mut melds, mut brisques, mut hands) = (0u64, 0u64, 0u64);
    for seed in 0..pairs {
        for swap in [false, true] {
            let mut agents = if swap {
                [rung_with(level, against), rung_with(level, style)]
            } else {
                [rung_with(level, style), rung_with(level, against)]
            };
            let mut game = Game::new(seeds(9_500_000) + seed, Side::A);
            tournament::play(&mut game, &mut agents, |g| g.hand_number() > 1);
            let hand = if game.hand_number() > 1 {
                game.previous_hand().unwrap()
            } else {
                game.hand()
            };
            // Side A dealt; the persona is side A unless swapped.
            let seat = if swap { Seat::Pone } else { Seat::Dealer };
            for s in hand.log().iter().filter(|s| s.seat == seat) {
                match s.source {
                    Source::Meld(_) => melds += u64::from(s.points),
                    Source::Brisques => brisques += u64::from(s.points),
                    _ => {}
                }
            }
            hands += 1;
        }
    }
    (melds as f64 / hands as f64, brisques as f64 / hands as f64)
}

/// Each rung against the one below it, in mirrored hands and games.
fn ladder(pairs: u64) {
    for level in 2..=bezique_core::agents::TOP {
        let upper = move |_: u64| rung(level);
        let lower = move |_: u64| rung(level - 1);
        let base = seeds(1_000_000 * u64::from(level));
        report_hands(
            &format!("rung {level} against rung {}, hands", level - 1),
            &tournament::hands(base, pairs, &upper, &lower),
        );
        report_games(
            &format!("rung {level} against rung {}, games", level - 1),
            &tournament::games(base + 500_000, pairs / 2, &upper, &lower),
            true,
        );
    }
}

fn riders(pairs: u64) {
    report_hands(
        "rung 2 against rung 2 without free riders, hands",
        &tournament::hands(seeds(3_000_000), pairs, &maxims, &no_riders),
    );
}

/// What a game between two players of one rung is made of: rung 2, or
/// `MEASURE_LEVEL`.
fn anatomy(games: u64) {
    let level = level_from_env(2);
    let mut hands_per_game = BTreeMap::new();
    let (mut doubles, mut mid_hand, mut first_dealer_won) = (0u64, 0u64, 0u64);
    let mut hands = 0u64;
    let mut by_seat = [0u64; 2];
    let mut by_source: BTreeMap<String, u64> = BTreeMap::new();
    let mut declared: BTreeMap<String, u64> = BTreeMap::new();
    let (mut with_riders, mut announcements, mut carried_scored) = (0u64, 0u64, 0u64);
    let (mut last_stock_margin, mut last_stock_won_hand) = (0i64, 0u64);
    for seed in 0..games {
        let mut game = Game::new(seeds(10_000_000) + seed, Side::A);
        let mut agents: [Box<dyn Agent>; 2] = [rung(level), rung(level)];
        while let Some(side) = game.to_move() {
            let view = View::of(&game, side);
            let action = agents[side.index()].choose(&view, &game.legal());
            if let Action::Declare(a) = &action {
                announcements += 1;
                if a.melds.len() > 1 {
                    with_riders += 1;
                }
                if !a.melds.contains(&a.score) {
                    carried_scored += 1;
                }
            }
            let number = game.hand_number();
            game.apply(action).unwrap();
            if game.hand_number() > number {
                let hand = game.previous_hand().unwrap();
                tally(hand, &mut by_seat, &mut by_source);
                // Combinations counted from the finished hands tallied, so
                // that they share the denominator (the full review of
                // 2026-10-02 found the rate inflated by the unfinished last
                // hands).
                for s in hand.log() {
                    if let Source::Meld(d) = s.source {
                        *declared.entry(kind(d.meld, hand.trump())).or_default() += 1;
                    }
                }
                hands += 1;
                let t = hand.tricks()[23];
                let (w, l) = (hand.score(t.winner), hand.score(t.winner.other()));
                last_stock_margin += i64::from(w) - i64::from(l);
                last_stock_won_hand += u64::from(w > l);
            }
        }
        let outcome = game.outcome().unwrap();
        *hands_per_game.entry(game.hand_number()).or_insert(0u64) += 1;
        doubles += u64::from(outcome.double);
        mid_hand += u64::from(game.hand().phase() != Phase::Over);
        first_dealer_won += u64::from(outcome.winner == Side::A);
    }
    let g = games as f64;
    let h = hands as f64;
    println!("Rung {level} against itself: {games} games, {hands} finished hands.");
    println!();
    let mean_hands: f64 = hands_per_game
        .iter()
        .map(|(&k, &v)| f64::from(k) * v as f64)
        .sum::<f64>()
        / g;
    println!("Hands a game: mean {mean_hands:.2}; distribution {hands_per_game:?}");
    println!(
        "Double games (loser under 500): {:.1}%",
        100.0 * doubles as f64 / g
    );
    println!(
        "Games ended in the middle of a hand: {:.1}%",
        100.0 * mid_hand as f64 / g
    );
    println!(
        "The first dealer won {:.1}% of games (standard error {:.1})",
        100.0 * first_dealer_won as f64 / g,
        100.0 * (0.25 / g).sqrt()
    );
    println!();
    println!(
        "Points a hand: dealer {:.1}, pone {:.1} (the pone leads first)",
        by_seat[Seat::Dealer.index()] as f64 / h,
        by_seat[Seat::Pone.index()] as f64 / h
    );
    let total: u64 = by_source.values().sum();
    println!("Where the points come from, per hand (both players):");
    for (source, points) in &by_source {
        println!(
            "  {source:<28} {:>6.1}  ({:.1}%)",
            *points as f64 / h,
            100.0 * *points as f64 / total as f64
        );
    }
    println!();
    println!("Combinations scored, per player-hand:");
    for (meld, count) in &declared {
        println!("  {meld:<28} {:.3}", *count as f64 / (2.0 * h));
    }
    println!(
        "Announcements carrying free riders: {:.1}% of {announcements}; carried scores later scored: {carried_scored}",
        100.0 * with_riders as f64 / announcements as f64
    );
    println!();
    println!(
        "The winner of the last stock trick (trick 24) out-scored the loser over the hand in {:.1}% of hands, by {:.1} points on average",
        100.0 * last_stock_won_hand as f64 / h,
        last_stock_margin as f64 / h
    );
}

/// A combination's kind, the four common marriages counted together.
fn kind(meld: Meld, trump: Suit) -> String {
    match meld {
        Meld::Marriage(suit) if suit == trump => "royal marriage".to_string(),
        Meld::Marriage(_) => "common marriage".to_string(),
        other => melds::name(other, trump),
    }
}

fn tally(hand: &Hand, by_seat: &mut [u64; 2], by_source: &mut BTreeMap<String, u64>) {
    for s in hand.log() {
        by_seat[s.seat.index()] += u64::from(s.points);
        let name = match s.source {
            Source::TurnedSeven => "the turned seven".to_string(),
            Source::Brisques => "brisques".to_string(),
            Source::Meld(d) => format!("meld: {}", kind(d.meld, hand.trump())),
            Source::Dix => "the dix".to_string(),
            Source::LastTrick => "the last trick".to_string(),
        };
        *by_source.entry(name).or_default() += u64::from(s.points);
    }
}

/// The style statistics a persona is told apart by, per pair of mirrored
/// hands: the persona's total minus balanced's over both seatings (halved,
/// a difference a hand), except the last, the mean tricks taken when a
/// combination was scored (earlier is lower), a difference of means.
const STATS: [&str; 6] = [
    "margin (points a hand)",
    "combination points a hand",
    "brisque points a hand",
    "double beziques a hand",
    "combinations scored a hand",
    "tricks taken when scoring (mean)",
];

fn personality_pair(style: &Style, level: u8, seed: u64) -> [f64; 6] {
    let balanced = Style::default();
    // [points, melds, brisques, doubles, count, trick-sum] for persona, balanced.
    let mut got = [[0f64; 6]; 2];
    for swap in [false, true] {
        let mut agents = if swap {
            [rung_with(level, &balanced), rung_with(level, style)]
        } else {
            [rung_with(level, style), rung_with(level, &balanced)]
        };
        let mut game = Game::new(seed, Side::A);
        tournament::play(&mut game, &mut agents, |g| g.hand_number() > 1);
        let hand = if game.hand_number() > 1 {
            game.previous_hand().unwrap()
        } else {
            game.hand()
        };
        let mine = if swap { Seat::Pone } else { Seat::Dealer };
        let theirs = if swap { Seat::Dealer } else { Seat::Pone };
        for (who, seat) in [(0, mine), (1, theirs)] {
            got[who][0] += f64::from(hand.score(seat));
            for s in hand.log().iter().filter(|s| s.seat == seat) {
                match s.source {
                    Source::Meld(d) => {
                        got[who][1] += f64::from(s.points);
                        got[who][4] += 1.0;
                        got[who][5] += f64::from(s.tricks);
                        if d.meld == melds::Meld::DoubleBezique {
                            got[who][3] += 1.0;
                        }
                    }
                    Source::Brisques => got[who][2] += f64::from(s.points),
                    _ => {}
                }
            }
        }
    }
    let mean_tricks = |g: &[f64; 6]| if g[4] > 0.0 { Some(g[5] / g[4]) } else { None };
    let when = match (mean_tricks(&got[0]), mean_tricks(&got[1])) {
        (Some(a), Some(b)) => a - b,
        _ => 0.0,
    };
    [
        (got[0][0] - got[1][0]) / 2.0,
        (got[0][1] - got[1][1]) / 2.0,
        (got[0][2] - got[1][2]) / 2.0,
        (got[0][3] - got[1][3]) / 2.0,
        (got[0][4] - got[1][4]) / 2.0,
        when,
    ]
}

/// Each persona against balanced at one level, sequentially: batches of
/// `BATCH` mirrored pairs, a look after each, the stopping boundary
/// O'Brien-Fleming for `LOOKS` equally spaced looks (z_k = 2.09 sqrt(LOOKS/k)),
/// none before the floor, a ceiling of `LOOKS` batches. Stops when the
/// persona's headline style statistic has crossed the boundary and the
/// strength difference's 95% interval lies within the tolerance, or when the
/// strength difference crosses the boundary (a persona that costs or gains
/// skill), or at the ceiling.
fn personality(level: u8, only: Option<&str>) {
    const BATCH: u64 = 1000;
    const LOOKS: u64 = 16;
    const FLOOR: u64 = 2000;
    let tol = opponent::TOLERANCE;
    println!(
        "personality at level {level}: persona against balanced, mirrored first hands; batch {BATCH}, floor {FLOOR}, ceiling {} pairs, O'Brien-Fleming z_k = {:.3} sqrt({LOOKS}/k) (shared with the sequential rule), tolerance ±{tol} points a hand (the same z_k interval)",
        BATCH * LOOKS,
        tournament::obf_constant(LOOKS)
    );
    println!("seeds from {}", seeds(20_000_000));
    // The headline statistic of each persona (index into STATS).
    let headline = |key: &str| match key {
        "collector" => 1,
        "hunter" => 2,
        "cautious" => 5,
        "gambler" => 3,
        _ => 4,
    };
    for p in opponent::personas()
        .into_iter()
        .filter(|p| p.key != "balanced")
    {
        if only.is_some_and(|k| k != p.key) {
            continue;
        }
        let mut rows: Vec<[f64; 6]> = Vec::new();
        let (stop, k) = loop {
            let first = seeds(20_000_000) + rows.len() as u64;
            let mut tuned = p.style.clone();
            for kv in std::env::var("MEASURE_VARIANT")
                .unwrap_or_default()
                .split(',')
            {
                match kv.split_once('=') {
                    Some(("bezique", v)) => tuned.bezique = parsed(v, "a whole number in MEASURE_VARIANT"),
                    Some(("leaning", "plain")) => {
                        tuned.leaning = bezique_core::agents::Leaning::Plain
                    }
                    Some(("sequence", v)) => tuned.sequence = parsed(v, "a whole number in MEASURE_VARIANT"),
                    Some(("marriage", v)) => tuned.marriage = parsed(v, "a whole number in MEASURE_VARIANT"),
                    Some(("four", v)) => tuned.four = parsed(v, "a whole number in MEASURE_VARIANT"),
                    Some(("take_brisque", v)) => tuned.take_brisque = parsed(v, "a whole number in MEASURE_VARIANT"),
                    Some(("lead_brisques", v)) => tuned.lead_brisques = parsed(v, "a whole number in MEASURE_VARIANT"),
                    _ => {}
                }
            }
            let style = &tuned;
            let threads = 2u64;
            let mut part: Vec<(u64, [f64; 6])> = std::thread::scope(|scope| {
                let workers: Vec<_> = (0..threads)
                    .map(|t| {
                        scope.spawn(move || {
                            (t..BATCH)
                                .step_by(threads as usize)
                                .map(|i| (i, personality_pair(style, level, first + i)))
                                .collect::<Vec<_>>()
                        })
                    })
                    .collect();
                workers
                    .into_iter()
                    .flat_map(|w| w.join().unwrap())
                    .collect()
            });
            part.sort_by_key(|&(i, _)| i);
            rows.extend(part.into_iter().map(|(_, r)| r));
            let n = rows.len() as u64;
            let k = n / BATCH;
            let z = tournament::obf_z(LOOKS, k);
            let (m, se) = mean_se(&rows, 0);
            let (hm, hse) = mean_se(&rows, headline(p.key));
            let strength_off = m + z * se < -tol || m - z * se > tol;
            let inside = (m - z * se) > -tol && (m + z * se) < tol;
            let style_clear = hse > 0.0 && hm.abs() >= z * hse;
            if n >= FLOOR && (strength_off || (inside && style_clear)) {
                let why = if strength_off {
                    "strength crossed the boundary"
                } else {
                    "style clear and strength inside the tolerance"
                };
                break (why, k);
            }
            if k >= LOOKS {
                break ("ceiling", k);
            }
        };
        let n = rows.len();
        let z = tournament::obf_z(LOOKS, k);
        println!(
            "\n{} ({}), look {k} of {LOOKS}: [floor, ceiling, used] = [{FLOOR}, {}, {n}] pairs; stopped: {stop}; boundary at last look z = {z:.2}",
            p.key,
            p.name,
            BATCH * LOOKS
        );
        for (i, name) in STATS.iter().enumerate() {
            let (m, se) = mean_se(&rows, i);
            let mark = if i == headline(p.key) {
                "  <- headline"
            } else {
                ""
            };
            println!(
                "    {name:<36} {m:+8.3}  interval at z_k [{:+.3}, {:+.3}]  z {:+.1}{mark}",
                m - z * se,
                m + z * se,
                if se > 0.0 { m / se } else { 0.0 }
            );
        }
    }
}

fn mean_se(rows: &[[f64; 6]], i: usize) -> (f64, f64) {
    let n = rows.len() as f64;
    let m = rows.iter().map(|r| r[i]).sum::<f64>() / n;
    let v = rows.iter().map(|r| (r[i] - m).powi(2)).sum::<f64>() / (n - 1.0).max(1.0);
    (m, (v / n).sqrt())
}
