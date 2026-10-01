//! One hand, from the deal to the last trick.
//!
//! While the stock lasts, each trick is led, answered with any card, and
//! taken; its winner may then declare (or exchange the seven of trumps)
//! before both draw, the winner first. When the stock is exhausted the cards
//! on the table go back into the hands and the last eight tricks are played
//! under the follow-and-win rule. Every point is entered in a log as it is
//! scored. `docs/RULES.md` §2–§6 gives the rules; `docs/DESIGN.md` §7 the
//! shape.

use crate::cards::{Card, CardSet, Rank, Suit};
use crate::deal::{Deal, Seat, Stock};
use crate::melds::{Announcement, Declared, Holdings};
use crate::tricks;

/// What the hand is waiting for.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    /// A card to lead.
    Lead,
    /// A card to answer the lead.
    Follow,
    /// The trick's winner to declare, exchange the seven, or pass, before the
    /// draw. Only while the stock lasts.
    Declare,
    /// All 32 tricks played.
    Over,
}

/// A move by the seat whose turn it is.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Action {
    /// Lead or answer with this card, from the hand or the table.
    Play(Card),
    /// Announce and score.
    Declare(Announcement),
    /// Exchange this seven of trumps for the turn-up, or show it if the
    /// turn-up is already a seven of trumps.
    Dix(Card),
    /// Draw without declaring.
    Pass,
}

/// Where a score came from.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Source {
    /// The dealer turned up a seven.
    TurnedSeven,
    /// Aces and tens taken in a trick.
    Brisques,
    /// A combination scored (announced now or carried).
    Meld(Declared),
    /// The seven of trumps exchanged or shown.
    Dix,
    /// The last trick of all.
    LastTrick,
}

/// One entry in the log of scores.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Scored {
    pub seat: Seat,
    pub points: u32,
    pub source: Source,
    /// How many tricks had been taken when it was scored.
    pub tricks: u8,
}

/// A trick, once taken.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Trick {
    pub leader: Seat,
    pub led: Card,
    pub second: Card,
    pub winner: Seat,
}

#[derive(Clone, Debug)]
pub struct Hand {
    trump: Suit,
    stock: Stock,
    holdings: [Holdings; 2],
    /// Who leads the trick in progress, or the next one.
    leader: Seat,
    led: Option<Card>,
    phase: Phase,
    tricks: Vec<Trick>,
    /// Tricks taken in all, those before an arranged position included.
    taken: u8,
    won: [CardSet; 2],
    /// Sevens of trumps that have scored, so that none scores twice.
    sevens_scored: CardSet,
    /// Concealed cards each seat has let the other see.
    seen: [CardSet; 2],
    log: Vec<Scored>,
}

impl Hand {
    /// A hand as dealt: the pone to lead, and 10 to the dealer for a turned
    /// seven.
    pub fn new(deal: Deal) -> Hand {
        let (hands, trump, stock) = deal.into_parts();
        let turned = stock.turn_up().expect("a fresh stock has its turn-up");
        let mut hand = Hand::arranged(trump, stock, hands.map(Holdings::new), Seat::Pone, 0);
        if is_seven_of(turned, trump) {
            hand.sevens_scored = hand.sevens_scored.with(turned);
            hand.score_to(Seat::Dealer, 10, Source::TurnedSeven);
        }
        hand
    }

    /// A position part-way through a hand, for tests and analysis: `leader` to
    /// lead, `tricks_taken` tricks already played (their cards gone).
    pub fn arranged(
        trump: Suit,
        stock: Stock,
        holdings: [Holdings; 2],
        leader: Seat,
        tricks_taken: u8,
    ) -> Hand {
        Hand {
            trump,
            stock,
            holdings,
            leader,
            led: None,
            phase: Phase::Lead,
            tricks: Vec::new(),
            taken: tricks_taken,
            won: [CardSet::EMPTY; 2],
            sevens_scored: CardSet::EMPTY,
            seen: [CardSet::EMPTY; 2],
            log: Vec::new(),
        }
    }

    pub fn trump(&self) -> Suit {
        self.trump
    }

    pub fn stock(&self) -> &Stock {
        &self.stock
    }

    pub fn holdings(&self, seat: Seat) -> &Holdings {
        &self.holdings[seat.index()]
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    /// The card led to the trick in progress, if any.
    pub fn led(&self) -> Option<Card> {
        self.led
    }

    /// Who leads the trick in progress (or the next): after a trick, its
    /// winner.
    pub fn leader(&self) -> Seat {
        self.leader
    }

    /// Whose turn it is, or `None` when the hand is over.
    pub fn to_move(&self) -> Option<Seat> {
        match self.phase {
            Phase::Lead | Phase::Declare => Some(self.leader),
            Phase::Follow => Some(self.leader.other()),
            Phase::Over => None,
        }
    }

    /// True once the stock is exhausted: the last eight tricks.
    pub fn in_endgame(&self) -> bool {
        self.stock.is_empty()
    }

    pub fn tricks(&self) -> &[Trick] {
        &self.tricks
    }

    /// The cards a seat has taken in tricks.
    pub fn won(&self, seat: Seat) -> CardSet {
        self.won[seat.index()]
    }

    pub fn log(&self) -> &[Scored] {
        &self.log
    }

    /// The cards in `seat`'s hand that the other player has seen: those on
    /// the table, and concealed ones shown or taken in view (by the dix, the
    /// second seven shown, the turn-up taken at the last stock trick, and
    /// table cards taken back into the hand).
    pub fn known(&self, seat: Seat) -> CardSet {
        let holdings = &self.holdings[seat.index()];
        self.seen[seat.index()]
            .union(holdings.shown)
            .intersect(holdings.cards())
    }

    /// A seat's points this hand.
    pub fn score(&self, seat: Seat) -> u32 {
        self.log
            .iter()
            .filter(|s| s.seat == seat)
            .map(|s| s.points)
            .sum()
    }

    /// Every move open to the seat whose turn it is. Concealed twins are one
    /// move; cards on the table are offered one by one, since their histories
    /// may differ.
    pub fn legal(&self) -> Vec<Action> {
        let Some(seat) = self.to_move() else {
            return Vec::new();
        };
        let holdings = &self.holdings[seat.index()];
        match self.phase {
            Phase::Lead => distinct(holdings, holdings.cards())
                .map(Action::Play)
                .collect(),
            Phase::Follow => distinct(holdings, self.playable(seat))
                .map(Action::Play)
                .collect(),
            Phase::Declare => {
                let mut moves = vec![Action::Pass];
                moves.extend(self.dix_cards(seat).map(Action::Dix));
                moves.extend(
                    holdings
                        .options(self.trump)
                        .into_iter()
                        .map(Action::Declare),
                );
                moves
            }
            Phase::Over => Vec::new(),
        }
    }

    /// Makes a move for the seat whose turn it is. A move the rules forbid is
    /// refused with a reason fit to show a player, and changes nothing.
    pub fn apply(&mut self, action: Action) -> Result<(), String> {
        let seat = self.to_move().ok_or("the hand is over")?;
        if let Action::Play(card) | Action::Dix(card) = action {
            if !card.is_valid() {
                return Err(format!("there is no card numbered {}", card.0));
            }
        }
        match (self.phase, action) {
            (Phase::Lead, Action::Play(card)) => {
                if !self.holdings[seat.index()].cards().contains(card) {
                    return Err(format!("you do not hold {}", card.code()));
                }
                self.holdings[seat.index()].play(card);
                self.led = Some(card);
                self.phase = Phase::Follow;
            }
            (Phase::Follow, Action::Play(card)) => {
                if !self.holdings[seat.index()].cards().contains(card) {
                    return Err(format!("you do not hold {}", card.code()));
                }
                if !self.playable(seat).contains(card) {
                    return Err(self.why_not(seat, card));
                }
                self.holdings[seat.index()].play(card);
                self.take_trick(card);
            }
            (Phase::Declare, Action::Pass) => self.draw(),
            (Phase::Declare, Action::Dix(seven)) => {
                // Either twin will do; only the list of moves is deduplicated.
                let unscored = self.holdings[seat.index()]
                    .concealed
                    .minus(self.sevens_scored);
                if !(unscored.contains(seven) && is_seven_of(seven, self.trump)) {
                    return Err(
                        "only a seven of trumps in your hand, not yet scored, may be exchanged"
                            .to_string(),
                    );
                }
                self.sevens_scored = self.sevens_scored.with(seven);
                let turned = self
                    .stock
                    .turn_up()
                    .expect("the turn-up is there while declaring lasts");
                let in_view = if is_seven_of(turned, self.trump) {
                    seven
                } else {
                    let taken = self.stock.exchange(seven);
                    let holdings = &mut self.holdings[seat.index()];
                    holdings.play(seven);
                    holdings.receive(taken);
                    taken
                };
                self.seen[seat.index()] = self.seen[seat.index()].with(in_view);
                self.score_to(seat, 10, Source::Dix);
                self.draw();
            }
            (Phase::Declare, Action::Declare(announcement)) => {
                let points = self.holdings[seat.index()].apply(self.trump, &announcement)?;
                self.score_to(seat, points, Source::Meld(announcement.score));
                self.draw();
            }
            (Phase::Declare, Action::Play(_)) => {
                return Err("declare, exchange the seven or pass before the next trick".to_string());
            }
            (_, Action::Pass | Action::Dix(_) | Action::Declare(_)) => {
                return Err(
                    "you may declare only after winning a trick, while the stock lasts".to_string(),
                );
            }
            (Phase::Over, _) => unreachable!("no one is to move"),
        }
        Ok(())
    }

    /// The cards `seat` may play now: anything while leading or while the
    /// stock lasts; in the last eight tricks, what the follow-and-win rule
    /// allows.
    fn playable(&self, seat: Seat) -> CardSet {
        let cards = self.holdings[seat.index()].cards();
        match (self.phase, self.led) {
            (Phase::Follow, Some(led)) if self.in_endgame() => {
                tricks::endgame_replies(cards, led, self.trump)
            }
            _ => cards,
        }
    }

    /// Why `card` breaks the follow-and-win rule.
    fn why_not(&self, seat: Seat, card: Card) -> String {
        let led = self.led.expect("answering a lead");
        let cards = self.holdings[seat.index()].cards();
        let suit = led.suit().name();
        if !cards.in_suit(led.suit()).is_empty() {
            if card.suit() == led.suit() {
                format!(
                    "you must beat the {} with a higher {suit} card while you can",
                    led.code()
                )
            } else {
                format!("you must follow {suit} while you can")
            }
        } else {
            format!("with no {suit}, you must trump while you can")
        }
    }

    /// The sevens of trumps `seat` may score now.
    fn dix_cards(&self, seat: Seat) -> impl Iterator<Item = Card> + '_ {
        let holdings = &self.holdings[seat.index()];
        distinct(holdings, holdings.concealed.minus(self.sevens_scored))
            .filter(move |&c| is_seven_of(c, self.trump))
    }

    fn take_trick(&mut self, second: Card) {
        let led = self.led.take().expect("a trick in progress");
        let winner = tricks::winner(self.leader, led, second, self.trump);
        self.won[winner.index()] = self.won[winner.index()].with(led).with(second);
        self.tricks.push(Trick {
            leader: self.leader,
            led,
            second,
            winner,
        });
        self.taken += 1;
        let brisques = [led, second].iter().filter(|c| c.is_brisque()).count() as u32;
        if brisques > 0 {
            self.score_to(winner, 10 * brisques, Source::Brisques);
        }
        self.leader = winner;
        self.phase = if self.taken == 32 {
            self.score_to(winner, 10, Source::LastTrick);
            Phase::Over
        } else if self.in_endgame() {
            Phase::Lead
        } else {
            Phase::Declare
        };
    }

    /// The winner draws, then the loser; when that empties the stock, the
    /// cards on the table go back into the hands for the last eight tricks.
    fn draw(&mut self) {
        for seat in [self.leader, self.leader.other()] {
            let face_up = self.stock.face_down() == 0;
            let card = self
                .stock
                .draw()
                .expect("the stock holds an even number of cards");
            self.holdings[seat.index()].receive(card);
            if face_up {
                self.seen[seat.index()] = self.seen[seat.index()].with(card);
            }
        }
        if self.stock.is_empty() {
            for seat in Seat::BOTH {
                let holdings = &mut self.holdings[seat.index()];
                self.seen[seat.index()] = self.seen[seat.index()].union(holdings.shown);
                holdings.take_up();
            }
        }
        self.phase = Phase::Lead;
    }

    fn score_to(&mut self, seat: Seat, points: u32, source: Source) {
        self.log.push(Scored {
            seat,
            points,
            source,
            tricks: self.taken,
        });
    }
}

fn is_seven_of(card: Card, trump: Suit) -> bool {
    card.rank() == Rank::SEVEN && card.suit() == trump
}

/// The cards of `allowed`, with concealed twins counted once (the lower copy):
/// they are the same move. Cards on the table are all listed, since their
/// histories may differ.
fn distinct(holdings: &Holdings, allowed: CardSet) -> impl Iterator<Item = Card> {
    let concealed = allowed.intersect(holdings.concealed);
    let shown = allowed.intersect(holdings.shown);
    let mut seen = CardSet::EMPTY;
    let mut once = CardSet::EMPTY;
    for card in concealed.cards() {
        if seen.intersect(card.card_type().twins()).is_empty() {
            once = once.with(card);
        }
        seen = seen.with(card);
    }
    once.union(shown).cards()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::CardType;
    use crate::melds::Meld;
    use crate::rng::Rng;

    /// A pack stacked so that the deal gives the pone and the dealer these
    /// cards, turns up `turn_up`, and puts `stock` (top first) on top of the
    /// stock; everything else follows in order. Each code takes the next free
    /// copy of its card.
    fn stacked(pone: &str, dealer: &str, turn_up: &str, stock: &str) -> [Card; 64] {
        let mut used = CardSet::EMPTY;
        let mut take = |code: &str| {
            let card_type = CardType::parse(code).unwrap();
            let card = card_type
                .twins()
                .minus(used)
                .lowest()
                .expect("only two of each");
            used = used.with(card);
            card
        };
        let pone: Vec<Card> = pone.split_whitespace().map(&mut take).collect();
        let dealer: Vec<Card> = dealer.split_whitespace().map(&mut take).collect();
        let turn_up = take(turn_up);
        let top: Vec<Card> = stock.split_whitespace().map(&mut take).collect();
        let mut order = [Card(0); 64];
        for (i, at) in [0, 1, 2, 6, 7, 10, 11, 12].into_iter().enumerate() {
            order[at] = pone[i];
        }
        for (i, at) in [3, 4, 5, 8, 9, 13, 14, 15].into_iter().enumerate() {
            order[at] = dealer[i];
        }
        order[16] = turn_up;
        let rest: Vec<Card> = top
            .into_iter()
            .chain(CardSet::PACK.minus(used).cards())
            .collect();
        order[17..].copy_from_slice(&rest);
        order
    }

    fn c(code: &str, copy: u8) -> Card {
        Card::new(CardType::parse(code).unwrap(), copy)
    }

    fn set(codes: &str) -> CardSet {
        CardSet::parse(codes).unwrap()
    }

    const PONE: &str = "AS 7C 8C 9C 7D 8D 9D JC";
    const DEALER: &str = "TS 7S 8S 9S JS 8H 9H KC";

    fn hand(pone: &str, dealer: &str, turn_up: &str, stock: &str) -> Hand {
        Hand::new(Deal::from_order(&stacked(pone, dealer, turn_up, stock)))
    }

    fn play(h: &mut Hand, code: &str) {
        let seat = h.to_move().unwrap();
        let card = h
            .holdings(seat)
            .cards()
            .cards()
            .find(|card| card.card_type() == CardType::parse(code).unwrap())
            .unwrap_or_else(|| panic!("{seat:?} does not hold {code}"));
        h.apply(Action::Play(card)).unwrap();
    }

    #[test]
    fn the_pone_leads_first() {
        let h = Hand::new(Deal::shuffled(&mut Rng::seeded(4)));
        assert_eq!(h.to_move(), Some(Seat::Pone));
        assert_eq!(h.phase(), Phase::Lead);
        assert!(!h.in_endgame());
        assert_eq!(
            h.legal().len(),
            h.holdings(Seat::Pone).concealed.len() as usize
                - duplicates(h.holdings(Seat::Pone).concealed)
        );
    }

    fn duplicates(cards: CardSet) -> usize {
        CardType::all().filter(|&t| cards.count_of(t) == 2).count()
    }

    #[test]
    fn a_turned_seven_scores_the_dealer_ten() {
        let h = hand(PONE, DEALER, "7H", "");
        assert_eq!(h.score(Seat::Dealer), 10);
        assert_eq!(h.log()[0].source, Source::TurnedSeven);
        let h = hand(PONE, DEALER, "KH", "");
        assert_eq!(h.score(Seat::Dealer), 0);
        assert!(h.log().is_empty());
    }

    #[test]
    fn brisques_score_as_the_trick_is_taken() {
        let mut h = hand(PONE, DEALER, "KH", "");
        play(&mut h, "AS");
        assert_eq!(h.phase(), Phase::Follow);
        assert_eq!(h.to_move(), Some(Seat::Dealer));
        play(&mut h, "TS");
        assert_eq!(h.score(Seat::Pone), 20, "the ace and the ten");
        assert_eq!(h.tricks().len(), 1);
        assert_eq!(h.tricks()[0].winner, Seat::Pone);
        assert_eq!(h.won(Seat::Pone).len(), 2);
        assert_eq!(h.phase(), Phase::Declare);
        assert_eq!(h.to_move(), Some(Seat::Pone));
    }

    #[test]
    fn the_winner_passes_and_both_draw_the_winner_first() {
        let mut h = hand(PONE, DEALER, "KH", "QC QD");
        play(&mut h, "AS");
        play(&mut h, "8H");
        assert_eq!(
            h.tricks()[0].winner,
            Seat::Dealer,
            "a trump takes a plain lead"
        );
        assert_eq!(h.score(Seat::Dealer), 10);
        assert!(h.legal().contains(&Action::Pass));
        h.apply(Action::Pass).unwrap();
        assert!(h.holdings(Seat::Dealer).concealed.contains(c("QC", 0)));
        assert!(h.holdings(Seat::Pone).concealed.contains(c("QD", 0)));
        assert_eq!(h.holdings(Seat::Dealer).cards().len(), 8);
        assert_eq!(h.holdings(Seat::Pone).cards().len(), 8);
        assert_eq!(h.stock().len(), 46);
        assert_eq!((h.phase(), h.to_move()), (Phase::Lead, Some(Seat::Dealer)));
    }

    #[test]
    fn declaring_scores_and_lays_the_cards_on_the_table() {
        let mut h = hand(PONE, DEALER, "KH", "QC");
        play(&mut h, "AS");
        play(&mut h, "8H");
        h.apply(Action::Pass).unwrap();
        // The dealer drew the queen of clubs, to marry the king.
        play(&mut h, "9S");
        play(&mut h, "7D");
        assert_eq!(h.to_move(), Some(Seat::Dealer));
        let marriage = h
            .legal()
            .into_iter()
            .find(
                |a| matches!(a, Action::Declare(x) if x.score.meld == Meld::Marriage(Suit::CLUBS)),
            )
            .expect("the marriage in clubs is offered");
        h.apply(marriage).unwrap();
        assert_eq!(h.score(Seat::Dealer), 30);
        assert_eq!(h.holdings(Seat::Dealer).shown, set("KC QC"));
        assert!(
            matches!(h.log().last().unwrap().source, Source::Meld(d) if d.meld == Meld::Marriage(Suit::CLUBS))
        );
        // The king may be led from the table.
        assert!(h.legal().contains(&Action::Play(c("KC", 0))));
        play(&mut h, "KC");
        assert_eq!(h.holdings(Seat::Dealer).shown, set("QC"));
    }

    #[test]
    fn the_dix_exchanges_the_seven_for_the_turn_up() {
        let mut h = hand(PONE, "TS 7S 8S 9S JS 8H 7H KC", "KH", "");
        play(&mut h, "AS");
        play(&mut h, "8H");
        assert!(h.legal().contains(&Action::Dix(c("7H", 0))));
        h.apply(Action::Dix(c("7H", 0))).unwrap();
        assert_eq!(h.score(Seat::Dealer), 20, "the ace taken, and the seven");
        assert_eq!(h.log().last().unwrap().source, Source::Dix);
        assert_eq!(h.stock().turn_up(), Some(c("7H", 0)));
        assert!(h.holdings(Seat::Dealer).concealed.contains(c("KH", 0)));
        assert!(!h.holdings(Seat::Dealer).cards().contains(c("7H", 0)));
        assert_eq!(
            h.holdings(Seat::Dealer).cards().len(),
            8,
            "and the draw followed"
        );
    }

    #[test]
    fn each_seven_of_trumps_scores_once() {
        let mut h = hand("7H 7H AS 8C 9C 7D 8D 9D", DEALER, "KH", "");
        play(&mut h, "AS");
        play(&mut h, "7S");
        h.apply(Action::Dix(c("7H", 0))).unwrap();
        play(&mut h, "8C");
        play(&mut h, "8S");
        // The turn-up is now a seven of trumps: the other is shown for 10.
        assert!(h.legal().contains(&Action::Dix(c("7H", 1))));
        h.apply(Action::Dix(c("7H", 1))).unwrap();
        assert_eq!(h.stock().turn_up(), Some(c("7H", 0)));
        assert!(
            h.holdings(Seat::Pone).concealed.contains(c("7H", 1)),
            "shown, and kept"
        );
        play(&mut h, "9C");
        play(&mut h, "9S");
        assert!(!h.legal().iter().any(|a| matches!(a, Action::Dix(_))));
        assert!(h.apply(Action::Dix(c("7H", 1))).is_err());
    }

    #[test]
    fn with_a_seven_turned_up_the_other_is_shown_and_neither_scores_twice() {
        // The pone's seven is the first copy; the turned seven the second.
        let mut h = hand("7H AS 7C 8C 9C 7D 8D 9D", DEALER, "7H", "");
        assert_eq!(h.score(Seat::Dealer), 10);
        play(&mut h, "AS");
        play(&mut h, "7S");
        assert!(h.legal().contains(&Action::Dix(c("7H", 0))));
        h.apply(Action::Dix(c("7H", 0))).unwrap();
        assert_eq!(h.score(Seat::Pone), 20);
        assert_eq!(h.stock().turn_up(), Some(c("7H", 1)));
        play(&mut h, "7C");
        play(&mut h, "8S");
        assert!(!h.legal().iter().any(|a| matches!(a, Action::Dix(_))));
    }

    #[test]
    fn either_twin_of_the_seven_may_be_exchanged() {
        let mut h = hand("7H 7H AS 8C 9C 7D 8D 9D", DEALER, "KH", "");
        play(&mut h, "AS");
        play(&mut h, "7S");
        h.apply(Action::Dix(c("7H", 1))).unwrap();
        assert!(h.holdings(Seat::Pone).concealed.contains(c("7H", 0)));
        assert_eq!(h.stock().turn_up(), Some(c("7H", 1)));
    }

    #[test]
    fn ids_that_name_no_card_are_refused() {
        let mut h = hand(PONE, DEALER, "KH", "");
        let before = format!("{h:?}");
        assert!(h.apply(Action::Play(Card(64))).is_err());
        assert!(h.apply(Action::Play(Card(255))).is_err());
        assert_eq!(format!("{h:?}"), before);
        play(&mut h, "AS");
        play(&mut h, "8H");
        assert!(h.apply(Action::Dix(Card(200))).is_err());
        let bogus = crate::melds::Declared {
            meld: Meld::Marriage(Suit(4)),
            cards: CardSet::EMPTY,
        };
        let refused = h.apply(Action::Declare(Announcement {
            melds: vec![bogus],
            score: bogus,
        }));
        assert!(refused.is_err());
    }

    #[test]
    fn refusals_change_nothing() {
        let mut h = hand(PONE, DEALER, "KH", "");
        let before = format!("{h:?}");
        assert!(
            h.apply(Action::Play(c("TS", 0))).is_err(),
            "the dealer's card"
        );
        assert!(
            h.apply(Action::Pass).is_err(),
            "nothing to pass on while leading"
        );
        assert!(h.apply(Action::Dix(c("7C", 0))).is_err());
        assert_eq!(format!("{h:?}"), before);
    }

    /// Both hands at the last stock trick: one face-down card and the turn-up
    /// left, the dealer having announced bezique and four kings, carrying the
    /// spade marriage.
    fn at_the_last_stock_trick() -> Hand {
        let trump = Suit::HEARTS;
        let mut dealer = Holdings::new(set("KS KH KD KC 7C 8C"));
        let kings = dealer
            .candidates(trump)
            .into_iter()
            .find(|d| d.meld == Meld::Four(Rank::KING))
            .unwrap();
        dealer
            .apply(
                trump,
                &Announcement {
                    melds: vec![kings],
                    score: kings,
                },
            )
            .unwrap();
        dealer.receive(c("QS", 0));
        dealer.receive(c("JD", 0));
        let forty = dealer
            .options(trump)
            .into_iter()
            .find(|a| a.score.meld == Meld::Bezique)
            .unwrap();
        dealer.apply(trump, &forty).unwrap();
        assert_eq!(dealer.carried().len(), 1);
        let pone = Holdings::new(set("AH TH 9S 9D 8D 7D AC TC"));
        let stock = Stock::new(&[c("QH", 0)], Some(c("9H", 0)));
        Hand::arranged(trump, stock, [dealer, pone], Seat::Pone, 23)
    }

    #[test]
    fn at_the_last_stock_trick_the_winner_declares_takes_the_last_card_and_the_loser_the_turn_up() {
        let mut h = at_the_last_stock_trick();
        assert_eq!(
            h.tricks().len(),
            0,
            "arranged: earlier tricks are not recorded"
        );
        play(&mut h, "9S");
        play(&mut h, "KS");
        assert_eq!(h.phase(), Phase::Declare);
        assert_eq!(h.to_move(), Some(Seat::Dealer));
        // Still time to declare (the carried marriage was lost: its king played).
        assert!(h.holdings(Seat::Dealer).carried().is_empty());
        h.apply(Action::Pass).unwrap();
        assert!(h.stock().is_empty());
        assert!(h.in_endgame());
        assert!(h.holdings(Seat::Dealer).cards().contains(c("QH", 0)));
        assert!(
            h.holdings(Seat::Pone).cards().contains(c("9H", 0)),
            "the loser takes the turn-up"
        );
        for seat in Seat::BOTH {
            assert!(
                h.holdings(seat).shown.is_empty(),
                "table cards go back into the hand"
            );
            assert_eq!(h.holdings(seat).cards().len(), 8);
        }
        assert_eq!((h.phase(), h.to_move()), (Phase::Lead, Some(Seat::Dealer)));
    }

    #[test]
    fn in_the_last_eight_tricks_the_second_must_follow_and_win() {
        let mut h = at_the_last_stock_trick();
        play(&mut h, "9S");
        play(&mut h, "KS");
        h.apply(Action::Pass).unwrap();
        // The dealer leads the queen of hearts; the pone holds AH TH 9H.
        play(&mut h, "QH");
        let legal: Vec<Action> = h.legal();
        assert_eq!(
            legal,
            vec![Action::Play(c("TH", 0)), Action::Play(c("AH", 0))]
        );
        let refused = h.apply(Action::Play(c("9H", 0))).unwrap_err();
        assert!(refused.contains("hearts"), "{refused}");
        assert!(h.apply(Action::Play(c("AC", 0))).is_err());
        play(&mut h, "TH");
        assert_eq!(h.tricks().last().unwrap().winner, Seat::Pone);
        assert_eq!(
            h.phase(),
            Phase::Lead,
            "no declaring once the stock is gone"
        );
    }

    #[test]
    fn nothing_concealed_is_known_at_the_deal() {
        let h = hand(PONE, DEALER, "KH", "");
        assert!(h.known(Seat::Dealer).is_empty());
        assert!(h.known(Seat::Pone).is_empty());
    }

    #[test]
    fn the_card_taken_by_the_dix_is_known_until_played() {
        let mut h = hand(PONE, "TS 7S 8S 9S JS 8H 7H KC", "KH", "");
        play(&mut h, "AS");
        play(&mut h, "8H");
        h.apply(Action::Dix(c("7H", 0))).unwrap();
        assert_eq!(h.known(Seat::Dealer), CardSet::of(&[c("KH", 0)]));
        play(&mut h, "KH");
        assert!(
            h.known(Seat::Dealer).is_empty(),
            "played, so no longer in the hand"
        );
    }

    #[test]
    fn the_second_seven_shown_is_known() {
        let mut h = hand("7H AS 7C 8C 9C 7D 8D 9D", DEALER, "7H", "");
        play(&mut h, "AS");
        play(&mut h, "7S");
        h.apply(Action::Dix(c("7H", 0))).unwrap();
        assert_eq!(h.known(Seat::Pone), CardSet::of(&[c("7H", 0)]));
    }

    #[test]
    fn table_cards_count_as_known_and_stay_known_when_taken_up() {
        let mut h = at_the_last_stock_trick();
        assert_eq!(h.known(Seat::Dealer), h.holdings(Seat::Dealer).shown);
        play(&mut h, "9S");
        play(&mut h, "KS");
        h.apply(Action::Pass).unwrap();
        // KH KD KC QS JD were on the table; the turn-up went to the pone.
        assert_eq!(h.known(Seat::Dealer), set("KH KD KC QS JD"));
        assert_eq!(h.known(Seat::Pone), CardSet::of(&[c("9H", 0)]));
    }

    #[test]
    fn the_last_trick_scores_ten_and_ends_the_hand() {
        let trump = Suit::HEARTS;
        let h0 = Hand::arranged(
            trump,
            Stock::new(&[], None),
            [Holdings::new(set("AS")), Holdings::new(set("7S"))],
            Seat::Dealer,
            31,
        );
        let mut h = h0.clone();
        play(&mut h, "AS");
        play(&mut h, "7S");
        assert_eq!(h.score(Seat::Dealer), 20, "the ace, and the last trick");
        assert_eq!(h.log().last().unwrap().source, Source::LastTrick);
        assert_eq!(h.phase(), Phase::Over);
        assert_eq!(h.to_move(), None);
        assert!(h.legal().is_empty());
    }
}
