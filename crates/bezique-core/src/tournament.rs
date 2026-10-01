//! Measuring agents against each other, always in mirrored pairs.
//!
//! Deal luck swamps skill, so every deal is played twice, the agents changing
//! places, and only the difference counts (`docs/DESIGN.md` §10.5). Two
//! measures:
//!
//! - **Games**: whole games to 1,000. The unit is the pair: one agent wins both
//!   games, or they split.
//! - **Hands**: single hands, by points. Each pair gives the first agent's
//!   points minus the second's, summed over both seatings, which cancels the
//!   cards and leaves the play. Far more sensitive than games.

use crate::deal::Seat;
use crate::game::{Game, Side};
use crate::observation::{Agent, View};

/// Builds a fresh agent for one game, given the game's seed: agents that draw
/// random numbers take their streams from it, so a run is reproducible.
pub type Make<'a> = &'a dyn Fn(u64) -> Box<dyn Agent>;

/// Mirrored games between two agents.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Games {
    /// Games won by each agent.
    pub wins: [u32; 2],
    /// Doubles won by each agent (the loser under 500).
    pub doubles: [u32; 2],
    /// Pairs in which one agent won both games.
    pub swept: [u32; 2],
    /// Pairs split one each.
    pub split: u32,
    /// Hands played in all.
    pub hands: u32,
}

impl Games {
    pub fn pairs(&self) -> u32 {
        self.swept[0] + self.swept[1] + self.split
    }

    /// The first agent's share of games won.
    pub fn share(&self) -> f64 {
        f64::from(self.wins[0]) / f64::from(self.wins[0] + self.wins[1])
    }

    /// The standard error of `share`, from the spread of the pairs (each pair
    /// scores 1, ½ or 0 for the first agent).
    pub fn standard_error(&self) -> f64 {
        let scores = std::iter::repeat_n(1.0, self.swept[0] as usize)
            .chain(std::iter::repeat_n(0.0, self.swept[1] as usize))
            .chain(std::iter::repeat_n(0.5, self.split as usize));
        standard_error(scores)
    }
}

/// Plays `pairs` mirrored pairs of games from consecutive seeds starting at
/// `first_seed`.
pub fn games(first_seed: u64, pairs: u64, a: Make, b: Make) -> Games {
    let mut result = Games::default();
    for seed in first_seed..first_seed + pairs {
        let mut first_won = 0;
        for swap in [false, true] {
            let mut agents = if swap {
                [b(seed), a(seed)]
            } else {
                [a(seed), b(seed)]
            };
            let mut game = Game::new(seed, Side::A);
            play(&mut game, &mut agents, |_| false);
            let outcome = game.outcome().expect("played to the end");
            let agent = usize::from((outcome.winner == Side::A) == swap);
            result.wins[agent] += 1;
            result.doubles[agent] += u32::from(outcome.double);
            result.hands += game.hand_number();
            first_won += u32::from(agent == 0);
        }
        match first_won {
            2 => result.swept[0] += 1,
            0 => result.swept[1] += 1,
            _ => result.split += 1,
        }
    }
    result
}

/// Mirrored single hands between two agents.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Hands {
    pub pairs: u32,
    /// The first agent's points minus the second's, summed over each pair.
    pub margins: Vec<i64>,
}

impl Hands {
    /// The first agent's mean margin per hand (half a pair).
    pub fn mean(&self) -> f64 {
        let n = self.margins.len().max(1) as f64;
        self.margins.iter().map(|&m| m as f64 / 2.0).sum::<f64>() / n
    }

    /// The standard error of `mean`.
    pub fn standard_error(&self) -> f64 {
        standard_error(self.margins.iter().map(|&m| m as f64 / 2.0))
    }
}

/// Plays `pairs` mirrored pairs of single hands (the first hand of a game,
/// from scores of nothing).
pub fn hands(first_seed: u64, pairs: u64, a: Make, b: Make) -> Hands {
    let mut result = Hands::default();
    for seed in first_seed..first_seed + pairs {
        let mut margin = 0i64;
        for swap in [false, true] {
            let mut agents = if swap {
                [b(seed), a(seed)]
            } else {
                [a(seed), b(seed)]
            };
            let mut game = Game::new(seed, Side::A);
            play(&mut game, &mut agents, |g| g.hand_number() > 1);
            let hand = if game.hand_number() > 1 {
                game.previous_hand().expect("the first hand, finished")
            } else {
                game.hand()
            };
            // Side A dealt the first hand.
            let (dealer, pone) = (
                i64::from(hand.score(Seat::Dealer)),
                i64::from(hand.score(Seat::Pone)),
            );
            margin += if swap { pone - dealer } else { dealer - pone };
        }
        result.pairs += 1;
        result.margins.push(margin);
    }
    result
}

/// The standard error of the mean of `values`.
fn standard_error(values: impl Iterator<Item = f64>) -> f64 {
    let values: Vec<f64> = values.collect();
    let n = values.len() as f64;
    if n < 2.0 {
        return 0.0;
    }
    let mean = values.iter().sum::<f64>() / n;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n - 1.0);
    (variance / n).sqrt()
}

/// Plays `game` until it ends or `stop` says so, agent `i` playing
/// `Side::BOTH[i]`.
pub fn play(game: &mut Game, agents: &mut [Box<dyn Agent>; 2], stop: impl Fn(&Game) -> bool) {
    while let Some(side) = game.to_move() {
        if stop(game) {
            return;
        }
        let legal = game.legal();
        let view = View::of(game, side);
        let action = agents[side.index()].choose(&view, &legal);
        game.apply(action).expect("agents choose among legal moves");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::{Legal, Maxims};

    fn maxims(_: u64) -> Box<dyn Agent> {
        Box::new(Maxims::default())
    }

    fn legal(_: u64) -> Box<dyn Agent> {
        Box::new(Legal)
    }

    #[test]
    fn identical_deterministic_agents_split_every_pair() {
        let g = games(0, 6, &maxims, &maxims);
        assert_eq!(g.pairs(), 6);
        assert_eq!(g.split, 6);
        assert_eq!(g.wins, [6, 6]);
        assert_eq!(g.share(), 0.5);
        assert_eq!(g.standard_error(), 0.0);
        assert!(g.hands >= 2 * 6 * 2, "a game is at least two hands");
    }

    #[test]
    fn identical_deterministic_agents_have_no_margin_in_any_hand() {
        let h = hands(0, 20, &maxims, &maxims);
        assert_eq!(h.pairs, 20);
        assert!(h.margins.iter().all(|&m| m == 0), "{:?}", h.margins);
        assert_eq!(h.mean(), 0.0);
    }

    #[test]
    fn the_maxims_have_a_positive_margin_over_legality() {
        let h = hands(100, 200, &maxims, &legal);
        assert!(
            h.mean() > 3.0 * h.standard_error(),
            "mean {} se {}",
            h.mean(),
            h.standard_error()
        );
        let reversed = hands(100, 200, &legal, &maxims);
        assert_eq!(reversed.mean(), -h.mean(), "the measure is antisymmetric");
    }

    #[test]
    fn games_are_counted_consistently() {
        let g = games(0, 10, &maxims, &legal);
        assert_eq!(g.wins[0] + g.wins[1], 20);
        assert_eq!(2 * g.swept[0] + g.split, g.wins[0]);
        assert_eq!(2 * g.swept[1] + g.split, g.wins[1]);
        assert!(g.doubles[0] <= g.wins[0] && g.doubles[1] <= g.wins[1]);
    }
}
