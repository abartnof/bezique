//! A game: hands dealt in turn until one player reaches 1,000.
//!
//! The first score to take a player to 1,000 wins, at that moment, mid-hand if
//! need be (`docs/RULES.md` §7 and "Interpretations"); it is a double game if
//! the loser has not reached 500. Before the first hand the players cut, and
//! the higher card chooses who deals (Foster, p. 339).

use crate::cards::Card;
use crate::deal::{Deal, Seat};
use crate::hand::{Action, Hand};
use crate::rng::Rng;

pub const GAME: u32 = 1000;
pub const DOUBLE_BELOW: u32 = 500;

/// One of the two players, for the whole game. Seats change with every deal;
/// sides do not.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Side {
    A,
    B,
}

impl Side {
    pub const BOTH: [Side; 2] = [Side::A, Side::B];

    pub fn other(self) -> Side {
        match self {
            Side::A => Side::B,
            Side::B => Side::A,
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }
}

/// The cut for deal: each side shows a card; equal ranks cut again.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Cut {
    /// Every pair of cards shown, in order; all but the last were equal.
    pub rounds: Vec<[Card; 2]>,
    /// The side that cut higher, and so chooses who deals.
    pub chooser: Side,
}

/// Cuts until one card ranks higher than the other.
pub fn cut_for_deal(rng: &mut Rng) -> Cut {
    let mut rounds = Vec::new();
    loop {
        let a = Card(rng.below(64) as u8);
        let b = Card((u64::from(a.0) + 1 + rng.below(63)) as u8 % 64);
        rounds.push([a, b]);
        if a.rank() != b.rank() {
            let chooser = if a.rank() > b.rank() {
                Side::A
            } else {
                Side::B
            };
            return Cut { rounds, chooser };
        }
    }
}

/// How a game ended.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Outcome {
    pub winner: Side,
    /// The loser had not reached 500.
    pub double: bool,
}

#[derive(Clone, Debug)]
pub struct Game {
    seed: u64,
    deals: Rng,
    first_dealer: Side,
    /// Points from finished hands.
    banked: [u32; 2],
    hand: Hand,
    /// How many hands have been dealt, this one included.
    hand_number: u32,
    outcome: Option<Outcome>,
}

impl Game {
    /// A game whose cards are fixed by `seed`, `first_dealer` dealing first.
    pub fn new(seed: u64, first_dealer: Side) -> Game {
        let mut deals = Rng::stream(seed, DEALS);
        let hand = Hand::new(Deal::shuffled(&mut deals));
        let mut game = Game {
            seed,
            deals,
            first_dealer,
            banked: [0, 0],
            hand,
            hand_number: 1,
            outcome: None,
        };
        game.check_for_game();
        game
    }

    /// A game part-way through, for tests: `banked` points already scored, and
    /// `hand` in play with `dealer` dealing it.
    pub fn arranged(seed: u64, banked: [u32; 2], dealer: Side, hand: Hand) -> Game {
        let mut game = Game {
            seed,
            deals: Rng::stream(seed, DEALS),
            first_dealer: dealer,
            banked,
            hand,
            hand_number: 1,
            outcome: None,
        };
        game.check_for_game();
        game
    }

    /// The seed that fixes the cards.
    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn hand(&self) -> &Hand {
        &self.hand
    }

    pub fn hand_number(&self) -> u32 {
        self.hand_number
    }

    /// The side dealing the current hand.
    pub fn dealer(&self) -> Side {
        if self.hand_number % 2 == 1 {
            self.first_dealer
        } else {
            self.first_dealer.other()
        }
    }

    /// Which side sits in `seat` this hand.
    pub fn side_in(&self, seat: Seat) -> Side {
        match seat {
            Seat::Dealer => self.dealer(),
            Seat::Pone => self.dealer().other(),
        }
    }

    /// Which seat `side` holds this hand.
    pub fn seat_of(&self, side: Side) -> Seat {
        if side == self.dealer() {
            Seat::Dealer
        } else {
            Seat::Pone
        }
    }

    /// A side's points: finished hands plus the hand in play.
    pub fn total(&self, side: Side) -> u32 {
        self.banked[side.index()] + self.hand.score(self.seat_of(side))
    }

    pub fn outcome(&self) -> Option<Outcome> {
        self.outcome
    }

    /// The side to move, or `None` once the game is over.
    pub fn to_move(&self) -> Option<Side> {
        if self.outcome.is_some() {
            return None;
        }
        self.hand.to_move().map(|seat| self.side_in(seat))
    }

    pub fn legal(&self) -> Vec<Action> {
        if self.outcome.is_some() {
            return Vec::new();
        }
        self.hand.legal()
    }

    /// Makes a move in the hand; ends the game the moment a side reaches 1,000,
    /// and deals the next hand when this one ends without that.
    pub fn apply(&mut self, action: Action) -> Result<(), String> {
        if self.outcome.is_some() {
            return Err("the game is over".to_string());
        }
        self.hand.apply(action)?;
        self.check_for_game();
        if self.outcome.is_none() && self.hand.to_move().is_none() {
            for side in Side::BOTH {
                self.banked[side.index()] = self.total(side);
            }
            self.hand_number += 1;
            self.hand = Hand::new(Deal::shuffled(&mut self.deals));
            // A turned seven can be the point that wins.
            self.check_for_game();
        }
        Ok(())
    }

    /// Ends the game if a side has reached 1,000. Every score belongs to one
    /// player, and the game is checked after each, so at most one side can
    /// have crossed.
    fn check_for_game(&mut self) {
        if let Some(&winner) = Side::BOTH.iter().find(|&&side| self.total(side) >= GAME) {
            self.outcome = Some(Outcome {
                winner,
                double: self.total(winner.other()) < DOUBLE_BELOW,
            });
        }
    }
}

/// The stream the deals are drawn from; the cut and the agents have others.
const DEALS: u64 = 0;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::{CardSet, CardType, Suit};
    use crate::deal::Stock;
    use crate::melds::Holdings;

    fn c(code: &str) -> Card {
        Card::new(CardType::parse(code).unwrap(), 0)
    }

    /// The last trick of a hand: the dealer leads the ace of spades to the
    /// pone's seven.
    fn last_trick() -> Hand {
        Hand::arranged(
            Suit::HEARTS,
            Stock::new(&[], None),
            [
                Holdings::new(CardSet::of(&[c("AS")])),
                Holdings::new(CardSet::of(&[c("7S")])),
            ],
            Seat::Dealer,
            31,
        )
    }

    fn play_it_out(game: &mut Game) {
        while game.to_move().is_some() {
            let first = game.legal()[0].clone();
            game.apply(first).unwrap();
        }
    }

    #[test]
    fn sides() {
        assert_eq!(Side::A.other(), Side::B);
        assert_ne!(Side::A.index(), Side::B.index());
    }

    #[test]
    fn the_cut_never_ends_on_equal_ranks_and_the_higher_card_chooses() {
        for seed in 0..500 {
            let cut = cut_for_deal(&mut Rng::seeded(seed));
            let [a, b] = *cut.rounds.last().unwrap();
            assert_ne!(a.rank(), b.rank());
            let higher = if a.rank() > b.rank() {
                Side::A
            } else {
                Side::B
            };
            assert_eq!(cut.chooser, higher);
            for [x, y] in &cut.rounds[..cut.rounds.len() - 1] {
                assert_eq!(x.rank(), y.rank());
            }
        }
    }

    #[test]
    fn the_first_dealer_deals_and_the_other_leads() {
        let game = Game::new(7, Side::B);
        assert_eq!(game.dealer(), Side::B);
        assert_eq!(game.side_in(Seat::Dealer), Side::B);
        assert_eq!(game.seat_of(Side::A), Seat::Pone);
        assert_eq!(game.to_move(), Some(Side::A));
        assert_eq!(game.hand_number(), 1);
    }

    #[test]
    fn a_seed_fixes_the_cards() {
        let a = Game::new(11, Side::A);
        let b = Game::new(11, Side::A);
        assert_eq!(format!("{:?}", a.hand()), format!("{:?}", b.hand()));
        let c = Game::new(12, Side::A);
        assert_ne!(format!("{:?}", a.hand()), format!("{:?}", c.hand()));
    }

    #[test]
    fn hands_follow_with_the_deal_alternating_and_the_points_banked() {
        let mut game = Game::arranged(5, [100, 200], Side::A, last_trick());
        // Side A deals, so holds the dealer's ace: 10 for it, 10 for the trick.
        game.apply(game.legal()[0].clone()).unwrap();
        game.apply(game.legal()[0].clone()).unwrap();
        assert_eq!(game.hand_number(), 2);
        assert_eq!(game.dealer(), Side::B);
        assert_eq!(
            game.total(Side::A),
            120 + game.hand().score(game.seat_of(Side::A))
        );
        assert_eq!(
            game.total(Side::B),
            200 + game.hand().score(game.seat_of(Side::B))
        );
        assert!(game.outcome().is_none());
    }

    #[test]
    fn the_first_to_one_thousand_wins_at_once() {
        let mut game = Game::arranged(5, [990, 400], Side::A, last_trick());
        game.apply(game.legal()[0].clone()).unwrap();
        game.apply(game.legal()[0].clone()).unwrap();
        assert_eq!(
            game.outcome(),
            Some(Outcome {
                winner: Side::A,
                double: true
            })
        );
        assert_eq!(game.total(Side::A), 1010);
        assert_eq!(game.to_move(), None);
        assert!(game.legal().is_empty());
        assert!(game.apply(Action::Pass).is_err());
        assert_eq!(game.hand_number(), 1, "no further hand is dealt");
    }

    #[test]
    fn a_loser_with_five_hundred_is_not_doubled() {
        let mut game = Game::arranged(5, [990, 500], Side::A, last_trick());
        play_it_out(&mut game);
        assert_eq!(
            game.outcome(),
            Some(Outcome {
                winner: Side::A,
                double: false
            })
        );
    }

    #[test]
    fn the_game_can_end_in_the_middle_of_a_hand() {
        // Side B, the pone here, needs only the ten for the first brisque.
        let mut game = Game::new(3, Side::A);
        game = Game::arranged(3, [0, 995], Side::A, game.hand().clone());
        while game.outcome().is_none() {
            let first = game.legal()[0].clone();
            game.apply(first).unwrap();
        }
        assert_eq!(game.hand_number(), 1);
        assert!(game.hand().tricks().len() < 32, "over before the hand was");
        assert_eq!(game.outcome().unwrap().winner, Side::B);
    }

    #[test]
    fn whole_games_end_with_one_side_over_the_line() {
        for seed in 0..40 {
            let mut game = Game::new(seed, Side::A);
            let mut moves = 0;
            while game.to_move().is_some() {
                let legal = game.legal();
                game.apply(legal[(seed as usize + moves) % legal.len()].clone())
                    .unwrap();
                moves += 1;
                assert!(moves < 20_000);
            }
            let Outcome { winner, double } = game.outcome().unwrap();
            assert!(game.total(winner) >= GAME);
            assert!(game.total(winner.other()) < GAME);
            assert_eq!(double, game.total(winner.other()) < DOUBLE_BELOW);
        }
    }
}
