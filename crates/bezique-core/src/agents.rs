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
use crate::lookahead::Lookahead;
use crate::melds::{Class, Meld};
use crate::observation::{Agent, View};
use crate::solver::{Endgame, Solver};
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
    /// How much readier it is to lead a plain-suit ace or ten while the stock
    /// lasts, to bring the brisque home (Foster: the lead is worth having
    /// when "you can bring home a plain-suit ten").
    pub lead_brisques: i32,
    /// Which of several cards of nearly equal worth it plays: a temperament,
    /// nearly free in points, as measured (`measurements/README.md`).
    pub leaning: Leaning,
}

/// Among cards within a point of the cheapest, which to play. Only near-ties
/// are touched, so a leaning changes how an opponent looks, not how well it
/// plays (`docs/DESIGN.md` §10.1).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Leaning {
    /// The cheapest, the lowest-numbered of equals.
    #[default]
    Plain,
    /// From its shortest suit, to be void of it (Foster: pick one or two
    /// suits and discard the rest).
    Concentrate,
    /// A card already on the table, showing nothing new (Pardon: play a
    /// declared card so as not to expose the hand).
    Close,
}

impl Default for Style {
    fn default() -> Style {
        Style {
            trump: 8,
            bezique: 20,
            marriage: 7,
            royal: 6,
            sequence: 5,
            four: 8,
            dix: 4,
            carried: 4,
            shown: 2,
            take_brisque: 20,
            take_to_declare: 20,
            urgency: 16,
            lead_brisques: 0,
            leaning: Leaning::Plain,
        }
    }
}

/// What a rung can do beyond the maxims. Each is kept only if it measurably
/// improves the rung below (`measurements/README.md`): watching the
/// opponent's table (counting its cards as gone, taking tricks to stop it
/// declaring) was tried on 2026-10-02 and gained nothing measurable, so it
/// is not here.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Skills {
    /// Remembers every card played, and so knows when a combination can no
    /// longer be made (alone, +1.8 points a hand over rung 2, se 0.6;
    /// `measurements/skills-final-style.txt`).
    pub memory: bool,
    /// Plays the last eight tricks exactly, which needs the memory to know
    /// the opponent's hand; without it, the skill does nothing (with memory,
    /// +18.6 points a hand over rung 2, se 0.6).
    pub endgame: bool,
    /// Wins the last trick before the stock runs out whenever it can: the
    /// winner makes the last declaration and leads into the endgame (+2.5
    /// points a hand over rung 3, se 0.4; confirmed +1.8, se 0.4).
    pub last_stock_trick: bool,
}

/// Rungs 2 to 4, and rung 5 away from the end of the stock: the maxims of the period books, weighed by a [`Style`],
/// with the [`Skills`] of the rung.
#[derive(Clone, Debug, Default)]
pub struct Maxims {
    pub style: Style,
    pub skills: Skills,
    /// Cards played before the position it sees, counted by type: what a
    /// rung with memory recalls in a world rebuilt part-way through a hand
    /// (rung 5's), whose view holds no tricks from before. Empty in play,
    /// where the view has every trick. Counted by type because the world's
    /// view numbers twins afresh.
    pub remembered: [u8; 32],
}

impl Agent for Maxims {
    fn choose(&mut self, view: &View, legal: &[Action]) -> Action {
        let style = &self.style;
        let in_endgame = view.stock == 0 && view.turn_up.is_none();
        if self.skills.endgame && self.skills.memory && in_endgame && view.phase != Phase::Declare {
            if let Some(card) = solve(view) {
                return Action::Play(card);
            }
        }
        let mut gone = CardSet::EMPTY;
        if self.skills.memory {
            gone = gone.union(view.played());
        }
        let remembered = if self.skills.memory {
            self.remembered
        } else {
            [0; 32]
        };
        let think = Think {
            view,
            style,
            gone,
            remembered,
            last_stock_trick: self.skills.last_stock_trick,
        };
        let card = match (view.phase, view.led) {
            (Phase::Declare, _) => return declaration(view, legal, true),
            (Phase::Follow, Some(led)) => follow(&think, legal, led),
            _ if in_endgame => endgame_lead(view, legal),
            _ => lead(&think, cards(legal)),
        };
        Action::Play(card)
    }
}

/// What one decision is weighed with.
struct Think<'a> {
    view: &'a View,
    style: &'a Style,
    /// Cards known to be out of reach: those played, to a rung that
    /// remembers them.
    gone: CardSet,
    /// More out of reach, by type: played before the view's first trick.
    remembered: [u8; 32],
    last_stock_trick: bool,
}

/// The highest rung built so far. Levels above it play as it does.
pub const TOP: u8 = 5;

/// The opponent at a level of the ladder, in the balanced style.
pub fn rung(level: u8) -> Box<dyn Agent> {
    rung_with(level, &Style::default())
}

/// Rung `level` playing in `style` (rung 1 has no style to play in).
pub fn rung_with(level: u8, style: &Style) -> Box<dyn Agent> {
    let skills = |last_stock_trick| Skills {
        memory: true,
        endgame: true,
        last_stock_trick,
    };
    let style = style.clone();
    match level {
        0 | 1 => Box::new(Legal),
        2 => Box::new(Maxims {
            style,
            ..Maxims::default()
        }),
        3 => Box::new(Maxims {
            style,
            skills: skills(false),
            ..Maxims::default()
        }),
        4 => Box::new(Maxims {
            style,
            skills: skills(true),
            ..Maxims::default()
        }),
        _ => Box::new(Lookahead {
            style,
            ..Lookahead::default()
        }),
    }
}

/// What the opponent at a level can do, in a player's words.
pub fn describe(level: u8) -> &'static str {
    match level {
        0 | 1 => "plays any legal card, and declares whatever scores most",
        2 => "plays by the maxims of the old books",
        3 => "plays by the maxims, remembers every card, and so plays the last eight tricks perfectly",
        4 => "remembers every card, plays the last eight tricks perfectly, and fights for the last trick before the stock runs out",
        _ => "remembers every card, plays the last eight tricks perfectly, and near the end of the stock looks ahead at what you might hold",
    }
}

/// The best card in the last eight tricks, where both hands are known to a
/// player who has counted: its own, and every card not yet seen. `None` if
/// the count does not come out, as in a position arranged part-way through
/// a hand, whose earlier tricks the view does not have.
fn solve(view: &View) -> Option<Card> {
    let theirs = view.theirs.cards().union(view.unseen());
    let answering = u32::from(view.led.is_some());
    if theirs.len() + answering != view.mine.cards().len() {
        return None;
    }
    let position = Endgame {
        mover: view.mine.cards(),
        other: theirs,
        led: view.led,
    };
    Some(Solver::new(view.trump).best(position).0)
}

/// The cards among the legal moves.
fn cards(legal: &[Action]) -> impl Iterator<Item = Card> + '_ {
    legal.iter().filter_map(|a| match a {
        Action::Play(card) => Some(*card),
        _ => None,
    })
}

/// The card to lead while the stock lasts: the least worth keeping, plain
/// aces and tens counted cheaper by the style's `lead_brisques`.
fn lead(think: &Think, cards: impl Iterator<Item = Card>) -> Card {
    let trump = think.view.trump;
    let valued = cards.map(|c| {
        let pull = if c.is_brisque() && c.suit() != trump {
            think.style.lead_brisques
        } else {
            0
        };
        (c, keep_value(c, think) - pull)
    });
    pick(think, valued.collect())
}

/// The card to play of these, by value: the cheapest, or among those within
/// a point of it, the one the style leans to.
fn pick(think: &Think, valued: Vec<(Card, i32)>) -> Card {
    let cheapest = valued
        .iter()
        .map(|&(_, v)| v)
        .min()
        .expect("a card to play");
    // How near counts as a tie, by measurement: shedding a short suit costs
    // points unless the cards are of exactly equal worth.
    let slack = match think.style.leaning {
        Leaning::Concentrate => 0,
        _ => 1,
    };
    let near = valued.iter().filter(|&&(_, v)| v <= cheapest + slack);
    let mine = &think.view.mine;
    let held_in = |c: Card| mine.cards().in_suit(c.suit()).len();
    match think.style.leaning {
        Leaning::Plain => valued.iter().min_by_key(|&&(c, v)| (v, c.0)),
        Leaning::Concentrate => near.min_by_key(|&&(c, v)| (held_in(c), v, c.0)),
        Leaning::Close => near.min_by_key(|&&(c, v)| (!mine.shown.contains(c), v, c.0)),
    }
    .expect("a card to play")
    .0
}

/// How much a card is worth keeping in hand: its rank, more for a trump, and
/// while declaring lasts, more for each combination it can still serve in.
/// A card on the table with no such work left is cheaper, so that of two
/// alike the shown one goes first.
fn keep_value(card: Card, think: &Think) -> i32 {
    let (view, style, gone) = (think.view, think.style, think.gone);
    // Copies of a type not known to be out of reach.
    let available = |t: CardType| {
        2 - (gone.count_of(t) + u32::from(think.remembered[usize::from(t.0)])).min(2) as i32
    };
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
    let bezique_alive = available(CardType::new(Rank::QUEEN, Suit::SPADES)) >= 1
        && available(CardType::new(Rank::JACK, Suit::DIAMONDS)) >= 1;
    if bezique_card && level(Class::Bezique) < 2 && bezique_alive {
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
    let sequence_alive = sequence
        .iter()
        .all(|&r| available(CardType::new(r, trump)) >= 1);
    if card.suit() == trump
        && sequence.contains(&rank)
        && level(Class::Marriages) < 2
        && sequence.iter().filter(|&&r| holds(r, trump)).count() >= 3
        && sequence_alive
    {
        value += style.sequence;
        work = true;
    }
    let four_alive = Suit::ALL
        .iter()
        .map(|&s| available(CardType::new(rank, s)))
        .sum::<i32>()
        >= 4;
    if [Rank::ACE, Rank::KING, Rank::QUEEN, Rank::JACK].contains(&rank)
        && level(Class::Fours) == 0
        && held.cards().filter(|c| c.rank() == rank).count() >= 3
        && four_alive
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
fn follow(think: &Think, legal: &[Action], led: Card) -> Card {
    let (view, style) = (think.view, think.style);
    let trump = view.trump;
    let winners: Vec<Card> = cards(legal)
        .filter(|&c| tricks::second_wins(led, c, trump))
        .collect();
    if let Some(&winner) = winners.iter().min_by_key(|&&c| (keep_value(c, think), c.0)) {
        if think.last_stock_trick && view.stock == 1 {
            return winner;
        }
        let cost = keep_value(winner, think);
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
    let valued = pool
        .into_iter()
        .map(|c| {
            (
                c,
                keep_value(c, think) + if c.is_brisque() { 20 } else { 0 },
            )
        })
        .collect();
    pick(think, valued)
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
/// scores more; else pass. Of announcements scoring alike, the one leaving
/// most to score afterwards: otherwise the choice would fall to which copy
/// of a twin is which (full review of 2026-10-02).
fn declaration(view: &View, legal: &[Action], maxims: bool) -> Action {
    let trump = view.trump;
    let points = |a: &Action| match a {
        Action::Declare(x) => x.score.meld.points(trump),
        _ => 0,
    };
    // What is still to score after an announcement.
    let to_score = |a: &Action| match a {
        Action::Declare(x) => {
            let mut after = view.mine.clone();
            after.apply(trump, x).map_or(0, |_| {
                after.carried().iter().map(|d| d.meld.points(trump)).sum::<u32>()
            })
        }
        _ => 0,
    };
    let mut best = legal
        .iter()
        .filter(|a| matches!(a, Action::Declare(_)))
        .max_by_key(|a| (points(a), to_score(a)))
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
                let first = legal
                    .iter()
                    .filter(|a| {
                        matches!(a, Action::Declare(y) if y.score.meld == meld && y.score.cards.minus(within).is_empty())
                    })
                    .max_by_key(|a| to_score(a));
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
        let turn_up = CardSet::parse("7H").unwrap().cards().next().unwrap();
        let stock: Vec<Card> = CardSet::PACK
            .minus(dealer.cards())
            .minus(pone.cards())
            .without(turn_up)
            .cards()
            .collect();
        let hand = Hand::arranged(
            trump,
            Stock::new(&stock, Some(turn_up)),
            [dealer, pone],
            Seat::Dealer,
            0,
        )
        .unwrap();
        let game = Game::arranged(1, [0, 0], Side::A, hand).unwrap();
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

    /// A position arranged in the last tricks has no record of the tricks
    /// before it, so the count of unseen cards does not come out: the
    /// endgame skill stands aside rather than solve a wrong position.
    #[test]
    fn an_arranged_endgame_is_played_without_the_solver_not_panicked_on() {
        use crate::deal::{Seat, Stock};
        use crate::hand::Hand;
        use crate::melds::Holdings;
        let hand = Hand::arranged(
            Suit::HEARTS,
            Stock::new(&[], None),
            [
                Holdings::new(CardSet::parse("AS 7D").unwrap()),
                Holdings::new(CardSet::parse("TS AD").unwrap()),
            ],
            Seat::Dealer,
            30,
        )
        .unwrap();
        let game = Game::arranged(1, [0, 0], Side::A, hand).unwrap();
        let view = View::of(&game, Side::A);
        for level in 3..=TOP {
            let action = rung(level).choose(&view, &game.legal());
            assert!(game.legal().contains(&action));
        }
    }

    /// The endgame skill needs the memory: without it, the last eight tricks
    /// are played by the maxims, as at rung 2.
    #[test]
    fn without_memory_the_endgame_skill_does_nothing() {
        let mut forgetful = Maxims {
            skills: Skills {
                memory: false,
                endgame: true,
                last_stock_trick: false,
            },
            ..Maxims::default()
        };
        let mut two = Maxims::default();
        let mut checked = 0;
        for seed in 0..10 {
            let mut game = Game::new(600 + seed, Side::A);
            let mut agents = [rung(4), rung(4)];
            while let Some(side) = game.to_move() {
                let view = View::of(&game, side);
                let legal = game.legal();
                if view.stock == 0 && view.turn_up.is_none() {
                    assert_eq!(forgetful.choose(&view, &legal), two.choose(&view, &legal));
                    checked += 1;
                }
                let action = agents[side.index()].choose(&view, &legal);
                game.apply(action).unwrap();
            }
        }
        assert!(checked > 100, "{checked}");
    }

    /// At the first lead of the last eight tricks, between two rung-3
    /// players: the opponent's hand as counted from the view is the true one,
    /// type for type, and the solver's value is the margin the engine then
    /// scores with both playing on. Checks the solver's scoring against the
    /// engine's own, which `solver::reference_value` cannot (it shares the
    /// solver's step).
    #[test]
    fn the_counted_hand_is_true_and_the_solver_scores_as_the_engine_does() {
        use crate::hand::Phase;
        let types = |cards: CardSet| -> Vec<u8> { cards.cards().map(|c| c.card_type().0).collect() };
        let mut checked = 0;
        for seed in 0..150u64 {
            let mut game = Game::new(seed, Side::A);
            let mut agents = [rung(3), rung(3)];
            let mut step = |game: &mut Game| {
                let side = game.to_move().unwrap();
                let view = View::of(game, side);
                let action = agents[side.index()].choose(&view, &game.legal());
                game.apply(action).unwrap();
            };
            while game.to_move().is_some()
                && game.hand_number() == 1
                && !(game.hand().in_endgame() && game.hand().phase() == Phase::Lead)
            {
                step(&mut game);
            }
            if game.hand_number() != 1 || game.to_move().is_none() {
                continue;
            }
            let side = game.to_move().unwrap();
            let seat = game.seat_of(side);
            let view = View::of(&game, side);
            let theirs = view.theirs.cards().union(view.unseen());
            let truth = game.hand().holdings(seat.other()).cards();
            assert_eq!(types(theirs), types(truth), "seed {seed}");
            let value = Solver::new(view.trump).value(Endgame {
                mover: view.mine.cards(),
                other: theirs,
                led: None,
            });
            let margin = |h: &crate::hand::Hand| h.score(seat) as i32 - h.score(seat.other()) as i32;
            let before = margin(game.hand());
            while game.to_move().is_some() && game.hand_number() == 1 {
                step(&mut game);
            }
            if game.hand_number() == 1 && game.hand().phase() != Phase::Over {
                continue; // the game was won during the endgame
            }
            let hand = if game.hand_number() > 1 {
                game.previous_hand().unwrap()
            } else {
                game.hand()
            };
            assert_eq!(margin(hand) - before, value, "seed {seed}");
            checked += 1;
        }
        assert!(checked > 50, "{checked}");
    }

    /// Of announcements scoring alike, the one that keeps a carried score:
    /// whichever copy of a twin is which (full review of 2026-10-02). The
    /// dealer has K♥ and Q♥ twice on the table, one pair carrying a royal
    /// marriage "to score", and A♥ T♥ J♥ in hand: every sequence scores 250,
    /// but only those using the other pair keep the forty.
    #[test]
    fn a_carried_score_is_kept_whichever_twin_carries_it() {
        use crate::deal::{Seat, Stock};
        use crate::hand::Hand;
        use crate::melds::{Announced, Declared, Holdings};
        let c = |code: &str, copy: u8| Card::new(CardType::parse(code).unwrap(), copy);
        let set = |cards: &[Card]| CardSet::of(cards);
        for married in [0u8, 1] {
            let other = 1 - married;
            let declared = |meld, cards: &[Card], scored| Announced {
                declared: Declared { meld, cards: set(cards) },
                scored,
            };
            let dealer = Holdings {
                concealed: set(&[c("AH", 0), c("TH", 0), c("JH", 0), c("AS", 0)]),
                shown: set(&[c("KH", 0), c("KH", 1), c("QH", 0), c("QH", 1)]),
                announced: vec![
                    declared(Meld::Four(Rank::KING), &[c("KH", other), c("KS", 0), c("KD", 0), c("KC", 0)], true),
                    declared(Meld::Four(Rank::QUEEN), &[c("QH", other), c("QS", 0), c("QD", 0), c("QC", 0)], true),
                    declared(Meld::Marriage(Suit::HEARTS), &[c("KH", married), c("QH", married)], false),
                ],
            };
            let pone = Holdings::new(set(&[
                c("7D", 0), c("8D", 0), c("9D", 0), c("7C", 0), c("8C", 0), c("9C", 0), c("7S", 0), c("8S", 0),
            ]));
            let gone = set(&[c("KS", 0), c("KD", 0), c("KC", 0), c("QS", 0), c("QD", 0), c("QC", 0)]);
            let turn_up = c("7H", 0);
            let pool: Vec<Card> = CardSet::PACK
                .minus(dealer.cards())
                .minus(pone.cards())
                .minus(gone)
                .without(turn_up)
                .cards()
                .take(5)
                .collect();
            let hand = Hand::arranged(Suit::HEARTS, Stock::new(&pool, Some(turn_up)), [dealer, pone], Seat::Dealer, 21)
                .unwrap();
            let mut game = Game::arranged(1, [0, 0], Side::A, hand).unwrap();
            game.apply(Action::Play(c("AS", 0))).unwrap();
            game.apply(Action::Play(c("7D", 0))).unwrap();
            let view = View::of(&game, Side::A);
            let action = rung(2).choose(&view, &game.legal());
            assert!(matches!(&action, Action::Declare(a) if a.score.meld == Meld::Sequence), "{action:?}");
            game.apply(action).unwrap();
            let carried = game.hand().holdings(Seat::Dealer).carried();
            assert_eq!(carried.len(), 1, "married copy {married}: {carried:?}");
        }
    }
}
