//! Every hint is classified, none panics, and the hinted command is legal.

use bezique_core::hints::{Param, KEYS};
use bezique_core::session::{Prompt, Session, Settings};
use std::collections::BTreeMap;

fn tally(seeds: std::ops::Range<u64>, tally: &mut BTreeMap<&'static str, usize>) -> usize {
    let mut hints = 0;
    for seed in seeds {
        let mut s = Session::new(
            seed,
            Settings {
                level: 2,
                ..Settings::default()
            },
        );
        assert!(s.send("set hints on"));
        let mut step = 0usize;
        loop {
            step += 1;
            assert!(step < 3000);
            if let Some((command, text, reason)) = s.advice() {
                hints += 1;
                assert!(!text.is_empty());
                assert!(KEYS.contains(&reason.key), "unlisted key {}", reason.key);
                for (_, p) in &reason.params {
                    if let Param::Card(c) = p {
                        assert!(c.len() == 2, "{c}");
                    }
                }
                let mut probe = Session::replay(seed, s.settings().clone(), s.record()).unwrap();
                assert!(probe.send(&command), "seed {seed}: {command}");
                *tally.entry(reason.key).or_default() += 1;
            }
            // Follow the hint on every third decision, else a varied legal card,
            // so the positions are not all the top rung's own.
            let command = match s.prompt() {
                Prompt::ChooseDealer => "dealer them".to_string(),
                Prompt::Play { legal } => {
                    format!("play {}", legal[(step * 7 + seed as usize) % legal.len()].0)
                }
                Prompt::Declare { options } => {
                    format!("declare {}", (step + seed as usize) % options.len())
                }
                Prompt::Over => break,
            };
            assert!(s.send(&command), "{command}");
        }
    }
    hints
}

#[test]
fn every_hint_is_classified_and_legal() {
    let mut counts = BTreeMap::new();
    let hints = tally(0..10, &mut counts);
    assert!(hints > 300, "{hints}");
    let generic = counts.get("generic").copied().unwrap_or(0);
    println!("{hints} hints: {counts:?}");
    assert!(
        generic * 10 < hints,
        "too many unclassified: {generic} of {hints}"
    );
    // The common cases all occur.
    for key in [
        "take_brisque",
        "declare_now",
        "lead_low_nonbrisque",
        "throw_lowest",
    ] {
        assert!(counts.contains_key(key), "never {key}: {counts:?}");
    }
}
