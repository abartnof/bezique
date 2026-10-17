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

/// Every string in a JSON value, and the code of every card object, with
/// the events kept to those of hand `hand` (earlier hands' cards are no
/// longer hidden) and the record left out (it names the person's own ids).
fn said_in(value: &Value, hand: u64, out: &mut Vec<String>) {
    match value {
        Value::String(s) => out.push(s.clone()),
        Value::Object(map) => {
            for (key, v) in map {
                match key.as_str() {
                    "record" => {}
                    "events" => {
                        for e in v.as_array().into_iter().flatten() {
                            if e["hand"].as_u64() == Some(hand) {
                                said_in(e, hand, out);
                            }
                        }
                    }
                    _ => said_in(v, hand, out),
                }
            }
        }
        Value::Array(items) => items.iter().for_each(|v| said_in(v, hand, out)),
        _ => {}
    }
}

/// Whether `text` names `code` as a word ("the KH", not "KHAKI").
fn names(text: &str, code: &str) -> bool {
    text.match_indices(code).any(|(i, _)| {
        let before = text[..i].chars().next_back();
        let after = text[i + code.len()..].chars().next();
        !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
    })
}

/// The cards the person cannot know of: the opponent's unseen ones and the
/// face-down stock.
fn hidden(session: &bezique_core::session::Session) -> CardSet {
    let game = session.game().expect("a game");
    let hand = game.hand();
    let theirs = game.seat_of(Side::B);
    let mut hidden = hand.holdings(theirs).concealed.minus(hand.known(theirs));
    let mut stock = hand.stock().clone();
    while stock.face_down() > 0 {
        hidden = hidden.with(stock.draw().unwrap());
    }
    hidden
}

/// No state shows more cards of a type than the person could know of: their
/// own, and those visible or seen. Hidden cards (the opponent's unseen ones
/// and the face-down stock) never appear, counted type by type; and a type
/// whose two copies are both hidden is named nowhere this hand, in any
/// string, the narration and the error included.
#[test]
fn no_state_shows_a_hidden_card() {
    for seed in 10..16 {
        let mut session = sit_down(3, seed, 0, 0);
        assert!(session.send("set tracker on"));
        loop {
            let json = state(&session);
            let s: Value = serde_json::from_str(&json).unwrap();
            if let Some(game) = session.game() {
                let mine = game.seat_of(Side::A);
                let hidden = hidden(&session);
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
                let mut said = Vec::new();
                said_in(&s, s["hand_number"].as_u64().unwrap(), &mut said);
                for card_type in CardType::all().filter(|&t| hidden.count_of(t) == 2) {
                    let code = card_type.code();
                    if let Some(text) = said.iter().find(|t| names(t, &code)) {
                        panic!("seed {seed}: {code} is wholly hidden but the state says {text:?}");
                    }
                }
            }
            let Some(command) = first(&s) else { break };
            assert!(session.send(&command));
        }
    }
}

/// The checks above catch leaks planted in real states: a hidden card told
/// in an event, the hidden hand spelled out in the error.
#[test]
fn the_leak_checks_catch_planted_leaks() {
    let mut session = sit_down(3, 12, 0, 0);
    for _ in 0..10 {
        let s: Value = serde_json::from_str(&state(&session)).unwrap();
        session.send(&first(&s).unwrap());
    }
    let hidden = hidden(&session);
    let wholly = CardType::all()
        .find(|&t| hidden.count_of(t) == 2)
        .expect("a type wholly hidden")
        .code();
    let s: Value = serde_json::from_str(&state(&session)).unwrap();
    let hand = s["hand_number"].as_u64().unwrap();
    for plant in 0..2 {
        let mut leaked = s.clone();
        if plant == 0 {
            leaked["events"].as_array_mut().unwrap().push(serde_json::json!({
                "kind": "drew", "hand": hand, "text": format!("Your opponent holds the {wholly}.")
            }));
        } else {
            leaked["error"] = Value::String(format!("they hold {wholly}"));
        }
        let mut said = Vec::new();
        said_in(&leaked, hand, &mut said);
        assert!(
            said.iter().any(|t| names(t, &wholly)),
            "plant {plant} missed"
        );
    }
    // And the honest state says nothing of it.
    let mut said = Vec::new();
    said_in(&s, hand, &mut said);
    assert!(!said.iter().any(|t| names(t, &wholly)));
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

/// The state names the sitting's persona, and a persona number (from 1)
/// picks the one at that place in the list clients read.
#[test]
fn the_state_names_the_persona_from_the_list() {
    let list: Value = serde_json::from_str(&bezique_core::opponent::personas_json()).unwrap();
    let list = list.as_array().unwrap();
    assert!(list.len() >= 4);
    for (i, p) in list.iter().enumerate() {
        let session = sit_down(2, 5, i as u32 + 1, 0);
        let s: Value = serde_json::from_str(&state(&session)).unwrap();
        assert_eq!(s["persona"], p["key"]);
    }
}
