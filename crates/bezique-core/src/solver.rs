//! The last eight tricks, solved exactly.
//!
//! Once the stock is exhausted both hands are known to a player who has kept
//! count (`docs/DESIGN.md` §8), the second player's choices are cut down hard
//! by the follow-and-win rule, and only brisques (10 each) and the last trick
//! (10) remain to score. So the rest of the hand is a small game of perfect
//! information, and alpha-beta search with a table of solved positions
//! settles it exactly (§10.3).
//!
//! Values are the side to move's points minus the other's, from here to the
//! end of the hand. Twins are interchangeable at this stage (nothing is
//! declared any more), so moves are deduplicated by type.

use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

use crate::cards::{Card, CardSet, Suit};
use crate::tricks;

/// A position in the last eight tricks.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Endgame {
    /// The hand of the player to move.
    pub mover: CardSet,
    /// The other player's hand.
    pub other: CardSet,
    /// The card led to the trick in progress, by the other player, if the
    /// mover is answering it.
    pub led: Option<Card>,
}

/// Solves endgames for one suit of trumps, remembering what it has solved.
pub struct Solver {
    trump: Suit,
    /// Searched positions, keyed by type counts so that twins share an
    /// entry, with what the search proved of each.
    table: HashMap<Key, (i32, Bound), BuildHasherDefault<Mix>>,
}

/// The mover's hand and the other's, as type counts; the led card's type,
/// or 255.
type Key = (u64, u64, u8);

/// What a stored value proves: alpha-beta search cut short proves only a
/// bound.
#[derive(Clone, Copy, Debug)]
enum Bound {
    Exact,
    /// The value is at least this.
    Lower,
    /// The value is at most this.
    Upper,
}

/// Beyond any value: a hand is worth at most 170 either way.
const INFINITY: i32 = 1_000;

impl Solver {
    pub fn new(trump: Suit) -> Solver {
        Solver {
            trump,
            table: HashMap::with_capacity_and_hasher(4096, Default::default()),
        }
    }

    /// The mover's points minus the other's, from here to the last trick,
    /// with best play on both sides.
    pub fn value(&mut self, position: Endgame) -> i32 {
        self.search(position, -INFINITY, INFINITY)
    }

    /// A best card for the mover, and its value. Of cards worth the same,
    /// the lowest.
    pub fn best(&mut self, position: Endgame) -> (Card, i32) {
        let mut best: Option<(Card, i32)> = None;
        for card in moves(position, self.trump) {
            // A window just below the best so far: a card worth as much is
            // valued exactly, a worse one only shown to be worse.
            let floor = best.map_or(-INFINITY, |(_, value)| value - 1);
            let value = self.after(position, card, floor, INFINITY);
            if best.is_none_or(|(_, b)| value > b) {
                best = Some((card, value));
            }
        }
        best.expect("a player to move has a card")
    }

    /// The value of `position` if it lies strictly between `alpha` and
    /// `beta`; otherwise a bound on the far side of the one it fails.
    fn search(&mut self, position: Endgame, mut alpha: i32, mut beta: i32) -> i32 {
        if position.mover.is_empty() && position.led.is_none() {
            return 0;
        }
        let key = (
            canonical(position.mover),
            canonical(position.other),
            position.led.map_or(255, |c| c.card_type().0),
        );
        let known = self.table.get(&key).copied();
        if let Some((value, bound)) = known {
            match bound {
                Bound::Exact => return value,
                Bound::Lower => alpha = alpha.max(value),
                Bound::Upper => beta = beta.min(value),
            }
            if alpha >= beta {
                return value;
            }
        }
        let floor = alpha;
        let mut best = -INFINITY;
        for card in ordered(position, self.trump).iter() {
            best = best.max(self.after(position, card, alpha, beta));
            alpha = alpha.max(best);
            if alpha >= beta {
                break;
            }
        }
        let bound = if best <= floor {
            Bound::Upper
        } else if best >= beta {
            Bound::Lower
        } else {
            Bound::Exact
        };
        // A stored bound and the bound just proved can meet: at least v
        // before, at most v now (the window's floor was v), is exactly v.
        let bound = match (known, bound) {
            (Some((v, Bound::Lower)), Bound::Upper) if floor == v => Bound::Exact,
            (Some((v, Bound::Upper)), Bound::Lower) if beta == v => Bound::Exact,
            _ => bound,
        };
        self.table.insert(key, (best, bound));
        best
    }

    /// The mover's value of playing `card`, searched in the window
    /// (`alpha`, `beta`).
    fn after(&mut self, position: Endgame, card: Card, alpha: i32, beta: i32) -> i32 {
        let rest = position.mover.without(card);
        match position.led {
            None => -self.search(
                Endgame {
                    mover: position.other,
                    other: rest,
                    led: Some(card),
                },
                -beta,
                -alpha,
            ),
            Some(led) => {
                let points = trick_points(led, card, rest, position.other);
                if tricks::second_wins(led, card, self.trump) {
                    let next = Endgame {
                        mover: rest,
                        other: position.other,
                        led: None,
                    };
                    points + self.search(next, alpha - points, beta - points)
                } else {
                    let next = Endgame {
                        mover: position.other,
                        other: rest,
                        led: None,
                    };
                    -points - self.search(next, -beta - points, -alpha - points)
                }
            }
        }
    }
}

/// What the trick of `led` and `second` scores: its brisques, and the last
/// trick if no cards remain.
fn trick_points(led: Card, second: Card, rest: CardSet, other: CardSet) -> i32 {
    let mut points = 10 * [led, second].iter().filter(|c| c.is_brisque()).count() as i32;
    if rest.is_empty() && other.is_empty() {
        points += 10;
    }
    points
}

/// A fast hasher for the table's keys, which are already well mixed.
#[derive(Default)]
struct Mix(u64);

impl Hasher for Mix {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.write_u64(u64::from(b));
        }
    }

    fn write_u8(&mut self, x: u8) {
        self.write_u64(u64::from(x));
    }

    fn write_u64(&mut self, x: u64) {
        self.0 = (self.0.rotate_left(26) ^ x).wrapping_mul(0x9e37_79b9_7f4a_7c15);
    }
}

/// The plain minimax this solver must agree with: no table, no pruning, every
/// card tried.
pub fn reference_value(position: Endgame, trump: Suit) -> i32 {
    if position.mover.is_empty() && position.led.is_none() {
        return 0;
    }
    let allowed = match position.led {
        Some(led) => tricks::endgame_replies(position.mover, led, trump),
        None => position.mover,
    };
    allowed
        .cards()
        .map(|card| step(position, card, trump, |p| reference_value(p, trump)))
        .max()
        .expect("a player to move has a card")
}

/// The mover plays `card`: the value to the mover, given `value` for the
/// positions that follow.
fn step(position: Endgame, card: Card, trump: Suit, mut value: impl FnMut(Endgame) -> i32) -> i32 {
    let rest = position.mover.without(card);
    match position.led {
        // Leading: the other answers, and its value is the mover's loss.
        None => -value(Endgame {
            mover: position.other,
            other: rest,
            led: Some(card),
        }),
        // Answering: the trick is decided and scored.
        Some(led) => {
            let mover_wins = tricks::second_wins(led, card, trump);
            let points = trick_points(led, card, rest, position.other);
            if mover_wins {
                points
                    + value(Endgame {
                        mover: rest,
                        other: position.other,
                        led: None,
                    })
            } else {
                -points
                    - value(Endgame {
                        mover: position.other,
                        other: rest,
                        led: None,
                    })
            }
        }
    }
}

/// The cards the mover may play, twins counted once.
fn moves(position: Endgame, trump: Suit) -> Vec<Card> {
    let allowed = match position.led {
        Some(led) => tricks::endgame_replies(position.mover, led, trump),
        None => position.mover,
    };
    let mut seen = CardSet::EMPTY;
    let mut out = Vec::new();
    for card in allowed.cards() {
        if seen.intersect(card.card_type().twins()).is_empty() {
            out.push(card);
        }
        seen = seen.with(card);
    }
    out
}

/// The moves in the order most likely to cut the search short: answering,
/// the cards that win the trick first; leading, the highest first.
/// Kept on the stack: the search asks for them at every node.
fn ordered(position: Endgame, trump: Suit) -> Moves {
    let allowed = match position.led {
        Some(led) => tricks::endgame_replies(position.mover, led, trump),
        None => position.mover,
    };
    let mut moves = Moves {
        cards: [Card(0); 16],
        len: 0,
    };
    let mut seen = CardSet::EMPTY;
    for card in allowed.cards() {
        if seen.intersect(card.card_type().twins()).is_empty() {
            moves.cards[moves.len] = card;
            moves.len += 1;
        }
        seen = seen.with(card);
    }
    let cards = &mut moves.cards[..moves.len];
    match position.led {
        Some(led) => cards.sort_by_key(|&c| {
            (
                !tricks::second_wins(led, c, trump),
                !c.is_brisque(),
                std::cmp::Reverse(c.rank()),
            )
        }),
        None => cards.sort_by_key(|&c| std::cmp::Reverse(c.rank())),
    }
    moves
}

/// At most sixteen moves: a hand holds eight cards.
struct Moves {
    cards: [Card; 16],
    len: usize,
}

impl Moves {
    fn iter(&self) -> impl Iterator<Item = Card> + '_ {
        self.cards[..self.len].iter().copied()
    }
}

/// A hand with twins made interchangeable: of each pair of twins, the lower
/// bit set if one is held, both if both are. Twins are adjacent bits, so
/// this is a few operations on the whole set.
fn canonical(hand: CardSet) -> u64 {
    const LOWER: u64 = 0x5555_5555_5555_5555;
    let (lower, upper) = (hand.0 & LOWER, (hand.0 >> 1) & LOWER);
    (lower | upper) | ((lower & upper) << 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::Rng;

    fn set(codes: &str) -> CardSet {
        CardSet::parse(codes).unwrap()
    }

    const HEARTS: Suit = Suit::HEARTS;

    #[test]
    fn one_trick_left_the_winner_takes_its_brisques_and_the_last_trick() {
        // The mover leads the ace of spades; the other must follow with the
        // seven: 10 for the ace and 10 for the last trick.
        let position = Endgame {
            mover: set("AS"),
            other: set("7S"),
            led: None,
        };
        assert_eq!(Solver::new(HEARTS).value(position), 20);
        // Answering the ace with the seven, the mover loses 20.
        let answering = Endgame {
            mover: set("7S"),
            other: CardSet::EMPTY,
            led: Some(set("AS").cards().next().unwrap()),
        };
        assert_eq!(Solver::new(HEARTS).value(answering), -20);
    }

    #[test]
    fn the_follower_is_held_to_the_rules() {
        // Hearts trumps. The other leads the ten of spades; the mover holds
        // the ace of spades and must take it with that: +20 for two brisques
        // and +10 for the last trick.
        let ten = set("TS").cards().next().unwrap();
        let position = Endgame {
            mover: set("AS"),
            other: CardSet::EMPTY,
            led: Some(ten),
        };
        assert_eq!(Solver::new(HEARTS).value(position), 30);
    }

    #[test]
    fn the_best_lead_is_found() {
        // Two tricks left, hearts trumps; the other holds the seven of hearts
        // and the king of spades. Led first, the ace of clubs is trumped (the
        // other, void in clubs, must trump): +10. Leading the trump ace first
        // draws the seven; then the ace of clubs takes the king: both aces and
        // the last trick, +30.
        let position = Endgame {
            mover: set("AH AC"),
            other: set("7H KS"),
            led: None,
        };
        let mut solver = Solver::new(HEARTS);
        let (card, value) = solver.best(position);
        assert_eq!((card.code(), value), ("AH".to_string(), 30));
    }

    /// Random endgames of up to five tricks each, solved both ways.
    #[test]
    fn agrees_with_plain_minimax() {
        let mut rng = Rng::seeded(343);
        for _ in 0..300 {
            let mut pack: Vec<Card> = CardSet::PACK.cards().collect();
            rng.shuffle(&mut pack);
            let n = 1 + rng.below(5) as usize;
            let trump = Suit(rng.below(4) as u8);
            let mover = CardSet::of(&pack[..n]);
            let answering = rng.below(2) == 0;
            let (other, led) = if answering {
                (CardSet::of(&pack[n..2 * n - 1]), Some(pack[2 * n - 1]))
            } else {
                (CardSet::of(&pack[n..2 * n]), None)
            };
            let position = Endgame { mover, other, led };
            assert_eq!(
                Solver::new(trump).value(position),
                reference_value(position, trump),
                "{position:?}, trumps {trump:?}"
            );
        }
    }

    /// Plain minimax with a table of solved positions and no pruning: slow,
    /// but quick enough for full endgames, which `reference_value` is not.
    fn memo_value(position: Endgame, trump: Suit, table: &mut HashMap<Key, i32>) -> i32 {
        if position.mover.is_empty() && position.led.is_none() {
            return 0;
        }
        let key = (
            canonical(position.mover),
            canonical(position.other),
            position.led.map_or(255, |c| c.card_type().0),
        );
        if let Some(&v) = table.get(&key) {
            return v;
        }
        let v = moves(position, trump)
            .into_iter()
            .map(|card| step(position, card, trump, |p| memo_value(p, trump, table)))
            .max()
            .unwrap();
        table.insert(key, v);
        v
    }

    /// Full endgames, eight tricks each, from the lead and in answer.
    #[test]
    fn full_endgames_agree_with_minimax_without_pruning() {
        let mut rng = Rng::seeded(81);
        for round in 0..40 {
            let mut pack: Vec<Card> = CardSet::PACK.cards().collect();
            rng.shuffle(&mut pack);
            let trump = Suit(rng.below(4) as u8);
            let position = if round % 2 == 0 {
                Endgame {
                    mover: CardSet::of(&pack[..8]),
                    other: CardSet::of(&pack[8..16]),
                    led: None,
                }
            } else {
                Endgame {
                    mover: CardSet::of(&pack[..8]),
                    other: CardSet::of(&pack[8..15]),
                    led: Some(pack[15]),
                }
            };
            let expected = memo_value(position, trump, &mut HashMap::new());
            let mut solver = Solver::new(trump);
            assert_eq!(solver.value(position), expected, "{position:?}");
            let (card, value) = solver.best(position);
            assert_eq!(value, expected);
            // The lowest of the best cards, as before the search was pruned.
            let first_best = moves(position, trump)
                .into_iter()
                .find(|&c| {
                    step(position, c, trump, |p| {
                        memo_value(p, trump, &mut HashMap::new())
                    }) == expected
                })
                .unwrap();
            assert_eq!(card, first_best);
        }
    }

    /// One solver answering many positions, as rung 5 uses it: what is
    /// remembered from one search must not spoil the next.
    #[test]
    fn a_solver_reused_across_positions_stays_exact() {
        let mut rng = Rng::seeded(82);
        let mut solver = Solver::new(Suit::CLUBS);
        let mut table = HashMap::new();
        for _ in 0..60 {
            let mut pack: Vec<Card> = CardSet::PACK.cards().collect();
            rng.shuffle(&mut pack);
            let n = 4 + rng.below(5) as usize;
            let position = Endgame {
                mover: CardSet::of(&pack[..n]),
                other: CardSet::of(&pack[n..2 * n]),
                led: None,
            };
            assert_eq!(
                solver.value(position),
                memo_value(position, Suit::CLUBS, &mut table)
            );
        }
    }

    #[test]
    fn a_full_endgame_is_quick() {
        let mut rng = Rng::seeded(8);
        let mut pack: Vec<Card> = CardSet::PACK.cards().collect();
        rng.shuffle(&mut pack);
        let position = Endgame {
            mover: CardSet::of(&pack[..8]),
            other: CardSet::of(&pack[8..16]),
            led: None,
        };
        let start = std::time::Instant::now();
        let value = Solver::new(Suit::SPADES).value(position);
        assert!(start.elapsed().as_millis() < 500, "{:?}", start.elapsed());
        assert!((-170..=170).contains(&value));
    }
}
