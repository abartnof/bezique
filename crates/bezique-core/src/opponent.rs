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
    /// The archetype's name, for the new-game dialog.
    pub name: &'static str,
    /// How it plays, in a player's words, to follow "it": used in the
    /// opponent's description.
    pub described: &'static str,
    /// One kind sentence for the new-game dialog.
    pub description: &'static str,
    /// Where the advice comes from (`docs/OPPONENT.md`).
    pub after: &'static str,
    pub style: Style,
}

/// The personas, the balanced one first. Personality is orthogonal to skill:
/// a persona only leans among choices of nearly equal worth (a [`Leaning`])
/// or shifts a few preference weights by a few points, and every persona is
/// measured to cost no more than [`TOLERANCE`] points a hand against
/// balanced at the same level (`docs/OPPONENT.md`).
pub fn personas() -> Vec<Persona> {
    let base = Style::default();
    vec![
        Persona {
            key: "balanced",
            name: "The Steady Hand",
            described: "plays a balanced game",
            description: "Plays the old books' maxims evenly, with no marked taste.",
            after: "the maxims of Pole, Pardon, Foster and Hoffmann",
            style: base.clone(),
        },
        Persona {
            key: "collector",
            name: "The Collector",
            described: "hoards kings and queens for marriages and fours",
            description: "Keeps its kings and queens back and loves to declare a marriage.",
            after: "Foster: the marriages and fours are the chief source of score",
            style: Style {
                marriage: base.marriage + 2,
                four: base.four + 2,
                leaning: Leaning::Collect,
                ..base.clone()
            },
        },
        Persona {
            key: "hunter",
            name: "The Brisque Hunter",
            described: "will spend a good card to win a brisque",
            description: "Chases the aces and tens, and will spend a good card to win one.",
            after: "Foster: the brisques decide a close hand, and are worth a good card",
            style: Style {
                take_brisque: base.take_brisque + 20,
                ..base.clone()
            },
        },
        Persona {
            key: "cautious",
            name: "The Careful Hand",
            described: "declares at once and plays from the table, showing you as little as it can",
            description: "Declares what it can at once, and shows you as little of its hand as it can.",
            after: "Pardon: play a declared card so as not to expose the hand",
            style: Style {
                prompt: true,
                leaning: Leaning::Close,
                ..base.clone()
            },
        },
        Persona {
            key: "gambler",
            name: "The Gambler",
            described: "plays for double bezique and the sequence, keeping those cards to the end",
            description: "Plays for the big prizes: double bezique and the sequence.",
            after: "Pardon: keep the bezique cards until near the end, even after declaring bezique",
            style: Style {
                bezique: base.bezique + 6,
                sequence: base.sequence,
                ..base.clone()
            },
        },
        Persona {
            key: "specialist",
            name: "The Specialist",
            described: "throws from its shortest suits, to be void of them",
            description: "Picks a suit or two to build on and sheds the rest.",
            after: "Foster: pick one or two suits and discard the rest",
            style: Style {
                leaning: Leaning::Concentrate,
                ..base
            },
        },
    ]
}

/// The most points a hand a persona may cost against balanced at the same
/// level: the tolerance within which personality is not skill.
pub const TOLERANCE: f64 = 2.0;

/// The personas as JSON for clients (`web3d/personas.json`): key, name and
/// description, in the order of [`personas`] (the protocol's persona
/// numbers are the places in this list from 1).
pub fn personas_json() -> String {
    let rows: Vec<String> = personas()
        .iter()
        .map(|p| {
            format!(
                "  {{\"key\": \"{}\", \"name\": \"{}\", \"description\": \"{}\"}}",
                p.key, p.name, p.description
            )
        })
        .collect();
    format!("[\n{}\n]\n", rows.join(",\n"))
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
            rungs: (1..=level).map(|r| rung_with(r, &persona.style)).collect(),
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

    #[test]
    fn four_to_six_personas_each_named_and_kindly_described() {
        let all = personas();
        assert!((4..=6).contains(&all.len()), "{}", all.len());
        for p in &all {
            assert!(p.name.starts_with("The "), "{}", p.name);
            assert!(
                p.description.ends_with('.') && p.description.len() <= 90,
                "{}",
                p.key
            );
            // The opponent is never named, and personality never talks of
            // strength.
            for word in ["Foster", "Pardon", "Pole", "Cady", "strong", "weak", "clever"] {
                assert!(!p.description.contains(word), "{}: {word}", p.key);
            }
        }
    }

    /// The list clients read is the engine's list (regenerate with
    /// `measure personas-json > web3d/personas.json`).
    #[test]
    fn the_clients_list_of_personas_is_the_engines() {
        assert_eq!(
            include_str!("../../../web3d/personas.json"),
            personas_json()
        );
    }

    /// Personality never touches skill: a persona moves at most two
    /// preference weights, by a few points, and never what a rung can do.
    #[test]
    fn a_persona_moves_a_few_weights_a_few_points() {
        let base = Style::default();
        for p in personas() {
            let s = &p.style;
            let moved = [
                s.trump - base.trump,
                s.bezique - base.bezique,
                s.marriage - base.marriage,
                s.royal - base.royal,
                s.sequence - base.sequence,
                s.four - base.four,
                s.dix - base.dix,
                s.carried - base.carried,
                s.shown - base.shown,
                s.take_brisque - base.take_brisque,
                s.take_to_declare - base.take_to_declare,
                s.urgency - base.urgency,
                s.lead_brisques - base.lead_brisques,
            ];
            assert!(moved.iter().all(|d| d.abs() <= 20), "{}: {moved:?}", p.key);
            assert!(moved.iter().filter(|&&d| d != 0).count() <= 2, "{}", p.key);
        }
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
