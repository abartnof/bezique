//! The deal: eight cards each, three-two-three, the seventeenth turned up for
//! trumps, and the rest left as the stock (Foster, p. 339).

use crate::cards::{Card, CardSet, Suit};
use crate::rng::Rng;

/// A seat at one hand. The pone (the non-dealer) receives the first packet
/// and leads to the first trick; the seats change hands with every deal.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Seat {
    Dealer,
    Pone,
}

impl Seat {
    pub const BOTH: [Seat; 2] = [Seat::Dealer, Seat::Pone];

    pub fn other(self) -> Seat {
        match self {
            Seat::Dealer => Seat::Pone,
            Seat::Pone => Seat::Dealer,
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }
}

/// The undealt cards: face down, drawn from the top, with the turn-up beside
/// them as the last card of all.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Stock {
    /// Face down, the top card last.
    face_down: Vec<Card>,
    turn_up: Option<Card>,
}

impl Stock {
    /// A stock from its face-down cards, top first, and the turn-up if it is
    /// still there: for arranging a position part-way through a hand.
    pub fn new(top_first: &[Card], turn_up: Option<Card>) -> Stock {
        Stock {
            face_down: top_first.iter().rev().copied().collect(),
            turn_up,
        }
    }

    /// Puts `seven` face up in place of the turn-up and returns the card it
    /// replaces: the exchange of the seven of trumps.
    pub fn exchange(&mut self, seven: Card) -> Card {
        self.turn_up
            .replace(seven)
            .expect("the turn-up is there to exchange while declaring lasts")
    }

    /// The cards still to be drawn, the turn-up included.
    pub fn len(&self) -> usize {
        self.face_down.len() + usize::from(self.turn_up.is_some())
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The face-down cards alone: what a player counts when counting the
    /// stock.
    pub fn face_down(&self) -> usize {
        self.face_down.len()
    }

    /// The face-up card, while it is still on the table.
    pub fn turn_up(&self) -> Option<Card> {
        self.turn_up
    }

    /// The top face-down card; when none is left, the turn-up; then nothing.
    /// So the loser of the last trick before the stock runs out takes the
    /// turn-up, as the rules require, without anyone having to arrange it.
    pub fn draw(&mut self) -> Option<Card> {
        self.face_down.pop().or_else(|| self.turn_up.take())
    }
}

/// A hand as dealt.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Deal {
    hands: [CardSet; 2],
    trump: Suit,
    stock: Stock,
}

impl Deal {
    /// Deals from a pack in order, `order[0]` on top: three cards to the pone,
    /// three to the dealer, two each, three each, then the seventeenth
    /// turned up.
    pub fn from_order(order: &[Card; 64]) -> Deal {
        let mut hands = [CardSet::EMPTY; 2];
        let mut next = order.iter().copied();
        for packet in [3, 2, 3] {
            for seat in [Seat::Pone, Seat::Dealer] {
                for card in next.by_ref().take(packet) {
                    hands[seat.index()] = hands[seat.index()].with(card);
                }
            }
        }
        let turn_up = next.next().expect("a full pack has a seventeenth card");
        let face_down = next.rev().collect();
        Deal {
            hands,
            trump: turn_up.suit(),
            stock: Stock {
                face_down,
                turn_up: Some(turn_up),
            },
        }
    }

    /// Shuffles a fresh pack and deals it.
    pub fn shuffled(rng: &mut Rng) -> Deal {
        Deal::from_order(&shuffled_pack(rng))
    }

    pub fn hand(&self, seat: Seat) -> CardSet {
        self.hands[seat.index()]
    }

    /// The suit of the turn-up. It stays trumps for the whole hand, after the
    /// turn-up itself has been exchanged or drawn.
    pub fn trump(&self) -> Suit {
        self.trump
    }

    pub fn stock(&self) -> &Stock {
        &self.stock
    }

    /// The hands (indexed by [`Seat::index`]), trumps and the stock, to be
    /// played out.
    pub fn into_parts(self) -> ([CardSet; 2], Suit, Stock) {
        (self.hands, self.trump, self.stock)
    }
}

/// The 64 cards in a uniformly random order.
pub fn shuffled_pack(rng: &mut Rng) -> [Card; 64] {
    let mut pack: [Card; 64] = std::array::from_fn(|i| Card(i as u8));
    rng.shuffle(&mut pack);
    pack
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::CardType;

    fn in_order() -> [Card; 64] {
        std::array::from_fn(|i| Card(i as u8))
    }

    fn cards(numbers: &[u8]) -> CardSet {
        CardSet::of(&numbers.iter().map(|&n| Card(n)).collect::<Vec<_>>())
    }

    #[test]
    fn the_packets_go_three_two_three_pone_first() {
        let deal = Deal::from_order(&in_order());
        assert_eq!(deal.hand(Seat::Pone), cards(&[0, 1, 2, 6, 7, 10, 11, 12]));
        assert_eq!(deal.hand(Seat::Dealer), cards(&[3, 4, 5, 8, 9, 13, 14, 15]));
        assert_eq!(deal.stock().turn_up(), Some(Card(16)));
    }

    #[test]
    fn the_turn_up_makes_trumps() {
        let deal = Deal::from_order(&in_order());
        assert_eq!(deal.trump(), Card(16).suit());
        let mut order = in_order();
        order.swap(16, 63);
        assert_eq!(Deal::from_order(&order).trump(), Suit::CLUBS);
    }

    #[test]
    fn eight_each_one_turned_up_forty_seven_in_the_stock_none_lost() {
        let mut rng = Rng::seeded(2026);
        for _ in 0..100 {
            let deal = Deal::shuffled(&mut rng);
            let (pone, dealer) = (deal.hand(Seat::Pone), deal.hand(Seat::Dealer));
            assert_eq!(pone.len(), 8);
            assert_eq!(dealer.len(), 8);
            assert!(pone.intersect(dealer).is_empty());
            assert_eq!(deal.stock().face_down(), 47);
            assert_eq!(deal.stock().len(), 48);
            let mut rest = CardSet::EMPTY;
            let mut stock = deal.stock().clone();
            while let Some(card) = stock.draw() {
                assert!(!rest.contains(card));
                rest = rest.with(card);
            }
            assert_eq!(pone.union(dealer).union(rest), CardSet::PACK);
        }
    }

    #[test]
    fn the_stock_is_drawn_from_the_top_and_the_turn_up_comes_last() {
        let mut stock = Deal::from_order(&in_order()).stock().clone();
        assert_eq!(stock.draw(), Some(Card(17)));
        assert_eq!(stock.draw(), Some(Card(18)));
        for expected in 19..64 {
            assert_eq!(stock.draw(), Some(Card(expected)));
        }
        assert_eq!(stock.face_down(), 0);
        assert_eq!(stock.turn_up(), Some(Card(16)));
        assert_eq!(stock.len(), 1);
        assert_eq!(stock.draw(), Some(Card(16)));
        assert_eq!(stock.turn_up(), None);
        assert!(stock.is_empty());
        assert_eq!(stock.draw(), None);
    }

    #[test]
    fn a_seed_fixes_the_deal() {
        let a = Deal::shuffled(&mut Rng::seeded(8));
        let b = Deal::shuffled(&mut Rng::seeded(8));
        let c = Deal::shuffled(&mut Rng::seeded(9));
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn a_stock_arranged_part_way_through_draws_in_order() {
        let mut stock = Stock::new(&[Card(40), Card(41)], Some(Card(7)));
        assert_eq!(stock.len(), 3);
        assert_eq!(stock.face_down(), 2);
        assert_eq!(stock.draw(), Some(Card(40)));
        assert_eq!(stock.draw(), Some(Card(41)));
        assert_eq!(stock.draw(), Some(Card(7)));
        assert!(Stock::new(&[], None).is_empty());
    }

    #[test]
    fn the_seven_takes_the_turn_ups_place() {
        let mut stock = Stock::new(&[Card(40)], Some(Card(30)));
        assert_eq!(stock.exchange(Card(16)), Card(30));
        assert_eq!(stock.turn_up(), Some(Card(16)));
        assert_eq!(stock.len(), 2);
        assert_eq!(stock.draw(), Some(Card(40)));
        assert_eq!(
            stock.draw(),
            Some(Card(16)),
            "the loser of the last trick takes the seven"
        );
    }

    #[test]
    fn seats() {
        assert_eq!(Seat::Dealer.other(), Seat::Pone);
        assert_eq!(Seat::Pone.other(), Seat::Dealer);
        assert_ne!(Seat::Dealer.index(), Seat::Pone.index());
        assert!(Seat::BOTH.iter().all(|s| s.index() < 2));
    }

    #[test]
    fn every_card_type_appears_twice_in_a_shuffled_pack() {
        let pack = shuffled_pack(&mut Rng::seeded(1));
        let set = CardSet::of(&pack);
        assert_eq!(set, CardSet::PACK);
        assert!(CardType::all().all(|t| set.count_of(t) == 2));
    }
}
