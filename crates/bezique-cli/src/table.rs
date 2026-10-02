//! A table in the terminal: a person against the opponent, or two opponents
//! watched. A client of the engine only: every rule, score and refusal comes
//! from `bezique-core`.

use std::io::{self, BufRead, Write};

use bezique_core::cards::{Card, CardSet, Suit};
use bezique_core::game::{Game, Outcome, Side};
use bezique_core::hand::Phase;
use bezique_core::observation::View;
use bezique_core::session::{DeclareOption, EventKind, Prompt, Session, Settings};
use bezique_core::words;

#[derive(Clone, Debug)]
pub struct Options {
    pub seed: u64,
    /// The opponent's rung, 1 to `agents::TOP`.
    pub level: u8,
    /// The opponent's persona by key, or `None` to draw one from the seed.
    pub persona: Option<String>,
    /// How often the opponent slips below its level, 0 to 1.
    pub erraticism: f64,
    /// Two computer players, no person.
    pub watch: bool,
    /// When watching, show both hands.
    pub reveal: bool,
    /// When watching, wait for Enter after each trick.
    pub pause: bool,
    /// Red hearts and diamonds.
    pub colour: bool,
}

/// How a sitting ended.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum End {
    Finished(Outcome),
    Quit,
}

pub struct Table<R, W> {
    input: R,
    out: W,
    options: Options,
    /// The side the person plays, unless watching.
    human: Option<Side>,
}

impl<R: BufRead, W: Write> Table<R, W> {
    pub fn new(input: R, out: W, options: Options) -> Table<R, W> {
        let human = (!options.watch).then_some(Side::A);
        Table {
            input,
            out,
            options,
            human,
        }
    }

    /// Sits down, cuts, and plays a game to its end.
    pub fn play(&mut self) -> io::Result<End> {
        if self.options.watch {
            self.watch()
        } else {
            self.sit_at_session()
        }
    }

    /// The person's game, as a client of the engine's session: the session
    /// holds the rules, the opponent, the record and undo; this draws the
    /// person's view and reads their answers.
    fn sit_at_session(&mut self) -> io::Result<End> {
        writeln!(
            self.out,
            "Bezique: two packs, 1,000 up. Seed {}. Type a card as AS or 10D (or its number); u undoes, h turns hints on and off, t shows every trick of the hand, q quits.",
            self.options.seed
        )?;
        let settings = Settings {
            level: self.options.level,
            persona: self.options.persona.clone(),
            erraticism: self.options.erraticism,
        };
        let mut session = Session::new(self.options.seed, settings);
        session.send("set play_forced on");
        writeln!(self.out, "{}", session.opponent())?;
        let mut told = 0;
        loop {
            for event in &session.events()[told..] {
                if matches!(
                    event.kind,
                    EventKind::Dealt { .. } | EventKind::GameEnds { .. }
                ) {
                    writeln!(self.out)?;
                }
                writeln!(self.out, "{}", event.text)?;
            }
            told = session.events().len();
            let answer = match session.prompt() {
                Prompt::Over => {
                    let outcome = session.outcome().expect("the game is over");
                    return Ok(End::Finished(outcome));
                }
                Prompt::ChooseDealer => {
                    write!(self.out, "You cut higher. Will you deal? [y/n] ")?;
                    self.out.flush()?;
                    match self.line()?.as_deref() {
                        None | Some("q") | Some("quit") => None,
                        Some("y") | Some("yes") => Some("dealer you".to_string()),
                        Some("n") | Some("no") => Some("dealer them".to_string()),
                        _ => continue,
                    }
                }
                Prompt::Play { legal } => self.ask_play(&session, legal)?,
                Prompt::Declare { options } => self.ask_option(&session, &options)?,
            };
            let Some(answer) = answer else {
                writeln!(self.out, "You leave the table.")?;
                return Ok(End::Quit);
            };
            match answer.as_str() {
                "undo" => {
                    if session.send("undo") {
                        told = session.events().len();
                        writeln!(self.out, "Taken back to your last decision.")?;
                    } else {
                        writeln!(
                            self.out,
                            "{}.",
                            session.error().unwrap_or("nothing to undo")
                        )?;
                    }
                }
                "hints" | "tracker" => {
                    let on = if answer == "hints" {
                        session.aids().hints
                    } else {
                        session.aids().tracker
                    };
                    session.send(&format!("set {answer} {}", if on { "off" } else { "on" }));
                }
                command => {
                    if !session.send(command) {
                        writeln!(
                            self.out,
                            "Not allowed: {}.",
                            session.error().unwrap_or("refused")
                        )?;
                    }
                }
            }
        }
    }

    /// Reads a card to play, or one of the other answers (u, h, t, q).
    fn ask_play(&mut self, session: &Session, mut plays: Vec<Card>) -> io::Result<Option<String>> {
        let view = session.view().expect("a game in progress");
        self.show_view(session, &view)?;
        plays.sort_by_key(|c| (c.suit(), std::cmp::Reverse(c.0)));
        let shown = view.mine.shown;
        let listing: Vec<String> = plays
            .iter()
            .enumerate()
            .map(|(i, &c)| {
                format!(
                    "{} {}{}",
                    i + 1,
                    self.card(c),
                    if shown.contains(c) { "*" } else { "" }
                )
            })
            .collect();
        loop {
            write!(
                self.out,
                "Play ({}; * is on the table): ",
                listing.join("  ")
            )?;
            self.out.flush()?;
            let Some(answer) = self.line()? else {
                return Ok(None);
            };
            match answer.as_str() {
                "q" | "quit" => return Ok(None),
                "u" | "undo" => return Ok(Some("undo".to_string())),
                "h" | "hint" => return Ok(Some("hints".to_string())),
                "t" | "tricks" => return Ok(Some("tracker".to_string())),
                _ => {}
            }
            match pick_card(&answer, &plays, shown, view.mine.cards()) {
                Ok(card) => return Ok(Some(format!("play {}", card.0))),
                // Held but not allowed: sent anyway, so that the engine's
                // reason is shown.
                Err(Some(held)) => return Ok(Some(format!("play {}", held.0))),
                Err(None) => writeln!(self.out, "{answer:?} is not one of your cards.")?,
            }
        }
    }

    /// Reads a choice in the turn of declaring.
    fn ask_option(
        &mut self,
        session: &Session,
        options: &[DeclareOption],
    ) -> io::Result<Option<String>> {
        // The table first (with the hint, if hints are on), so that `t`
        // shows the tricks here too.
        let view = session.view().expect("a game in progress");
        self.show_view(session, &view)?;
        writeln!(self.out, "You took the trick. Declare before you draw:")?;
        for (i, option) in options.iter().enumerate() {
            writeln!(self.out, "  {i}  {}", option.text)?;
        }
        loop {
            write!(self.out, "Choose [0-{}]: ", options.len() - 1)?;
            self.out.flush()?;
            let Some(answer) = self.line()? else {
                return Ok(None);
            };
            match answer.as_str() {
                "q" | "quit" => return Ok(None),
                "u" | "undo" => return Ok(Some("undo".to_string())),
                "h" | "hint" => return Ok(Some("hints".to_string())),
                "t" | "tricks" => return Ok(Some("tracker".to_string())),
                _ => {}
            }
            match answer.parse::<usize>() {
                Ok(n) if n < options.len() => return Ok(Some(format!("declare {n}"))),
                _ => writeln!(self.out, "Choose a number from 0 to {}.", options.len() - 1)?,
            }
        }
    }

    /// The table as the person sees it.
    fn show_view(&mut self, session: &Session, v: &View) -> io::Result<()> {
        let turn_up = v.turn_up.map_or("taken".to_string(), |c| self.card(c));
        writeln!(self.out)?;
        writeln!(
            self.out,
            "Hand {} · trumps {} {} · turn-up {} · stock {}",
            v.hand_number,
            v.trump.symbol(),
            v.trump.name(),
            turn_up,
            v.stock
        )?;
        writeln!(
            self.out,
            "Score: you {} · your opponent {}",
            v.totals.0, v.totals.1
        )?;
        let mut theirs = format!(
            "Your opponent: {} in hand",
            v.theirs.concealed.len() + v.their_hidden
        );
        if !v.theirs.concealed.is_empty() {
            theirs.push_str(&format!(" (among them {})", self.cards(v.theirs.concealed)));
        }
        self.describe_table(&mut theirs, &v.theirs, v.trump);
        writeln!(self.out, "{theirs}")?;
        let mut mine = format!("You: in hand {}", self.cards(v.mine.concealed));
        self.describe_table(&mut mine, &v.mine, v.trump);
        writeln!(self.out, "{mine}")?;
        if session.aids().tracker {
            let tricks: Vec<String> = v
                .tricks
                .iter()
                .map(|t| {
                    let mark = if t.winner == v.seat { "you" } else { "them" };
                    format!("{} {} ({mark})", self.card(t.led), self.card(t.second))
                })
                .collect();
            if !tricks.is_empty() {
                writeln!(self.out, "Tricks this hand: {}", tricks.join(" | "))?;
            }
        }
        if let Some(led) = v.led {
            let who = if v.leader == v.seat {
                "You"
            } else {
                "Your opponent"
            };
            writeln!(self.out, "{who} led the {}.", self.card(led))?;
        }
        if let Some((_, said)) = session.hint() {
            writeln!(self.out, "Hint: {said}")?;
        }
        Ok(())
    }

    fn describe_table(
        &self,
        line: &mut String,
        holdings: &bezique_core::melds::Holdings,
        trump: bezique_core::cards::Suit,
    ) {
        if !holdings.shown.is_empty() {
            line.push_str(&format!("; on the table {}", self.cards(holdings.shown)));
        }
        let carried = holdings.carried();
        if !carried.is_empty() {
            let said: Vec<String> = carried
                .iter()
                .map(|d| words::points(d.meld.points(trump)))
                .collect();
            line.push_str(&format!("; {} to score", said.join(" and ")));
        }
    }

    /// Two computer players, watched: the session plays and tells the game,
    /// one move at a time; this shows the table before each lead.
    fn watch(&mut self) -> io::Result<End> {
        writeln!(
            self.out,
            "Bezique: two packs, 1,000 up. Seed {}. Two computer players; q stops.",
            self.options.seed
        )?;
        let settings = Settings {
            level: self.options.level,
            persona: self.options.persona.clone(),
            erraticism: self.options.erraticism,
        };
        let mut session = Session::watch(self.options.seed, settings);
        for side in Side::BOTH {
            writeln!(self.out, "{}", session.player(side))?;
        }
        let mut told = 0;
        loop {
            for event in &session.events()[told..] {
                if matches!(
                    event.kind,
                    EventKind::Dealt { .. } | EventKind::GameEnds { .. }
                ) {
                    writeln!(self.out)?;
                }
                writeln!(self.out, "{}", event.text)?;
            }
            told = session.events().len();
            let game = session.game().expect("a watched game is dealt at once");
            if let Some(side) = game.to_move() {
                if game.hand().phase() == Phase::Lead {
                    self.show(game, side)?;
                    if self.options.pause && !self.wait()? {
                        return Ok(End::Quit);
                    }
                }
            }
            if !session.step() {
                let outcome = session.outcome().expect("the game is over");
                return Ok(End::Finished(outcome));
            }
        }
    }

    // ---- names and cards ---------------------------------------------------

    fn who(&self, side: Side) -> &'static str {
        match (self.human, side) {
            (Some(h), s) if h == s => "You",
            (Some(_), _) => "Your opponent",
            (None, Side::A) => "A",
            (None, Side::B) => "B",
        }
    }

    /// `who`, as it reads inside a sentence: "you", "your opponent", "A".
    fn whom(&self, side: Side) -> String {
        match self.who(side) {
            "You" => "you".to_string(),
            "Your opponent" => "your opponent".to_string(),
            name => name.to_string(),
        }
    }

    fn card(&self, card: Card) -> String {
        paint(card, self.options.colour)
    }

    fn cards(&self, cards: CardSet) -> String {
        by_suit(cards, self.options.colour)
    }

    // ---- the cut -----------------------------------------------------------

    // ---- asking the person ---------------------------------------------------

    fn line(&mut self) -> io::Result<Option<String>> {
        let mut line = String::new();
        if self.input.read_line(&mut line)? == 0 {
            return Ok(None);
        }
        Ok(Some(line.trim().to_ascii_lowercase()))
    }

    /// Waits for Enter; false on quitting.
    fn wait(&mut self) -> io::Result<bool> {
        write!(self.out, "(Enter to go on) ")?;
        self.out.flush()?;
        Ok(!matches!(
            self.line()?.as_deref(),
            None | Some("q") | Some("quit")
        ))
    }

    // ---- showing the table ---------------------------------------------------

    fn show(&mut self, game: &Game, side: Side) -> io::Result<()> {
        let hand = game.hand();
        let trump = hand.trump();
        let turn_up = hand
            .stock()
            .turn_up()
            .map_or("taken".to_string(), |c| self.card(c));
        let (me, them) = (side, side.other());
        writeln!(self.out)?;
        writeln!(
            self.out,
            "Hand {} · trumps {} {} · turn-up {} · stock {}",
            game.hand_number(),
            trump.symbol(),
            trump.name(),
            turn_up,
            hand.stock().face_down()
        )?;
        writeln!(
            self.out,
            "Score: {} {} · {} {}",
            self.whom(me),
            game.total(me),
            self.whom(them),
            game.total(them)
        )?;
        for s in [them, me] {
            let seat = game.seat_of(s);
            let holdings = hand.holdings(seat);
            let mut line = format!("{}:", self.who(s));
            let visible = Some(s) == self.human || self.options.reveal;
            if visible {
                line.push_str(&format!(" in hand {}", self.cards(holdings.concealed)));
            } else {
                let known = hand.known(seat).intersect(holdings.concealed);
                line.push_str(&format!(" {} in hand", holdings.concealed.len()));
                if !known.is_empty() {
                    line.push_str(&format!(" (among them {})", self.cards(known)));
                }
            }
            if !holdings.shown.is_empty() {
                line.push_str(&format!("; on the table {}", self.cards(holdings.shown)));
            }
            let carried = holdings.carried();
            if !carried.is_empty() {
                let said: Vec<String> = carried
                    .iter()
                    .map(|d| words::points(d.meld.points(trump)))
                    .collect();
                line.push_str(&format!("; {} to score", said.join(" and ")));
            }
            writeln!(self.out, "{line}")?;
        }
        if let Some(led) = hand.led() {
            writeln!(
                self.out,
                "{} led the {}.",
                self.who(game.side_in(hand.leader())),
                self.card(led)
            )?;
        }
        Ok(())
    }

    // ---- telling what happened -----------------------------------------------
}

/// A card's code, red for hearts and diamonds when colour is on.
pub fn paint(card: Card, colour: bool) -> String {
    let code = card.code();
    if colour && (card.suit() == Suit::HEARTS || card.suit() == Suit::DIAMONDS) {
        format!("\x1b[31m{code}\x1b[0m")
    } else {
        code
    }
}

/// Cards grouped by suit, high to low: `AS TS 7S | QH 8H`.
pub fn by_suit(cards: CardSet, colour: bool) -> String {
    let mut groups = Vec::new();
    for suit in Suit::ALL {
        let mut in_suit: Vec<Card> = cards.in_suit(suit).cards().collect();
        in_suit.sort_by(|a, b| b.cmp(a));
        if !in_suit.is_empty() {
            groups.push(
                in_suit
                    .iter()
                    .map(|&c| paint(c, colour))
                    .collect::<Vec<_>>()
                    .join(" "),
            );
        }
    }
    groups.join(" | ")
}

/// Reads the person's answer: a number from the list, or a code (`ks`,
/// `10d`), with `*` for the copy on the table. `Err(Some(card))` means a card
/// they hold but may not play now; `Err(None)` means no card of theirs.
pub fn pick_card(
    answer: &str,
    plays: &[Card],
    shown: CardSet,
    held: CardSet,
) -> Result<Card, Option<Card>> {
    if let Ok(n) = answer.parse::<usize>() {
        return plays.get(n.wrapping_sub(1)).copied().ok_or(None);
    }
    let (code, table) = match answer.strip_suffix('*') {
        Some(code) => (code, true),
        None => (answer, false),
    };
    let Ok(card_type) = bezique_core::cards::CardType::parse(code) else {
        return Err(None);
    };
    let matching = |c: &Card| c.card_type() == card_type && shown.contains(*c) == table;
    if let Some(&card) = plays.iter().find(|c| matching(c)) {
        return Ok(card);
    }
    // Without a star, a table copy will do if there is no other.
    if !table {
        if let Some(&card) = plays.iter().find(|c| c.card_type() == card_type) {
            return Ok(card);
        }
    }
    // Held but not playable: sent anyway, for the engine's reason. With a
    // star, only a copy on the table will do.
    Err(held
        .cards()
        .find(|&c| c.card_type() == card_type && (!table || shown.contains(c))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bezique_core::cards::CardType;

    fn c(code: &str, copy: u8) -> Card {
        Card::new(CardType::parse(code).unwrap(), copy)
    }

    fn options(watch: bool) -> Options {
        Options {
            seed: 7,
            level: 2,
            persona: None,
            erraticism: 0.0,
            watch,
            reveal: watch,
            pause: false,
            colour: false,
        }
    }

    fn sit(input: &str, options: Options) -> (End, String) {
        let mut out = Vec::new();
        let end = Table::new(input.as_bytes(), &mut out, options)
            .play()
            .unwrap();
        (end, String::from_utf8(out).unwrap())
    }

    #[test]
    fn cards_are_shown_as_they_are_typed() {
        let cards = CardSet::parse("AS TS 7S QH 8H 9D KC JC").unwrap();
        assert_eq!(by_suit(cards, false), "AS TS 7S | QH 8H | 9D | KC JC");
        assert_eq!(paint(c("QH", 0), true), "\x1b[31mQH\x1b[0m");
        assert_eq!(paint(c("QS", 0), true), "QS");
    }

    #[test]
    fn a_card_is_picked_by_code_number_or_star() {
        let plays = [c("KS", 0), c("KS", 1), c("9H", 0)];
        let shown = CardSet::of(&[c("KS", 1)]);
        let held = CardSet::of(&plays).with(c("AD", 0));
        assert_eq!(pick_card("ks", &plays, shown, held), Ok(c("KS", 0)));
        assert_eq!(pick_card("ks*", &plays, shown, held), Ok(c("KS", 1)));
        assert_eq!(pick_card("3", &plays, shown, held), Ok(c("9H", 0)));
        assert_eq!(pick_card("10d", &plays, shown, held), Err(None));
        assert_eq!(pick_card("ad", &plays, shown, held), Err(Some(c("AD", 0))));
        assert_eq!(pick_card("9", &plays, shown, held), Err(None));
        assert_eq!(pick_card("zz", &plays, shown, held), Err(None));
        // A star asks for the table copy: with none on the table, not the
        // one in the hand.
        assert_eq!(pick_card("9h*", &plays, shown, held), Err(None));
    }

    #[test]
    fn a_person_can_play_a_whole_game() {
        // Always the first card listed, the first declaration offered, and
        // yes to dealing.
        let (end, said) = sit(&"y\n1\n".repeat(3000), options(false));
        assert!(matches!(end, End::Finished(_)), "{said}");
        assert!(said.contains("the game,"), "{said}");
        assert!(said.contains("Hand 1."));
        assert!(said.contains("takes the trick") || said.contains("take the trick"));
    }

    #[test]
    fn two_computer_players_can_be_watched() {
        let (end, said) = sit("", options(true));
        assert!(matches!(end, End::Finished(_)));
        assert!(said.contains("A leads") || said.contains("B leads"));
        assert!(said.contains("The stock is exhausted"));
        assert!(
            said.contains("reaches") && said.contains("wins the game"),
            "{said}"
        );
    }

    #[test]
    fn the_opponent_is_described_at_the_start() {
        let mut o = options(false);
        o.level = 4;
        o.persona = Some("bezique".to_string());
        o.erraticism = 0.2;
        let (_, said) = sit("q\n", o);
        assert!(
            said.contains("Your opponent remembers every card"),
            "{said}"
        );
        assert!(said.contains("double bezique"), "{said}");
        assert!(said.contains("does not always play its best"), "{said}");
    }

    #[test]
    fn a_decision_can_be_taken_back() {
        let (_, said) = sit(&format!("y\n1\nu\n{}", "1\n".repeat(3000)), options(false));
        assert!(said.contains("Taken back to your last decision."), "{said}");
    }

    #[test]
    fn a_person_can_leave() {
        let (end, said) = sit("q\n", options(false));
        assert_eq!(end, End::Quit);
        assert!(!said.is_empty());
    }

    #[test]
    fn nonsense_is_answered_and_asked_again() {
        let (_, said) = sit(&format!("y\nzz\n{}", "1\n".repeat(3000)), options(false));
        assert!(said.contains("\"zz\" is not one of your cards."), "{said}");
    }
}
