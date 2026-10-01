//! What one player may know: the view every agent decides from.
//!
//! Every field answers *when was this shown or said, and to whom?*
//! (`docs/DESIGN.md` §8). A player knows their own cards; everything on the
//! table, with what it has been declared in; the opponent's concealed cards
//! only where shown (the card taken by the dix, the second seven, the turn-up
//! taken at the last stock trick, table cards taken back up); the turn-up, the
//! size of the stock, and every card played this hand. Memory of the played
//! cards is offered; whether an agent uses it is a matter of skill.

use crate::cards::{Card, CardSet, Suit};
use crate::deal::Seat;
use crate::game::{Game, Side};
use crate::hand::{Action, Phase, Scored, Trick};
use crate::melds::Holdings;

#[derive(Clone, Debug)]
pub struct View {
    pub side: Side,
    pub seat: Seat,
    pub trump: Suit,
    /// The face-up card beside the stock, while it is there.
    pub turn_up: Option<Card>,
    /// Face-down cards left in the stock.
    pub stock: usize,
    pub phase: Phase,
    /// Whose turn it is, if anyone's.
    pub to_move: Option<Seat>,
    pub leader: Seat,
    /// The card led to the trick in progress.
    pub led: Option<Card>,
    /// This player's hand, table and announcements.
    pub mine: Holdings,
    /// The opponent's table and announcements; `concealed` holds only the
    /// concealed cards this player has seen.
    pub theirs: Holdings,
    /// How many of the opponent's concealed cards this player has not seen.
    pub their_hidden: u32,
    /// Every trick taken this hand, in order.
    pub tricks: Vec<Trick>,
    /// Every score this hand, in order: all were made in view.
    pub log: Vec<Scored>,
    /// Points this hand, (mine, theirs).
    pub hand_scores: (u32, u32),
    /// Points in the game, finished hands included, (mine, theirs).
    pub totals: (u32, u32),
    pub hand_number: u32,
}

impl View {
    /// `side`'s view of `game` as it stands.
    pub fn of(game: &Game, side: Side) -> View {
        let hand = game.hand();
        let seat = game.seat_of(side);
        let other = seat.other();
        let them = hand.holdings(other);
        let seen = them.concealed.intersect(hand.known(other));
        View {
            side,
            seat,
            trump: hand.trump(),
            turn_up: hand.stock().turn_up(),
            stock: hand.stock().face_down(),
            phase: hand.phase(),
            to_move: hand.to_move(),
            leader: hand.leader(),
            led: hand.led(),
            mine: hand.holdings(seat).clone(),
            theirs: Holdings {
                concealed: seen,
                shown: them.shown,
                announced: them.announced.clone(),
            },
            their_hidden: them.concealed.len() - seen.len(),
            tricks: hand.tricks().to_vec(),
            log: hand.log().to_vec(),
            hand_scores: (hand.score(seat), hand.score(other)),
            totals: (game.total(side), game.total(side.other())),
            hand_number: game.hand_number(),
        }
    }

    /// The cards this player cannot account for: the opponent's unseen
    /// concealed cards and the face-down stock, together.
    pub fn unseen(&self) -> CardSet {
        let turn_up = self
            .turn_up
            .map_or(CardSet::EMPTY, |c| CardSet::EMPTY.with(c));
        CardSet::PACK
            .minus(self.mine.cards())
            .minus(self.theirs.cards())
            .minus(self.played())
            .minus(turn_up)
    }

    /// The cards played this hand, the trick in progress included.
    pub fn played(&self) -> CardSet {
        let mut cards = self.led.map_or(CardSet::EMPTY, |c| CardSet::EMPTY.with(c));
        for trick in &self.tricks {
            cards = cards.with(trick.led).with(trick.second);
        }
        cards
    }

    /// Whether this player is to move.
    pub fn my_turn(&self) -> bool {
        self.to_move == Some(self.seat)
    }
}

/// Anything that chooses moves from a view: the opponent, at any level.
pub trait Agent {
    /// Picks one of `legal` (never empty) for the player whose view it is.
    fn choose(&mut self, view: &View, legal: &[Action]) -> Action;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::Rng;

    /// Every card a view names anywhere.
    fn mentioned(v: &View) -> CardSet {
        let mut cards = v.mine.cards().union(v.theirs.cards()).union(v.played());
        for h in [&v.mine, &v.theirs] {
            for a in &h.announced {
                cards = cards.union(a.declared.cards);
            }
        }
        if let Some(t) = v.turn_up {
            cards = cards.with(t);
        }
        for s in &v.log {
            if let crate::hand::Source::Meld(d) = s.source {
                cards = cards.union(d.cards);
            }
        }
        cards
    }

    /// Plays random games and checks every view of every position.
    #[test]
    fn no_view_names_a_card_its_player_could_not_know() {
        for seed in 0..25 {
            let mut game = Game::new(seed, Side::A);
            let mut rng = Rng::stream(seed, 9);
            while game.to_move().is_some() {
                for side in Side::BOTH {
                    let v = View::of(&game, side);
                    let hand = game.hand();
                    let (mine, theirs) = (game.seat_of(side), game.seat_of(side).other());
                    assert_eq!(v.seat, mine);
                    assert_eq!(v.mine, *hand.holdings(mine));
                    // Of the opponent's concealed cards, only those seen.
                    assert_eq!(
                        v.theirs.concealed,
                        hand.holdings(theirs)
                            .concealed
                            .intersect(hand.known(theirs))
                    );
                    assert_eq!(v.theirs.shown, hand.holdings(theirs).shown);
                    let hidden = hand.holdings(theirs).concealed.minus(hand.known(theirs));
                    assert_eq!(v.their_hidden, hidden.len());
                    // What is unseen is exactly their hidden cards and the stock.
                    let mut stock = CardSet::EMPTY;
                    let mut s = hand.stock().clone();
                    while s.face_down() > 0 {
                        stock = stock.with(s.draw().unwrap());
                    }
                    assert_eq!(v.unseen(), hidden.union(stock));
                    assert!(
                        mentioned(&v).intersect(hidden.union(stock)).is_empty(),
                        "seed {seed}: a view names an unseen card"
                    );
                    assert_eq!(v.stock, hand.stock().face_down());
                }
                let legal = game.legal();
                game.apply(legal[rng.below(legal.len() as u64) as usize].clone())
                    .unwrap();
            }
        }
    }

    #[test]
    fn scores_and_turns_are_from_the_players_side() {
        let game = Game::new(5, Side::B);
        let a = View::of(&game, Side::A);
        let b = View::of(&game, Side::B);
        assert_eq!(a.seat, Seat::Pone);
        assert_eq!(b.seat, Seat::Dealer);
        assert!(a.my_turn());
        assert!(!b.my_turn());
        assert_eq!(a.totals.0, game.total(Side::A));
        assert_eq!(a.totals.1, game.total(Side::B));
        assert_eq!(b.totals.0, game.total(Side::B));
        assert_eq!(a.hand_number, 1);
        assert_eq!(a.stock, 47);
        assert_eq!(a.their_hidden, 8);
        assert_eq!(a.unseen().len(), 47 + 8);
    }
}
