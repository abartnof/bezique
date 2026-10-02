//! The race to 1,000: what a game is worth from any score.
//!
//! A game ends the moment a player reaches 1,000, mid-hand as often as not,
//! and is doubled if the loser has not reached 500. So the value of a
//! position for the game is not its points but the chance of winning from
//! the totals, and of winning double. That is estimated here from the play
//! itself: a library of hands' score logs, the first hand of each of 20,000
//! games between two rung-4 players (`data/hand-logs.bin`, made by `measure
//! hand-logs`), replayed score by score from the given totals, hand after
//! hand, the deal alternating, until one player crosses. First hands only:
//! a library of every finished hand would leave out those that end games,
//! which run bigger than most, and undervalue doubles. The same paths through the library serve every
//! estimate (common random numbers), so two nearby positions are compared
//! on the same futures and their difference is not swamped by noise.
//!
//! The players' scores in a hand do not depend on the totals (rung 4 never
//! reads them; rung 5 uses them only to seed its sampling), which is what
//! lets whole hands be replayed on any totals. The model is of rung 4's
//! play; stronger play scores a little differently.

use crate::game::{DOUBLE_BELOW, GAME};
use crate::rng::Rng;

/// The library: each hand's scores in order, positive to the dealer,
/// negative to the pone.
static LOGS: std::sync::OnceLock<Vec<Vec<i16>>> = std::sync::OnceLock::new();

/// The packed library: per score a byte, the high bit set for the pone and
/// the rest the points in tens; a zero ends a hand.
const PACKED: &[u8] = include_bytes!("../data/hand-logs.bin");

fn logs() -> &'static [Vec<i16>] {
    LOGS.get_or_init(|| unpack(PACKED))
}

/// The hands of a packed library.
pub fn unpack(bytes: &[u8]) -> Vec<Vec<i16>> {
    let mut hands = Vec::new();
    let mut hand = Vec::new();
    for &b in bytes {
        if b == 0 {
            hands.push(std::mem::take(&mut hand));
        } else {
            let points = 10 * i16::from(b & 0x7f);
            hand.push(if b & 0x80 == 0 { points } else { -points });
        }
    }
    hands
}

/// A library packed: the inverse of [`unpack`].
pub fn pack(hands: &[Vec<i16>]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for hand in hands {
        for &s in hand {
            debug_assert!(s != 0 && s % 10 == 0 && s.abs() <= 1270);
            let tens = (s.unsigned_abs() / 10) as u8;
            bytes.push(if s > 0 { tens } else { 0x80 | tens });
        }
        bytes.push(0);
    }
    bytes
}

/// The futures every estimate is made on.
pub const PATHS: u64 = 2000;

/// What the game is worth to a player with `mine` points against `theirs`,
/// the next hand dealt by that player if `i_deal`: the mean, over the fixed
/// futures, of +1 for a game won, +2 for one won double, −1 and −2 for the
/// same lost.
pub fn value(mine: u32, theirs: u32, i_deal: bool) -> f64 {
    let [won_double, won, lost, lost_double] = outcomes(mine, theirs, i_deal);
    2.0 * won_double + won - lost - 2.0 * lost_double
}

/// The shares of the fixed futures in which the player wins double, wins,
/// loses, and loses double, from these totals.
pub fn outcomes(mine: u32, theirs: u32, i_deal: bool) -> [f64; 4] {
    let mut counts = [0u32; 4];
    let mut count = |worth: f64| {
        let i = match worth as i32 {
            2 => 0,
            1 => 1,
            -1 => 2,
            _ => 3,
        };
        counts[i] += 1;
    };
    if mine >= GAME || theirs >= GAME {
        count(outcome(mine >= GAME, if mine >= GAME { theirs } else { mine }));
        return counts.map(f64::from);
    }
    let logs = logs();
    for path in 0..PATHS {
        let mut rng = Rng::stream(0x7ace, path);
        let (mut me, mut them, mut dealing) = (mine, theirs, i_deal);
        'game: loop {
            let hand = &logs[rng.below(logs.len() as u64) as usize];
            for &s in hand {
                let to_dealer = s > 0;
                let points = u32::from(s.unsigned_abs());
                if to_dealer == dealing {
                    me += points;
                } else {
                    them += points;
                }
                if me >= GAME {
                    count(outcome(true, them));
                    break 'game;
                }
                if them >= GAME {
                    count(outcome(false, me));
                    break 'game;
                }
            }
            dealing = !dealing;
        }
    }
    counts.map(|c| f64::from(c) / PATHS as f64)
}

/// A game won (`won`) or lost, the loser on `loser`: one game, or two if
/// the loser had not reached 500.
fn outcome(won: bool, loser: u32) -> f64 {
    let games = if loser < DOUBLE_BELOW { 2.0 } else { 1.0 };
    if won {
        games
    } else {
        -games
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_library_is_whole_hands() {
        let logs = logs();
        assert!(logs.len() >= 1000, "{}", logs.len());
        for hand in logs {
            // Brisques alone are 160 and the last trick 10.
            let total: i32 = hand.iter().map(|&s| i32::from(s.abs())).sum();
            assert!(total >= 170, "{total}");
        }
        assert_eq!(unpack(&pack(logs)), logs);
    }

    #[test]
    fn the_value_of_a_game_is_as_the_rules_say_at_its_ends() {
        assert_eq!(value(1000, 400, true), 2.0);
        assert_eq!(value(1010, 700, false), 1.0);
        assert_eq!(value(600, 1000, true), -1.0);
        // From nothing each, an even game: the deal's advantage is slight.
        let even = value(0, 0, true);
        assert!(even.abs() < 0.1, "{even}");
        // Antisymmetric, the deal changing hands with the seat.
        let (a, b) = (value(700, 300, true), value(300, 700, false));
        assert!((a + b).abs() < 1e-9, "{a} {b}");
    }

    /// The model doubles about as many games as play does: rung 4 against
    /// rung 4 doubles about 12% of games, rung 5 about 13% (the anatomy
    /// reports). A library short of big hands predicted under 10%.
    #[test]
    fn games_are_doubled_about_as_often_as_in_play() {
        let [won_double, _, _, lost_double] = outcomes(0, 0, true);
        let doubled = won_double + lost_double;
        assert!((0.105..0.145).contains(&doubled), "{doubled}");
    }

    #[test]
    fn more_points_are_worth_more() {
        for &(lo, hi, theirs) in &[(0u32, 200u32, 0u32), (400, 500, 300), (850, 950, 900), (480, 520, 950)] {
            assert!(value(hi, theirs, true) > value(lo, theirs, true), "{lo} {hi} {theirs}");
        }
        // Near the end, a lead of a hundred is close to a won game.
        assert!(value(950, 400, true) > 1.0);
    }
}
