//! The bezique pack: two piquet packs shuffled together, 64 cards.
//!
//! Every card exists twice. A [`CardType`] is a suit and a rank, one of 32; a
//! [`Card`] is one physical card, one of 64, numbered `type * 2 + copy`. The
//! twins are interleaved, so a suit occupies sixteen consecutive bits of a
//! [`CardSet`], the two copies of a card are adjacent, and a higher bit is a
//! higher card. `docs/DESIGN.md` §5 gives the reasoning.

use std::fmt;

const RANK_CHARS: [u8; 8] = *b"789JQKTA";
const SUIT_LETTERS: [u8; 4] = *b"SHDC";
const SUIT_SYMBOLS: [char; 4] = ['♠', '♥', '♦', '♣'];

/// A rank, as its place in bezique order from the seven (0) to the ace (7).
///
/// The ten ranks second, above the king, in cutting and in play.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Rank(pub u8);

impl Rank {
    pub const SEVEN: Rank = Rank(0);
    pub const EIGHT: Rank = Rank(1);
    pub const NINE: Rank = Rank(2);
    pub const JACK: Rank = Rank(3);
    pub const QUEEN: Rank = Rank(4);
    pub const KING: Rank = Rank(5);
    pub const TEN: Rank = Rank(6);
    pub const ACE: Rank = Rank(7);

    /// Low to high.
    pub const ALL: [Rank; 8] = [
        Rank::SEVEN,
        Rank::EIGHT,
        Rank::NINE,
        Rank::JACK,
        Rank::QUEEN,
        Rank::KING,
        Rank::TEN,
        Rank::ACE,
    ];

    /// Whether this names one of the eight ranks.
    pub fn is_valid(self) -> bool {
        self.0 < 8
    }

    pub fn letter(self) -> char {
        RANK_CHARS[self.0 as usize] as char
    }

    pub fn name(self) -> &'static str {
        [
            "seven", "eight", "nine", "jack", "queen", "king", "ten", "ace",
        ][self.0 as usize]
    }

    /// Aces and tens: the brisques, ten points to whoever takes them in a trick.
    pub fn is_brisque(self) -> bool {
        self >= Rank::TEN
    }
}

/// A suit. Suits have no order in bezique; the index fixes each card's slot,
/// in the order a hand is shown: spades, hearts, diamonds, clubs.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Suit(pub u8);

impl Suit {
    pub const SPADES: Suit = Suit(0);
    pub const HEARTS: Suit = Suit(1);
    pub const DIAMONDS: Suit = Suit(2);
    pub const CLUBS: Suit = Suit(3);
    pub const ALL: [Suit; 4] = [Suit::SPADES, Suit::HEARTS, Suit::DIAMONDS, Suit::CLUBS];

    /// Whether this names one of the four suits.
    pub fn is_valid(self) -> bool {
        self.0 < 4
    }

    pub fn letter(self) -> char {
        SUIT_LETTERS[self.0 as usize] as char
    }

    pub fn symbol(self) -> char {
        SUIT_SYMBOLS[self.0 as usize]
    }

    pub fn name(self) -> &'static str {
        ["spades", "hearts", "diamonds", "clubs"][self.0 as usize]
    }
}

/// A suit and a rank: what is printed on a card. There are 32, and the pack
/// holds two cards of each.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct CardType(pub u8);

impl CardType {
    pub fn new(rank: Rank, suit: Suit) -> CardType {
        CardType(suit.0 * 8 + rank.0)
    }

    pub fn rank(self) -> Rank {
        Rank(self.0 % 8)
    }

    pub fn suit(self) -> Suit {
        Suit(self.0 / 8)
    }

    /// Both copies of this card.
    pub fn twins(self) -> CardSet {
        CardSet(0b11 << (2 * self.0))
    }

    /// Two characters, rank then suit: `AS`, `TD`, `7C`.
    pub fn code(self) -> String {
        format!("{}{}", self.rank().letter(), self.suit().letter())
    }

    /// Reads a code. The ten may be written `T` or `10`, and letters in either
    /// case.
    pub fn parse(text: &str) -> Result<CardType, String> {
        let upper = text.trim().to_uppercase();
        let code = upper
            .strip_prefix("10")
            .map_or(upper.clone(), |suit| format!("T{suit}"));
        let chars: Vec<char> = code.chars().collect();
        if chars.len() != 2 {
            return Err(format!(
                "{:?} is not a card: write rank then suit, as AS or 10D",
                text.trim()
            ));
        }
        let rank = RANK_CHARS
            .iter()
            .position(|&r| char::from(r) == chars[0])
            .ok_or_else(|| {
                format!(
                    "{:?} is not a rank in the bezique pack (7 8 9 10 J Q K A)",
                    chars[0]
                )
            })?;
        let suit = SUIT_LETTERS
            .iter()
            .position(|&s| char::from(s) == chars[1])
            .ok_or_else(|| format!("{:?} is not a suit (S H D C)", chars[1]))?;
        Ok(CardType::new(Rank(rank as u8), Suit(suit as u8)))
    }

    /// Every card type, from the seven of spades to the ace of clubs.
    pub fn all() -> impl Iterator<Item = CardType> {
        (0..32).map(CardType)
    }
}

impl fmt::Display for CardType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.code())
    }
}

/// One physical card, `0..64`: its type times two, plus which copy it is.
///
/// The copy number distinguishes the twins and nothing else: they are equal in
/// play, and differ only in what each has been declared in.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Card(pub u8);

impl Card {
    /// Whether this names one of the 64 cards. Ids come from outside (a
    /// client's command), so the engine checks before using one.
    pub fn is_valid(self) -> bool {
        self.0 < 64
    }

    pub fn new(card_type: CardType, copy: u8) -> Card {
        // Checked in release too: copy 2 would silently name the next type.
        assert!(copy < 2, "a two-pack game has copies 0 and 1");
        Card(card_type.0 * 2 + copy)
    }

    pub fn card_type(self) -> CardType {
        CardType(self.0 / 2)
    }

    pub fn copy(self) -> u8 {
        self.0 % 2
    }

    pub fn rank(self) -> Rank {
        self.card_type().rank()
    }

    pub fn suit(self) -> Suit {
        self.card_type().suit()
    }

    pub fn is_brisque(self) -> bool {
        self.rank().is_brisque()
    }

    /// The code of its type: both copies of the ace of spades are `AS`.
    pub fn code(self) -> String {
        self.card_type().code()
    }

    #[inline]
    fn bit(self) -> u64 {
        // Checked in release too, where the shift would wrap and Card(64)
        // would silently name the seven of spades.
        assert!(self.is_valid(), "card id {} names no card", self.0);
        1 << self.0
    }
}

impl fmt::Debug for Card {
    /// The code with the copy, `QS#1`, so that a failing test says which twin.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}#{}", self.code(), self.copy())
    }
}

/// A set of physical cards, one bit each.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CardSet(pub u64);

impl CardSet {
    pub const EMPTY: CardSet = CardSet(0);
    /// All 64 cards.
    pub const PACK: CardSet = CardSet(u64::MAX);

    /// Every card of one suit, sixteen of them.
    pub fn suit(suit: Suit) -> CardSet {
        CardSet(0xFFFF << (16 * suit.0))
    }

    pub fn len(self) -> u32 {
        self.0.count_ones()
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn contains(self, card: Card) -> bool {
        self.0 & card.bit() != 0
    }

    pub fn with(self, card: Card) -> CardSet {
        CardSet(self.0 | card.bit())
    }

    pub fn without(self, card: Card) -> CardSet {
        CardSet(self.0 & !card.bit())
    }

    pub fn union(self, other: CardSet) -> CardSet {
        CardSet(self.0 | other.0)
    }

    pub fn intersect(self, other: CardSet) -> CardSet {
        CardSet(self.0 & other.0)
    }

    pub fn minus(self, other: CardSet) -> CardSet {
        CardSet(self.0 & !other.0)
    }

    pub fn in_suit(self, suit: Suit) -> CardSet {
        self.intersect(CardSet::suit(suit))
    }

    /// How many copies of this type the set holds: 0, 1 or 2.
    pub fn count_of(self, card_type: CardType) -> u32 {
        self.intersect(card_type.twins()).len()
    }

    /// The highest-numbered card: within one suit, the highest in rank (of
    /// twins, the second copy). Across suits the order means nothing.
    pub fn highest(self) -> Option<Card> {
        (self.0 != 0).then(|| Card(63 - self.0.leading_zeros() as u8))
    }

    /// The lowest-numbered card, by the same order.
    pub fn lowest(self) -> Option<Card> {
        (self.0 != 0).then(|| Card(self.0.trailing_zeros() as u8))
    }

    /// The cards in ascending order of their numbers: suit by suit, and within
    /// a suit from the sevens up to the aces.
    pub fn cards(self) -> impl Iterator<Item = Card> {
        let mut bits = self.0;
        std::iter::from_fn(move || {
            if bits == 0 {
                return None;
            }
            let at = bits.trailing_zeros() as u8;
            bits &= bits - 1;
            Some(Card(at))
        })
    }

    pub fn of(cards: &[Card]) -> CardSet {
        cards
            .iter()
            .fold(CardSet::EMPTY, |set, &card| set.with(card))
    }

    /// Reads codes separated by spaces. Each repeated code takes the next copy,
    /// so `QS QS JD` is both queens of spades and the first jack of diamonds;
    /// a third copy is an error.
    pub fn parse(text: &str) -> Result<CardSet, String> {
        let mut set = CardSet::EMPTY;
        for code in text.split_whitespace() {
            let card_type = CardType::parse(code)?;
            let copy = set.count_of(card_type);
            if copy == 2 {
                return Err(format!("there are only two of {card_type} in the pack"));
            }
            set = set.with(Card::new(card_type, copy as u8));
        }
        Ok(set)
    }
}

impl fmt::Debug for CardSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.cards()).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card(code: &str, copy: u8) -> Card {
        Card::new(CardType::parse(code).unwrap(), copy)
    }

    #[test]
    fn the_pack_is_sixty_four_cards_two_of_every_type() {
        assert_eq!(CardSet::PACK.len(), 64);
        for card_type in CardType::all() {
            assert_eq!(CardSet::PACK.count_of(card_type), 2, "{card_type}");
        }
        assert_eq!(CardType::all().count(), 32);
    }

    #[test]
    fn ranks_run_ace_ten_king_queen_jack_nine_eight_seven() {
        let high_to_low = [
            Rank::ACE,
            Rank::TEN,
            Rank::KING,
            Rank::QUEEN,
            Rank::JACK,
            Rank::NINE,
            Rank::EIGHT,
            Rank::SEVEN,
        ];
        for pair in high_to_low.windows(2) {
            assert!(
                pair[0] > pair[1],
                "{:?} should outrank {:?}",
                pair[0],
                pair[1]
            );
        }
    }

    #[test]
    fn every_code_round_trips() {
        for card_type in CardType::all() {
            assert_eq!(CardType::parse(&card_type.code()), Ok(card_type));
        }
        assert_eq!(CardType::new(Rank::TEN, Suit::DIAMONDS).code(), "TD");
        assert_eq!(CardType::new(Rank::SEVEN, Suit::CLUBS).code(), "7C");
        assert_eq!(CardType::new(Rank::QUEEN, Suit::SPADES).code(), "QS");
    }

    #[test]
    fn the_ten_may_be_written_either_way_and_case_does_not_matter() {
        let ten = CardType::new(Rank::TEN, Suit::HEARTS);
        assert_eq!(CardType::parse("TH"), Ok(ten));
        assert_eq!(CardType::parse("10H"), Ok(ten));
        assert_eq!(CardType::parse("th"), Ok(ten));
        assert_eq!(
            CardType::parse(" jd "),
            Ok(CardType::new(Rank::JACK, Suit::DIAMONDS))
        );
    }

    #[test]
    fn a_bad_code_is_refused_with_a_reason() {
        for bad in ["", "A", "XS", "AX", "ASS", "1S", "6S"] {
            let refusal = CardType::parse(bad).unwrap_err();
            assert!(!refusal.is_empty(), "{bad:?}");
        }
    }

    #[test]
    fn a_refusal_names_the_character_typed() {
        let refusal = CardType::parse("AŠ").unwrap_err();
        assert!(refusal.contains('Š'), "{refusal}");
        assert!(Card(63).is_valid() && !Card(64).is_valid());
        assert!(Suit(3).is_valid() && !Suit(4).is_valid());
        assert!(Rank(7).is_valid() && !Rank(8).is_valid());
    }

    #[test]
    fn twins_share_a_type_and_sit_side_by_side() {
        for card_type in CardType::all() {
            let first = Card::new(card_type, 0);
            let second = Card::new(card_type, 1);
            assert_eq!(first.card_type(), card_type);
            assert_eq!(second.card_type(), card_type);
            assert_eq!(second.0, first.0 + 1);
            assert_eq!(first.code(), second.code());
            assert_eq!(card_type.twins(), CardSet::of(&[first, second]));
        }
    }

    #[test]
    fn a_card_knows_its_rank_and_suit() {
        let ten_of_diamonds = card("TD", 1);
        assert_eq!(ten_of_diamonds.rank(), Rank::TEN);
        assert_eq!(ten_of_diamonds.suit(), Suit::DIAMONDS);
        assert_eq!(ten_of_diamonds.copy(), 1);
        assert_eq!(format!("{ten_of_diamonds:?}"), "TD#1");
    }

    #[test]
    fn the_suits_partition_the_pack_sixteen_apiece() {
        let mut seen = CardSet::EMPTY;
        for suit in Suit::ALL {
            let cards = CardSet::suit(suit);
            assert_eq!(cards.len(), 16);
            assert!(cards.intersect(seen).is_empty());
            assert!(cards.cards().all(|c| c.suit() == suit));
            seen = seen.union(cards);
        }
        assert_eq!(seen, CardSet::PACK);
    }

    #[test]
    fn a_higher_card_has_a_higher_number_within_its_suit() {
        for suit in Suit::ALL {
            for pair in Rank::ALL.windows(2) {
                let lower = Card::new(CardType::new(pair[0], suit), 1);
                let higher = Card::new(CardType::new(pair[1], suit), 0);
                assert!(higher > lower, "{higher:?} over {lower:?}");
            }
        }
    }

    #[test]
    fn the_highest_heart_of_nine_king_and_ten_is_the_ten() {
        let hand = CardSet::parse("9H KH TH 8S AS").unwrap();
        let hearts = hand.in_suit(Suit::HEARTS);
        assert_eq!(
            hearts.highest().map(Card::card_type),
            CardType::parse("TH").ok()
        );
        assert_eq!(
            hearts.lowest().map(Card::card_type),
            CardType::parse("9H").ok()
        );
        assert_eq!(CardSet::EMPTY.highest(), None);
    }

    #[test]
    fn parsing_a_set_takes_the_next_copy_for_each_repeat() {
        let set = CardSet::parse("QS QS JD").unwrap();
        assert_eq!(
            set,
            CardSet::of(&[card("QS", 0), card("QS", 1), card("JD", 0)])
        );
        assert_eq!(set.count_of(CardType::parse("QS").unwrap()), 2);
        assert_eq!(set.count_of(CardType::parse("JD").unwrap()), 1);
        assert_eq!(set.count_of(CardType::parse("AS").unwrap()), 0);
        assert!(CardSet::parse("QS QS QS").is_err());
        assert!(CardSet::parse("QS XX").is_err());
        assert_eq!(CardSet::parse("").unwrap(), CardSet::EMPTY);
    }

    #[test]
    fn adding_and_removing_cards() {
        let ace = card("AS", 0);
        let set = CardSet::EMPTY.with(ace);
        assert!(set.contains(ace));
        assert!(!set.contains(card("AS", 1)));
        assert_eq!(set.len(), 1);
        assert!(set.without(ace).is_empty());
        let two = CardSet::parse("AS 7C").unwrap();
        assert_eq!(two.minus(set), CardSet::parse("7C").unwrap());
    }

    #[test]
    fn the_cards_come_out_in_ascending_order() {
        let set = CardSet::parse("AC 7S TS 7S").unwrap();
        let order: Vec<Card> = set.cards().collect();
        assert_eq!(
            order,
            vec![card("7S", 0), card("7S", 1), card("TS", 0), card("AC", 0)]
        );
    }

    #[test]
    fn the_brisques_are_the_aces_and_tens_sixteen_in_the_pack() {
        let brisques = CardSet::PACK.cards().filter(|c| c.is_brisque()).count();
        assert_eq!(brisques, 16);
        assert!(card("AS", 0).is_brisque());
        assert!(card("TC", 1).is_brisque());
        assert!(!card("KH", 0).is_brisque());
    }

    #[test]
    fn names_and_symbols() {
        assert_eq!(Rank::TEN.name(), "ten");
        assert_eq!(Rank::TEN.letter(), 'T');
        assert_eq!(Suit::DIAMONDS.symbol(), '♦');
        assert_eq!(Suit::CLUBS.letter(), 'C');
        assert_eq!(Suit::HEARTS.name(), "hearts");
    }
}
