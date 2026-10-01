//! Rung 5: looking ahead at the end of the stock (`docs/DESIGN.md` §10.4).
//!
//! When few face-down cards remain, the rest of the hand is short: a few
//! stock tricks, then eight tricks of perfect information that the solver
//! settles exactly. So each card the player might play is tried in sampled
//! worlds: the cards it cannot see are dealt at random between the
//! opponent's hand and the stock, consistently with its view; the stock
//! tricks are played out by rung 4 on both sides; and the endgame they lead
//! into is valued with one call to the solver rather than played. The card
//! with the best total over the worlds is played.
//!
//! Everything is drawn from the view alone. The worlds are seeded from the
//! position, so the same position always gets the same decision: replaying
//! a game, or undoing a move, cannot re-roll it.

use crate::agents::{Maxims, Skills, Style};
use crate::cards::{Card, CardSet, Rank};
use crate::deal::{Seat, Stock};
use crate::game::Game;
use crate::hand::{Action, Hand, Phase, Source};
use crate::melds::Holdings;
use crate::observation::{Agent, View};
use crate::rng::Rng;
use crate::solver::{Endgame, Solver};

/// Rung 4 that, near the end of the stock, chooses its card by looking
/// ahead in sampled worlds.
#[derive(Clone, Debug)]
pub struct Lookahead {
    pub style: Style,
    /// Looks ahead once this many face-down cards or fewer remain.
    pub horizon: usize,
    /// Worlds sampled for each decision.
    pub worlds: usize,
}

impl Default for Lookahead {
    fn default() -> Lookahead {
        Lookahead {
            style: Style::default(),
            horizon: 9,
            worlds: 16,
        }
    }
}

impl Lookahead {
    /// Rung 4 in the same style: plays everything this rung does not look
    /// ahead for, and both sides of every world.
    fn rung4(&self) -> Maxims {
        Maxims {
            style: self.style.clone(),
            skills: Skills {
                memory: true,
                endgame: true,
                last_stock_trick: true,
            },
        }
    }

    /// The points `card` is worth to this player, from now to the end of the
    /// hand, in `world`.
    fn value_of(&self, world: &Game, seat: Seat, card: Card, solver: &mut Solver) -> i32 {
        let mut game = world.clone();
        if game.apply(Action::Play(card)).is_err() {
            return i32::MIN / 2;
        }
        let mut agents = [self.rung4(), self.rung4()];
        while let Some(side) = game.to_move() {
            if game.hand().in_endgame() {
                break;
            }
            let view = View::of(&game, side);
            let action = agents[side.index()].choose(&view, &game.legal());
            game.apply(action).expect("rung 4 plays legally");
        }
        let hand = game.hand();
        let mut value = hand.score(seat) as i32 - hand.score(seat.other()) as i32;
        if hand.in_endgame() && hand.phase() == Phase::Lead {
            let leader = hand.leader();
            let solved = solver.value(Endgame {
                mover: hand.holdings(leader).cards(),
                other: hand.holdings(leader.other()).cards(),
                led: None,
            });
            value += if leader == seat { solved } else { -solved };
        }
        value
    }
}

impl Agent for Lookahead {
    fn choose(&mut self, view: &View, legal: &[Action]) -> Action {
        let mut rung4 = self.rung4();
        let usual = rung4.choose(view, legal);
        let candidates: Vec<Card> = legal
            .iter()
            .filter_map(|a| match a {
                Action::Play(card) => Some(*card),
                _ => None,
            })
            .collect();
        if view.phase == Phase::Declare
            || view.turn_up.is_none()
            || view.stock > self.horizon
            || candidates.len() < 2
        {
            return usual;
        }
        let mut rng = Rng::seeded(fingerprint(view));
        let mut solver = Solver::new(view.trump);
        let mut totals = vec![0i64; candidates.len()];
        let mut sampled = 0;
        for _ in 0..self.worlds {
            let Some(world) = world(view, &mut rng) else {
                continue;
            };
            sampled += 1;
            for (total, &card) in totals.iter_mut().zip(&candidates) {
                *total += i64::from(self.value_of(&world, view.seat, card, &mut solver));
            }
        }
        if sampled == 0 {
            return usual;
        }
        // Rung 4's own card wins ties.
        let mut best = candidates
            .iter()
            .position(|&c| Action::Play(c) == usual)
            .unwrap_or(0);
        for (i, &total) in totals.iter().enumerate() {
            if total > totals[best] {
                best = i;
            }
        }
        Action::Play(candidates[best])
    }
}

/// A game consistent with `view`, its hidden cards dealt at random: the
/// opponent's unseen cards and the face-down stock, from the cards the
/// player cannot account for. The hand starts afresh at this point (its log
/// empty), so scores in it are points from here on. `None` if the view
/// admits no such position, which should not happen.
pub fn world(view: &View, rng: &mut Rng) -> Option<Game> {
    let mut hidden: Vec<Card> = view.unseen().cards().collect();
    if hidden.len() != view.their_hidden as usize + view.stock {
        return None;
    }
    rng.shuffle(&mut hidden);
    let (theirs_hidden, face_down) = hidden.split_at(view.their_hidden as usize);
    let other = view.seat.other();
    let mut theirs = Holdings {
        concealed: view.theirs.concealed,
        shown: view.theirs.shown,
        announced: view.theirs.announced.clone(),
    };
    for &card in theirs_hidden {
        theirs.concealed = theirs.concealed.with(card);
    }
    // Answering a lead: the world starts at the trick's start, the led card
    // back in the leader's hand, and replays the lead.
    if let Some(led) = view.led {
        theirs.concealed = theirs.concealed.with(led);
    }
    let mut holdings = [Holdings::default(), Holdings::default()];
    holdings[view.seat.index()] = view.mine.clone();
    holdings[other.index()] = theirs;
    let stock = Stock::new(face_down, view.turn_up);
    let hand = Hand::arranged(
        view.trump,
        stock,
        holdings,
        view.leader,
        view.tricks.len() as u8,
    )
    .ok()?
    .with_sevens_scored(sevens_scored(view));
    let dealer = if view.seat == Seat::Dealer {
        view.side
    } else {
        view.side.other()
    };
    let mut game = Game::arranged(0, [0, 0], dealer, hand).ok()?;
    if let Some(led) = view.led {
        game.apply(Action::Play(led)).ok()?;
    }
    Some(game)
}

/// The sevens of trumps in sight that may have scored: the turn-up, if it is
/// one, and those held by a player who has scored the dix. (A player who
/// exchanged one seven and later drew the other is counted as having scored
/// both; it costs at most ten points in a world, and only then.)
fn sevens_scored(view: &View) -> CardSet {
    let is_seven = |c: &Card| c.rank() == Rank::SEVEN && c.suit() == view.trump;
    let mut scored = CardSet::EMPTY;
    for card in view.turn_up.iter().filter(|c| is_seven(c)) {
        scored = scored.with(*card);
    }
    for (holdings, seat) in [(&view.mine, view.seat), (&view.theirs, view.seat.other())] {
        let dix = view
            .log
            .iter()
            .any(|s| s.seat == seat && s.source == Source::Dix);
        if dix {
            for card in holdings.cards().cards().filter(is_seven) {
                scored = scored.with(card);
            }
        }
    }
    scored
}

/// A seed for the worlds, from what the player can see.
fn fingerprint(view: &View) -> u64 {
    let parts = [
        view.mine.cards().0,
        view.theirs.cards().0,
        view.tricks.len() as u64,
        u64::from(view.hand_number),
        u64::from(view.totals.0) << 32 | u64::from(view.totals.1),
        view.led.map_or(64, |c| u64::from(c.0)),
        view.played().0,
    ];
    parts.iter().fold(0x6a09_e667_f3bc_c908, |h, &x| {
        (h ^ x).wrapping_mul(0x9e37_79b9_7f4a_7c15).rotate_left(29)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::rung;
    use crate::game::Side;

    /// Positions near the end of the stock, from games between two rung-4
    /// players, with the view of the player to move.
    fn positions(games: u64) -> Vec<(Game, View)> {
        let mut out = Vec::new();
        for seed in 0..games {
            let mut game = Game::new(50_000 + seed, Side::A);
            let mut agents = [rung(4), rung(4)];
            while let Some(side) = game.to_move() {
                let view = View::of(&game, side);
                if view.turn_up.is_some() && view.stock <= 7 && view.phase != Phase::Declare {
                    out.push((game.clone(), view.clone()));
                }
                let action = agents[side.index()].choose(&view, &game.legal());
                game.apply(action).unwrap();
            }
        }
        out
    }

    fn types(cards: CardSet) -> Vec<u8> {
        cards.cards().map(|c| c.card_type().0).collect()
    }

    #[test]
    fn every_world_agrees_with_the_view_it_was_drawn_from() {
        let positions = positions(30);
        assert!(positions.len() > 200, "{}", positions.len());
        let mut rng = Rng::seeded(1);
        for (game, view) in &positions {
            for _ in 0..3 {
                let world = world(view, &mut rng).expect("a world for every view");
                let seen = View::of(&world, view.side);
                assert_eq!(seen.seat, view.seat);
                assert_eq!(seen.mine.cards(), view.mine.cards());
                assert_eq!(seen.mine.announced, view.mine.announced);
                // The world's own view renumbers twins afresh (it does not
                // know what this player held earlier), so compare types.
                assert_eq!(types(seen.theirs.shown), types(view.theirs.shown));
                assert_eq!(seen.theirs.announced.len(), view.theirs.announced.len());
                assert_eq!(seen.turn_up.is_some(), view.turn_up.is_some());
                assert_eq!(seen.stock, view.stock);
                assert_eq!(seen.phase, view.phase);
                assert_eq!(seen.to_move, view.to_move);
                assert_eq!(seen.leader, view.leader);
                assert_eq!(seen.led.is_some(), view.led.is_some());
                // The world offers the moves the real game does.
                assert_eq!(world.legal(), game.legal());
            }
        }
    }

    #[test]
    fn it_plays_a_legal_card_and_the_same_one_every_time() {
        for (game, view) in positions(6) {
            let legal = game.legal();
            let mut agent = Lookahead::default();
            let first = agent.choose(&view, &legal);
            assert!(legal.contains(&first), "{first:?} not in {legal:?}");
            assert_eq!(agent.choose(&view, &legal), first);
        }
    }

    #[test]
    fn it_plays_as_rung_4_until_the_horizon() {
        let mut agent = Lookahead {
            horizon: 0,
            ..Lookahead::default()
        };
        let mut four = rung(4);
        for (game, view) in positions(4).into_iter().filter(|(_, v)| v.stock > 0) {
            let legal = game.legal();
            assert_eq!(agent.choose(&view, &legal), four.choose(&view, &legal));
        }
    }
}
