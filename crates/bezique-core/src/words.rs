//! How the game is said aloud: the period formulae for announcing.
//!
//! Foster (pp. 342–343) and Cady (p. 14) give the words: "Forty for bezique,
//! and twenty to score"; "Royal marriage, forty, and twenty to score";
//! "Two-fifty in trumps"; "Sixty and forty, and twenty, and twenty to score";
//! "Score forty". Fours are named by their value, "sixty queens", as the
//! period books do.

use crate::cards::Suit;
use crate::melds::{Announcement, Declared, Meld};

/// A score as it is said: `ten`, `forty`, `a hundred`, `two-fifty`.
pub fn points(points: u32) -> String {
    match points {
        10 => "ten".to_string(),
        20 => "twenty".to_string(),
        30 => "thirty".to_string(),
        40 => "forty".to_string(),
        50 => "fifty".to_string(),
        60 => "sixty".to_string(),
        80 => "eighty".to_string(),
        100 => "a hundred".to_string(),
        250 => "two-fifty".to_string(),
        500 => "five hundred".to_string(),
        other => other.to_string(),
    }
}

/// What is said on scoring `meld`: "Forty for bezique", "Sixty queens".
pub fn scoring(meld: Meld, trump: Suit) -> String {
    let said = points(meld.points(trump));
    match meld {
        Meld::Marriage(suit) if suit == trump => format!("Royal marriage, {said}"),
        Meld::Marriage(suit) => format!("{} for the marriage in {}", capital(&said), suit.name()),
        Meld::Sequence => format!("{} in trumps", capital(&said)),
        Meld::Bezique => format!("{} for bezique", capital(&said)),
        Meld::DoubleBezique => format!("{} for double bezique", capital(&said)),
        Meld::Four(rank) => format!("{} {}s", capital(&said), rank.name()),
    }
}

fn capital(text: &str) -> String {
    let mut chars = text.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

/// The whole announcement: what is scored, and what is then carried, e.g.
/// "Forty for bezique, and twenty to score." `carried_after` is what remains
/// to score once the announcement is made.
pub fn announcement(a: &Announcement, trump: Suit, carried_after: &[Declared]) -> String {
    let mut said = if a.melds.contains(&a.score) {
        scoring(a.score.meld, trump)
    } else {
        format!(
            "Score {}, for {}",
            points(a.score.meld.points(trump)),
            crate::melds::name(a.score.meld, trump)
        )
    };
    for carried in carried_after {
        said.push_str(&format!(", and {}", points(carried.meld.points(trump))));
    }
    if !carried_after.is_empty() {
        said.push_str(" to score");
    }
    said.push('.');
    said
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::{CardSet, Rank};

    fn d(meld: Meld) -> Declared {
        Declared {
            meld,
            cards: CardSet::EMPTY,
        }
    }

    const HEARTS: Suit = Suit::HEARTS;

    #[test]
    fn scores_said_aloud() {
        assert_eq!(points(10), "ten");
        assert_eq!(points(20), "twenty");
        // A trick can be worth thirty: two brisques and the last trick.
        assert_eq!(points(30), "thirty");
        assert_eq!(points(40), "forty");
        assert_eq!(points(60), "sixty");
        assert_eq!(points(80), "eighty");
        assert_eq!(points(100), "a hundred");
        assert_eq!(points(250), "two-fifty");
        assert_eq!(points(500), "five hundred");
    }

    #[test]
    fn each_combination_scored() {
        assert_eq!(scoring(Meld::Bezique, HEARTS), "Forty for bezique");
        assert_eq!(
            scoring(Meld::DoubleBezique, HEARTS),
            "Five hundred for double bezique"
        );
        assert_eq!(scoring(Meld::Sequence, HEARTS), "Two-fifty in trumps");
        assert_eq!(
            scoring(Meld::Marriage(HEARTS), HEARTS),
            "Royal marriage, forty"
        );
        assert_eq!(
            scoring(Meld::Marriage(Suit::CLUBS), HEARTS),
            "Twenty for the marriage in clubs"
        );
        assert_eq!(scoring(Meld::Four(Rank::ACE), HEARTS), "A hundred aces");
        assert_eq!(scoring(Meld::Four(Rank::KING), HEARTS), "Eighty kings");
        assert_eq!(scoring(Meld::Four(Rank::QUEEN), HEARTS), "Sixty queens");
        assert_eq!(scoring(Meld::Four(Rank::JACK), HEARTS), "Forty jacks");
    }

    #[test]
    fn fosters_forty_and_twenty_to_score() {
        let bezique = d(Meld::Bezique);
        let marriage = d(Meld::Marriage(Suit::SPADES));
        let a = Announcement {
            melds: vec![bezique, marriage],
            score: bezique,
        };
        assert_eq!(
            announcement(&a, HEARTS, &[marriage]),
            "Forty for bezique, and twenty to score."
        );
    }

    #[test]
    fn cadys_sixty_and_forty_and_twenty_and_twenty_to_score() {
        let queens = d(Meld::Four(Rank::QUEEN));
        let carried = [
            d(Meld::Marriage(HEARTS)),
            d(Meld::Marriage(Suit::CLUBS)),
            d(Meld::Marriage(Suit::DIAMONDS)),
        ];
        let a = Announcement {
            melds: vec![queens, carried[0], carried[1], carried[2]],
            score: queens,
        };
        assert_eq!(
            announcement(&a, HEARTS, &carried),
            "Sixty queens, and forty, and twenty, and twenty to score."
        );
    }

    #[test]
    fn scoring_what_was_carried() {
        let royal = d(Meld::Marriage(HEARTS));
        let rest = [
            d(Meld::Marriage(Suit::CLUBS)),
            d(Meld::Marriage(Suit::DIAMONDS)),
        ];
        let a = Announcement {
            melds: vec![],
            score: royal,
        };
        assert_eq!(
            announcement(&a, HEARTS, &rest),
            "Score forty, for the royal marriage, and twenty, and twenty to score."
        );
        assert_eq!(
            announcement(&a, HEARTS, &[]),
            "Score forty, for the royal marriage."
        );
    }

    #[test]
    fn a_plain_declaration() {
        let a = Announcement {
            melds: vec![d(Meld::Sequence)],
            score: d(Meld::Sequence),
        };
        assert_eq!(announcement(&a, HEARTS, &[]), "Two-fifty in trumps.");
    }
}
