//! Declaring: the combinations, who may declare what, and carried scores.
//!
//! `docs/RULES.md` §4 gives the rules (Foster, pp. 340–343) and
//! `docs/DESIGN.md` §6 the model. In brief: each combination has a class and
//! a level within it, and each card a history, which is the highest level at
//! which it has been announced in each class. A card may join a combination
//! only below that combination's level in its class; every announcement adds a
//! card from the hand; one combination is scored per trick won, and the rest of
//! an announcement is carried "to score" while its cards stay on the table.

use crate::cards::{Card, CardSet, CardType, Rank, Suit};

/// Foster's three classes of combination.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Class {
    /// Class A: marriages (level 1) and the sequence (level 2).
    Marriages,
    /// Class B: bezique (level 1) and double bezique (level 2).
    Bezique,
    /// Class C: four of a kind (level 1).
    Fours,
}

/// A combination, as a kind: what it is made of and what it scores.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Meld {
    /// King and queen of a suit: royal (40) in trumps, common (20) otherwise.
    Marriage(Suit),
    /// Ace, ten, king, queen and jack of trumps: 250.
    Sequence,
    /// Queen of spades and jack of diamonds: 40.
    Bezique,
    /// Both queens of spades and both jacks of diamonds: 500.
    DoubleBezique,
    /// Any four aces (100), kings (80), queens (60) or jacks (40).
    Four(Rank),
}

impl Meld {
    /// Every combination there is: four marriages (one of them royal), the
    /// sequence, bezique, double bezique, and four fours.
    pub fn all() -> [Meld; 11] {
        [
            Meld::Marriage(Suit::SPADES),
            Meld::Marriage(Suit::HEARTS),
            Meld::Marriage(Suit::DIAMONDS),
            Meld::Marriage(Suit::CLUBS),
            Meld::Sequence,
            Meld::Bezique,
            Meld::DoubleBezique,
            Meld::Four(Rank::ACE),
            Meld::Four(Rank::KING),
            Meld::Four(Rank::QUEEN),
            Meld::Four(Rank::JACK),
        ]
    }

    /// Whether this names a combination that exists: a marriage in one of
    /// the four suits, a four of aces, kings, queens or jacks, or one of the
    /// others.
    pub fn is_valid(self) -> bool {
        match self {
            Meld::Marriage(suit) => suit.is_valid(),
            Meld::Four(rank) => [Rank::ACE, Rank::KING, Rank::QUEEN, Rank::JACK].contains(&rank),
            _ => true,
        }
    }

    pub fn class(self) -> Class {
        match self {
            Meld::Marriage(_) | Meld::Sequence => Class::Marriages,
            Meld::Bezique | Meld::DoubleBezique => Class::Bezique,
            Meld::Four(_) => Class::Fours,
        }
    }

    /// Its level within its class: 2 for the sequence and double bezique, 1
    /// otherwise.
    pub fn level(self) -> u8 {
        match self {
            Meld::Sequence | Meld::DoubleBezique => 2,
            _ => 1,
        }
    }

    pub fn points(self, trump: Suit) -> u32 {
        match self {
            Meld::Marriage(suit) if suit == trump => 40,
            Meld::Marriage(_) => 20,
            Meld::Sequence => 250,
            Meld::Bezique => 40,
            Meld::DoubleBezique => 500,
            Meld::Four(Rank::ACE) => 100,
            Meld::Four(Rank::KING) => 80,
            Meld::Four(Rank::QUEEN) => 60,
            Meld::Four(Rank::JACK) => 40,
            Meld::Four(_) => 0,
        }
    }

    /// The card types it needs, each with how many copies; a four is any four
    /// cards of its rank, and so has no fixed list.
    fn needs(self, trump: Suit) -> Vec<(CardType, usize)> {
        let of = |rank, suit| CardType::new(rank, suit);
        let queen_of_spades = of(Rank::QUEEN, Suit::SPADES);
        let jack_of_diamonds = of(Rank::JACK, Suit::DIAMONDS);
        match self {
            Meld::Marriage(suit) => vec![(of(Rank::KING, suit), 1), (of(Rank::QUEEN, suit), 1)],
            Meld::Sequence => [Rank::ACE, Rank::TEN, Rank::KING, Rank::QUEEN, Rank::JACK]
                .map(|rank| (of(rank, trump), 1))
                .to_vec(),
            Meld::Bezique => vec![(queen_of_spades, 1), (jack_of_diamonds, 1)],
            Meld::DoubleBezique => vec![(queen_of_spades, 2), (jack_of_diamonds, 2)],
            Meld::Four(_) => vec![],
        }
    }

    /// Whether exactly these cards make this combination, trumps being
    /// `trump`. Fours may repeat a suit (Foster, p. 341).
    pub fn is_made_by(self, cards: CardSet, trump: Suit) -> bool {
        match self {
            Meld::Four(rank) => {
                self.points(trump) > 0
                    && cards.len() == 4
                    && cards.cards().all(|c| c.rank() == rank)
            }
            _ => {
                let needs = self.needs(trump);
                let total: usize = needs.iter().map(|&(_, n)| n).sum();
                cards.len() as usize == total
                    && needs.iter().all(|&(t, n)| cards.count_of(t) as usize == n)
            }
        }
    }
}

/// A combination made of particular cards.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Declared {
    pub meld: Meld,
    pub cards: CardSet,
}

/// A combination once announced: scored at once, or carried "to score".
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Announced {
    pub declared: Declared,
    pub scored: bool,
}

/// One turn of declaring (the dix aside): lay down the cards of `melds`
/// (possibly none) and score `score`, which is one of them or a combination
/// already carried.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Announcement {
    pub melds: Vec<Declared>,
    pub score: Declared,
}

/// What one player holds while the stock lasts: the concealed cards, the cards
/// shown on the table (still part of the hand), and every combination the
/// player has announced this hand.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Holdings {
    pub concealed: CardSet,
    pub shown: CardSet,
    pub announced: Vec<Announced>,
}

impl Holdings {
    pub fn new(concealed: CardSet) -> Holdings {
        Holdings {
            concealed,
            ..Holdings::default()
        }
    }

    /// Concealed and shown together: everything the player may play.
    pub fn cards(&self) -> CardSet {
        self.concealed.union(self.shown)
    }

    /// The highest level at which `card` has been announced in `class`, or 0.
    pub fn level(&self, card: Card, class: Class) -> u8 {
        self.announced
            .iter()
            .map(|a| a.declared)
            .filter(|d| d.meld.class() == class && d.cards.contains(card))
            .map(|d| d.meld.level())
            .max()
            .unwrap_or(0)
    }

    /// The combinations announced but not yet scored that could still be
    /// scored: every card still on the table, and none raised since into a
    /// higher combination of its class.
    pub fn carried(&self) -> Vec<Declared> {
        self.announced
            .iter()
            .filter(|a| !a.scored)
            .map(|a| a.declared)
            .filter(|d| {
                d.cards.minus(self.shown).is_empty()
                    && d.cards
                        .cards()
                        .all(|c| self.level(c, d.meld.class()) == d.meld.level())
            })
            .collect()
    }

    /// Every single combination that could be announced now, each with at
    /// least one concealed card. Concealed twins are interchangeable, so of
    /// two choices that differ only in which twin is laid, the one laying the
    /// lower copy is kept.
    pub fn candidates(&self, trump: Suit) -> Vec<Declared> {
        self.distinct_candidates(&self.every_candidate(trump))
    }

    /// `every`, with choices that differ only in which concealed twin is laid
    /// counted once (the lower copy).
    fn distinct_candidates(&self, every: &[Declared]) -> Vec<Declared> {
        let mut found: Vec<Declared> = Vec::new();
        for d in every {
            let d = Declared {
                meld: d.meld,
                cards: self.canonical(d.cards),
            };
            if !found.contains(&d) {
                found.push(d);
            }
        }
        found
    }

    /// Every way of making each combination from particular cards, twins
    /// counted separately.
    fn every_candidate(&self, trump: Suit) -> Vec<Declared> {
        let mut found = Vec::new();
        for meld in Meld::all() {
            let (class, level) = (meld.class(), meld.level());
            let eligible = |cards: CardSet| -> Vec<Card> {
                cards
                    .intersect(self.cards())
                    .cards()
                    .filter(|&c| self.level(c, class) < level)
                    .collect()
            };
            let choices: Vec<CardSet> = match meld {
                Meld::Four(rank) => {
                    let of_rank = Suit::ALL.iter().fold(CardSet::EMPTY, |s, &suit| {
                        s.union(CardType::new(rank, suit).twins())
                    });
                    subsets(&eligible(of_rank), 4)
                }
                _ => meld
                    .needs(trump)
                    .iter()
                    .fold(vec![CardSet::EMPTY], |partial, &(t, n)| {
                        let picks = subsets(&eligible(t.twins()), n);
                        partial
                            .iter()
                            .flat_map(|&p| picks.iter().map(move |&q| p.union(q)))
                            .collect()
                    }),
            };
            for cards in choices {
                if meld.is_made_by(cards, trump) && !cards.intersect(self.concealed).is_empty() {
                    found.push(Declared { meld, cards });
                }
            }
        }
        found
    }

    /// The same cards with each concealed type's copies renumbered lowest
    /// first: concealed twins are interchangeable.
    fn canonical(&self, cards: CardSet) -> CardSet {
        let concealed = cards.intersect(self.concealed);
        let mut out = cards.minus(concealed);
        for card in concealed.cards() {
            let lowest = self
                .concealed
                .intersect(card.card_type().twins())
                .minus(out)
                .lowest()
                .expect("a concealed card has itself to stand for");
            out = out.with(lowest);
        }
        out
    }

    /// Whether `announcement` is allowed, and if not, why.
    pub fn check(&self, trump: Suit, announcement: &Announcement) -> Result<(), String> {
        let Announcement { melds, score } = announcement;
        if let Some(bad) = melds.iter().chain([score]).find(|d| !d.meld.is_valid()) {
            return Err(format!("{:?} is not a combination", bad.meld));
        }
        for d in melds {
            let name = name(d.meld, trump);
            if !d.meld.is_made_by(d.cards, trump) {
                return Err(format!("those cards do not make {name}"));
            }
            if !d.cards.minus(self.cards()).is_empty() {
                return Err(format!("{name} needs cards you do not hold"));
            }
            if d.cards.intersect(self.concealed).is_empty() {
                return Err(format!(
                    "{name} must add a card from your hand to the table"
                ));
            }
            if let Some(spent) = d
                .cards
                .cards()
                .find(|&c| self.level(c, d.meld.class()) >= d.meld.level())
            {
                return Err(format!(
                    "{} has already been declared in a combination that rules out {name}",
                    spent.card_type()
                ));
            }
        }
        for (i, x) in melds.iter().enumerate() {
            for y in &melds[i + 1..] {
                if clash(*x, *y) {
                    return Err(format!(
                        "{} and {} share a card, and a card serves only once in a class",
                        name(x.meld, trump),
                        name(y.meld, trump)
                    ));
                }
            }
        }
        if melds.contains(score) {
            return Ok(());
        }
        if !self.carried().contains(score) {
            return Err(format!(
                "{} is neither announced now nor carried",
                name(score.meld, trump)
            ));
        }
        let covered = melds.iter().any(|d| {
            d.meld.class() == score.meld.class()
                && d.meld.level() > score.meld.level()
                && !d.cards.intersect(score.cards).is_empty()
        });
        if covered {
            return Err(format!(
                "{} is lost once its cards are declared in a higher combination",
                name(score.meld, trump)
            ));
        }
        Ok(())
    }

    /// Makes the announcement: lays its new cards, records its combinations,
    /// and returns the points scored.
    pub fn apply(&mut self, trump: Suit, announcement: &Announcement) -> Result<u32, String> {
        self.check(trump, announcement)?;
        let Announcement { melds, score } = announcement;
        let laid = melds
            .iter()
            .fold(CardSet::EMPTY, |s, d| s.union(d.cards))
            .intersect(self.concealed);
        self.concealed = self.concealed.minus(laid);
        self.shown = self.shown.union(laid);
        if !melds.contains(score) {
            let carried = self
                .announced
                .iter_mut()
                .find(|a| !a.scored && a.declared == *score)
                .expect("checked: the score is carried");
            carried.scored = true;
        }
        self.announced
            .extend(melds.iter().map(|&declared| Announced {
                declared,
                scored: declared == *score,
            }));
        Ok(score.meld.points(trump))
    }

    /// The announcements worth offering a person. For each combination that
    /// could be laid, the announcement that lays it together with every
    /// combination its new cards complete with the cards already shown (the
    /// "free riders", at most value-maximising set that does not repeat a
    /// card within a class), once for each choice of which to score now; and
    /// where it is worth more, the best set laying the same cards without
    /// that combination. Then each carried combination, scored alone.
    pub fn options(&self, trump: Suit) -> Vec<Announcement> {
        let every = self.every_candidate(trump);
        let mut out: Vec<Announcement> = Vec::new();
        let mut offer = |melds: Vec<Declared>, score: Declared| {
            let a = Announcement { melds, score };
            let same = |b: &Announcement| {
                b.score == a.score
                    && b.melds.len() == a.melds.len()
                    && a.melds.iter().all(|d| b.melds.contains(d))
            };
            if !out.iter().any(same) {
                out.push(a);
            }
        };
        let guarded = self
            .carried()
            .iter()
            .fold(CardSet::EMPTY, |s, d| s.union(d.cards));
        for main in self.distinct_candidates(&every) {
            let laid = main.cards.intersect(self.concealed);
            let riders: Vec<Declared> = every
                .iter()
                .copied()
                .filter(|r| {
                    *r != main
                        && r.cards.minus(self.shown.union(laid)).is_empty()
                        && !r.cards.intersect(laid).is_empty()
                        && !clash(*r, main)
                })
                .collect();
            let mut melds = vec![main];
            melds.extend(best_riders(&riders, trump, guarded.union(main.cards)));
            for &score in &melds {
                offer(melds.clone(), score);
            }
            // The same cards laid, without insisting on `main`: two new
            // cards that could each marry a twin already on the table make
            // two marriages, worth more than their own one, which would
            // leave the old cards unable ever to marry (full review of
            // 2026-10-02). Offered when it is worth more.
            let around: Vec<Declared> = every
                .iter()
                .copied()
                .filter(|r| {
                    r.cards.minus(self.shown.union(laid)).is_empty()
                        && !r.cards.intersect(laid).is_empty()
                })
                .collect();
            let wider = best_riders(&around, trump, guarded);
            let worth = |set: &[Declared]| set.iter().map(|d| d.meld.points(trump)).sum::<u32>();
            if worth(&wider) > worth(&melds) {
                for &score in &wider {
                    offer(wider.clone(), score);
                }
            }
        }
        for carried in self.carried() {
            offer(vec![], carried);
        }
        out
    }

    /// A card leaves the hand, played to a trick.
    pub fn play(&mut self, card: Card) {
        debug_assert!(self.cards().contains(card), "{card:?} is not held");
        self.concealed = self.concealed.without(card);
        self.shown = self.shown.without(card);
    }

    /// A card joins the hand, drawn from the stock.
    pub fn receive(&mut self, card: Card) {
        self.concealed = self.concealed.with(card);
    }

    /// When the stock is exhausted the cards on the table go back into the
    /// hand, and declaring is over.
    pub fn take_up(&mut self) {
        self.concealed = self.cards();
        self.shown = CardSet::EMPTY;
    }
}

/// Two combinations in one class that share a card cannot be announced
/// together.
fn clash(a: Declared, b: Declared) -> bool {
    a.meld.class() == b.meld.class() && !a.cards.intersect(b.cards).is_empty()
}

/// The free riders worth announcing: the set of greatest value with no two in
/// one class sharing a card; of sets equally valuable, the one that ties the
/// fewest cards to scores still to come (`guarded`: the cards of combinations
/// already carried), so that more cards stay free to play. There are only
/// ever a handful to choose among; past sixteen, riders are taken greedily,
/// most valuable first, rather than searched.
fn best_riders(riders: &[Declared], trump: Suit, guarded: CardSet) -> Vec<Declared> {
    let tied = |chosen: &[Declared]| chosen.iter().fold(guarded, |s, d| s.union(d.cards)).len();
    if riders.len() > 16 {
        let mut sorted = riders.to_vec();
        sorted.sort_by_key(|d| std::cmp::Reverse(d.meld.points(trump)));
        let mut chosen: Vec<Declared> = Vec::new();
        for rider in sorted {
            if chosen.iter().all(|&c| !clash(c, rider)) {
                chosen.push(rider);
            }
        }
        return chosen;
    }
    let mut best: (u32, std::cmp::Reverse<u32>, u32) = (0, std::cmp::Reverse(0), 0);
    for mask in 1u32..(1 << riders.len()) {
        let chosen: Vec<Declared> = (0..riders.len())
            .filter(|i| mask >> i & 1 == 1)
            .map(|i| riders[i])
            .collect();
        let compatible = chosen
            .iter()
            .enumerate()
            .all(|(i, &x)| chosen[i + 1..].iter().all(|&y| !clash(x, y)));
        if !compatible {
            continue;
        }
        let value: u32 = chosen.iter().map(|d| d.meld.points(trump)).sum();
        let key = (value, std::cmp::Reverse(tied(&chosen)), mask);
        if (key.0, key.1) > (best.0, best.1) {
            best = key;
        }
    }
    (0..riders.len())
        .filter(|i| best.2 >> i & 1 == 1)
        .map(|i| riders[i])
        .collect()
}

/// Every `n`-card subset of `cards`.
fn subsets(cards: &[Card], n: usize) -> Vec<CardSet> {
    if n == 0 {
        return vec![CardSet::EMPTY];
    }
    let mut out = Vec::new();
    for (i, &first) in cards.iter().enumerate() {
        for rest in subsets(&cards[i + 1..], n - 1) {
            out.push(rest.with(first));
        }
    }
    out
}

/// How a combination is named to a player.
pub fn name(meld: Meld, trump: Suit) -> String {
    match meld {
        Meld::Marriage(suit) if suit == trump => "the royal marriage".to_string(),
        Meld::Marriage(suit) => format!("the marriage in {}", suit.name()),
        Meld::Sequence => "the sequence".to_string(),
        Meld::Bezique => "bezique".to_string(),
        Meld::DoubleBezique => "double bezique".to_string(),
        Meld::Four(rank) => format!("four {}s", rank.name()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::Rng;

    const HEARTS: Suit = Suit::HEARTS;

    fn set(codes: &str) -> CardSet {
        CardSet::parse(codes).unwrap()
    }

    fn card(code: &str, copy: u8) -> Card {
        Card::new(CardType::parse(code).unwrap(), copy)
    }

    /// The candidate of this kind, if exactly one exists.
    fn the(h: &Holdings, meld: Meld) -> Declared {
        let found: Vec<Declared> = h
            .candidates(HEARTS)
            .into_iter()
            .filter(|d| d.meld == meld)
            .collect();
        assert_eq!(
            found.len(),
            1,
            "expected one {meld:?} among {:?}",
            h.candidates(HEARTS)
        );
        found[0]
    }

    fn can(h: &Holdings, meld: Meld) -> bool {
        h.candidates(HEARTS).iter().any(|d| d.meld == meld)
    }

    /// Announces exactly these combinations, scoring the first.
    fn announce(h: &mut Holdings, melds: &[Declared]) -> u32 {
        let a = Announcement {
            melds: melds.to_vec(),
            score: melds[0],
        };
        h.apply(HEARTS, &a).unwrap()
    }

    /// Announces the one combination of this kind, alone, and scores it.
    fn declare(h: &mut Holdings, meld: Meld) -> u32 {
        let d = the(h, meld);
        announce(h, &[d])
    }

    fn score_carried(h: &mut Holdings, meld: Meld) -> u32 {
        let carried = *h
            .carried()
            .iter()
            .find(|d| d.meld == meld)
            .expect("carried");
        h.apply(
            HEARTS,
            &Announcement {
                melds: vec![],
                score: carried,
            },
        )
        .unwrap()
    }

    fn meld_kinds(a: &Announcement) -> Vec<Meld> {
        a.melds.iter().map(|d| d.meld).collect()
    }

    // ---- the combinations themselves -------------------------------------

    #[test]
    fn the_scores() {
        assert_eq!(Meld::Marriage(Suit::SPADES).points(HEARTS), 20);
        assert_eq!(Meld::Marriage(HEARTS).points(HEARTS), 40);
        assert_eq!(Meld::Sequence.points(HEARTS), 250);
        assert_eq!(Meld::Bezique.points(HEARTS), 40);
        assert_eq!(Meld::DoubleBezique.points(HEARTS), 500);
        assert_eq!(Meld::Four(Rank::ACE).points(HEARTS), 100);
        assert_eq!(Meld::Four(Rank::KING).points(HEARTS), 80);
        assert_eq!(Meld::Four(Rank::QUEEN).points(HEARTS), 60);
        assert_eq!(Meld::Four(Rank::JACK).points(HEARTS), 40);
    }

    #[test]
    fn classes_and_levels() {
        assert_eq!(
            (
                Meld::Marriage(HEARTS).class(),
                Meld::Marriage(HEARTS).level()
            ),
            (Class::Marriages, 1)
        );
        assert_eq!(
            (Meld::Sequence.class(), Meld::Sequence.level()),
            (Class::Marriages, 2)
        );
        assert_eq!(
            (Meld::Bezique.class(), Meld::Bezique.level()),
            (Class::Bezique, 1)
        );
        assert_eq!(
            (Meld::DoubleBezique.class(), Meld::DoubleBezique.level()),
            (Class::Bezique, 2)
        );
        assert_eq!(
            (Meld::Four(Rank::ACE).class(), Meld::Four(Rank::ACE).level()),
            (Class::Fours, 1)
        );
        assert_eq!(Meld::all().len(), 11);
    }

    #[test]
    fn what_makes_each_combination() {
        assert!(Meld::Marriage(Suit::CLUBS).is_made_by(set("KC QC"), HEARTS));
        assert!(!Meld::Marriage(Suit::CLUBS).is_made_by(set("KC QD"), HEARTS));
        assert!(!Meld::Marriage(Suit::CLUBS).is_made_by(set("KC QC JC"), HEARTS));
        assert!(Meld::Sequence.is_made_by(set("AH TH KH QH JH"), HEARTS));
        assert!(
            !Meld::Sequence.is_made_by(set("AS TS KS QS JS"), HEARTS),
            "only in trumps"
        );
        assert!(Meld::Bezique.is_made_by(set("QS JD"), HEARTS));
        assert!(!Meld::Bezique.is_made_by(set("QD JS"), HEARTS));
        assert!(Meld::DoubleBezique.is_made_by(set("QS QS JD JD"), HEARTS));
        assert!(!Meld::DoubleBezique.is_made_by(set("QS JD"), HEARTS));
        assert!(Meld::Four(Rank::KING).is_made_by(set("KS KH KD KC"), HEARTS));
        assert!(
            Meld::Four(Rank::KING).is_made_by(set("KS KS KH KD"), HEARTS),
            "a suit may repeat"
        );
        assert!(!Meld::Four(Rank::KING).is_made_by(set("KS KS KH"), HEARTS));
        assert!(
            !Meld::Four(Rank::TEN).is_made_by(set("TS TH TD TC"), HEARTS),
            "four tens score nothing"
        );
    }

    #[test]
    fn nothing_to_declare_in_a_hand_of_odds_and_ends() {
        let h = Holdings::new(set("7S 8H 9D TC AS KH QD JC"));
        assert!(h.candidates(HEARTS).is_empty());
        assert!(h.options(HEARTS).is_empty());
    }

    #[test]
    fn concealed_twins_are_one_choice() {
        let h = Holdings::new(set("KC QC QC 7S"));
        let marriages: Vec<Declared> = h
            .candidates(HEARTS)
            .into_iter()
            .filter(|d| d.meld == Meld::Marriage(Suit::CLUBS))
            .collect();
        assert_eq!(
            marriages,
            vec![Declared {
                meld: Meld::Marriage(Suit::CLUBS),
                cards: CardSet::of(&[card("KC", 0), card("QC", 0)])
            }]
        );
    }

    #[test]
    fn laid_cards_go_to_the_table_and_score() {
        let mut h = Holdings::new(set("KC QC 7S"));
        let points = declare(&mut h, Meld::Marriage(Suit::CLUBS));
        assert_eq!(points, 20);
        assert_eq!(h.shown, set("KC QC"));
        assert_eq!(h.concealed, set("7S"));
        assert_eq!(h.cards(), set("KC QC 7S"));
        assert!(h.carried().is_empty());
    }

    // ---- the new-card rule (rule 4) --------------------------------------

    #[test]
    fn a_combination_wholly_on_the_table_cannot_be_declared() {
        // Four kings and four queens shown without the marriages announced:
        // the marriages are lost, since no new card could ever join them.
        let mut h = Holdings::new(set("KS KH KD KC QS QH QD QC"));
        declare(&mut h, Meld::Four(Rank::KING));
        let queens = Declared {
            meld: Meld::Four(Rank::QUEEN),
            cards: set("QS QH QD QC"),
        };
        h.apply(
            HEARTS,
            &Announcement {
                melds: vec![queens],
                score: queens,
            },
        )
        .unwrap();
        assert!(
            h.candidates(HEARTS).is_empty(),
            "{:?}",
            h.candidates(HEARTS)
        );
        let marriage = Declared {
            meld: Meld::Marriage(Suit::CLUBS),
            cards: set("KC QC"),
        };
        let refused = h.check(
            HEARTS,
            &Announcement {
                melds: vec![marriage],
                score: marriage,
            },
        );
        assert!(refused.unwrap_err().contains("from your hand"));
    }

    // ---- Foster's worked examples (RULES.md, Worked examples) ------------

    /// Example 1. Hearts trumps, holding double bezique, four queens and the
    /// royal marriage: all three may be announced but only one scores, and
    /// declaring the double first forfeits the single bezique.
    #[test]
    fn foster_the_600_that_cannot_be_had() {
        let hand = Holdings::new(set("QS QS JD JD QH QD KH 7C"));
        let mut all_at_once = hand.clone();
        let double = the(&all_at_once, Meld::DoubleBezique);
        let queens = the(&all_at_once, Meld::Four(Rank::QUEEN));
        let royal = the(&all_at_once, Meld::Marriage(HEARTS));
        assert_eq!(announce(&mut all_at_once, &[double, queens, royal]), 500);
        assert_eq!(
            all_at_once.carried().len(),
            2,
            "the queens and the marriage are carried"
        );
        assert!(
            !can(&all_at_once, Meld::Bezique),
            "the double first forfeits the single"
        );

        let mut wisely = hand.clone();
        let single = wisely
            .candidates(HEARTS)
            .into_iter()
            .find(|d| d.meld == Meld::Bezique)
            .unwrap();
        assert_eq!(announce(&mut wisely, &[single]), 40);
        assert_eq!(declare(&mut wisely, Meld::DoubleBezique), 500);
    }

    /// Example 2. Four kings shown, the king of spades among them; showing
    /// bezique completes a spade marriage: "Forty for bezique, and twenty to
    /// score."
    #[test]
    fn foster_forty_and_twenty_to_score() {
        let mut h = Holdings::new(set("KS KH KD KC 7C 8C"));
        declare(&mut h, Meld::Four(Rank::KING));
        h.receive(card("QS", 0));
        h.receive(card("JD", 0));
        let offered = h.options(HEARTS);
        let forty = offered
            .iter()
            .find(|a| a.score.meld == Meld::Bezique)
            .expect("bezique is offered");
        assert_eq!(
            meld_kinds(forty),
            vec![Meld::Bezique, Meld::Marriage(Suit::SPADES)]
        );
        let mut after = h.clone();
        assert_eq!(after.apply(HEARTS, forty).unwrap(), 40);
        assert_eq!(
            after.carried().iter().map(|d| d.meld).collect::<Vec<_>>(),
            vec![Meld::Marriage(Suit::SPADES)]
        );
        assert_eq!(
            score_carried(&mut after.clone(), Meld::Marriage(Suit::SPADES)),
            20
        );

        // Lost if the king is played before a trick is won to score it.
        after.play(card("KS", 0));
        assert!(after.carried().is_empty());
    }

    /// Example 3. With twenty to score, the royal marriage and then the
    /// sequence are declared, the twenty still carried throughout.
    #[test]
    fn foster_the_carried_score_waits() {
        let mut h = Holdings::new(set("KS KH KD KC 7C 8C"));
        declare(&mut h, Meld::Four(Rank::KING));
        h.receive(card("QS", 0));
        h.receive(card("JD", 0));
        let forty = h
            .options(HEARTS)
            .into_iter()
            .find(|a| a.score.meld == Meld::Bezique)
            .unwrap();
        h.apply(HEARTS, &forty).unwrap();
        h.receive(card("QH", 0));
        assert_eq!(declare(&mut h, Meld::Marriage(HEARTS)), 40);
        h.receive(card("AH", 0));
        h.receive(card("TH", 0));
        h.receive(card("JH", 0));
        assert_eq!(declare(&mut h, Meld::Sequence), 250);
        assert_eq!(
            h.carried().iter().map(|d| d.meld).collect::<Vec<_>>(),
            vec![Meld::Marriage(Suit::SPADES)]
        );
    }

    /// Example 4. Eighty kings, then sixty queens, with the marriages the
    /// queens make announced to score.
    #[test]
    fn foster_kings_then_queens() {
        let mut h = Holdings::new(set("KS KD KC KC QS QD QC 7H"));
        declare(&mut h, Meld::Four(Rank::KING));
        h.receive(card("QH", 0));
        let sixty = h
            .options(HEARTS)
            .into_iter()
            .find(|a| a.score.meld == Meld::Four(Rank::QUEEN))
            .expect("four queens is offered");
        let mut kinds = meld_kinds(&sixty);
        let mut expected = vec![
            Meld::Four(Rank::QUEEN),
            Meld::Marriage(Suit::CLUBS),
            Meld::Marriage(Suit::DIAMONDS),
            Meld::Marriage(Suit::SPADES),
        ];
        kinds.sort_by_key(|m| format!("{m:?}"));
        expected.sort_by_key(|m| format!("{m:?}"));
        assert_eq!(kinds, expected);
        assert_eq!(h.apply(HEARTS, &sixty).unwrap(), 60);
        assert_eq!(h.carried().len(), 3);
    }

    /// Example 5. Four kings announced, two played, both kings of spades left;
    /// double bezique scores 500 and makes two spade marriages to score.
    #[test]
    fn foster_double_bezique_with_marriages() {
        let mut h = Holdings::new(set("KS KS KH KD 7C 8C 9C 7D"));
        declare(&mut h, Meld::Four(Rank::KING));
        h.play(card("KH", 0));
        h.play(card("KD", 0));
        for code in ["QS", "QS", "JD", "JD"] {
            let copy = h.cards().count_of(CardType::parse(code).unwrap()) as u8;
            h.receive(card(code, copy));
        }
        let five_hundred = h
            .options(HEARTS)
            .into_iter()
            .find(|a| a.score.meld == Meld::DoubleBezique)
            .expect("double bezique is offered");
        let marriages = five_hundred
            .melds
            .iter()
            .filter(|d| d.meld == Meld::Marriage(Suit::SPADES))
            .count();
        assert_eq!(marriages, 2, "{five_hundred:?}");
        assert_eq!(h.apply(HEARTS, &five_hundred).unwrap(), 500);
        assert_eq!(h.carried().len(), 2);
    }

    /// Example 6. After the sequence, no royal marriage, not even with a new
    /// queen beside its king.
    #[test]
    fn foster_no_inferior_declaration() {
        let mut h = Holdings::new(set("AH TH KH QH JH 7C 8C 9C"));
        declare(&mut h, Meld::Sequence);
        assert!(!can(&h, Meld::Marriage(HEARTS)));
        h.receive(card("QH", 1));
        assert!(
            !can(&h, Meld::Marriage(HEARTS)),
            "the king is spent in the sequence"
        );
        h.receive(card("KH", 1));
        assert!(
            can(&h, Meld::Marriage(HEARTS)),
            "a fresh king and queen are a fresh marriage"
        );
    }

    /// Example 7. Repetitions in the same class are barred.
    #[test]
    fn foster_no_repetition() {
        let mut kings = Holdings::new(set("KS KH KD KC 7C"));
        declare(&mut kings, Meld::Four(Rank::KING));
        kings.play(card("KD", 0));
        kings.receive(card("KD", 1));
        assert!(!can(&kings, Meld::Four(Rank::KING)));

        let mut marriage = Holdings::new(set("KS QS 7C"));
        declare(&mut marriage, Meld::Marriage(Suit::SPADES));
        marriage.play(card("KS", 0));
        marriage.receive(card("KS", 1));
        assert!(!can(&marriage, Meld::Marriage(Suit::SPADES)));

        let mut bezique = Holdings::new(set("QS JD 7C"));
        declare(&mut bezique, Meld::Bezique);
        bezique.play(card("JD", 0));
        bezique.receive(card("JD", 1));
        assert!(!can(&bezique, Meld::Bezique));
    }

    /// Example 8. A second single bezique scores 40, but the double is then out
    /// of reach.
    #[test]
    fn foster_the_second_single_spoils_the_double() {
        let mut h = Holdings::new(set("QS JD 7C 8C"));
        declare(&mut h, Meld::Bezique);
        h.receive(card("QS", 1));
        h.receive(card("JD", 1));
        assert!(can(&h, Meld::DoubleBezique));
        let second = h
            .candidates(HEARTS)
            .into_iter()
            .find(|d| d.meld == Meld::Bezique)
            .expect("the second single is legal");
        assert_eq!(second.cards, CardSet::of(&[card("QS", 1), card("JD", 1)]));
        assert_eq!(announce(&mut h, &[second]), 40);
        assert!(!can(&h, Meld::DoubleBezique));
    }

    // ---- Cady's worked examples ------------------------------------------

    /// Example 9. "Sixty and forty, and twenty, and twenty to score." Then,
    /// "Score forty, and twenty, and twenty to score."
    #[test]
    fn cady_three_marriages_to_score() {
        let mut h = Holdings::new(set("KS KH KD KC 7S 8S 9S 7D"));
        declare(&mut h, Meld::Four(Rank::KING));
        h.play(card("KS", 0));
        for code in ["QS", "QH", "QD", "QC"] {
            h.receive(card(code, 0));
        }
        let sixty = h
            .options(HEARTS)
            .into_iter()
            .find(|a| a.score.meld == Meld::Four(Rank::QUEEN))
            .unwrap();
        assert_eq!(
            sixty.melds.len(),
            4,
            "sixty, forty, twenty, twenty: {sixty:?}"
        );
        assert_eq!(h.apply(HEARTS, &sixty).unwrap(), 60);
        assert_eq!(score_carried(&mut h, Meld::Marriage(HEARTS)), 40);
        assert_eq!(h.carried().len(), 2);
    }

    /// Example 10. A carried score is lost when its cards are raised into a
    /// higher combination of the same class.
    #[test]
    fn cady_carried_scores_are_lost_upward() {
        // A royal marriage carried, then the sequence laid over it.
        let mut h = Holdings::new(set("KS KH KD KC 7S 8S"));
        declare(&mut h, Meld::Four(Rank::KING));
        h.receive(card("QH", 0));
        h.receive(card("QS", 0));
        h.receive(card("QD", 0));
        h.receive(card("QC", 0));
        let sixty = h
            .options(HEARTS)
            .into_iter()
            .find(|a| a.score.meld == Meld::Four(Rank::QUEEN))
            .unwrap();
        h.apply(HEARTS, &sixty).unwrap();
        assert!(h.carried().iter().any(|d| d.meld == Meld::Marriage(HEARTS)));
        for code in ["AH", "TH", "JH"] {
            h.receive(card(code, 0));
        }
        let sequence = the(&h, Meld::Sequence);
        let royal = *h
            .carried()
            .iter()
            .find(|d| d.meld == Meld::Marriage(HEARTS))
            .unwrap();
        let both = Announcement {
            melds: vec![sequence],
            score: royal,
        };
        assert!(
            h.check(HEARTS, &both).is_err(),
            "the marriage cannot be scored as the sequence covers it"
        );
        announce(&mut h, &[sequence]);
        assert!(!h.carried().iter().any(|d| d.meld == Meld::Marriage(HEARTS)));
    }

    #[test]
    fn cady_a_carried_single_is_lost_to_the_double() {
        // Four queens laid with a jack of diamonds already shown (from four
        // jacks): the single bezique rides free and is carried; adding the
        // other pair for the double then loses it.
        let mut h = Holdings::new(set("JS JH JD JC 7C 8C"));
        declare(&mut h, Meld::Four(Rank::JACK));
        for code in ["QS", "QH", "QD", "QC"] {
            h.receive(card(code, 0));
        }
        let sixty = h
            .options(HEARTS)
            .into_iter()
            .find(|a| a.score.meld == Meld::Four(Rank::QUEEN))
            .unwrap();
        assert!(
            sixty.melds.iter().any(|d| d.meld == Meld::Bezique),
            "{sixty:?}"
        );
        h.apply(HEARTS, &sixty).unwrap();
        assert!(h.carried().iter().any(|d| d.meld == Meld::Bezique));
        h.receive(card("QS", 1));
        h.receive(card("JD", 1));
        assert_eq!(declare(&mut h, Meld::DoubleBezique), 500);
        assert!(!h.carried().iter().any(|d| d.meld == Meld::Bezique));
    }

    // ---- choosing free riders --------------------------------------------

    /// Of two equally good free riders, the one that leaves more cards free:
    /// a marriage on the queen of spades that already guards a carried
    /// bezique, not on the other, which can then be played.
    #[test]
    fn a_free_rider_doubles_up_on_a_card_already_guarding_a_score() {
        let mut h = Holdings::new(set("QS QS QH QD JD 7C 8C 9C"));
        let queens = Declared {
            meld: Meld::Four(Rank::QUEEN),
            cards: set("QS QS QH QD"),
        };
        let bezique = Declared {
            meld: Meld::Bezique,
            cards: CardSet::of(&[card("QS", 1), card("JD", 0)]),
        };
        h.apply(
            HEARTS,
            &Announcement {
                melds: vec![queens, bezique],
                score: queens,
            },
        )
        .unwrap();
        assert_eq!(h.carried(), vec![bezique]);
        for code in ["KS", "KH", "KD", "KC"] {
            h.receive(card(code, 0));
        }
        let kings = h
            .options(HEARTS)
            .into_iter()
            .find(|a| a.score.meld == Meld::Four(Rank::KING))
            .unwrap();
        let spades = kings
            .melds
            .iter()
            .find(|d| d.meld == Meld::Marriage(Suit::SPADES))
            .expect("the spade marriage rides");
        assert!(
            spades.cards.contains(card("QS", 1)),
            "{spades:?}: the queen already guarding the bezique"
        );
    }

    /// With more than sixteen possible riders the best set is still found,
    /// not a truncated one. The position holds eleven cards, which play
    /// cannot produce (with eight, more than sixteen riders needs freak
    /// tables, six unmarried queens and the like): it tests the greedy
    /// branch, which is optimal on value in every reachable case found by
    /// the full review of 2026-10-02 but ignores the tie-break of tying the
    /// fewest cards to scores still to come.
    #[test]
    fn many_free_riders_are_not_cut_short() {
        let h = Holdings {
            concealed: CardSet::of(&[card("QS", 0), card("JD", 0)]),
            shown: CardSet::of(&[
                card("QS", 1),
                card("QH", 0),
                card("QH", 1),
                card("QD", 0),
                card("QD", 1),
                card("QC", 0),
                card("JS", 0),
                card("JH", 0),
                card("JC", 0),
            ]),
            announced: vec![],
        };
        let forty = h
            .options(HEARTS)
            .into_iter()
            .find(|a| a.score.meld == Meld::Bezique && a.melds[0].meld == Meld::Bezique)
            .unwrap();
        let kinds: Vec<Meld> = forty.melds.iter().map(|d| d.meld).collect();
        assert!(
            kinds.contains(&Meld::Four(Rank::QUEEN)) && kinds.contains(&Meld::Four(Rank::JACK)),
            "{kinds:?}"
        );
    }

    // ---- what check() accepts and options() does not offer ---------------
    //
    // Characterisation, not a wanted behaviour: both are open questions for
    // the maintainer (docs/reviews/rules-engine.md, findings 1 and 2).

    /// Finding 1. With a royal marriage carried, a fresh (second) bezique may
    /// be announced while the marriage is the combination scored: `check`
    /// accepts it. `options` offers the bezique scored, and the marriage
    /// scored alone, but never the pair.
    #[test]
    fn check_allows_scoring_a_carried_combination_while_announcing_a_new_one_but_options_does_not_offer_it(
    ) {
        let mut h = Holdings::new(set("KH QH QS JD 7S 8S 9S 7D"));
        let marriage = the(&h, Meld::Marriage(HEARTS));
        let first = the(&h, Meld::Bezique);
        announce(&mut h, &[first, marriage]);
        assert_eq!(h.carried(), vec![marriage]);
        h.receive(card("QS", 1));
        h.receive(card("JD", 1));
        let second = the(&h, Meld::Bezique);
        assert_eq!(second.cards, CardSet::of(&[card("QS", 1), card("JD", 1)]));
        let wanted = Announcement {
            melds: vec![second],
            score: marriage,
        };
        assert_eq!(h.check(HEARTS, &wanted), Ok(()));
        let options = h.options(HEARTS);
        assert!(
            options.iter().any(|a| a.score == second),
            "the new bezique, scored, is offered"
        );
        assert!(
            options
                .iter()
                .any(|a| a.melds.is_empty() && a.score == marriage),
            "the carried marriage, scored alone, is offered"
        );
        assert!(
            !options
                .iter()
                .any(|a| !a.melds.is_empty() && a.score == marriage),
            "{options:?}: announcing the bezique and scoring the marriage is not offered"
        );
    }

    /// Finding 2. A bezique laid with the spade king already shown: the
    /// queen of spades also marries the king, a free rider. `check` accepts
    /// the bezique alone; `options` never offers it without the marriage.
    #[test]
    fn check_allows_declining_a_free_rider_but_options_always_announces_it() {
        let h = Holdings {
            concealed: set("QS JD 7C 8C"),
            shown: set("KS"),
            announced: vec![],
        };
        let bezique = the(&h, Meld::Bezique);
        let alone = Announcement {
            melds: vec![bezique],
            score: bezique,
        };
        assert_eq!(h.check(HEARTS, &alone), Ok(()));
        let options = h.options(HEARTS);
        assert!(
            options.iter().any(|a| a.melds.len() == 2),
            "{options:?}: the bezique with its marriage"
        );
        assert!(
            !options.iter().any(|a| a.melds == vec![bezique]),
            "{options:?}: the bezique alone is not offered"
        );
    }

    // ---- checking announcements ------------------------------------------

    #[test]
    fn refusals() {
        let h = Holdings::new(set("KC QC QS JD 7H"));
        let marriage = the(&h, Meld::Marriage(Suit::CLUBS));
        let bezique = the(&h, Meld::Bezique);
        let ok = Announcement {
            melds: vec![marriage, bezique],
            score: bezique,
        };
        assert_eq!(h.check(HEARTS, &ok), Ok(()));

        let not_announced = Announcement {
            melds: vec![marriage],
            score: bezique,
        };
        assert!(h.check(HEARTS, &not_announced).is_err());

        let wrong_cards = Declared {
            meld: Meld::Marriage(Suit::CLUBS),
            cards: set("KC QS"),
        };
        assert!(h
            .check(
                HEARTS,
                &Announcement {
                    melds: vec![wrong_cards],
                    score: wrong_cards
                }
            )
            .is_err());

        let not_held = Declared {
            meld: Meld::Marriage(Suit::SPADES),
            cards: set("KS QS"),
        };
        assert!(h
            .check(
                HEARTS,
                &Announcement {
                    melds: vec![not_held],
                    score: not_held
                }
            )
            .is_err());

        let twice = Announcement {
            melds: vec![marriage, marriage],
            score: marriage,
        };
        assert!(
            h.check(HEARTS, &twice).is_err(),
            "no card twice in one class"
        );

        let nothing_carried = Announcement {
            melds: vec![],
            score: marriage,
        };
        assert!(h.check(HEARTS, &nothing_carried).is_err());
    }

    #[test]
    fn combinations_that_cannot_exist_are_refused_not_panicked_on() {
        let h = Holdings::new(set("KC QC TS TH TD TC"));
        for meld in [
            Meld::Marriage(Suit(4)),
            Meld::Four(Rank(8)),
            Meld::Four(Rank::TEN),
        ] {
            let d = Declared {
                meld,
                cards: set("KC QC"),
            };
            assert!(
                h.check(
                    HEARTS,
                    &Announcement {
                        melds: vec![d],
                        score: d
                    }
                )
                .is_err(),
                "{meld:?}"
            );
        }
        assert!(!Meld::Four(Rank::TEN).is_valid());
        assert!(Meld::Four(Rank::JACK).is_valid());
    }

    #[test]
    fn the_same_card_may_serve_in_different_classes_at_once() {
        // Q♠ in a spade marriage and a bezique in one announcement.
        let h = Holdings::new(set("KS QS JD 7H"));
        let a = Announcement {
            melds: vec![
                the(&h, Meld::Bezique),
                the(&h, Meld::Marriage(Suit::SPADES)),
            ],
            score: the(&h, Meld::Bezique),
        };
        assert_eq!(h.check(HEARTS, &a), Ok(()));
        let mut after = h.clone();
        after.apply(HEARTS, &a).unwrap();
        assert_eq!(after.shown, set("KS QS JD"));
    }

    // ---- references ------------------------------------------------------

    /// Concealed twins are interchangeable: rewrite a set so that its concealed
    /// cards of each type are the lowest concealed copies.
    fn canonical(h: &Holdings, cards: CardSet) -> CardSet {
        let mut out = cards.intersect(h.shown);
        for card_type in CardType::all() {
            let wanted = cards.intersect(h.concealed).count_of(card_type);
            for card in h
                .concealed
                .intersect(card_type.twins())
                .cards()
                .take(wanted as usize)
            {
                out = out.with(card);
            }
        }
        out
    }

    /// Every combination the cards in `from` could form, whatever the rules
    /// say about declaring it.
    fn well_formed(from: CardSet, trump: Suit) -> Vec<Declared> {
        let mine: Vec<Card> = from.cards().take(12).collect();
        let mut found = Vec::new();
        for mask in 0u32..(1 << mine.len()) {
            if !matches!(mask.count_ones(), 2 | 4 | 5) {
                continue;
            }
            let picked: Vec<Card> = (0..mine.len())
                .filter(|i| mask >> i & 1 == 1)
                .map(|i| mine[i])
                .collect();
            let cards = CardSet::of(&picked);
            for meld in Meld::all() {
                if meld.is_made_by(cards, trump) {
                    found.push(Declared { meld, cards });
                }
            }
        }
        found
    }

    /// Foster's rules read literally, over every subset of the player's cards:
    /// a combination of the right cards, at least one of them from the hand,
    /// none of them already in a combination of the same class that is higher
    /// (rule 2) or of equal value (rule 3).
    fn reference_candidates(h: &Holdings, trump: Suit) -> Vec<Declared> {
        let mine: Vec<Card> = h.cards().cards().collect();
        let mut found = Vec::new();
        for mask in 0u32..(1 << mine.len()) {
            if !matches!(mask.count_ones(), 2 | 4 | 5) {
                continue;
            }
            let cards = CardSet::of(
                &mine
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| mask >> i & 1 == 1)
                    .map(|(_, &c)| c)
                    .collect::<Vec<_>>(),
            );
            for meld in Meld::all() {
                if !meld.is_made_by(cards, trump) || cards.intersect(h.concealed).is_empty() {
                    continue;
                }
                let spent = h.announced.iter().any(|past| {
                    past.declared.meld.class() == meld.class()
                        && past.declared.meld.level() >= meld.level()
                        && !past.declared.cards.intersect(cards).is_empty()
                });
                let found_it = Declared {
                    meld,
                    cards: canonical(h, cards),
                };
                if !spent && !found.contains(&found_it) {
                    found.push(found_it);
                }
            }
        }
        found
    }

    /// `RULES.md` §4 checked clause by clause, for comparison with `check`.
    /// `fresh` is `reference_candidates(h, trump)`, computed once per position.
    fn reference_check(h: &Holdings, trump: Suit, fresh: &[Declared], a: &Announcement) -> bool {
        let laid_anything_new = a
            .melds
            .iter()
            .all(|d| !d.cards.intersect(h.concealed).is_empty());
        let made = a
            .melds
            .iter()
            .all(|d| d.meld.is_made_by(d.cards, trump) && d.cards.minus(h.cards()).is_empty());
        let allowed = a.melds.iter().all(|d| {
            fresh.contains(&Declared {
                meld: d.meld,
                cards: canonical(h, d.cards),
            })
        });
        let mut no_clash = true;
        for (i, x) in a.melds.iter().enumerate() {
            for y in &a.melds[i + 1..] {
                if x.meld.class() == y.meld.class() && !x.cards.intersect(y.cards).is_empty() {
                    no_clash = false;
                }
            }
        }
        let scored_ok = a.melds.contains(&a.score)
            || (h
                .announced
                .iter()
                .any(|p| !p.scored && p.declared == a.score)
                && a.score.cards.minus(h.shown).is_empty()
                && !h
                    .announced
                    .iter()
                    .chain(
                        a.melds
                            .iter()
                            .map(|&d| Announced {
                                declared: d,
                                scored: false,
                            })
                            .collect::<Vec<_>>()
                            .iter(),
                    )
                    .any(|p| {
                        p.declared.meld.class() == a.score.meld.class()
                            && p.declared.meld.level() > a.score.meld.level()
                            && !p.declared.cards.intersect(a.score.cards).is_empty()
                    }));
        laid_anything_new && made && allowed && no_clash && scored_ok
    }

    /// A random hand part-way through the stock: announcements made, cards
    /// played, cards drawn.
    fn random_holdings(rng: &mut Rng, trump: Suit) -> Holdings {
        let mut pack: Vec<Card> = CardSet::PACK.cards().collect();
        // Courts and bezique cards, so that combinations are common.
        pack.retain(|c| c.rank() >= Rank::JACK || rng.below(4) == 0);
        rng.shuffle(&mut pack);
        let mut pack = pack.into_iter();
        let mut h = Holdings::new(CardSet::of(&pack.by_ref().take(8).collect::<Vec<_>>()));
        for _ in 0..rng.below(10) {
            match rng.below(3) {
                0 => {
                    let options = h.options(trump);
                    if !options.is_empty() {
                        let pick = &options[rng.below(options.len() as u64) as usize];
                        h.apply(trump, pick).unwrap();
                    }
                }
                1 => {
                    let mine: Vec<Card> = h.cards().cards().collect();
                    h.play(mine[rng.below(mine.len() as u64) as usize]);
                    if let Some(card) = pack.next() {
                        h.receive(card);
                    }
                }
                _ => {}
            }
        }
        h
    }

    #[test]
    fn candidates_agree_with_fosters_rules_read_literally() {
        let mut rng = Rng::seeded(341);
        for _ in 0..3000 {
            let trump = Suit(rng.below(4) as u8);
            let h = random_holdings(&mut rng, trump);
            let mut fast = h.candidates(trump);
            let mut slow = reference_candidates(&h, trump);
            let key = |d: &Declared| (format!("{:?}", d.meld), d.cards.0);
            fast.sort_by_key(key);
            slow.sort_by_key(key);
            assert_eq!(fast, slow, "{h:?}, trumps {trump:?}");
        }
    }

    #[test]
    fn check_agrees_with_the_rules_clause_by_clause() {
        let mut rng = Rng::seeded(342);
        for _ in 0..3000 {
            let trump = Suit(rng.below(4) as u8);
            let h = random_holdings(&mut rng, trump);
            let fresh = reference_candidates(&h, trump);
            // Proposals: some sensible, some not.
            let mut pool = h.candidates(trump);
            pool.extend(h.carried());
            // Well formed but not necessarily allowed: wholly on the table, already
            // declared, and so on.
            pool.extend(well_formed(h.cards(), trump));
            let mine: Vec<Card> = h.cards().cards().collect();
            for _ in 0..3 {
                let mut cards = CardSet::EMPTY;
                for _ in 0..(2 + rng.below(4)) {
                    if !mine.is_empty() {
                        cards = cards.with(mine[rng.below(mine.len() as u64) as usize]);
                    }
                }
                pool.push(Declared {
                    meld: Meld::all()[rng.below(11) as usize],
                    cards,
                });
            }
            for _ in 0..20 {
                let n = rng.below(4) as usize;
                let melds: Vec<Declared> = (0..n)
                    .map(|_| pool[rng.below(pool.len() as u64) as usize])
                    .collect();
                let score = if !melds.is_empty() && rng.below(3) > 0 {
                    melds[rng.below(n as u64) as usize]
                } else {
                    pool[rng.below(pool.len() as u64) as usize]
                };
                let a = Announcement { melds, score };
                assert_eq!(
                    h.check(trump, &a).is_ok(),
                    reference_check(&h, trump, &fresh, &a),
                    "{a:?} on {h:?}, trumps {trump:?}"
                );
            }
        }
    }

    #[test]
    fn every_option_is_legal_and_scores_what_it_says() {
        let mut rng = Rng::seeded(343);
        for _ in 0..2000 {
            let trump = Suit(rng.below(4) as u8);
            let h = random_holdings(&mut rng, trump);
            for option in h.options(trump) {
                assert_eq!(h.check(trump, &option), Ok(()), "{option:?}");
                let mut after = h.clone();
                assert_eq!(
                    after.apply(trump, &option),
                    Ok(option.score.meld.points(trump))
                );
                assert_eq!(
                    after.cards(),
                    h.cards(),
                    "declaring moves cards, never loses them"
                );
            }
        }
    }

    /// K♠ and Q♠ each on the table unmarried (one from four kings, one from a
    /// bezique), their twins drawn: the two cross-marriages, twenty now and
    /// twenty to score, are offered, and are worth more than marrying the new
    /// pair, which would leave the old cards unable to marry (full review of
    /// 2026-10-02).
    #[test]
    fn the_cross_marriages_are_offered_when_worth_more() {
        let card = |code: &str, copy: u8| Card::new(CardType::parse(code).unwrap(), copy);
        let mut h = Holdings::new(CardSet::of(&[
            card("KS", 1),
            card("KH", 0),
            card("KD", 0),
            card("KC", 0),
            card("QS", 1),
            card("JD", 0),
            card("7C", 0),
            card("8C", 0),
        ]));
        let kings = Declared {
            meld: Meld::Four(Rank::KING),
            cards: CardSet::of(&[card("KS", 1), card("KH", 0), card("KD", 0), card("KC", 0)]),
        };
        h.apply(
            HEARTS,
            &Announcement {
                melds: vec![kings],
                score: kings,
            },
        )
        .unwrap();
        let bezique = Declared {
            meld: Meld::Bezique,
            cards: CardSet::of(&[card("QS", 1), card("JD", 0)]),
        };
        h.apply(
            HEARTS,
            &Announcement {
                melds: vec![bezique],
                score: bezique,
            },
        )
        .unwrap();
        h.play(card("7C", 0));
        h.play(card("8C", 0));
        h.receive(card("KS", 0));
        h.receive(card("QS", 0));
        let options = h.options(HEARTS);
        for o in &options {
            assert_eq!(h.check(HEARTS, o), Ok(()), "{o:?}");
        }
        let marriages = |o: &Announcement| {
            o.melds
                .iter()
                .filter(|d| d.meld == Meld::Marriage(Suit::SPADES))
                .count()
        };
        assert!(options.iter().any(|o| marriages(o) == 2), "{options:?}");
    }
}
