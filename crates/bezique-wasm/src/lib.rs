//! The table as a protocol: JSON out, one-line commands in.
//!
//! A client (the browser page, or anything that replaces it) holds no game
//! logic. It asks for the state, draws it, and sends back one command at a
//! time (`bezique_core::session` lists them):
//!
//! ```text
//! dealer you             having cut higher, deal first (or: dealer them)
//! play 23                lead or answer with your card numbered 23
//! declare 2              choose option 2 of the turn of declaring (0: draw)
//! undo                   take back the last decision
//! set hints on           switch an aid: hints, play_forced, tracker
//! ```
//!
//! The state's shape is documented in `docs/PROTOCOL.md`. Every field comes
//! from the person's view, so a client cannot show more than the person at
//! the table could know.
//!
//! Compiled to WebAssembly the crate exports four functions (see [`ffi`]),
//! passing strings through linear memory, with no `wasm-bindgen`.

use bezique_core::cards::{Card, CardSet, Suit};
use bezique_core::game::Side;
use bezique_core::hand::Phase;
use bezique_core::observation::View;
use bezique_core::opponent;
use bezique_core::session::{Event, EventKind, Prompt, Session, Settings};
use bezique_core::words;

/// Bumped whenever the state changes shape in a way a client would notice.
pub const PROTOCOL: u32 = 1;

/// A sitting from the numbers a page can pass: the level, the seed, the
/// persona (0 to draw one, else its place in `opponent::personas`, from 1)
/// and erraticism in thousandths.
pub fn sit_down(level: u32, seed: u32, persona: u32, erraticism_permille: u32) -> Session {
    let personas = opponent::personas();
    let persona = (persona as usize)
        .checked_sub(1)
        .and_then(|i| personas.get(i))
        .map(|p| p.key.to_string());
    Session::new(
        u64::from(seed),
        Settings {
            level: level.clamp(1, u32::from(bezique_core::agents::TOP)) as u8,
            persona,
            erraticism: f64::from(erraticism_permille.min(1000)) / 1000.0,
        },
    )
}

// ---------------------------------------------------------------------------
// Writing JSON. Twenty lines rather than a dependency.
// ---------------------------------------------------------------------------

fn text(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn object(fields: &[(&str, String)]) -> String {
    let body: Vec<String> = fields
        .iter()
        .map(|(k, v)| format!("{}:{}", text(k), v))
        .collect();
    format!("{{{}}}", body.join(","))
}

fn list(items: impl IntoIterator<Item = String>) -> String {
    format!("[{}]", items.into_iter().collect::<Vec<_>>().join(","))
}

fn or_null(value: Option<String>) -> String {
    value.unwrap_or_else(|| "null".to_string())
}

// ---------------------------------------------------------------------------
// The state.
// ---------------------------------------------------------------------------

fn card(c: Card) -> String {
    object(&[("id", c.0.to_string()), ("card", text(&c.code()))])
}

/// Cards in the order a hand is read: spades, hearts, diamonds, clubs; high
/// to low within each.
fn cards(set: CardSet) -> String {
    let mut ordered = Vec::new();
    for suit in Suit::ALL {
        let mut in_suit: Vec<Card> = set.in_suit(suit).cards().collect();
        in_suit.sort_by(|a, b| b.cmp(a));
        ordered.extend(in_suit);
    }
    list(ordered.into_iter().map(card))
}

fn carried(holdings: &bezique_core::melds::Holdings, trump: Suit) -> String {
    list(holdings.carried().iter().map(|d| {
        object(&[
            ("points", d.meld.points(trump).to_string()),
            ("said", text(&words::points(d.meld.points(trump)))),
            ("cards", cards(d.cards)),
        ])
    }))
}

fn whose(you: bool) -> String {
    text(if you { "you" } else { "them" })
}

fn event(e: &Event) -> String {
    let mut fields: Vec<(&str, String)> =
        vec![("hand", e.hand.to_string()), ("text", text(&e.text))];
    let kind = match &e.kind {
        EventKind::Cut { yours, theirs } => {
            fields.extend([("yours", card(*yours)), ("theirs", card(*theirs))]);
            "cut"
        }
        EventKind::FirstDealer { you } => {
            fields.push(("dealer", whose(*you)));
            "first_dealer"
        }
        EventKind::Dealt { you_deal, turn_up } => {
            fields.extend([("dealer", whose(*you_deal)), ("turn_up", card(*turn_up))]);
            "dealt"
        }
        EventKind::Played { you, card: c } => {
            fields.extend([("who", whose(*you)), ("card", card(*c))]);
            "played"
        }
        EventKind::TookTrick { you, points } => {
            fields.extend([("who", whose(*you)), ("points", points.to_string())]);
            "took_trick"
        }
        EventKind::Declared {
            you,
            cards: laid,
            points,
        } => {
            fields.extend([
                ("who", whose(*you)),
                ("laid", list(laid.iter().map(|c| card(*c)))),
                ("points", points.to_string()),
            ]);
            "declared"
        }
        EventKind::Dix { you, taken } => {
            fields.extend([("who", whose(*you)), ("taken", or_null(taken.map(card)))]);
            "dix"
        }
        EventKind::Drew { card: c } => {
            fields.push(("card", card(*c)));
            "drew"
        }
        EventKind::StockOut {
            you_took_last,
            turn_up,
        } => {
            fields.extend([
                ("last_card", whose(*you_took_last)),
                ("turn_up", card(*turn_up)),
            ]);
            "stock_out"
        }
        EventKind::HandEnds { yours, theirs } => {
            fields.extend([("you", yours.to_string()), ("them", theirs.to_string())]);
            "hand_ends"
        }
        EventKind::GameEnds { you_won, double } => {
            fields.extend([("winner", whose(*you_won)), ("double", double.to_string())]);
            "game_ends"
        }
    };
    fields.insert(0, ("kind", text(kind)));
    object(&fields)
}

fn prompt(p: &Prompt) -> String {
    match p {
        Prompt::ChooseDealer => object(&[("kind", text("choose_dealer"))]),
        Prompt::Play { legal } => object(&[
            ("kind", text("play")),
            ("legal", list(legal.iter().map(|c| c.0.to_string()))),
        ]),
        Prompt::Declare { options } => object(&[
            ("kind", text("declare")),
            (
                "options",
                list(options.iter().map(|o| {
                    object(&[
                        ("text", text(&o.text)),
                        ("cards", list(o.cards.iter().map(|c| card(*c)))),
                        ("points", o.points.to_string()),
                    ])
                })),
            ),
        ]),
        Prompt::Over => object(&[("kind", text("over"))]),
    }
}

fn phase(view: Option<&View>, p: &Prompt) -> &'static str {
    match (view, p) {
        (None, _) => "choose_dealer",
        (_, Prompt::Over) => "over",
        (Some(v), _) => match v.phase {
            Phase::Lead => "lead",
            Phase::Follow => "follow",
            Phase::Declare => "declare",
            Phase::Over => "over",
        },
    }
}

/// The whole state, as the person may know it.
pub fn state(session: &Session) -> String {
    let view = session.view();
    let p = session.prompt();
    let aids = session.aids();
    let mut fields: Vec<(&str, String)> = vec![
        ("protocol", PROTOCOL.to_string()),
        ("level", session.settings().level.to_string()),
        ("opponent", text(&session.opponent())),
        ("phase", text(phase(view.as_ref(), &p))),
    ];
    if let Some(v) = &view {
        let trump = v.trump;
        let tricks_played = if aids.tracker {
            list(v.tricks.iter().map(|t| {
                object(&[
                    ("leader", whose(t.leader == v.seat)),
                    ("led", card(t.led)),
                    ("second", card(t.second)),
                    ("winner", whose(t.winner == v.seat)),
                ])
            }))
        } else {
            "null".to_string()
        };
        let last_trick = v.tricks.last().map(|t| {
            object(&[
                ("leader", whose(t.leader == v.seat)),
                ("led", card(t.led)),
                ("second", card(t.second)),
                ("winner", whose(t.winner == v.seat)),
            ])
        });
        let trick = v
            .led
            .map(|led| object(&[("leader", whose(v.leader == v.seat)), ("led", card(led))]));
        fields.extend([
            ("hand_number", v.hand_number.to_string()),
            (
                "you_deal",
                (v.seat == bezique_core::deal::Seat::Dealer).to_string(),
            ),
            ("trump", text(trump.name())),
            ("trump_symbol", text(&trump.symbol().to_string())),
            ("turn_up", or_null(v.turn_up.map(card))),
            ("stock", v.stock.to_string()),
            (
                "totals",
                object(&[
                    ("you", v.totals.0.to_string()),
                    ("them", v.totals.1.to_string()),
                ]),
            ),
            (
                "hand_scores",
                object(&[
                    ("you", v.hand_scores.0.to_string()),
                    ("them", v.hand_scores.1.to_string()),
                ]),
            ),
            ("hand", cards(v.mine.concealed)),
            ("table", cards(v.mine.shown)),
            ("carried", carried(&v.mine, trump)),
            ("their_table", cards(v.theirs.shown)),
            ("their_carried", carried(&v.theirs, trump)),
            ("their_known", cards(v.theirs.concealed)),
            ("their_hidden", v.their_hidden.to_string()),
            ("trick", or_null(trick)),
            ("last_trick", or_null(last_trick)),
            ("tricks_played", tricks_played),
        ]);
    }
    let outcome = session.outcome().map(|o| {
        object(&[
            ("winner", whose(o.winner == Side::A)),
            ("double", o.double.to_string()),
        ])
    });
    let hint = session
        .hint()
        .map(|(command, said)| object(&[("command", text(&command)), ("text", text(&said))]));
    fields.extend([
        ("prompt", prompt(&p)),
        (
            "aids",
            object(&[
                ("hints", aids.hints.to_string()),
                ("play_forced", aids.play_forced.to_string()),
                ("tracker", aids.tracker.to_string()),
            ]),
        ),
        ("can_undo", session.can_undo().to_string()),
        ("record", list(session.record().iter().map(|r| text(r)))),
        ("hint", or_null(hint)),
        ("events", list(session.events().iter().map(event))),
        ("outcome", or_null(outcome)),
        ("error", or_null(session.error().map(text))),
    ]);
    object(&fields)
}

/// The WebAssembly exports. The host asks for a buffer with `bezique_alloc`,
/// writes a command into it and calls `bezique_send`; it reads the state
/// from `bezique_state` (a pointer) and `bezique_state_len`. One sitting per
/// module instance.
pub mod ffi {
    use super::{sit_down, state, Session};
    use std::cell::RefCell;

    thread_local! {
        static SESSION: RefCell<Option<Session>> = const { RefCell::new(None) };
        static OUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
        static IN: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    }

    /// A buffer of `len` bytes for the host to write a command into. Valid
    /// until the next call to this function.
    #[no_mangle]
    pub extern "C" fn bezique_alloc(len: usize) -> *mut u8 {
        IN.with(|buffer| {
            let mut buffer = buffer.borrow_mut();
            buffer.clear();
            buffer.resize(len, 0);
            buffer.as_mut_ptr()
        })
    }

    /// Sits down at a new table, discarding any old one: level 1 to 5; persona
    /// 0 to draw one, else its place in the list from 1; erraticism in
    /// thousandths.
    #[no_mangle]
    pub extern "C" fn bezique_new(level: u32, seed: u32, persona: u32, erraticism_permille: u32) {
        SESSION
            .with(|s| *s.borrow_mut() = Some(sit_down(level, seed, persona, erraticism_permille)));
        render();
    }

    /// Carries out the command in the `len` bytes just written; 1 if accepted.
    #[no_mangle]
    pub extern "C" fn bezique_send(len: usize) -> u32 {
        let command = IN.with(|buffer| {
            let buffer = buffer.borrow();
            String::from_utf8_lossy(&buffer[..len.min(buffer.len())]).into_owned()
        });
        let accepted = SESSION.with(|s| match s.borrow_mut().as_mut() {
            Some(session) => session.send(&command),
            None => false,
        });
        render();
        u32::from(accepted)
    }

    /// Where the state, as UTF-8 JSON, begins.
    #[no_mangle]
    pub extern "C" fn bezique_state() -> *const u8 {
        OUT.with(|out| out.borrow().as_ptr())
    }

    #[no_mangle]
    pub extern "C" fn bezique_state_len() -> usize {
        OUT.with(|out| out.borrow().len())
    }

    fn render() {
        let json = SESSION.with(|s| {
            s.borrow()
                .as_ref()
                .map_or_else(|| "null".to_string(), state)
        });
        OUT.with(|out| *out.borrow_mut() = json.into_bytes());
    }
}
