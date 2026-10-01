//! One hand, from the deal to the last trick.
//!
//! While the stock lasts, each trick is led, answered with any card, and
//! taken; its winner may then declare (or exchange the seven of trumps)
//! before both draw, the winner first. When the stock is exhausted the cards
//! on the table go back into the hands and the last eight tricks are played
//! under the follow-and-win rule. Every point is entered in a log as it is
//! scored. `docs/RULES.md` §2–§6 gives the rules; `docs/DESIGN.md` §7 the
//! shape.

use crate::cards::{Card, CardSet, CardType, Rank, Suit};
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
    /// For each seat and card type, how many of its concealed cards of that
    /// type the other player has seen and not since seen leave. Counted by
    /// type, because twins cannot be told apart: a player who saw a K♥ taken
    /// and then a K♥ played cannot know whether another remains.
    seen: [[u8; 32]; 2],
    /// Every card each seat has held, for telling its own twins apart.
    handled: [CardSet; 2],
    /// Cards in the order they first became public.
    appeared: Vec<Card>,
    log: Vec<Scored>,
    /// Each seat's points this hand: the log's sums, kept as it grows.
    scores: [u32; 2],
}

impl Hand {
    /// A hand as dealt: the pone to lead, and 10 to the dealer for a turned
    /// seven.
    pub fn new(deal: Deal) -> Hand {
        let (hands, trump, stock) = deal.into_parts();
        let turned = stock.turn_up().expect("a fresh stock has its turn-up");
        let mut hand = Hand::arranged(trump, stock, hands.map(Holdings::new), Seat::Pone, 0)
            .expect("a fresh deal is a possible position");
        if is_seven_of(turned, trump) {
            hand.sevens_scored = hand.sevens_scored.with(turned);
            hand.score_to(Seat::Dealer, 10, Source::TurnedSeven);
        }
        hand
    }

    /// A position part-way through a hand, for tests, analysis and search:
    /// `leader` to lead, `tricks_taken` tricks already played (their cards
    /// gone). Refused unless it is a position a hand can reach: every card in
    /// one place, the cards and tricks adding up to the pack, eight cards each
    /// while the stock lasts, a turn-up while face-down cards remain, an even
    /// number to draw, and nothing on the table once the stock is gone. Which
    /// sevens of trumps have already scored is not part of the position; say
    /// so with [`Hand::with_sevens_scored`]. Nor is what each player has seen
    /// of the other's concealed cards (the card taken by the dix, a seven
    /// shown): an arranged position starts with nothing seen, and the tricks
    /// before it are gone.
    pub fn arranged(
        trump: Suit,
        stock: Stock,
        holdings: [Holdings; 2],
        leader: Seat,
        tricks_taken: u8,
    ) -> Result<Hand, String> {
        if tricks_taken >= 32 {
            return Err(format!(
                "{tricks_taken} tricks taken: a hand has 32, and one after its last is over"
            ));
        }
        let held = [holdings[0].cards(), holdings[1].cards()];
        for h in &holdings {
            if !h.concealed.intersect(h.shown).is_empty() {
                return Err("a card both concealed and on the table".to_string());
            }
        }
        if !held[0].intersect(held[1]).is_empty()
            || !held[0].union(held[1]).intersect(stock.cards()).is_empty()
        {
            return Err("a card in two places".to_string());
        }
        if stock.cards().len() as usize != stock.len() {
            return Err("a card twice in the stock".to_string());
        }
        if stock.turn_up().is_some_and(|c| c.suit() != trump) {
            return Err("the turn-up is not of the trump suit".to_string());
        }
        if held[0].len() > 8 || held[1].len() > 8 {
            return Err("a player holds more than eight cards".to_string());
        }
        if stock.face_down() > 0 && stock.turn_up().is_none() {
            return Err("face-down cards remain but the turn-up is gone".to_string());
        }
        if stock.len() % 2 == 1 {
            return Err("an odd number of cards left to draw".to_string());
        }
        if held[0].len() != held[1].len() {
            return Err("the players hold different numbers of cards".to_string());
        }
        let in_play = 2 * held[0].len() as usize + stock.len();
        if in_play != 64 - 2 * usize::from(tricks_taken) {
            return Err(format!(
                "{in_play} cards in play after {tricks_taken} tricks"
            ));
        }
        if !stock.is_empty() && held[0].len() != 8 {
            return Err("while the stock lasts each player holds eight".to_string());
        }
        // While the stock lasts a card that has been declared stays on the
        // table until it is played; a concealed one has no history.
        if !stock.is_empty()
            && holdings.iter().any(|h| {
                h.announced
                    .iter()
                    .any(|a| !a.declared.cards.intersect(h.concealed).is_empty())
            })
        {
            return Err("a concealed card that has been declared".to_string());
        }
        if stock.is_empty() && holdings.iter().any(|h| !h.shown.is_empty()) {
            return Err("cards on the table once the stock is gone".to_string());
        }
        let mut hand = Hand {
            trump,
            stock,
            holdings: holdings.clone(),
            leader,
            led: None,
            phase: Phase::Lead,
            tricks: Vec::new(),
            taken: tricks_taken,
            won: [CardSet::EMPTY; 2],
            sevens_scored: CardSet::EMPTY,
            seen: [[0; 32]; 2],
            handled: [holdings[0].cards(), holdings[1].cards()],
            appeared: Vec::new(),
            log: Vec::new(),
            scores: [0, 0],
        };
        let public = hand
            .stock
            .turn_up()
            .into_iter()
            .chain(hand.holdings[0].shown.cards())
            .chain(hand.holdings[1].shown.cards());
        let public: Vec<Card> = public.collect();
        for card in public {
            hand.appear(card);
        }
        Ok(hand)
    }

    /// The same position, with these sevens of trumps recorded as having
    /// scored, so that none scores again.
    pub fn with_sevens_scored(mut self, sevens: CardSet) -> Hand {
        self.sevens_scored = sevens;
        self
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
    ///
    /// Concealed known cards are counted by type; of twins, the copies that
    /// were actually seen stand first, then the lower.
    pub fn known(&self, seat: Seat) -> CardSet {
        let holdings = &self.holdings[seat.index()];
        let mut known = holdings.shown;
        for (index, &count) in self.seen[seat.index()].iter().enumerate() {
            let twins = holdings.concealed.intersect(CardType(index as u8).twins());
            // The copies that were actually seen stand first.
            let mut copies: Vec<Card> = twins.cards().collect();
            copies.sort_by_key(|c| (!self.appeared.contains(c), c.0));
            for &card in copies.iter().take(count as usize) {
                known = known.with(card);
            }
        }
        known
    }

    /// Every card `seat` has held this hand: the ones it can tell apart from
    /// their twins.
    pub fn handled(&self, seat: Seat) -> CardSet {
        self.handled[seat.index()]
    }

    /// The cards that have become public, in the order they did: the turn-up,
    /// cards laid on the table, cards played, sevens shown.
    pub fn appeared(&self) -> &[Card] {
        &self.appeared
    }

    fn appear(&mut self, card: Card) {
        if !self.appeared.contains(&card) {
            self.appeared.push(card);
        }
    }

    /// `card` comes into the other player's sight while concealed in `seat`'s
    /// hand.
    fn comes_to_light(&mut self, seat: Seat, card: Card) {
        let count = &mut self.seen[seat.index()][card.card_type().0 as usize];
        *count = (*count + 1).min(2);
    }

    /// A card of `card`'s type leaves `seat`'s concealed cards (played or
    /// laid): the other player can no longer be sure a seen one remains.
    fn leaves_concealed(&mut self, seat: Seat, card: Card) {
        if self.holdings[seat.index()].concealed.contains(card) {
            let count = &mut self.seen[seat.index()][card.card_type().0 as usize];
            *count = count.saturating_sub(1);
        }
    }

    /// A seat's points this hand.
    pub fn score(&self, seat: Seat) -> u32 {
        self.scores[seat.index()]
    }

    /// The moves to offer the seat whose turn it is. Every card it may play
    /// (concealed twins as one move; cards on the table one by one, since
    /// their histories may differ); in the turn of declaring, passing, the
    /// dix, and the announcements of `Holdings::options`, which are the
    /// worthwhile ones, not every announcement the rules allow. `apply`
    /// accepts any announcement `Holdings::check` allows.
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
                self.leaves_concealed(seat, card);
                self.holdings[seat.index()].play(card);
                self.appear(card);
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
                self.leaves_concealed(seat, card);
                self.holdings[seat.index()].play(card);
                self.appear(card);
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
                if is_seven_of(turned, self.trump) {
                    // Shown and kept.
                    self.comes_to_light(seat, seven);
                } else {
                    let taken = self.stock.exchange(seven);
                    self.leaves_concealed(seat, seven);
                    let holdings = &mut self.holdings[seat.index()];
                    holdings.play(seven);
                    holdings.receive(taken);
                    self.handled[seat.index()] = self.handled[seat.index()].with(taken);
                    self.comes_to_light(seat, taken);
                }
                self.appear(seven);
                self.score_to(seat, 10, Source::Dix);
                self.draw();
            }
            (Phase::Declare, Action::Declare(announcement)) => {
                let before = self.holdings[seat.index()].concealed;
                let points = self.holdings[seat.index()].apply(self.trump, &announcement)?;
                let laid = before.minus(self.holdings[seat.index()].concealed);
                for card in laid.cards() {
                    let count = &mut self.seen[seat.index()][card.card_type().0 as usize];
                    *count = count.saturating_sub(1);
                    self.appear(card);
                }
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
            self.handled[seat.index()] = self.handled[seat.index()].with(card);
            if face_up {
                self.comes_to_light(seat, card);
            }
        }
        if self.stock.is_empty() {
            for seat in Seat::BOTH {
                for card in self.holdings[seat.index()].shown.cards() {
                    self.comes_to_light(seat, card);
                }
                self.holdings[seat.index()].take_up();
            }
        }
        self.phase = Phase::Lead;
    }

    fn score_to(&mut self, seat: Seat, points: u32, source: Source) {
        self.scores[seat.index()] += points;
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
        Hand::arranged(trump, stock, [dealer, pone], Seat::Pone, 23).unwrap()
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
    fn an_impossible_position_is_refused() {
        let eight = |codes: &str| Holdings::new(set(codes));
        let (a, b) = (
            eight("AS TS KS QS JS 9S 8S 7S"),
            eight("AH TH KH QH JH 9H 8H 7H"),
        );
        let rest: Vec<Card> = CardSet::PACK
            .minus(a.cards())
            .minus(b.cards())
            .cards()
            .collect();
        // The turn-up's suit is trumps.
        let trump = rest[0].suit();
        let fine = Hand::arranged(
            trump,
            Stock::new(&rest[1..], Some(rest[0])),
            [a.clone(), b.clone()],
            Seat::Pone,
            0,
        );
        assert!(fine.is_ok());
        let bad = [
            // Face-down cards without the turn-up.
            Hand::arranged(
                trump,
                Stock::new(&rest[..47], None),
                [a.clone(), b.clone()],
                Seat::Pone,
                0,
            ),
            // An odd number of cards to draw.
            Hand::arranged(
                trump,
                Stock::new(&rest[2..], Some(rest[0])),
                [a.clone(), b.clone()],
                Seat::Pone,
                0,
            ),
            // Cards and tricks that do not add up to the pack.
            Hand::arranged(
                trump,
                Stock::new(&[], None),
                [eight("AS"), eight("7S")],
                Seat::Pone,
                0,
            ),
            Hand::arranged(
                trump,
                Stock::new(&[], None),
                [eight("AS 9C"), eight("7S 8C")],
                Seat::Pone,
                31,
            ),
            Hand::arranged(
                trump,
                Stock::new(&[], None),
                [eight("AS"), eight("7S")],
                Seat::Pone,
                255,
            ),
            // A concealed card with a declaring history, while the stock
            // lasts: declared cards stay on the table until played.
            Hand::arranged(
                trump,
                Stock::new(&rest[1..], Some(rest[0])),
                [
                    Holdings {
                        announced: vec![crate::melds::Announced {
                            declared: crate::melds::Declared {
                                meld: crate::melds::Meld::Marriage(Suit::SPADES),
                                cards: set("KS QS"),
                            },
                            scored: true,
                        }],
                        ..a.clone()
                    },
                    b.clone(),
                ],
                Seat::Pone,
                0,
            ),
            // A hand after its last trick: over, not a position to play.
            Hand::arranged(
                trump,
                Stock::new(&[], None),
                [Holdings::default(), Holdings::default()],
                Seat::Pone,
                32,
            ),
            // A card in two places.
            Hand::arranged(
                trump,
                Stock::new(&rest[1..], Some(rest[0])),
                [a.clone(), a.clone()],
                Seat::Pone,
                0,
            ),
            // A card twice in the stock, and another nowhere.
            Hand::arranged(
                trump,
                Stock::new(&[&rest[1..2], &rest[1..47]].concat(), Some(rest[0])),
                [a.clone(), b.clone()],
                Seat::Pone,
                0,
            ),
            // The turn-up face down as well.
            Hand::arranged(
                trump,
                Stock::new(&rest[..47], Some(rest[0])),
                [a.clone(), b.clone()],
                Seat::Pone,
                0,
            ),
            // A turn-up not of the trump suit.
            Hand::arranged(
                Suit((trump.0 + 1) % 4),
                Stock::new(&rest[1..], Some(rest[0])),
                [a.clone(), b.clone()],
                Seat::Pone,
                0,
            ),
            // More than eight cards each once the stock is gone.
            Hand::arranged(
                trump,
                Stock::new(&[], None),
                [
                    Holdings::new(a.cards().union(b.cards())),
                    Holdings::new(CardSet::PACK.minus(a.cards()).minus(b.cards())),
                ],
                Seat::Pone,
                0,
            ),
        ];
        for (i, position) in bad.iter().enumerate() {
            assert!(position.is_err(), "position {i} should be refused");
        }
        // Cards still on the table once the stock is gone.
        let mut shown = Holdings::new(set("KS QS"));
        let marriage = shown.candidates(trump)[0];
        shown
            .apply(
                trump,
                &Announcement {
                    melds: vec![marriage],
                    score: marriage,
                },
            )
            .unwrap();
        assert!(Hand::arranged(
            trump,
            Stock::new(&[], None),
            [shown, eight("7C 8C")],
            Seat::Pone,
            30
        )
        .is_err());
    }

    #[test]
    fn an_arranged_hand_can_be_told_which_sevens_have_scored() {
        let trump = Suit::HEARTS;
        let pone = Holdings::new(set("7H AS 8C 9C 7D 8D 9D JC"));
        let dealer = Holdings::new(set("TS 7S 8S 9S JS 8H 9H KC"));
        let mut rest: Vec<Card> = CardSet::PACK
            .minus(pone.cards())
            .minus(dealer.cards())
            .cards()
            .collect();
        let turn_up = rest.remove(rest.iter().position(|c| c.suit() == trump).unwrap());
        let stock = Stock::new(&rest, Some(turn_up));
        let mut h = Hand::arranged(trump, stock, [dealer, pone], Seat::Pone, 0)
            .unwrap()
            .with_sevens_scored(CardSet::of(&[c("7H", 0)]));
        play(&mut h, "AS");
        play(&mut h, "7S");
        assert!(!h.legal().iter().any(|a| matches!(a, Action::Dix(_))));
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

    /// A seen card and its unseen twin cannot be told apart: once a card of
    /// the type is played, the seen one may be gone.
    #[test]
    fn a_seen_card_is_known_only_until_one_like_it_is_played() {
        let mut h = hand(PONE, "TS 7S 8S 9S JS 8H 7H KH", "KH", "");
        play(&mut h, "AS");
        play(&mut h, "8H");
        h.apply(Action::Dix(c("7H", 0))).unwrap();
        assert_eq!(
            h.known(Seat::Dealer)
                .count_of(CardType::parse("KH").unwrap()),
            1
        );
        play(&mut h, "KH");
        assert!(
            h.known(Seat::Dealer).is_empty(),
            "{:?}",
            h.known(Seat::Dealer)
        );
    }

    #[test]
    fn the_order_of_appearance_is_kept() {
        let mut h = hand(PONE, DEALER, "KH", "");
        assert_eq!(h.appeared(), &[c("KH", 0)]);
        play(&mut h, "AS");
        play(&mut h, "8H");
        assert_eq!(h.appeared(), &[c("KH", 0), c("AS", 0), c("8H", 0)]);
        assert!(h.handled(Seat::Pone).contains(c("AS", 0)));
        assert!(!h.handled(Seat::Pone).contains(c("8H", 0)));
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
        )
        .unwrap();
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
