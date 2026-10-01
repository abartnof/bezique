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
use crate::melds::{Class, Meld};
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

/// What rung 2 and above decide by: how much a card is worth keeping, and
/// how much it may be worth for it to be spent taking a trick. The default
/// is calibrated by mirrored measurement; a persona is a different set.
#[derive(Clone, Debug, PartialEq)]
pub struct Style {
    /// Extra worth of a trump.
    pub trump: i32,
    /// A bezique card (Q♠ or J♦) while the single or double can still be
    /// made with it.
    pub bezique: i32,
    /// A king or queen whose partner is held, before they are married:
    /// plain suit, and trumps.
    pub marriage: i32,
    pub royal: i32,
    /// A trump ace, ten, king, queen or jack while three of the five are held
    /// and the sequence is not yet declared.
    pub sequence: i32,
    /// An ace, king, queen or jack while three of the rank are held and no
    /// four of it declared.
    pub four: i32,
    /// The seven of trumps, while declaring lasts.
    pub dix: i32,
    /// A card carried "to score", until it scores.
    pub carried: i32,
    /// How much cheaper a card on the table is once it has no work left
    /// (Foster: of two cards alike, play the one already shown).
    pub shown: i32,
    /// The most a card may be worth for it to be spent taking a brisque.
    pub take_brisque: i32,
    /// The most a card may be worth for it to be spent winning the lead in
    /// order to declare,
    pub take_to_declare: i32,
    /// plus this much for every hundred points the declaration would score.
    pub urgency: i32,
}

impl Default for Style {
    fn default() -> Style {
        Style {
            trump: 5,
            bezique: 14,
            marriage: 4,
            royal: 6,
            sequence: 5,
            four: 4,
            dix: 4,
            carried: 4,
            shown: 2,
            take_brisque: 20,
            take_to_declare: 7,
            urgency: 16,
        }
    }
}

/// Rung 2: the maxims of the period books, weighed by a [`Style`].
#[derive(Clone, Debug, Default)]
pub struct Maxims {
    pub style: Style,
}

impl Agent for Maxims {
    fn choose(&mut self, view: &View, legal: &[Action]) -> Action {
        let style = &self.style;
        let card = match (view.phase, view.led) {
            (Phase::Declare, _) => return declaration(view, legal, true),
            (Phase::Follow, Some(led)) => follow(view, legal, led, style),
            _ if view.stock == 0 && view.turn_up.is_none() => endgame_lead(view, legal),
            _ => cheapest(view, cards(legal), style),
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
fn cheapest(view: &View, cards: impl Iterator<Item = Card>, style: &Style) -> Card {
    cards
        .min_by_key(|&c| (keep_value(c, view, style), c.0))
        .expect("a card to play")
}

/// How much a card is worth keeping in hand: its rank, more for a trump, and
/// while declaring lasts, more for each combination it can still serve in.
/// A card on the table with no such work left is cheaper, so that of two
/// alike the shown one goes first.
fn keep_value(card: Card, view: &View, style: &Style) -> i32 {
    let rank = card.rank();
    let trump = view.trump;
    let mine = &view.mine;
    let held = mine.cards();
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
        value += style.trump;
    }
    let declaring = view.stock > 0;
    if !declaring {
        return value;
    }
    let holds = |rank: Rank, suit: Suit| held.count_of(CardType::new(rank, suit)) > 0;
    let level = |class| mine.level(card, class);
    let mut work = false;
    let card_type = card.card_type();
    let bezique_card = card_type == CardType::new(Rank::QUEEN, Suit::SPADES)
        || card_type == CardType::new(Rank::JACK, Suit::DIAMONDS);
    if bezique_card && level(Class::Bezique) < 2 {
        value += style.bezique;
        work = true;
    }
    let partner = match rank {
        Rank::KING => Some(Rank::QUEEN),
        Rank::QUEEN => Some(Rank::KING),
        _ => None,
    };
    if partner.is_some_and(|p| holds(p, card.suit())) && level(Class::Marriages) == 0 {
        value += if card.suit() == trump {
            style.royal
        } else {
            style.marriage
        };
        work = true;
    }
    let sequence = [Rank::ACE, Rank::TEN, Rank::KING, Rank::QUEEN, Rank::JACK];
    if card.suit() == trump
        && sequence.contains(&rank)
        && level(Class::Marriages) < 2
        && sequence.iter().filter(|&&r| holds(r, trump)).count() >= 3
    {
        value += style.sequence;
        work = true;
    }
    if [Rank::ACE, Rank::KING, Rank::QUEEN, Rank::JACK].contains(&rank)
        && level(Class::Fours) == 0
        && held.cards().filter(|c| c.rank() == rank).count() >= 3
    {
        value += style.four;
        work = true;
    }
    if rank == Rank::SEVEN && card.suit() == trump {
        value += style.dix;
        work = true;
    }
    if mine.carried().iter().any(|d| d.cards.contains(card)) {
        value += style.carried;
        work = true;
    }
    if !work && mine.shown.contains(card) {
        value -= style.shown;
    }
    value
}

/// Answering a lead while the stock lasts (or under the follow-and-win rule,
/// which leaves fewer choices): take a brisque, or the lead when there is
/// something to declare, if it can be done cheaply; otherwise throw the least
/// valuable card, sparing brisques.
fn follow(view: &View, legal: &[Action], led: Card, style: &Style) -> Card {
    let trump = view.trump;
    let winners: Vec<Card> = cards(legal)
        .filter(|&c| tricks::second_wins(led, c, trump))
        .collect();
    if let Some(&winner) = winners
        .iter()
        .min_by_key(|&&c| (keep_value(c, view, style), c.0))
    {
        let cost = keep_value(winner, view, style);
        // What winning would let it declare: the more, the more it may spend.
        let waiting = if view.stock > 0 {
            view.mine
                .candidates(trump)
                .into_iter()
                .chain(view.mine.carried())
                .map(|d| d.meld.points(trump) as i32)
                .max()
                .unwrap_or(0)
        } else {
            0
        };
        let worth_declaring = style.take_to_declare + waiting * style.urgency / 100;
        let in_endgame = view.stock == 0 && view.turn_up.is_none();
        if in_endgame
            || (led.is_brisque() && cost <= style.take_brisque)
            || (waiting > 0 && cost <= worth_declaring)
        {
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
                keep_value(c, view, style) + if c.is_brisque() { 20 } else { 0 },
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
            play_out(&mut game, &mut [&mut Legal, &mut Maxims::default()]);
            assert!(game.outcome().is_some());
        }
    }

    #[test]
    fn the_maxims_beat_bare_legality() {
        let (maxims, legal) = games_won(0..40, &mut Maxims::default(), &mut Legal);
        assert!(
            maxims >= 56,
            "the maxims won {maxims} of 80 mirrored games, the baseline {legal}"
        );
    }

    /// Pardon: keep the bezique cards until near the end, even after
    /// declaring bezique; the double may yet come.
    #[test]
    fn a_declared_bezique_is_kept_for_the_double() {
        use crate::cards::CardSet;
        use crate::deal::{Seat, Stock};
        use crate::hand::Hand;
        use crate::melds::{Announcement, Holdings};
        let trump = Suit::HEARTS;
        let mut dealer = Holdings::new(CardSet::parse("QS JD AS TC TD AC AD TS").unwrap());
        let bezique = dealer
            .candidates(trump)
            .into_iter()
            .find(|d| d.meld == Meld::Bezique)
            .unwrap();
        dealer
            .apply(
                trump,
                &Announcement {
                    melds: vec![bezique],
                    score: bezique,
                },
            )
            .unwrap();
        let pone = Holdings::new(CardSet::parse("7C 8C 9C 7D 8D 9D 7S 8S").unwrap());
        let stock: Vec<Card> = CardSet::parse("9S 9H JC QC KC JS KS JH QH KH")
            .unwrap()
            .cards()
            .collect();
        let hand = Hand::arranged(
            trump,
            Stock::new(
                &stock,
                Some(CardSet::parse("7H").unwrap().cards().next().unwrap()),
            ),
            [dealer, pone],
            Seat::Dealer,
            4,
        );
        let game = Game::arranged(1, [0, 0], Side::A, hand);
        let view = View::of(&game, Side::A);
        let lead = Maxims::default().choose(&view, &game.legal());
        let Action::Play(card) = lead else {
            panic!("a lead")
        };
        assert!(
            card.card_type() != CardType::new(Rank::QUEEN, Suit::SPADES)
                && card.card_type() != CardType::new(Rank::JACK, Suit::DIAMONDS),
            "led the {card:?}, a card the double needs"
        );
    }

    #[test]
    fn identical_agents_split_mirrored_games_evenly() {
        // Deterministic agents on mirrored deals: every pair is one win each.
        let (a, b) = games_won(0..10, &mut Maxims::default(), &mut Maxims::default());
        assert_eq!((a, b), (10, 10));
    }
}
