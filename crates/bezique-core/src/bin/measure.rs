//! Measurements of the game and of its opponents.
//!
//! ```text
//! cargo run --release -p bezique-core --bin measure -- anatomy [games]
//! cargo run --release -p bezique-core --bin measure -- riders [pairs]
//! cargo run --release -p bezique-core --bin measure -- ladder [pairs]
//! cargo run --release -p bezique-core --bin measure -- tune <field> <pairs> <value>...
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
        Some(s) => s
            .parse()
            .unwrap_or_else(|_| panic!("{s:?} is not a number of pairs or games")),
    };
    match args.first().map(String::as_str) {
        Some("anatomy") => anatomy(n(2000)),
        Some("riders") => riders(n(2000)),
        Some("ladder") => ladder(n(1000)),
        Some("tune") => tune(&args[1..]),
        Some("skills") => skills(n(2000)),
        Some("last-stock-trick") => last_stock_trick(n(2000)),
        Some("personas") => personas(n(1000)),
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

fn no_riders(_: u64) -> Box<dyn Agent> {
    Box::new(NoRiders(Maxims::default()))
}

fn report_hands(name: &str, h: &tournament::Hands) {
    println!(
        "{name}: {:+.2} points a hand for the first ({} mirrored pairs, standard error {:.2}; seeds {}-{})",
        h.mean(),
        h.pairs,
        h.standard_error(),
        h.first_seed,
        h.first_seed + u64::from(h.pairs) - 1
    );
}

fn report_games(name: &str, g: &tournament::Games) {
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
            let shift: u64 = s
                .parse()
                .unwrap_or_else(|_| panic!("MEASURE_SEED={s:?} is not a plain number"));
            default + shift
        }
    }
}

/// Rung 2 with one weight of its style changed, against the default, in
/// mirrored hands: a positive margin means the change plays better.
fn tune(args: &[String]) {
    let field = args.first().expect("a field of Style");
    let pairs: u64 = args
        .get(1)
        .and_then(|s| s.parse().ok())
        .expect("a number of pairs");
    for value in &args[2..] {
        let value: i32 = value.parse().expect("a whole number");
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
            other => panic!("no field {other}"),
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
    let horizon = rest.first().map_or(9, |s| s.parse().expect("a horizon"));
    let worlds = rest
        .get(1)
        .map_or(16, |s| s.parse().expect("a number of worlds"));
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
    let horizon = rest.first().map_or(9, |s| s.parse().expect("a horizon"));
    let worlds = rest
        .get(1)
        .map_or(32, |s| s.parse().expect("a number of worlds"));
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
            z: 3.0,
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
            z: 3.0,
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
        z: 3.0,
        worth: 0.02,
    };
    let (g, stop) = tournament::games_until(seeds(16_000_000), rule, &six, &five);
    report_games("rung 6 against rung 5, games", &g);
    println!(
        "    sequential on game points: batches of {}, looks from {} pairs, stop at |z| >= {} or when the effect is within ±{} at two standard errors, cap {}; stopped: {stop:?}",
        rule.batch, rule.min, rule.z, rule.worth, rule.max
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
    report_hands(name, &h);
    println!(
        "    sequential: batches of {}, looks from {} pairs, stop at |z| >= {} or when the effect is within ±{} at two standard errors, cap {}; stopped: {stop:?}",
        rule.batch, rule.min, rule.z, rule.worth, rule.max
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
    let top = std::env::var("MEASURE_LEVEL")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(bezique_core::agents::TOP);
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
    let level: u8 = std::env::var("MEASURE_LEVEL")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(2);
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
