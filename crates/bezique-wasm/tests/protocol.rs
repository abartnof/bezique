//! The protocol, driven as a client drives it: commands in, JSON out.

use bezique_core::cards::{CardSet, CardType};
use bezique_core::game::Side;
use bezique_wasm::{sit_down, state};
use serde_json::Value;

/// The command a scripted client sends: the first thing offered.
fn first(state: &Value) -> Option<String> {
    let prompt = &state["prompt"];
    match prompt["kind"].as_str()? {
        "choose_dealer" => Some("dealer you".to_string()),
        "play" => Some(format!("play {}", prompt["legal"][0])),
        "declare" => Some(format!(
            "declare {}",
            prompt["options"].as_array()?.len() - 1
        )),
        _ => None,
    }
}

/// Every card object (`{"id", "card"}`) in a JSON value, outside the events.
fn cards_in(value: &Value, out: &mut Vec<(u64, String)>) {
    match value {
        Value::Object(map) => {
            if let (Some(id), Some(code)) = (
                map.get("id").and_then(Value::as_u64),
                map.get("card").and_then(Value::as_str),
            ) {
                out.push((id, code.to_string()));
            }
            for (key, v) in map {
                if key != "events" {
                    cards_in(v, out);
                }
            }
        }
        Value::Array(items) => items.iter().for_each(|v| cards_in(v, out)),
        _ => {}
    }
}

#[test]
fn whole_games_through_the_protocol() {
    for seed in 0..6 {
        let mut session = sit_down(4, seed, 0, 100);
        let mut moves = 0;
        loop {
            let s: Value = serde_json::from_str(&state(&session)).expect("the state is JSON");
            assert_eq!(s["protocol"], 1);
            let Some(command) = first(&s) else { break };
            assert!(session.send(&command), "{command}: {}", s["error"]);
            moves += 1;
            assert!(moves < 3000);
        }
        let s: Value = serde_json::from_str(&state(&session)).unwrap();
        assert_eq!(s["phase"], "over");
        assert!(s["outcome"]["winner"].is_string());
    }
}

/// No state shows more cards of a type than the person could know of: their
/// own, and those visible or seen. Hidden cards (the opponent's unseen ones
/// and the face-down stock) never appear, counted type by type.
#[test]
fn no_state_shows_a_hidden_card() {
    for seed in 10..16 {
        let mut session = sit_down(3, seed, 0, 0);
        assert!(session.send("set tracker on"));
        loop {
            let json = state(&session);
            let s: Value = serde_json::from_str(&json).unwrap();
            if let Some(game) = session.game() {
                let hand = game.hand();
                let (mine, theirs) = (game.seat_of(Side::A), game.seat_of(Side::B));
                let hidden_theirs = hand.holdings(theirs).concealed.minus(hand.known(theirs));
                let mut stock = CardSet::EMPTY;
                let mut st = hand.stock().clone();
                while st.face_down() > 0 {
                    stock = stock.with(st.draw().unwrap());
                }
                let hidden = hidden_theirs.union(stock);
                let mut shown = Vec::new();
                cards_in(&s, &mut shown);
                shown.sort();
                shown.dedup();
                for card_type in CardType::all() {
                    let code = card_type.code();
                    let named = shown.iter().filter(|(_, c)| *c == code).count() as u32;
                    let could_know = 2 - hidden.count_of(card_type);
                    assert!(
                        named <= could_know,
                        "seed {seed}: {named} of {code} shown, but only {could_know} can be known ({:?} hidden)",
                        hidden.count_of(card_type)
                    );
                }
                let _ = mine;
            }
            let Some(command) = first(&s) else { break };
            assert!(session.send(&command));
        }
    }
}

#[test]
fn a_refused_command_is_reported_in_the_state() {
    let mut session = sit_down(2, 3, 0, 0);
    assert!(!session.send("play 999"));
    let s: Value = serde_json::from_str(&state(&session)).unwrap();
    assert!(s["error"].is_string());
}

#[test]
fn a_record_reloads_the_sitting() {
    let mut session = sit_down(2, 21, 2, 0);
    for _ in 0..40 {
        let s: Value = serde_json::from_str(&state(&session)).unwrap();
        if let Some(c) = first(&s) {
            session.send(&c);
        }
    }
    let record: Vec<String> = session.record().to_vec();
    let again =
        bezique_core::session::Session::replay(21, session.settings().clone(), &record).unwrap();
    assert_eq!(state(&again), state(&session));
}
