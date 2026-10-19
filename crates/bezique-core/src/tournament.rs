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

/// Builds a fresh agent for one game, given a seed of its own (derived from
/// the game's seed and the agent's place, so two agents never share one):
/// agents that draw random numbers take their streams from it, so a run is
/// reproducible. Shared between threads, so it must be `Sync`.
pub type Make<'a> = &'a (dyn Fn(u64) -> Box<dyn Agent> + Sync);

/// The seed agent `slot` (0 for the first, 1 for the second) is built with
/// for the pair dealt from `seed`: the same in both seatings.
fn agent_seed(seed: u64, slot: u64) -> u64 {
    crate::rng::Rng::stream(seed, crate::rng::purpose::agent(slot as usize)).next_u64()
}

/// `f` of every seed from `first` on, `count` of them, in seed order,
/// computed on every core: the result does not depend on how many.
fn each_seed<T: Send>(first: u64, count: u64, f: impl Fn(u64) -> T + Sync) -> Vec<T> {
    let threads = std::env::var("MEASURE_THREADS")
        .ok()
        .and_then(|s| s.parse().ok())
        .or_else(|| std::thread::available_parallelism().ok().map(usize::from))
        .unwrap_or(1)
        .clamp(1, 64) as u64;
    let f = &f;
    let mut out: Vec<(u64, T)> = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    (t..count)
                        .step_by(threads as usize)
                        .map(|i| (i, f(first + i)))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        workers
            .into_iter()
            .flat_map(|w| w.join().expect("a measuring thread panicked"))
            .collect()
    });
    out.sort_by_key(|&(i, _)| i);
    out.into_iter().map(|(_, t)| t).collect()
}

/// Mirrored games between two agents.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Games {
    /// The seed of the first pair; pairs are dealt from consecutive seeds.
    pub first_seed: u64,
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
    /// For each pair, the first agent's game points minus the second's (a
    /// game won is 1, a double 2): what the race to 1,000 and the double at
    /// 500 are played for.
    pub points: Vec<i64>,
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

    /// The first agent's mean game points a game, a double counting two.
    pub fn points_a_game(&self) -> f64 {
        let n = self.points.len().max(1) as f64;
        self.points.iter().map(|&p| p as f64 / 2.0).sum::<f64>() / n
    }

    /// The standard error of `points_a_game`.
    pub fn points_standard_error(&self) -> f64 {
        standard_error(self.points.iter().map(|&p| p as f64 / 2.0))
    }
}

/// One mirrored pair of games: each seating's winner (as an agent slot) and
/// whether it was a double, and the hands played.
struct PairOfGames {
    winners: [(usize, bool); 2],
    hands: u32,
}

/// Plays `pairs` mirrored pairs of games from consecutive seeds starting at
/// `first_seed`.
pub fn games(first_seed: u64, pairs: u64, a: Make, b: Make) -> Games {
    let played = each_seed(first_seed, pairs, |seed| {
        let mut pair = PairOfGames {
            winners: [(0, false); 2],
            hands: 0,
        };
        for (i, swap) in [false, true].into_iter().enumerate() {
            let (sa, sb) = (agent_seed(seed, 0), agent_seed(seed, 1));
            let mut agents = if swap { [b(sb), a(sa)] } else { [a(sa), b(sb)] };
            let mut game = Game::new(seed, Side::A);
            play(&mut game, &mut agents, |_| false);
            let outcome = game.outcome().expect("played to the end");
            // Agent 0 plays side A unless swapped.
            let agent = usize::from((outcome.winner == Side::A) == swap);
            pair.winners[i] = (agent, outcome.double);
            pair.hands += game.hand_number();
        }
        pair
    });
    let mut result = Games {
        first_seed,
        ..Games::default()
    };
    for pair in played {
        let mut first_won = 0;
        let mut points = 0i64;
        for (agent, double) in pair.winners {
            result.wins[agent] += 1;
            result.doubles[agent] += u32::from(double);
            first_won += u32::from(agent == 0);
            let value = if double { 2 } else { 1 };
            points += if agent == 0 { value } else { -value };
        }
        result.hands += pair.hands;
        result.points.push(points);
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
    /// The seed of the first pair; pairs are dealt from consecutive seeds.
    pub first_seed: u64,
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
    let margins = each_seed(first_seed, pairs, |seed| {
        let mut margin = 0i64;
        for swap in [false, true] {
            let (sa, sb) = (agent_seed(seed, 0), agent_seed(seed, 1));
            let mut agents = if swap { [b(sb), a(sa)] } else { [a(sa), b(sb)] };
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
        margin
    });
    Hands {
        first_seed,
        pairs: margins.len() as u32,
        margins,
    }
}

/// The O'Brien-Fleming constant `c` for `looks` equally spaced looks at a
/// two-sided 5% level: the boundary at look `k` is `z_k = c sqrt(looks/k)`
/// (see `obf_z`). Found by integrating the chance that a standard random
/// walk stays inside `±c sqrt(looks)` for `looks` steps, and bisecting on
/// `c` until that chance is 95%. About 1.96 for one look, 2.04 for five, 2.08
/// for ten, 2.12 for sixteen, 2.15 for thirty-two. Cached.
pub fn obf_constant(looks: u64) -> f64 {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    static CACHE: OnceLock<Mutex<HashMap<u64, f64>>> = OnceLock::new();
    let looks = looks.max(1);
    let cache = CACHE.get_or_init(Mutex::default);
    if let Some(&c) = cache.lock().expect("cache").get(&looks) {
        return c;
    }
    let (mut lo, mut hi) = (1.9f64, 2.6f64);
    for _ in 0..24 {
        let mid = (lo + hi) / 2.0;
        if walk_crossing_chance(mid, looks) > 0.05 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let c = (lo + hi) / 2.0;
    cache.lock().expect("cache").insert(looks, c);
    c
}

/// The chance that the sum of `k` standard normals leaves `±c sqrt(looks)`
/// at any of the steps `1..=looks`, by stepping its density on a grid.
fn walk_crossing_chance(c: f64, looks: u64) -> f64 {
    let bound = c * (looks as f64).sqrt();
    let n = (2.0 * bound / 0.07).ceil() as usize;
    let h = 2.0 * bound / n as f64;
    let x: Vec<f64> = (0..n).map(|i| -bound + (i as f64 + 0.5) * h).collect();
    let pdf = |d: f64| (-0.5 * d * d).exp() / (2.0 * std::f64::consts::PI).sqrt();
    let reach = (6.5 / h) as usize;
    let mut density: Vec<f64> = x.iter().map(|&v| pdf(v)).collect();
    for _ in 1..looks {
        let mut next = vec![0.0; n];
        for (i, out) in next.iter_mut().enumerate() {
            let from = i.saturating_sub(reach);
            let to = (i + reach + 1).min(n);
            *out = (from..to)
                .map(|j| density[j] * pdf(x[i] - x[j]) * h)
                .sum();
        }
        density = next;
    }
    1.0 - density.iter().sum::<f64>() * h
}

/// The two-sided boundary at look `k` of `looks`: `c sqrt(looks/k)`, which
/// keeps the chance of ever crossing it, over all looks, at 5% when there is
/// no effect. Used for stopping on a clear effect and, as the half-width of
/// the interval, for stopping on no effect worth having.
pub fn obf_z(looks: u64, k: u64) -> f64 {
    let looks = looks.max(1);
    obf_constant(looks) * (looks as f64 / k.clamp(1, looks) as f64).sqrt()
}

/// A sequential design, fixed before a run starts (the project's policy:
/// stop as soon as the answer is clear, never run an enormous n for its own
/// sake). Pairs are played in batches, so there are `looks()` = `max / batch`
/// looks at most, the `k`th after `k` batches. After each, with `z_k` the
/// O'Brien-Fleming boundary for that look (`obf_z`), the run stops if the
/// effect is clear (`|mean| >= z_k se`), or if it is clear there is no effect
/// worth having (the interval `mean ± z_k se`, the same width, lies within
/// `±worth`), or at `max` pairs. No look is taken before `min` pairs (the
/// floor); looks before it are skipped, which only makes the rule more
/// conservative.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sequential {
    pub batch: u64,
    pub min: u64,
    pub max: u64,
    /// The smallest effect worth having, in the statistic's units.
    pub worth: f64,
}

/// Why a sequential run stopped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stop {
    /// The effect is clear.
    Clear,
    /// Whatever the effect is, it is smaller than the smallest worth having.
    NothingWorthHaving,
    /// The cap was reached first.
    Cap,
}

impl Sequential {
    /// The most looks a run can take (the ceiling in batches).
    pub fn looks(&self) -> u64 {
        self.max.div_ceil(self.batch.max(1)).max(1)
    }

    /// The look that `pairs` pairs is.
    pub fn look(&self, pairs: u64) -> u64 {
        pairs.div_ceil(self.batch.max(1)).clamp(1, self.looks())
    }

    /// The boundary `z_k` at the look `pairs` pairs is.
    pub fn z(&self, pairs: u64) -> f64 {
        obf_z(self.looks(), self.look(pairs))
    }

    /// Whether to stop, given the statistic's mean and standard error after
    /// `pairs` pairs.
    pub fn stop(&self, pairs: u64, mean: f64, se: f64) -> Option<Stop> {
        if pairs >= self.min && se > 0.0 {
            let z = self.z(pairs);
            if mean.abs() >= z * se {
                return Some(Stop::Clear);
            }
            if mean + z * se < self.worth && mean - z * se > -self.worth {
                return Some(Stop::NothingWorthHaving);
            }
        }
        (pairs >= self.max).then_some(Stop::Cap)
    }

    /// One line for a report: the interval at the last look, its boundary,
    /// the floor, the ceiling and the number used.
    pub fn describe(&self, pairs: u64, mean: f64, se: f64, stop: Stop) -> String {
        let z = self.z(pairs);
        format!(
            "sequential (O'Brien-Fleming, look {} of {}): {mean:+.3} ± {:.3} = [{:+.3}, {:+.3}] at boundary z = {z:.2}; [floor, ceiling, used] = [{}, {}, {pairs}] pairs, batches of {}, no effect worth having within ±{}; stopped: {stop:?}",
            self.look(pairs),
            self.looks(),
            z * se,
            mean - z * se,
            mean + z * se,
            self.min,
            self.max,
            self.batch,
            self.worth
        )
    }
}

/// Mirrored hands, played in batches until `rule` says stop.
pub fn hands_until(first_seed: u64, rule: Sequential, a: Make, b: Make) -> (Hands, Stop) {
    let mut all = Hands {
        first_seed,
        ..Hands::default()
    };
    loop {
        let more = hands(first_seed + u64::from(all.pairs), rule.batch, a, b);
        all.pairs += more.pairs;
        all.margins.extend(more.margins);
        if let Some(stop) = rule.stop(u64::from(all.pairs), all.mean(), all.standard_error()) {
            return (all, stop);
        }
    }
}

/// Mirrored games, played in batches until `rule` says stop, judged by game
/// points a game (a double counting two).
pub fn games_until(first_seed: u64, rule: Sequential, a: Make, b: Make) -> (Games, Stop) {
    let mut all = Games {
        first_seed,
        ..Games::default()
    };
    loop {
        let more = games(first_seed + u64::from(all.pairs()), rule.batch, a, b);
        for i in 0..2 {
            all.wins[i] += more.wins[i];
            all.doubles[i] += more.doubles[i];
            all.swept[i] += more.swept[i];
        }
        all.split += more.split;
        all.hands += more.hands;
        all.points.extend(more.points);
        let pairs = u64::from(all.pairs());
        if let Some(stop) = rule.stop(pairs, all.points_a_game(), all.points_standard_error()) {
            return (all, stop);
        }
    }
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
    use crate::deal::Seat;

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

    /// The maxims beat bare legality, whichever is named first: the measure
    /// credits the right agent.
    #[test]
    fn games_credit_the_right_agent() {
        let g = games(0, 10, &maxims, &legal);
        assert!(g.wins[0] > g.wins[1], "{g:?}");
        assert!(g.points_a_game() > 0.0);
        let reversed = games(0, 10, &legal, &maxims);
        assert_eq!(reversed.wins, [g.wins[1], g.wins[0]]);
        assert_eq!(reversed.points_a_game(), -g.points_a_game());
    }

    /// The hand margin, recounted from each game's own totals: what each side
    /// had banked when hand 2 was dealt. `Game::total` adds the new hand's
    /// score, which is not hand 1's: when hand 2's turn-up is the seven of
    /// trumps its dealer scores 10 at once, so that is taken off again.
    fn banked_margin(seed: u64, swap: bool) -> (i64, bool) {
        let mut agents: [Box<dyn Agent>; 2] = if swap {
            [legal(0), maxims(0)]
        } else {
            [maxims(0), legal(0)]
        };
        let mut game = Game::new(seed, Side::A);
        play(&mut game, &mut agents, |g| g.hand_number() > 1);
        let banked = |side: Side| {
            let new_hand = if game.hand_number() > 1 {
                game.hand().score(game.seat_of(side))
            } else {
                0
            };
            i64::from(game.total(side)) - i64::from(new_hand)
        };
        let seven = game.hand_number() > 1
            && (game.hand().score(Seat::Dealer) > 0 || game.hand().score(Seat::Pone) > 0);
        // The maxims played side A unless swapped.
        let (a, b) = (banked(Side::A), banked(Side::B));
        (if swap { b - a } else { a - b }, seven)
    }

    #[test]
    fn hand_margins_match_an_independent_count() {
        let h = hands(40, 30, &maxims, &legal);
        for (i, &margin) in h.margins.iter().enumerate() {
            let seed = 40 + i as u64;
            let expected: i64 = [false, true].iter().map(|&s| banked_margin(seed, s).0).sum();
            assert_eq!(margin, expected, "seed {seed}");
        }
    }

    /// Seeds whose second hand turns up the seven of trumps (10 to its dealer
    /// before a card is played) are counted by hand 1 alone.
    #[test]
    fn a_turned_up_seven_is_not_part_of_the_first_hand() {
        let hit: Vec<u64> = (0..400)
            .filter(|&seed| banked_margin(seed, false).1)
            .take(3)
            .collect();
        assert!(!hit.is_empty(), "no seed in 0..400 turns up the seven of trumps");
        for seed in hit {
            let h = hands(seed, 1, &maxims, &legal);
            let expected: i64 = [false, true].iter().map(|&s| banked_margin(seed, s).0).sum();
            assert_eq!(h.margins[0], expected, "seed {seed}");
            // The naive total would have counted the 10.
            let mut agents: [Box<dyn Agent>; 2] = [maxims(0), legal(0)];
            let mut game = Game::new(seed, Side::A);
            play(&mut game, &mut agents, |g| g.hand_number() > 1);
            let naive = i64::from(game.total(Side::A)) - i64::from(game.total(Side::B));
            assert_ne!(naive, banked_margin(seed, false).0, "seed {seed}");
        }
    }

    #[test]
    fn a_sequential_run_stops_when_the_answer_is_clear() {
        let rule = Sequential {
            batch: 20,
            min: 40,
            max: 100,
            worth: 1.0,
        };
        // The maxims against bare legality: clear at the first look.
        let (h, stop) = hands_until(100, rule, &maxims, &legal);
        assert_eq!((stop, h.pairs), (Stop::Clear, 40));
        // An agent against itself: no effect at all, and no variance; the
        // cap ends it.
        let (h, stop) = hands_until(100, rule, &maxims, &maxims);
        assert_eq!((stop, h.pairs), (Stop::Cap, 100));
        // The rule itself (K = 5: z_5 = 2.04, z_3 = 2.63).
        assert_eq!(rule.looks(), 5);
        assert_eq!(rule.stop(100, 0.1, 0.2), Some(Stop::NothingWorthHaving));
        assert_eq!(rule.stop(80, 0.1, 0.5), None, "an interval of ±1.0 is too wide");
        assert_eq!(rule.stop(60, 0.5, 0.2), None);
        assert_eq!(rule.stop(60, 1.4, 0.5), Some(Stop::Clear));
        assert_eq!(rule.stop(30, 9.0, 1.0), None, "no look before the minimum");
        // Early looks are held to a higher boundary than late ones.
        assert!(rule.z(40) > rule.z(60) && rule.z(60) > rule.z(100));
        assert!((rule.z(100) - obf_constant(5)).abs() < 1e-12);
    }

    /// The constants agree with the published O'Brien-Fleming values (Jennison
    /// and Turnbull: 1.96 for one look, 2.04 for five, 2.08 for ten) and
    /// with an independent calculation (2.116 for sixteen, 2.146 for 32).
    #[test]
    fn the_obrien_fleming_constants_are_the_published_ones() {
        for (looks, expected) in [(1, 1.96), (2, 1.977), (5, 2.040), (10, 2.083), (16, 2.116)] {
            let c = obf_constant(looks);
            assert!((c - expected).abs() < 0.006, "K = {looks}: {c}");
        }
    }

    /// `n` runs of an effect of `effect` (unit variance, so `se = 1/sqrt(k)`
    /// after `k` batches of one), the rule applied at every look: how often
    /// each way of stopping happens.
    fn stops(rule: Sequential, effect: f64, n: u64) -> (f64, f64) {
        let mut rng = crate::rng::Rng::seeded(2026);
        let mut normal = move || {
            let unit = |r: &mut crate::rng::Rng| (r.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
            let (u, v) = (1.0 - unit(&mut rng), unit(&mut rng));
            (-2.0 * u.ln()).sqrt() * (2.0 * std::f64::consts::PI * v).cos()
        };
        let (mut clear, mut nothing) = (0u64, 0u64);
        for _ in 0..n {
            let mut sum = 0.0;
            for k in 1..=rule.max {
                sum += effect + normal();
                match rule.stop(k, sum / k as f64, 1.0 / (k as f64).sqrt()) {
                    Some(Stop::Clear) => {
                        clear += 1;
                        break;
                    }
                    Some(Stop::NothingWorthHaving) => {
                        nothing += 1;
                        break;
                    }
                    _ => {}
                }
            }
        }
        (clear as f64 / n as f64, nothing as f64 / n as f64)
    }

    /// With no effect, the chance of ever calling one clear, over all the
    /// looks, is the 5% the boundary is built for.
    #[test]
    fn the_boundary_holds_its_type_one_error_under_the_null() {
        for looks in [5, 16, 40] {
            let rule = Sequential {
                batch: 1,
                min: 1,
                max: looks,
                worth: 0.0,
            };
            let (clear, _) = stops(rule, 0.0, 20_000);
            assert!((0.035..0.062).contains(&clear), "K = {looks}: {clear}");
        }
    }

    /// With an effect exactly as big as the smallest worth having, the chance
    /// of ever declaring "nothing worth having" is held to one tail of 5%;
    /// a fixed two-standard-error interval looked at every batch is not.
    #[test]
    fn the_futility_stop_holds_its_error_at_the_edge_of_worth() {
        for looks in [5, 16, 40] {
            let rule = Sequential {
                batch: 1,
                min: 1,
                max: looks,
                worth: 1.0,
            };
            let (_, nothing) = stops(rule, rule.worth, 20_000);
            assert!(nothing < 0.035, "K = {looks}: {nothing}");
        }
        // And it does stop when the answer is plainly none: no effect, a
        // wide tolerance.
        let rule = Sequential {
            batch: 1,
            min: 1,
            max: 16,
            worth: 3.0,
        };
        let (clear, nothing) = stops(rule, 0.0, 2000);
        assert!(clear < 0.01 && nothing > 0.9, "{clear} {nothing}");
    }

    /// Batches follow one another with no seed missed or repeated: a
    /// sequential run that goes to its cap is the plain run of as many pairs.
    #[test]
    fn sequential_games_cover_the_same_seeds_as_a_plain_run() {
        let rule = Sequential {
            batch: 3,
            min: 9,
            max: 9,
            worth: 0.0,
        };
        let (g, _) = games_until(30, rule, &maxims, &legal);
        let plain = games(30, 9, &maxims, &legal);
        assert_eq!(g.pairs(), 9, "it went to the cap, or the look at the cap");
        assert_eq!(
            (g.wins, g.doubles, g.points.clone()),
            (plain.wins, plain.doubles, plain.points)
        );
        let (h, _) = hands_until(30, rule, &maxims, &legal);
        assert_eq!(h.margins, hands(30, 9, &maxims, &legal).margins);
    }

    /// The same results on one thread as on several.
    #[test]
    fn threads_do_not_change_results() {
        let many = hands(7, 16, &maxims, &legal);
        let one: Vec<i64> = each_seed(7, 16, |seed| hands(seed, 1, &maxims, &legal).margins[0]);
        assert_eq!(many.margins, one);
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
