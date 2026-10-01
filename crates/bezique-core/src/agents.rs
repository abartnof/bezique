//! The opponents, as a ladder of capabilities (`docs/DESIGN.md` §10.1).
//!
//! Each rung reads only its [`View`]. So far:
//!
//! 1. **Plays legally**: plays its lowest card and declares whatever scores
//!    most.
//! 2. **The period maxims** (Pole, Pardon, Foster, Hoffmann; literature review
//!    §7): throw the cards that matter least; take a brisque when it can be
//!    had cheaply; win a trick when there is something to declare; keep the
//!    bezique, marriage and sequence cards and the makings of a four; declare
//!    the single bezique before the double and the royal marriage before the
//!    sequence while there is time; exchange the seven of trumps.

use crate::cards::{Card, CardSet, CardType, Rank, Suit};
use crate::game::Game;
use crate::hand::{Action, Phase};
use crate::melds::Meld;
use crate::observation::{Agent, View};
use crate::tricks;

/// Rung 1: legal, and no more.
#[derive(Clone, Debug, Default)]
pub struct Legal;

impl Agent for Legal {
    fn choose(&mut self, view: &View, legal: &[Action]) -> Action {
        if view.phase == Phase::Declare {
            return declaration(view, legal, false);
        }
        let lowest = cards(legal)
            .min_by_key(|c| (c.rank(), c.0))
            .expect("a card to play");
        Action::Play(lowest)
    }
}

/// Rung 2: the maxims of the period books.
#[derive(Clone, Debug, Default)]
pub struct Maxims;

impl Agent for Maxims {
    fn choose(&mut self, view: &View, legal: &[Action]) -> Action {
        let card = match (view.phase, view.led) {
            (Phase::Declare, _) => return declaration(view, legal, true),
            (Phase::Follow, Some(led)) => follow(view, legal, led),
            _ if view.stock == 0 && view.turn_up.is_none() => endgame_lead(view, legal),
            _ => cheapest(view, cards(legal)),
        };
        Action::Play(card)
    }
}

/// The cards among the legal moves.
fn cards(legal: &[Action]) -> impl Iterator<Item = Card> + '_ {
    legal.iter().filter_map(|a| match a {
        Action::Play(card) => Some(*card),
        _ => None,
    })
}

/// The card least worth keeping.
fn cheapest(view: &View, cards: impl Iterator<Item = Card>) -> Card {
    cards
        .min_by_key(|&c| (keep_value(c, view), c.0))
        .expect("a card to play")
}

/// How much a card is worth keeping in hand: its rank, more for a trump, and
/// while declaring lasts, more for its part in combinations to come.
fn keep_value(card: Card, view: &View) -> i32 {
    let rank = card.rank();
    let trump = view.trump;
    let mine = view.mine.cards();
    let mut value = match rank {
        Rank::SEVEN => 0,
        Rank::EIGHT => 1,
        Rank::NINE => 2,
        Rank::JACK => 3,
        Rank::QUEEN => 4,
        Rank::KING => 5,
        Rank::TEN => 10,
        _ => 11,
    };
    if card.suit() == trump {
        value += 5;
    }
    let declaring = view.stock > 0;
    if !declaring {
        return value;
    }
    let held = |rank: Rank, suit: Suit| mine.count_of(CardType::new(rank, suit)) > 0;
    let card_type = card.card_type();
    if card_type == CardType::new(Rank::QUEEN, Suit::SPADES)
        || card_type == CardType::new(Rank::JACK, Suit::DIAMONDS)
    {
        value += 6;
    }
    let partner = match rank {
        Rank::KING => Some(Rank::QUEEN),
        Rank::QUEEN => Some(Rank::KING),
        _ => None,
    };
    if partner.is_some_and(|p| held(p, card.suit())) {
        value += if card.suit() == trump { 6 } else { 4 };
    }
    let sequence = [Rank::ACE, Rank::TEN, Rank::KING, Rank::QUEEN, Rank::JACK];
    if card.suit() == trump
        && sequence.contains(&rank)
        && sequence.iter().filter(|&&r| held(r, trump)).count() >= 3
    {
        value += 5;
    }
    if [Rank::ACE, Rank::KING, Rank::QUEEN, Rank::JACK].contains(&rank)
        && mine.cards().filter(|c| c.rank() == rank).count() >= 3
    {
        value += 4;
    }
    if rank == Rank::SEVEN && card.suit() == trump {
        value += 4;
    }
    if view.mine.carried().iter().any(|d| d.cards.contains(card)) {
        value += 4;
    } else if view.mine.shown.contains(card) {
        // Foster: of two cards alike, play the one already on the table; it
        // tells the opponent nothing new.
        value -= 2;
    }
    value
}

/// Answering a lead while the stock lasts (or under the follow-and-win rule,
/// which leaves fewer choices): take a brisque, or the lead when there is
/// something to declare, if it can be done cheaply; otherwise throw the least
/// valuable card, sparing brisques.
fn follow(view: &View, legal: &[Action], led: Card) -> Card {
    let trump = view.trump;
    let winners: Vec<Card> = cards(legal)
        .filter(|&c| tricks::second_wins(led, c, trump))
        .collect();
    if let Some(&winner) = winners.iter().min_by_key(|&&c| (keep_value(c, view), c.0)) {
        let cost = keep_value(winner, view);
        let something_to_declare = view.stock > 0
            && (!view.mine.candidates(trump).is_empty() || !view.mine.carried().is_empty());
        let in_endgame = view.stock == 0 && view.turn_up.is_none();
        if in_endgame || (led.is_brisque() && cost <= 12) || (something_to_declare && cost <= 7) {
            return winner;
        }
    }
    let losers: Vec<Card> = cards(legal).filter(|c| !winners.contains(c)).collect();
    let pool = if losers.is_empty() {
        cards(legal).collect()
    } else {
        losers
    };
    pool.into_iter()
        .min_by_key(|&c| {
            (
                keep_value(c, view) + if c.is_brisque() { 20 } else { 0 },
                c.0,
            )
        })
        .expect("a card to play")
}

/// Leading to the last eight tricks: a plain-suit ace, which can only be
/// trumped; else the highest trump, to draw the opponent's; else the
/// lowest card.
fn endgame_lead(view: &View, legal: &[Action]) -> Card {
    let trump = view.trump;
    let mine: Vec<Card> = cards(legal).collect();
    if let Some(&ace) = mine
        .iter()
        .find(|c| c.rank() == Rank::ACE && c.suit() != trump)
    {
        return ace;
    }
    let trumps: Vec<Card> = mine.iter().copied().filter(|c| c.suit() == trump).collect();
    if trumps.len() >= 3 {
        return *trumps
            .iter()
            .max_by_key(|c| (c.rank(), c.0))
            .expect("trumps");
    }
    *mine
        .iter()
        .min_by_key(|c| (c.rank(), c.0))
        .expect("a card to lead")
}

/// The turn of declaring: the announcement scoring most; with `maxims`, the
/// single bezique before the double and the royal marriage before the
/// sequence while there is time to win another trick; the dix when nothing
/// scores more; else pass.
fn declaration(view: &View, legal: &[Action], maxims: bool) -> Action {
    let trump = view.trump;
    let points = |a: &Action| match a {
        Action::Declare(x) => x.score.meld.points(trump),
        _ => 0,
    };
    let mut best = legal
        .iter()
        .filter(|a| matches!(a, Action::Declare(_)))
        .max_by_key(|a| points(a))
        .cloned();
    if maxims && view.stock >= 10 {
        if let Some(Action::Declare(x)) = &best {
            let bezique_declared = view
                .mine
                .announced
                .iter()
                .any(|a| a.declared.meld == Meld::Bezique);
            let smaller = match x.score.meld {
                Meld::DoubleBezique if !bezique_declared => Some((Meld::Bezique, CardSet::PACK)),
                Meld::Sequence => Some((Meld::Marriage(trump), x.score.cards)),
                _ => None,
            };
            if let Some((meld, within)) = smaller {
                let first = legal.iter().find(|a| {
                    matches!(a, Action::Declare(y) if y.score.meld == meld && y.score.cards.minus(within).is_empty())
                });
                if let Some(first) = first {
                    best = Some(first.clone());
                }
            }
        }
    }
    let dix = legal.iter().find(|a| matches!(a, Action::Dix(_))).cloned();
    match (best, dix) {
        (Some(best), _) if points(&best) > 10 => best,
        (_, Some(dix)) => dix,
        (Some(best), None) => best,
        (None, None) => Action::Pass,
    }
}

/// Plays a game to its end, `agents[i]` playing side `Side::BOTH[i]`.
pub fn play_out(game: &mut Game, agents: &mut [&mut dyn Agent; 2]) {
    while let Some(side) = game.to_move() {
        let legal = game.legal();
        let view = View::of(game, side);
        let action = agents[side.index()].choose(&view, &legal);
        game.apply(action).expect("agents choose among legal moves");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Outcome, Side};

    fn games_won(
        seeds: std::ops::Range<u64>,
        first: &mut dyn Agent,
        second: &mut dyn Agent,
    ) -> (u32, u32) {
        let mut wins = (0, 0);
        for seed in seeds {
            // Mirrored: the same cards twice, the agents changing places.
            for swap in [false, true] {
                let mut game = Game::new(seed, Side::A);
                let mut agents: [&mut dyn Agent; 2] = if swap {
                    [&mut *second, &mut *first]
                } else {
                    [&mut *first, &mut *second]
                };
                play_out(&mut game, &mut agents);
                let Outcome { winner, .. } = game.outcome().unwrap();
                let first_won = (winner == Side::A) != swap;
                if first_won {
                    wins.0 += 1;
                } else {
                    wins.1 += 1;
                }
            }
        }
        wins
    }

    #[test]
    fn both_rungs_play_whole_games_legally() {
        for seed in 0..10 {
            let mut game = Game::new(seed, Side::A);
            play_out(&mut game, &mut [&mut Legal, &mut Maxims]);
            assert!(game.outcome().is_some());
        }
    }

    #[test]
    fn the_maxims_beat_bare_legality() {
        let (maxims, legal) = games_won(0..40, &mut Maxims, &mut Legal);
        assert!(
            maxims >= 56,
            "the maxims won {maxims} of 80 mirrored games, the baseline {legal}"
        );
    }

    #[test]
    fn identical_agents_split_mirrored_games_evenly() {
        // Deterministic agents on mirrored deals: every pair is one win each.
        let (a, b) = games_won(0..10, &mut Maxims, &mut Maxims);
        assert_eq!((a, b), (10, 10));
    }
}
