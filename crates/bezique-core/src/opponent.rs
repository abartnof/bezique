//! The opponent a player sits down against: a level, a persona, and how
//! erratic it is (`docs/DESIGN.md` §10.1).
//!
//! - The **level** is the rung of the ladder: what it is capable of.
//! - The **persona** is how it chooses among options of nearly equal worth: a
//!   style taken from the period writers' disagreements. A persona must be a
//!   style, not a handicap, so each is calibrated by mirrored measurement to
//!   cost no more than about two points a hand against the balanced default
//!   (`measurements/README.md`).
//! - **Erraticism** is how consistently it plays to its level: on each
//!   decision it may slip a rung, and each further rung with the same
//!   probability again, as people do, rather than playing uniformly worse.
//!
//! The opponent is never given a name: it is described by how it plays.

use crate::agents::{rung_with, Leaning, Style, TOP};
use crate::hand::Action;
use crate::observation::{Agent, View};
use crate::rng::Rng;

/// A way of playing.
#[derive(Clone, Debug, PartialEq)]
pub struct Persona {
    /// A short key, for settings and measurement.
    pub key: &'static str,
    /// How it plays, in a player's words.
    pub described: &'static str,
    /// Where the advice comes from.
    pub after: &'static str,
    pub style: Style,
}

/// The personas, the balanced one first.
pub fn personas() -> Vec<Persona> {
    let base = Style::default();
    vec![
        Persona {
            key: "balanced",
            described: "plays a balanced game",
            after: "the maxims of Pole, Pardon, Foster and Hoffmann",
            style: base.clone(),
        },
        Persona {
            key: "concentrates",
            described: "throws from its shortest suits, to be void of them",
            after: "Foster: pick one or two suits and discard the rest",
            style: Style {
                leaning: Leaning::Concentrate,
                ..base.clone()
            },
        },
        Persona {
            key: "close",
            described: "plays from the table first, showing you as little as it can",
            after: "Pardon: play a declared card so as not to expose the hand",
            style: Style {
                leaning: Leaning::Close,
                ..base.clone()
            },
        },
        Persona {
            key: "bezique",
            described: "will give almost anything for double bezique",
            after:
                "Pardon: keep the bezique cards until near the end, even after declaring bezique",
            style: Style {
                bezique: base.bezique + 20,
                ..base
            },
        },
    ]
}

/// The persona with this key.
pub fn persona(key: &str) -> Option<Persona> {
    personas().into_iter().find(|p| p.key == key)
}

/// An opponent that plays at `level` most of the time, and on each decision
/// slips a rung with probability `erraticism` (and another with that
/// probability again), never below rung 1.
pub struct Erratic {
    rungs: Vec<Box<dyn Agent>>,
    erraticism: f64,
    rng: Rng,
}

impl Erratic {
    pub fn new(level: u8, persona: &Persona, erraticism: f64, rng: Rng) -> Erratic {
        // Levels above the top would only repeat it, and its slips would
        // land on rungs no weaker.
        let level = level.clamp(1, TOP);
        Erratic {
            rungs: (1..=level)
                .map(|r| rung_with(r, &persona.style))
                .collect(),
            erraticism: erraticism.clamp(0.0, 1.0),
            rng,
        }
    }

    /// The rung it plays this decision at (an index: 0 is rung 1).
    fn draw_rung(&mut self) -> usize {
        let threshold = (self.erraticism * 1_000_000.0) as u64;
        let mut rung = self.rungs.len() - 1;
        while rung > 0 && self.rng.below(1_000_000) < threshold {
            rung -= 1;
        }
        rung
    }
}

impl Erratic {
    /// Makes a decision already known (replaying, for undo) without thinking
    /// it again: only the draw of the rung, so that its stream stays where
    /// it would have been. The rungs themselves keep no state.
    pub fn replay(&mut self) {
        self.draw_rung();
    }
}

impl Agent for Erratic {
    fn choose(&mut self, view: &View, legal: &[Action]) -> Action {
        let rung = self.draw_rung();
        self.rungs[rung].choose(view, legal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Game, Side};
    use crate::rng::purpose;

    #[test]
    fn a_level_past_the_top_is_the_top() {
        let p = persona("balanced").unwrap();
        let high = Erratic::new(9, &p, 0.5, Rng::seeded(1));
        assert_eq!(high.rungs.len(), usize::from(TOP));
        assert_eq!(Erratic::new(0, &p, 0.5, Rng::seeded(1)).rungs.len(), 1);
    }

    #[test]
    fn the_personas_are_distinct_and_described() {
        let all = personas();
        assert!(all.len() >= 4);
        assert_eq!(all[0].key, "balanced");
        assert_eq!(all[0].style, Style::default());
        for (i, p) in all.iter().enumerate() {
            assert!(!p.described.is_empty() && !p.after.is_empty(), "{}", p.key);
            for q in &all[i + 1..] {
                assert_ne!(p.key, q.key);
                assert_ne!(p.style, q.style, "{} and {} play alike", p.key, q.key);
            }
        }
        assert_eq!(persona("balanced").unwrap().key, "balanced");
        assert!(persona("no such").is_none());
    }

    /// Plays a game with these agents and returns every move made.
    fn moves_of(seed: u64, mut a: Box<dyn Agent>, mut b: Box<dyn Agent>) -> Vec<Action> {
        let mut game = Game::new(seed, Side::A);
        let mut made = Vec::new();
        while let Some(side) = game.to_move() {
            let view = View::of(&game, side);
            let agent = if side == Side::A { &mut a } else { &mut b };
            let action = agent.choose(&view, &game.legal());
            made.push(action.clone());
            game.apply(action).unwrap();
        }
        made
    }

    #[test]
    fn with_no_erraticism_it_plays_exactly_as_its_rung() {
        let balanced = persona("balanced").unwrap();
        let steady = Erratic::new(3, &balanced, 0.0, Rng::stream(1, purpose::agent(0)));
        let plain = rung_with(3, &balanced.style);
        assert_eq!(
            moves_of(4, Box::new(steady), rung_with(2, &balanced.style)),
            moves_of(4, plain, rung_with(2, &balanced.style))
        );
    }

    #[test]
    fn an_erratic_opponent_is_reproducible_and_does_slip() {
        let balanced = persona("balanced").unwrap();
        let make = || {
            Box::new(Erratic::new(
                3,
                &balanced,
                0.4,
                Rng::stream(9, purpose::agent(0)),
            )) as Box<dyn Agent>
        };
        let first = moves_of(5, make(), rung_with(3, &balanced.style));
        assert_eq!(first, moves_of(5, make(), rung_with(3, &balanced.style)));
        let steady = moves_of(
            5,
            rung_with(3, &balanced.style),
            rung_with(3, &balanced.style),
        );
        assert_ne!(first, steady, "at 0.4 it should slip somewhere in a game");
    }

    #[test]
    fn slips_never_go_below_rung_one() {
        let balanced = persona("balanced").unwrap();
        let mut e = Erratic::new(2, &balanced, 0.99, Rng::seeded(3));
        for _ in 0..1000 {
            assert!(e.draw_rung() < 2);
        }
    }
}
