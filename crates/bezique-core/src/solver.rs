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
    /// Solved positions, keyed by type counts so that twins share an entry.
    table: HashMap<(u64, u64, Option<u8>), i32>,
}

impl Solver {
    pub fn new(trump: Suit) -> Solver {
        Solver {
            trump,
            table: HashMap::new(),
        }
    }

    /// The mover's points minus the other's, from here to the last trick,
    /// with best play on both sides.
    pub fn value(&mut self, position: Endgame) -> i32 {
        if position.mover.is_empty() && position.led.is_none() {
            return 0;
        }
        let key = (
            counts(position.mover),
            counts(position.other),
            position.led.map(|c| c.card_type().0),
        );
        if let Some(&value) = self.table.get(&key) {
            return value;
        }
        let value = moves(position, self.trump)
            .into_iter()
            .map(|card| self.after(position, card))
            .max()
            .expect("a player to move has a card");
        self.table.insert(key, value);
        value
    }

    /// A best card for the mover, and its value.
    pub fn best(&mut self, position: Endgame) -> (Card, i32) {
        moves(position, self.trump)
            .into_iter()
            .map(|card| (card, self.after(position, card)))
            .max_by_key(|&(card, value)| (value, std::cmp::Reverse(card.0)))
            .expect("a player to move has a card")
    }

    /// The mover's value of playing `card`.
    fn after(&mut self, position: Endgame, card: Card) -> i32 {
        step(position, card, self.trump, |p| self.value(p))
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
            let mut points = 10 * [led, card].iter().filter(|c| c.is_brisque()).count() as i32;
            if rest.is_empty() && position.other.is_empty() {
                points += 10;
            }
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

/// A hand as counts of each type, two bits apiece.
fn counts(hand: CardSet) -> u64 {
    let mut key = 0u64;
    for card_type in crate::cards::CardType::all() {
        key |= u64::from(hand.count_of(card_type)) << (2 * card_type.0);
    }
    key
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
