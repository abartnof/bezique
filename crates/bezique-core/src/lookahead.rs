//! Rung 5: looking ahead at the end of the stock (`docs/DESIGN.md` §10.4).
//!
//! When few face-down cards remain, the rest of the hand is short: a few
//! stock tricks, then eight tricks of perfect information that the solver
//! settles exactly. So each card the player might play is tried in sampled
//! worlds: the cards it cannot see are dealt at random between the
//! opponent's hand and the stock, consistently with its view; the stock
//! tricks are played out by rung 4 on both sides; and the endgame they lead
//! into is valued with one call to the solver rather than played. The card
//! with the best total over the worlds is played. A world is rebuilt without
//! the tricks before it, so the rollout players are told what was played
//! before, by type, and recall it as rung 4 does in play.
//!
//! Everything is drawn from the view alone. The worlds are seeded from the
//! position, so the same position always gets the same decision: replaying
//! a game, or undoing a move, cannot re-roll it.

use crate::agents::{Maxims, Skills, Style};
use crate::cards::{Card, CardSet, Rank};
use crate::deal::{Seat, Stock};
use crate::game::{Game, Side, GAME};
use crate::hand::{Action, Hand, Phase, Source};
use crate::melds::Holdings;
use crate::observation::{Agent, View};
#[cfg(feature = "race")]
use crate::race;
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
    /// Looks ahead in the turn of declaring too, not only in play. Off at
    /// rung 5: it measured worse (`measurements/lookahead-declarations.txt`).
    pub declarations: bool,
    /// The rollout players recall the cards played before the world began
    /// (by type), as rung 4 does in play. Off only to measure what it is
    /// worth: without it rung 4 in a world chose differently from rung 4 in
    /// the real game in about one decision in twenty-five.
    pub rollout_memory: bool,
    /// What a world is worth: the hand's points, or the game.
    pub objective: Objective,
}

/// What looking ahead plays for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Objective {
    /// The most points over the rest of the hand (rung 5).
    #[default]
    Hand,
    /// The game (rung 6): once either player is within [`NEAR`] of 1,000, a
    /// world is worth what the game is worth at its end, in game points (a
    /// game won 1, a double 2, the same lost negative): exactly, if the
    /// game ends in it; else, from the totals when the hand ends, by
    /// [`race::value`]; of cards equal in that, the one scoring most in the
    /// hand. Further from 1,000 it plays as rung 5. Only the cards played in
    /// the last stock tricks are chosen this way: declaring stays rung 4's,
    /// the last eight tricks the solver's (which maximises the hand's
    /// points), and the race model is built from rung 4's play.
    Game,
}

/// How near 1,000 either player must be for rung 6 to play for the game. A
/// player scores 450 or more in about one hand in twenty (in the race
/// library), so a game can rarely end within a hand from further off.
pub const NEAR: u32 = 450;

impl Default for Lookahead {
    fn default() -> Lookahead {
        Lookahead {
            style: Style::default(),
            horizon: 9,
            worlds: 16,
            declarations: false,
            rollout_memory: true,
            objective: Objective::Hand,
        }
    }
}

impl Lookahead {
    /// Rung 4 in the same style: plays everything this rung does not look
    /// ahead for, and both sides of every world, recalling `remembered`, the
    /// cards played before the world began (by type).
    fn rung4(&self, remembered: [u8; 32]) -> Maxims {
        Maxims {
            style: self.style.clone(),
            skills: Skills {
                memory: true,
                endgame: true,
                last_stock_trick: true,
            },
            remembered,
        }
    }

    /// The points `action` is worth to this player, from now to the end of
    /// the hand, in `world`.
    fn value_of(
        &self,
        world: &Game,
        seat: Seat,
        action: &Action,
        remembered: [u8; 32],
        solver: &mut Solver,
    ) -> i32 {
        let mut game = world.clone();
        if game.apply(action.clone()).is_err() {
            return i32::MIN / 2;
        }
        let mut agents = [self.rung4(remembered), self.rung4(remembered)];
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

impl Lookahead {
    /// Whether this decision is played for the game rather than the hand.
    fn for_the_game(&self, view: &View) -> bool {
        cfg!(feature = "race")
            && self.objective == Objective::Game
            && view.totals.0.max(view.totals.1) + NEAR >= GAME
    }

    /// What `action` is worth to `side` for the game, in millionths of a
    /// game point, in `world` (whose game starts from the real totals), with
    /// the hand's points from here as a second measure for ties: the stock
    /// played out by rung 4, the last eight tricks by the solver on both
    /// sides, as the real players play them, so that who reaches 1,000 first
    /// is the engine's to say; then, if the hand ends with the game still
    /// on, the race from the hand's final totals.
    fn game_value_of(
        &self,
        world: &Game,
        side: Side,
        action: &Action,
        remembered: [u8; 32],
        solver: &mut Solver,
    ) -> (i64, i64) {
        let mut game = world.clone();
        if game.apply(action.clone()).is_err() {
            return (i64::MIN / 4, 0);
        }
        let number = game.hand_number();
        let mut agents = [self.rung4(remembered), self.rung4(remembered)];
        while let Some(mover) = game.to_move() {
            if game.hand_number() != number {
                break;
            }
            let hand = game.hand();
            let action = if hand.in_endgame() {
                let seat = hand.to_move().expect("a player to move");
                let (card, _) = solver.best(Endgame {
                    mover: hand.holdings(seat).cards(),
                    other: hand.holdings(seat.other()).cards(),
                    led: hand.led(),
                });
                Action::Play(card)
            } else {
                let view = View::of(&game, mover);
                agents[mover.index()].choose(&view, &game.legal())
            };
            game.apply(action)
                .expect("rung 4 and the solver play legally");
        }
        let hand_ended = game.hand_number() != number;
        let hand = if hand_ended {
            game.previous_hand().expect("the hand just ended")
        } else {
            game.hand()
        };
        let seat = world.seat_of(side);
        let margin = i64::from(hand.score(seat)) - i64::from(hand.score(seat.other()));
        let worth = if hand_ended {
            // The totals the hand ended on, and who deals next. Whatever the
            // next deal did is not this world's: every world deals the same
            // next hand, so its turned seven must not count.
            let banked = |s: Side| game.total(s) - game.hand().score(game.seat_of(s));
            race_value(banked(side), banked(side.other()), game.dealer() == side)
        } else {
            let outcome = game.outcome().expect("the game ended within the hand");
            let games = if outcome.double { 2.0 } else { 1.0 };
            if outcome.winner == side {
                games
            } else {
                -games
            }
        };
        ((1_000_000.0 * worth).round() as i64, margin)
    }
}

impl Agent for Lookahead {
    fn choose(&mut self, view: &View, legal: &[Action]) -> Action {
        let mut rung4 = self.rung4([0; 32]);
        let usual = rung4.choose(view, legal);
        let candidates: Vec<Action> = legal.to_vec();
        if (view.phase == Phase::Declare && !self.declarations)
            || view.turn_up.is_none()
            || view.stock > self.horizon
            || candidates.len() < 2
        {
            return usual;
        }
        let remembered = if self.rollout_memory {
            before_the_world(view)
        } else {
            [0; 32]
        };
        let for_the_game = self.for_the_game(view);
        let banked = if for_the_game {
            game_banked(view)
        } else {
            [0, 0]
        };
        let mut rng = Rng::seeded(fingerprint(view));
        let mut solver = Solver::new(view.trump);
        // Per candidate: the value summed over the worlds, and (playing for
        // the game) the hand's points, which break ties in game value.
        let mut totals = vec![(0i64, 0i64); candidates.len()];
        let mut sampled = 0;
        for _ in 0..self.worlds {
            let Some(world) = world_with(view, &mut rng, banked) else {
                continue;
            };
            sampled += 1;
            for (total, action) in totals.iter_mut().zip(&candidates) {
                let (value, margin) = if for_the_game {
                    self.game_value_of(&world, view.side, action, remembered, &mut solver)
                } else {
                    let points = self.value_of(&world, view.seat, action, remembered, &mut solver);
                    (i64::from(points), 0)
                };
                total.0 += value;
                total.1 += margin;
            }
        }
        if sampled == 0 {
            return usual;
        }
        // Rung 4's own choice wins ties.
        let mut best = candidates.iter().position(|a| *a == usual).unwrap_or(0);
        for (i, &total) in totals.iter().enumerate() {
            if total > totals[best] {
                best = i;
            }
        }
        candidates[best].clone()
    }
}

/// The cards played before a world built from `view` begins, counted by
/// type: every trick taken, but for the one the world replays in the turn of
/// declaring. Both players saw them, so both rollout players recall them.
fn before_the_world(view: &View) -> [u8; 32] {
    let replayed = usize::from(view.phase == Phase::Declare);
    let taken = view.tricks.len().saturating_sub(replayed);
    let mut counts = [0u8; 32];
    for trick in &view.tricks[..taken] {
        for card in [trick.led, trick.second] {
            counts[usize::from(card.card_type().0)] += 1;
        }
    }
    counts
}

/// A game consistent with `view`, its hidden cards dealt at random: the
/// opponent's unseen cards and the face-down stock, from the cards the
/// player cannot account for. The hand starts afresh at this point (its log
/// empty), so scores in it are points from here on. `None` if the view
/// admits no such position, which should not happen.
pub fn world(view: &View, rng: &mut Rng) -> Option<Game> {
    world_with(view, rng, [0, 0])
}

/// [`race::value`], where it is built in.
#[cfg(feature = "race")]
fn race_value(mine: u32, theirs: u32, i_deal: bool) -> f64 {
    race::value(mine, theirs, i_deal)
}

/// Without the race model rung 6 never plays for the game
/// (`Lookahead::for_the_game`), so this is never asked.
#[cfg(not(feature = "race"))]
fn race_value(_: u32, _: u32, _: bool) -> f64 {
    unreachable!("playing for the game needs the race feature")
}

/// The totals a world's game starts from, by side, when playing for the
/// game: the real totals, less what replaying the trick just taken (in the
/// turn of declaring) will score again.
fn game_banked(view: &View) -> [u32; 2] {
    let (mut mine, mut theirs) = view.totals;
    if view.phase == Phase::Declare {
        if let Some(last) = view.tricks.last() {
            let brisques = 10
                * [last.led, last.second]
                    .iter()
                    .filter(|c| c.is_brisque())
                    .count() as u32;
            if last.winner == view.seat {
                mine -= brisques;
            } else {
                theirs -= brisques;
            }
        }
    }
    let mut banked = [0, 0];
    banked[view.side.index()] = mine;
    banked[view.side.other().index()] = theirs;
    banked
}

/// [`world`], its game starting from `banked` points by side.
pub fn world_with(view: &View, rng: &mut Rng, banked: [u32; 2]) -> Option<Game> {
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
    let mut holdings = [Holdings::default(), Holdings::default()];
    holdings[view.seat.index()] = view.mine.clone();
    holdings[other.index()] = theirs;
    // The world starts at the start of the trick in progress, or, in the
    // turn of declaring, of the trick just taken, and replays it: its cards
    // go back to the hands that played them.
    let mut replay = Vec::new();
    let mut leader = view.leader;
    let mut tricks = view.tricks.len();
    if let Some(led) = view.led {
        replay.push((leader, led));
    } else if view.phase == Phase::Declare {
        let last = view.tricks.last()?;
        leader = last.leader;
        tricks -= 1;
        replay.push((last.leader, last.led));
        replay.push((last.leader.other(), last.second));
    }
    for &(seat, card) in &replay {
        let h = &mut holdings[seat.index()];
        // A card in its player's announcements was played from the table.
        if h.announced.iter().any(|a| a.declared.cards.contains(card)) {
            h.shown = h.shown.with(card);
        } else {
            h.concealed = h.concealed.with(card);
        }
    }
    let stock = Stock::new(face_down, view.turn_up);
    let hand = Hand::arranged(view.trump, stock, holdings, leader, tricks as u8)
        .ok()?
        .with_sevens_scored(sevens_scored(view));
    let dealer = if view.seat == Seat::Dealer {
        view.side
    } else {
        view.side.other()
    };
    let mut game = Game::arranged(0, banked, dealer, hand).ok()?;
    for (_, card) in replay {
        game.apply(Action::Play(card)).ok()?;
    }
    if game.to_move() != Some(view.side) || game.hand().phase() != view.phase {
        return None;
    }
    Some(game)
}

/// The sevens of trumps that have scored, exactly. While the stock lasts a
/// seven of trumps scores only by being turned up at the deal, exchanged for
/// the turn-up (which it then becomes, until the last card is drawn), or
/// shown when the turn-up is already the other seven. So if the turn-up is
/// not a seven of trumps, none has scored; if it is, it has, and its twin
/// too when two sevens appear in the log.
fn sevens_scored(view: &View) -> CardSet {
    let Some(turn_up) = view.turn_up else {
        return CardSet::EMPTY;
    };
    if turn_up.rank() != Rank::SEVEN || turn_up.suit() != view.trump {
        return CardSet::EMPTY;
    }
    let scored = view
        .log
        .iter()
        .filter(|s| matches!(s.source, Source::Dix | Source::TurnedSeven))
        .count();
    if scored >= 2 {
        turn_up.card_type().twins()
    } else {
        CardSet::EMPTY.with(turn_up)
    }
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
                if view.turn_up.is_some() && view.stock <= 7 {
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

    /// The player at the root of a world sees what it sees in the real
    /// game, its memory included: rung 4 there chooses as rung 4 does in the
    /// real game. (Without the cards played before the world, recalled by
    /// type, it chose differently in about one decision in twenty-five.)
    #[test]
    fn in_a_world_rung_4_chooses_as_in_the_real_game() {
        let mut rng = Rng::seeded(3);
        let la = Lookahead::default();
        let mut checked = 0;
        for (game, view) in positions(30) {
            let legal = game.legal();
            if view.phase == Phase::Declare || legal.len() < 2 {
                continue;
            }
            let real = rung(4).choose(&view, &legal);
            let world = world(&view, &mut rng).unwrap();
            let seen = View::of(&world, view.side);
            let mut four = la.rung4(before_the_world(&view));
            assert_eq!(four.choose(&seen, &world.legal()), real, "{view:?}");
            checked += 1;
        }
        assert!(checked > 150, "{checked}");
    }

    /// Positions near the end of the stock in the later hands of games
    /// between two rung-4 players, where either is within reach of 1,000.
    #[cfg(feature = "race")]
    fn late_positions(games: u64) -> Vec<(Game, View)> {
        let mut out = Vec::new();
        for seed in 0..games {
            let mut game = Game::new(70_000 + seed, Side::A);
            let mut agents = [rung(4), rung(4)];
            while let Some(side) = game.to_move() {
                let view = View::of(&game, side);
                if view.turn_up.is_some()
                    && view.stock <= 9
                    && view.totals.0.max(view.totals.1) + NEAR >= GAME
                {
                    out.push((game.clone(), view.clone()));
                }
                let action = agents[side.index()].choose(&view, &game.legal());
                game.apply(action).unwrap();
            }
        }
        out
    }

    #[test]
    #[cfg(feature = "race")]
    fn far_from_the_end_rung_6_plays_as_rung_5() {
        let mut six = Lookahead {
            objective: Objective::Game,
            ..Lookahead::default()
        };
        let mut five = rung(5);
        let far: Vec<_> = positions(4)
            .into_iter()
            .filter(|(_, v)| v.totals.0.max(v.totals.1) + NEAR < GAME)
            .collect();
        assert!(far.len() > 50, "{}", far.len());
        for (game, view) in far {
            let legal = game.legal();
            assert_eq!(six.choose(&view, &legal), five.choose(&view, &legal));
        }
    }

    #[test]
    #[cfg(feature = "race")]
    fn near_the_end_rung_6_plays_legally_and_the_same_every_time() {
        let positions = late_positions(12);
        assert!(positions.len() > 30, "{}", positions.len());
        let mut six = Lookahead {
            objective: Objective::Game,
            ..Lookahead::default()
        };
        let mut differs = 0;
        for (game, view) in positions
            .iter()
            .filter(|(g, _)| g.legal().len() > 1)
            .take(60)
        {
            let legal = game.legal();
            let first = six.choose(view, &legal);
            assert!(legal.contains(&first));
            assert_eq!(six.choose(view, &legal), first);
            differs += usize::from(rung(5).choose(view, &legal) != first);
        }
        // Playing for the game is not the same as playing for points.
        assert!(differs > 0);
    }

    /// A world built to play for the game starts from the real totals, in
    /// every phase, the replayed trick's brisques allowed for.
    #[test]
    #[cfg(feature = "race")]
    fn a_world_for_the_game_starts_from_the_real_totals() {
        let mut rng = Rng::seeded(5);
        for (_, view) in late_positions(12) {
            let world = world_with(&view, &mut rng, game_banked(&view)).unwrap();
            assert_eq!(
                View::of(&world, view.side).totals,
                view.totals,
                "{:?}",
                view.phase
            );
        }
    }

    #[test]
    fn it_plays_a_legal_card_and_the_same_one_every_time() {
        for (game, view) in positions(6) {
            let legal = game.legal();
            let mut agent = Lookahead {
                declarations: true,
                ..Lookahead::default()
            };
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
