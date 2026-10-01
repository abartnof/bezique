//! What one player may know: the view every agent decides from.
//!
//! Every field answers *when was this shown or said, and to whom?*
//! (`docs/DESIGN.md` §8). A player knows their own cards; everything on the
//! table, with what it has been declared in; the opponent's concealed cards
//! only where shown (the card taken by the dix, the second seven, the turn-up
//! taken at the last stock trick, table cards taken back up); the turn-up, the
//! size of the stock, and every card played this hand. Memory of the played
//! cards is offered; whether an agent uses it is a matter of skill.

use crate::cards::{Card, CardSet, CardType, Suit};
use crate::deal::Seat;
use crate::game::{Game, Side};
use crate::hand::{Action, Phase, Scored, Source, Trick};
use crate::melds::{Announced, Declared, Holdings};

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
        let mut visible: Vec<Card> = Vec::new();
        for trick in hand.tricks() {
            visible.extend([trick.led, trick.second]);
        }
        visible.extend(hand.led());
        visible.extend(hand.stock().turn_up());
        // The opponent's table in an order the viewer can see: by the
        // announcements each card has been in, which are public. Bit order
        // would number two twins on the table by physical copy, and the
        // records would then tell which copy was held when (finding O1 of
        // the review of 2026-10-02).
        let mut table: Vec<(Vec<usize>, Card)> = them
            .shown
            .cards()
            .map(|card| {
                let records = them
                    .announced
                    .iter()
                    .enumerate()
                    .filter(|(_, a)| a.declared.cards.contains(card))
                    .map(|(i, _)| i)
                    .collect();
                (records, card)
            })
            .collect();
        table.sort_by(|x, y| x.0.cmp(&y.0));
        visible.extend(table.into_iter().map(|(_, card)| card));
        visible.extend(seen.cards());
        let relabel = Relabel::new(hand.handled(seat), &visible);
        // Once the stock is gone, cards declared earlier may be back in hand
        // and indistinguishable from their twins: their record keeps types.
        let endgame = hand.in_endgame();
        let record = |cards: CardSet| {
            if endgame {
                by_type(cards)
            } else {
                relabel.set(cards)
            }
        };
        View {
            side,
            seat,
            trump: hand.trump(),
            turn_up: hand.stock().turn_up().map(|c| relabel.card(c)),
            stock: hand.stock().face_down(),
            phase: hand.phase(),
            // The game's turn, not the hand's: a game won mid-hand leaves
            // the hand unfinished, with no one to move.
            to_move: game.to_move().map(|s| game.seat_of(s)),
            leader: hand.leader(),
            led: hand.led().map(|c| relabel.card(c)),
            mine: hand.holdings(seat).clone(),
            theirs: Holdings {
                concealed: relabel.set(seen),
                shown: relabel.set(them.shown),
                announced: them
                    .announced
                    .iter()
                    .map(|a| Announced {
                        declared: Declared {
                            meld: a.declared.meld,
                            cards: record(a.declared.cards),
                        },
                        scored: a.scored,
                    })
                    .collect(),
            },
            their_hidden: them.concealed.len() - seen.len(),
            tricks: hand
                .tricks()
                .iter()
                .map(|t| Trick {
                    led: relabel.card(t.led),
                    second: relabel.card(t.second),
                    ..*t
                })
                .collect(),
            log: hand
                .log()
                .iter()
                .map(|s| Scored {
                    source: match s.source {
                        Source::Meld(d) if s.seat == other => Source::Meld(Declared {
                            meld: d.meld,
                            cards: record(d.cards),
                        }),
                        other => other,
                    },
                    ..*s
                })
                .collect(),
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

/// Which physical copy of a twin appears is the engine's choice, not
/// something a player can see: real twins cannot be told apart. So in a
/// player's view the twins of a type that player has never held are
/// renumbered from what is visible now: a lone visible copy is always copy 0;
/// two visible copies are numbered in the order of where they lie (the
/// tricks in order, the turn-up, the table, the cards seen in hand), which is
/// public. On the opponent's table, cards are ordered by the announcements
/// they have been in, never by copy. Otherwise the copy numbers would tell where a twin lies, or
/// whether a card seen and later played was the one seen. Twins of which the
/// player has held one are left alone: the player can tell its own from the
/// other.
struct Relabel {
    to: [u8; 64],
}

impl Relabel {
    /// `visible`: the cards not the player's own that it can see now, in the
    /// order of where they lie.
    fn new(handled: CardSet, visible: &[Card]) -> Relabel {
        let mut to: [u8; 64] = std::array::from_fn(|i| i as u8);
        for card_type in CardType::all() {
            let twins = card_type.twins();
            if !twins.intersect(handled).is_empty() {
                continue;
            }
            let copies: Vec<Card> = twins.cards().collect();
            let mut order: Vec<Card> = Vec::new();
            for &card in visible {
                if card.card_type() == card_type && !order.contains(&card) {
                    order.push(card);
                }
            }
            if let Some(&first) = order.first() {
                let second = copies.iter().copied().find(|&c| c != first).expect("twins");
                to[first.0 as usize] = copies[0].0;
                to[second.0 as usize] = copies[1].0;
            }
        }
        Relabel { to }
    }

    fn card(&self, card: Card) -> Card {
        Card(self.to[card.0 as usize])
    }

    fn set(&self, cards: CardSet) -> CardSet {
        cards
            .cards()
            .fold(CardSet::EMPTY, |s, c| s.with(self.card(c)))
    }
}

/// The same cards by type only: each type's copies renumbered from 0. For
/// records whose cards may since have gone back into a hand, where linking
/// them to cards seen now would say which twin is which.
fn by_type(cards: CardSet) -> CardSet {
    let mut out = CardSet::EMPTY;
    for card_type in CardType::all() {
        for copy in 0..cards.count_of(card_type) {
            out = out.with(Card::new(card_type, copy as u8));
        }
    }
    out
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

    /// The cards a view shows where they are now: hands, tables, tricks, the
    /// turn-up.
    fn live(v: &View) -> CardSet {
        let turn_up = v.turn_up.map_or(CardSet::EMPTY, |c| CardSet::EMPTY.with(c));
        v.mine
            .cards()
            .union(v.theirs.cards())
            .union(v.played())
            .union(turn_up)
    }

    /// Card types with their counts, for comparing sets up to which twin is
    /// which.
    fn types(cards: CardSet) -> Vec<u32> {
        CardType::all().map(|t| cards.count_of(t)).collect()
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
                    let seen = hand
                        .holdings(theirs)
                        .concealed
                        .intersect(hand.known(theirs));
                    assert_eq!(types(v.theirs.concealed), types(seen));
                    assert_eq!(types(v.theirs.shown), types(hand.holdings(theirs).shown));
                    let hidden = hand.holdings(theirs).concealed.minus(hand.known(theirs));
                    assert_eq!(v.their_hidden, hidden.len());
                    // What is unseen is their hidden cards and the stock, type
                    // for type.
                    let mut stock = CardSet::EMPTY;
                    let mut s = hand.stock().clone();
                    while s.face_down() > 0 {
                        stock = stock.with(s.draw().unwrap());
                    }
                    assert_eq!(types(v.unseen()), types(hidden.union(stock)), "seed {seed}");
                    assert!(live(&v).intersect(v.unseen()).is_empty());
                    assert_eq!(v.stock, hand.stock().face_down());
                }
                let legal = game.legal();
                game.apply(legal[rng.below(legal.len() as u64) as usize].clone())
                    .unwrap();
            }
        }
    }

    /// Copy numbers carry no information: of a type whose twins a player
    /// has never held, a lone visible copy is always shown as copy 0.
    #[test]
    fn a_lone_twin_the_player_never_held_is_always_copy_zero() {
        let mut lone = 0;
        for seed in 100..140 {
            let mut game = Game::new(seed, Side::A);
            let mut rng = Rng::stream(seed, 9);
            while game.to_move().is_some() {
                for side in Side::BOTH {
                    let v = View::of(&game, side);
                    let handled = game.hand().handled(game.seat_of(side));
                    let visible = live(&v).minus(v.mine.cards());
                    for card_type in CardType::all() {
                        let twins = card_type.twins();
                        if twins.intersect(handled).is_empty() && visible.count_of(card_type) == 1 {
                            lone += 1;
                            let shown = visible.intersect(twins).cards().next().unwrap();
                            assert_eq!(shown.copy(), 0, "seed {seed}: {shown:?}");
                        }
                    }
                }
                let legal = game.legal();
                game.apply(legal[rng.below(legal.len() as u64) as usize].clone())
                    .unwrap();
            }
        }
        assert!(lone > 1000, "the case must arise: {lone}");
    }

    #[test]
    fn no_one_is_to_move_once_the_game_is_won_mid_hand() {
        let game = Game::new(3, Side::A);
        let mut game = Game::arranged(3, [0, 995], Side::A, game.hand().clone()).unwrap();
        while game.outcome().is_none() {
            let first = game.legal()[0].clone();
            game.apply(first).unwrap();
        }
        assert!(
            game.hand().to_move().is_some(),
            "the hand itself is unfinished"
        );
        for side in Side::BOTH {
            let v = View::of(&game, side);
            assert!(!v.my_turn());
            assert_eq!(v.to_move, None);
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
