//! Measurements of the game and of its opponents.
//!
//! ```text
//! cargo run --release -p bezique-core --bin measure -- anatomy [games]
//! cargo run --release -p bezique-core --bin measure -- riders [pairs]
//! cargo run --release -p bezique-core --bin measure -- ladder [pairs]
//! cargo run --release -p bezique-core --bin measure -- tune <field> <pairs> <value>...
//! ```
//!
//! Each prints a short report; the reports kept are in `measurements/`.

use std::collections::BTreeMap;

use bezique_core::agents::{Legal, Maxims, Style};
use bezique_core::cards::Suit;
use bezique_core::deal::Seat;
use bezique_core::game::{Game, Side};
use bezique_core::hand::{Action, Hand, Phase, Source};
use bezique_core::melds::{self, Announcement, Meld};
use bezique_core::observation::{Agent, View};
use bezique_core::tournament;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let n = |default: u64| args.get(1).and_then(|s| s.parse().ok()).unwrap_or(default);
    match args.first().map(String::as_str) {
        Some("anatomy") => anatomy(n(2000)),
        Some("riders") => riders(n(2000)),
        Some("ladder") => ladder(n(1000)),
        Some("tune") => tune(&args[1..]),
        _ => eprintln!("measure anatomy|riders|ladder [n] | tune <field> <pairs> <value>..."),
    }
}

fn maxims(_: u64) -> Box<dyn Agent> {
    Box::new(Maxims::default())
}

fn legal(_: u64) -> Box<dyn Agent> {
    Box::new(Legal)
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
        "{name}: {:+.2} points a hand for the first ({} mirrored pairs, standard error {:.2})",
        h.mean(),
        h.pairs,
        h.standard_error()
    );
}

fn report_games(name: &str, g: &tournament::Games) {
    println!(
        "{name}: the first won {} of {} games, {:.1}% (standard error {:.1}); pairs swept {}-{}, split {}; doubles {}-{}; {:.2} hands a game",
        g.wins[0],
        g.wins[0] + g.wins[1],
        100.0 * g.share(),
        100.0 * g.standard_error(),
        g.swept[0],
        g.swept[1],
        g.split,
        g.doubles[0],
        g.doubles[1],
        f64::from(g.hands) / f64::from(g.wins[0] + g.wins[1])
    );
}

/// The first seed of a tuning run: `MEASURE_SEED` if set, so that a value
/// chosen on one run can be confirmed on fresh deals.
fn seed_base() -> u64 {
    std::env::var("MEASURE_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(4_000_000)
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
            "urgency" => &mut style.urgency,
            other => panic!("no field {other}"),
        };
        *slot = value;
        let variant = move |_: u64| -> Box<dyn Agent> {
            Box::new(Maxims {
                style: style.clone(),
            })
        };
        report_hands(
            &format!("{field} = {value}"),
            &tournament::hands(seed_base(), pairs, &variant, &maxims),
        );
    }
}

fn ladder(pairs: u64) {
    report_hands(
        "rung 2 against rung 1, hands",
        &tournament::hands(1_000_000, pairs, &maxims, &legal),
    );
    report_games(
        "rung 2 against rung 1, games",
        &tournament::games(2_000_000, pairs, &maxims, &legal),
    );
}

fn riders(pairs: u64) {
    report_hands(
        "rung 2 against rung 2 without free riders, hands",
        &tournament::hands(3_000_000, pairs, &maxims, &no_riders),
    );
}

/// What a game between two rung-2 players is made of.
fn anatomy(games: u64) {
    let mut hands_per_game = BTreeMap::new();
    let (mut doubles, mut mid_hand, mut first_dealer_won) = (0u64, 0u64, 0u64);
    let mut hands = 0u64;
    let mut by_seat = [0u64; 2];
    let mut by_source: BTreeMap<String, u64> = BTreeMap::new();
    let mut declared: BTreeMap<String, u64> = BTreeMap::new();
    let (mut with_riders, mut announcements, mut carried_scored) = (0u64, 0u64, 0u64);
    let (mut last_stock_margin, mut last_stock_won_hand) = (0i64, 0u64);
    for seed in 0..games {
        let mut game = Game::new(10_000_000 + seed, Side::A);
        let mut agents: [Box<dyn Agent>; 2] =
            [Box::new(Maxims::default()), Box::new(Maxims::default())];
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
                *declared
                    .entry(kind(a.score.meld, game.hand().trump()))
                    .or_default() += 1;
            }
            let number = game.hand_number();
            game.apply(action).unwrap();
            if game.hand_number() > number {
                let hand = game.previous_hand().unwrap();
                tally(hand, &mut by_seat, &mut by_source);
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
    println!("Rung 2 against itself: {games} games, {hands} finished hands.");
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
