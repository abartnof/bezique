//! Who wins a trick, and what the second player may play to one.
//!
//! While the stock lasts the second player may play anything (Foster, pp.
//! 339–340), so only the winner needs working out. In the last eight tricks
//! the second player "must now not only follow suit, but must win the trick if
//! he can, either with a superior card of the same suit, or with a trump"
//! (p. 343).

use crate::cards::{Card, CardSet, Suit};
use crate::deal::Seat;

/// Whether the card played second takes the trick: a higher card of the suit
/// led, or a trump to a plain-suit lead. An identical card does not: the
/// leader wins (p. 340).
pub fn second_wins(led: Card, second: Card, trump: Suit) -> bool {
    if second.suit() == led.suit() {
        second.rank() > led.rank()
    } else {
        second.suit() == trump
    }
}

/// The seat that takes the trick.
pub fn winner(leader: Seat, led: Card, second: Card, trump: Suit) -> Seat {
    if second_wins(led, second, trump) {
        leader.other()
    } else {
        leader
    }
}

/// The cards the second player may play to `led` in the last eight tricks:
/// a higher card of the suit if he has one; else any card of the suit; void
/// in it, a trump if he has one (any trump wins a plain-suit lead); else
/// anything.
pub fn endgame_replies(hand: CardSet, led: Card, trump: Suit) -> CardSet {
    let following = hand.in_suit(led.suit());
    if !following.is_empty() {
        let higher = following.intersect(above(led));
        return if higher.is_empty() { following } else { higher };
    }
    let trumps = hand.in_suit(trump);
    if trumps.is_empty() {
        hand
    } else {
        trumps
    }
}

/// Every card ranking above `card` in its own suit: a mask of the bits above
/// its pair of twins, cut to the suit.
fn above(card: Card) -> CardSet {
    let past_twins = 2 * (u32::from(card.card_type().0) + 1);
    CardSet(u64::MAX.checked_shl(past_twins).unwrap_or(0)).in_suit(card.suit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::CardType;
    use crate::rng::Rng;

    fn c(code: &str) -> Card {
        Card::new(CardType::parse(code).unwrap(), 0)
    }

    fn twin(code: &str) -> Card {
        Card::new(CardType::parse(code).unwrap(), 1)
    }

    fn set(codes: &str) -> CardSet {
        CardSet::parse(codes).unwrap()
    }

    const CLUBS: Suit = Suit::CLUBS;

    #[test]
    fn the_ten_beats_the_king_of_its_suit() {
        assert!(second_wins(c("KH"), c("TH"), CLUBS));
        assert!(!second_wins(c("AH"), c("TH"), CLUBS));
    }

    #[test]
    fn identical_cards_go_to_the_leader() {
        assert!(!second_wins(c("QS"), twin("QS"), CLUBS));
        assert!(!second_wins(twin("QS"), c("QS"), CLUBS));
        assert!(!second_wins(c("7C"), twin("7C"), CLUBS), "trumps too");
    }

    #[test]
    fn a_trump_takes_a_plain_lead_however_small() {
        assert!(second_wins(c("AH"), c("7C"), CLUBS));
    }

    #[test]
    fn a_plain_card_of_another_suit_loses_however_high() {
        assert!(!second_wins(c("7H"), c("AS"), CLUBS));
    }

    #[test]
    fn a_trump_lead_is_beaten_only_by_a_higher_trump() {
        assert!(second_wins(c("KC"), c("TC"), CLUBS));
        assert!(!second_wins(c("KC"), c("QC"), CLUBS));
        assert!(!second_wins(c("7C"), c("AH"), CLUBS));
    }

    #[test]
    fn the_winner_is_a_seat() {
        assert_eq!(winner(Seat::Pone, c("KH"), c("TH"), CLUBS), Seat::Dealer);
        assert_eq!(winner(Seat::Pone, c("TH"), c("KH"), CLUBS), Seat::Pone);
        assert_eq!(winner(Seat::Dealer, c("AH"), c("7C"), CLUBS), Seat::Pone);
    }

    #[test]
    fn in_the_endgame_follow_and_beat_if_you_can() {
        assert_eq!(endgame_replies(set("9H KH AS"), c("QH"), CLUBS), set("KH"));
        assert_eq!(
            endgame_replies(set("9H KH TH AS"), c("QH"), CLUBS),
            set("KH TH")
        );
    }

    #[test]
    fn unable_to_beat_any_card_of_the_suit_will_do() {
        assert_eq!(
            endgame_replies(set("9H 8H AS"), c("QH"), CLUBS),
            set("9H 8H")
        );
    }

    #[test]
    fn an_identical_card_does_not_beat_so_does_not_oblige() {
        let hand = CardSet::of(&[twin("QH"), c("9H")]);
        assert_eq!(endgame_replies(hand, c("QH"), CLUBS), hand);
    }

    #[test]
    fn void_in_the_suit_led_you_must_trump() {
        assert_eq!(
            endgame_replies(set("7C KC AS"), c("AH"), CLUBS),
            set("7C KC")
        );
    }

    #[test]
    fn void_and_without_trumps_anything_goes() {
        assert_eq!(endgame_replies(set("AS 9D"), c("AH"), CLUBS), set("AS 9D"));
    }

    #[test]
    fn a_trump_lead_must_be_overtrumped_if_possible() {
        assert_eq!(endgame_replies(set("TC 8C AH"), c("KC"), CLUBS), set("TC"));
        assert_eq!(endgame_replies(set("8C AH"), c("KC"), CLUBS), set("8C"));
        assert_eq!(endgame_replies(set("AH 9S"), c("KC"), CLUBS), set("AH 9S"));
    }

    #[test]
    fn the_ace_of_clubs_led_leaves_nothing_higher() {
        assert_eq!(
            endgame_replies(set("TC 7C"), c("AC"), Suit::HEARTS),
            set("TC 7C")
        );
        assert_eq!(endgame_replies(set("TC 7C"), c("AC"), CLUBS), set("TC 7C"));
    }

    /// Foster's sentence read literally: among the cards that follow suit (or,
    /// holding none, among all of them), those that win if any do, else all.
    fn reference(hand: CardSet, led: Card, trump: Suit) -> CardSet {
        let following = hand.in_suit(led.suit());
        let pool = if following.is_empty() {
            hand
        } else {
            following
        };
        let winning = CardSet::of(
            &pool
                .cards()
                .filter(|&card| second_wins(led, card, trump))
                .collect::<Vec<_>>(),
        );
        if winning.is_empty() {
            pool
        } else {
            winning
        }
    }

    #[test]
    fn agrees_with_fosters_sentence_on_random_endgames() {
        let mut rng = Rng::seeded(343);
        for _ in 0..20_000 {
            let mut pack: Vec<Card> = CardSet::PACK.cards().collect();
            rng.shuffle(&mut pack);
            let size = 1 + rng.below(8) as usize;
            let hand = CardSet::of(&pack[..size]);
            let led = pack[size];
            let trump = Suit(rng.below(4) as u8);
            let replies = endgame_replies(hand, led, trump);
            assert_eq!(
                replies,
                reference(hand, led, trump),
                "{hand:?} to {led:?}, trumps {trump:?}"
            );
            assert!(!replies.is_empty() && replies.minus(hand).is_empty());
        }
    }
}
