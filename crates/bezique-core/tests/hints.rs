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
                let card = |name: &str| {
                    reason.params.iter().find_map(|(k, p)| match p {
                        Param::Card(c) if *k == name => Some(c.clone()),
                        _ => None,
                    })
                };
                match reason.key {
                    // The guard names a card other than the one it plays, and the
                    // master lead is an ace or ten with no higher card of its suit
                    // named as the card played.
                    "nothing_decisive" => assert_ne!(card("card"), card("kept")),
                    "lead_master_brisque" => {
                        let c = card("card").unwrap();
                        assert!(c.starts_with('T') || c.starts_with('A'), "{c}");
                    }
                    "take_sparing_pieces" => assert_ne!(card("card"), card("kept")),
                    "solved_endgame" => {
                        assert!(reason.params.iter().any(|(k, _)| *k == "count"));
                    }
                    _ => {}
                }
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
    // About 1 in 65 in the 10-seed run; the old catch-alls are gone.
    assert!(
        generic * 30 < hints,
        "too many unclassified: {generic} of {hints}"
    );
    assert!(!counts.contains_key("lead_cheapest"), "{counts:?}");
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
