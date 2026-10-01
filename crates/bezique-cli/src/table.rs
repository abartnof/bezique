//! A table in the terminal: a person against the opponent, or two opponents
//! watched. A client of the engine only: every rule, score and refusal comes
//! from `bezique-core`.

use std::io::{self, BufRead, Write};

use bezique_core::agents::{Legal, Maxims};
use bezique_core::cards::{Card, CardSet, Rank, Suit};
use bezique_core::deal::Seat;
use bezique_core::game::{cut_for_deal, Game, Outcome, Side};
use bezique_core::hand::{Action, Hand, Phase, Source};
use bezique_core::observation::{Agent, View};
use bezique_core::rng::{purpose, Rng};
use bezique_core::words;

#[derive(Clone, Debug)]
pub struct Options {
    pub seed: u64,
    /// The opponent's rung: 1 plays legally, 2 plays the period maxims.
    pub level: u8,
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
        writeln!(
            self.out,
            "Bezique: two packs, 1,000 up. Seed {}. Type a card as AS or 10D (or its number); q quits.",
            self.options.seed
        )?;
        let Some(first_dealer) = self.cut()? else {
            return Ok(End::Quit);
        };
        let mut game = Game::new(self.options.seed, first_dealer);
        let mut agents: [Box<dyn Agent>; 2] = [self.agent(), self.agent()];
        self.hand_begins(&game)?;
        while let Some(side) = game.to_move() {
            if self.options.watch && game.hand().phase() == Phase::Lead {
                self.show(&game, side)?;
                if self.options.pause && !self.wait()? {
                    return Ok(End::Quit);
                }
            }
            let action = if Some(side) == self.human {
                match self.ask(&game, side)? {
                    Some(action) => action,
                    None => {
                        writeln!(self.out, "You leave the table.")?;
                        return Ok(End::Quit);
                    }
                }
            } else {
                let view = View::of(&game, side);
                agents[side.index()].choose(&view, &game.legal())
            };
            let before = game.hand().clone();
            let number = game.hand_number();
            game.apply(action.clone())
                .expect("only legal moves are made");
            self.narrate(&before, number, &game, side, &action)?;
        }
        let outcome = game.outcome().expect("the game is over");
        self.game_ends(&game, outcome)?;
        Ok(End::Finished(outcome))
    }

    fn agent(&self) -> Box<dyn Agent> {
        match self.options.level {
            1 => Box::new(Legal),
            _ => Box::new(Maxims::default()),
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

    /// A verb agreeing with `who`: "You take", "Your opponent takes".
    fn does(&self, side: Side, verb: &str) -> String {
        let who = self.who(side);
        if who == "You" {
            format!("{who} {verb}")
        } else if verb.ends_with("sh") || verb.ends_with("ch") {
            format!("{who} {verb}es")
        } else {
            format!("{who} {verb}s")
        }
    }

    fn card(&self, card: Card) -> String {
        paint(card, self.options.colour)
    }

    fn cards(&self, cards: CardSet) -> String {
        by_suit(cards, self.options.colour)
    }

    // ---- the cut -----------------------------------------------------------

    /// Cuts for the deal; returns who deals first, or `None` on quitting.
    fn cut(&mut self) -> io::Result<Option<Side>> {
        let cut = cut_for_deal(&mut Rng::stream(self.options.seed, purpose::CUT));
        for (i, [a, b]) in cut.rounds.iter().enumerate() {
            let (cuts, other) = (self.does(Side::A, "cut"), self.whom(Side::B));
            writeln!(
                self.out,
                "{cuts} the {}; {other} the {}.",
                self.card(*a),
                self.card(*b)
            )?;
            if i + 1 < cut.rounds.len() {
                writeln!(self.out, "Equal ranks: cut again.")?;
            }
        }
        if Some(cut.chooser) == self.human {
            loop {
                write!(self.out, "You cut higher. Will you deal? [y/n] ")?;
                self.out.flush()?;
                match self.line()?.as_deref() {
                    None | Some("q") | Some("quit") => return Ok(None),
                    Some("y") | Some("yes") => return Ok(Some(cut.chooser)),
                    Some("n") | Some("no") => return Ok(Some(cut.chooser.other())),
                    _ => {}
                }
            }
        }
        writeln!(
            self.out,
            "{} higher and chooses to deal.",
            self.does(cut.chooser, "cut")
        )?;
        Ok(Some(cut.chooser))
    }

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

    /// The person's move, or `None` if they leave.
    fn ask(&mut self, game: &Game, side: Side) -> io::Result<Option<Action>> {
        let legal = game.legal();
        if game.hand().phase() == Phase::Declare {
            return self.ask_declare(game, side, &legal);
        }
        let mut plays: Vec<Card> = legal
            .iter()
            .filter_map(|a| match a {
                Action::Play(c) => Some(*c),
                _ => None,
            })
            .collect();
        // Numbered in the order the hand is shown: by suit, high to low.
        plays.sort_by_key(|c| (c.suit(), std::cmp::Reverse(c.0)));
        if plays.len() == 1 {
            writeln!(self.out, "You must play the {}.", self.card(plays[0]))?;
            return Ok(Some(Action::Play(plays[0])));
        }
        self.show(game, side)?;
        let shown = game.hand().holdings(game.seat_of(side)).shown;
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
            if answer == "q" || answer == "quit" {
                return Ok(None);
            }
            match pick_card(
                &answer,
                &plays,
                shown,
                game.hand().holdings(game.seat_of(side)).cards(),
            ) {
                Ok(card) => return Ok(Some(Action::Play(card))),
                Err(Some(held)) => {
                    // Held but not allowed: the engine's reason is the lesson.
                    let refusal = game.clone().apply(Action::Play(held)).unwrap_err();
                    writeln!(self.out, "Not allowed: {refusal}.")?;
                }
                Err(None) => writeln!(self.out, "{answer:?} is not one of your cards.")?,
            }
        }
    }

    fn ask_declare(
        &mut self,
        game: &Game,
        side: Side,
        legal: &[Action],
    ) -> io::Result<Option<Action>> {
        let options: Vec<&Action> = legal.iter().filter(|a| **a != Action::Pass).collect();
        if options.is_empty() {
            return Ok(Some(Action::Pass));
        }
        writeln!(self.out, "You took the trick. Declare before you draw:")?;
        writeln!(self.out, "  0  Nothing: draw.")?;
        for (i, option) in options.iter().enumerate() {
            writeln!(
                self.out,
                "  {}  {}",
                i + 1,
                self.describe(game.hand(), game.seat_of(side), option)
            )?;
        }
        loop {
            write!(self.out, "Choose [0-{}]: ", options.len())?;
            self.out.flush()?;
            let Some(answer) = self.line()? else {
                return Ok(None);
            };
            if answer == "q" || answer == "quit" {
                return Ok(None);
            }
            match answer.parse::<usize>() {
                Ok(0) => return Ok(Some(Action::Pass)),
                Ok(n) if n <= options.len() => return Ok(Some(options[n - 1].clone())),
                _ => writeln!(self.out, "Choose a number from 0 to {}.", options.len())?,
            }
        }
    }

    /// An option of the turn of declaring, in words.
    fn describe(&self, hand: &Hand, seat: Seat, action: &Action) -> String {
        let trump = hand.trump();
        match action {
            Action::Declare(a) => {
                let mut after = hand.holdings(seat).clone();
                after.apply(trump, a).expect("offered, so legal");
                words::announcement(a, trump, &after.carried())
            }
            Action::Dix(_) => match hand.stock().turn_up() {
                Some(t) if t.rank() == Rank::SEVEN && t.suit() == trump => {
                    "Show the other seven of trumps: ten.".to_string()
                }
                Some(t) => format!(
                    "Exchange the seven of trumps for the {}: ten.",
                    self.card(t)
                ),
                None => "The seven of trumps: ten.".to_string(),
            },
            Action::Pass => "Nothing: draw.".to_string(),
            Action::Play(c) => format!("Play the {}.", self.card(*c)),
        }
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

    fn narrate(
        &mut self,
        before: &Hand,
        number: u32,
        game: &Game,
        side: Side,
        action: &Action,
    ) -> io::Result<()> {
        let new_hand = game.hand_number() > number;
        let after = if new_hand {
            game.previous_hand().expect("the hand just finished")
        } else {
            game.hand()
        };
        let side_of = |seat: Seat| -> Side {
            // Seats belong to the hand the move was made in.
            let dealer = if new_hand {
                game.dealer().other()
            } else {
                game.dealer()
            };
            if seat == Seat::Dealer {
                dealer
            } else {
                dealer.other()
            }
        };
        let trump = before.trump();
        match (before.phase(), action) {
            (Phase::Lead, Action::Play(c)) => {
                let from_table = before.holdings(before.leader()).shown.contains(*c);
                let place = if from_table { " from the table" } else { "" };
                writeln!(
                    self.out,
                    "{} the {}{place}.",
                    self.does(side, "lead"),
                    self.card(*c)
                )?;
            }
            (Phase::Follow, Action::Play(c)) => {
                writeln!(
                    self.out,
                    "{} the {}.",
                    self.does(side, "play"),
                    self.card(*c)
                )?;
                let trick = after.tricks().last().expect("a trick was taken");
                let winner = side_of(trick.winner);
                let mut taken = self.does(winner, "take");
                taken.push_str(" the trick");
                let new_scores = &after.log()[before.log().len()..];
                let mut extras = Vec::new();
                for s in new_scores {
                    match s.source {
                        Source::Brisques => {
                            extras.push(format!("{} for brisques", words::points(s.points)))
                        }
                        Source::LastTrick => extras.push("ten for the last trick".to_string()),
                        _ => {}
                    }
                }
                if !extras.is_empty() {
                    taken.push_str(&format!(": {}", extras.join(", and ")));
                }
                writeln!(self.out, "{taken}.")?;
            }
            (Phase::Declare, Action::Declare(a)) => {
                let seat = before.leader();
                let said = words::announcement(a, trump, &after.holdings(seat).carried());
                writeln!(self.out, "{}: \"{said}\"", self.does(side, "declare"))?;
            }
            (Phase::Declare, Action::Dix(_)) => {
                let turned = before
                    .stock()
                    .turn_up()
                    .expect("there while declaring lasts");
                if turned.rank() == Rank::SEVEN && turned.suit() == trump {
                    writeln!(
                        self.out,
                        "{} the other seven of trumps: ten.",
                        self.does(side, "show")
                    )?;
                } else {
                    writeln!(
                        self.out,
                        "{} the seven of trumps for the {}: ten.",
                        self.does(side, "exchange"),
                        self.card(turned)
                    )?;
                }
            }
            _ => {}
        }
        if before.phase() == Phase::Declare {
            self.draws(before, after, side)?;
        }
        if new_hand || (after.phase() == Phase::Over) {
            self.hand_ends(after, game, new_hand)?;
        }
        if new_hand && game.outcome().is_none() {
            self.hand_begins(game)?;
        }
        Ok(())
    }

    /// The draws after a turn of declaring, and the end of the stock.
    fn draws(&mut self, before: &Hand, after: &Hand, winner: Side) -> io::Result<()> {
        let winner_seat = before.leader();
        let last = before.stock().face_down() == 1;
        // The turn-up, which goes to the loser at the end or to the winner by
        // the dix, is told separately.
        let turn_up = before
            .stock()
            .turn_up()
            .map_or(CardSet::EMPTY, |c| CardSet::EMPTY.with(c));
        if last {
            let card = before
                .stock()
                .turn_up()
                .map(|c| self.card(c))
                .unwrap_or_default();
            let loser = winner.other();
            let (does, whom) = (self.does(winner, "take"), self.whom(loser));
            writeln!(
                self.out,
                "{does} the last card; {whom} the turn-up, the {card}."
            )?;
        }
        if let Some(human) = self.human {
            let seat = if human == winner {
                winner_seat
            } else {
                winner_seat.other()
            };
            let drawn = after
                .holdings(seat)
                .cards()
                .minus(before.holdings(seat).cards())
                .minus(turn_up);
            if !drawn.is_empty() {
                writeln!(self.out, "You draw the {}.", self.cards(drawn))?;
            }
        }
        if last {
            writeln!(
                self.out,
                "The stock is exhausted. The cards on the table go back into the hands; from now on the second player must follow suit and win the trick if able."
            )?;
        }
        Ok(())
    }

    fn hand_begins(&mut self, game: &Game) -> io::Result<()> {
        let hand = game.hand();
        let turn_up = hand.stock().turn_up().expect("a fresh stock");
        writeln!(self.out)?;
        writeln!(
            self.out,
            "Hand {}. {}. The {} is turned up: {} are trumps.",
            game.hand_number(),
            self.does(game.dealer(), "deal"),
            self.card(turn_up),
            hand.trump().name()
        )?;
        if hand.log().iter().any(|s| s.source == Source::TurnedSeven) {
            writeln!(
                self.out,
                "{} ten for turning up the seven.",
                self.does(game.dealer(), "score")
            )?;
        }
        Ok(())
    }

    fn hand_ends(&mut self, finished: &Hand, game: &Game, new_hand: bool) -> io::Result<()> {
        let dealer = if new_hand {
            game.dealer().other()
        } else {
            game.dealer()
        };
        let seat_of = |s: Side| {
            if s == dealer {
                Seat::Dealer
            } else {
                Seat::Pone
            }
        };
        if finished.phase() != Phase::Over {
            return Ok(());
        }
        let banked = |s: Side| {
            game.total(s)
                - if new_hand {
                    game.hand().score(game.seat_of(s))
                } else {
                    0
                }
        };
        writeln!(
            self.out,
            "End of the hand: {} {}, {} {}. The game: {} {}, {} {}.",
            self.whom(Side::A),
            finished.score(seat_of(Side::A)),
            self.whom(Side::B),
            finished.score(seat_of(Side::B)),
            self.whom(Side::A),
            banked(Side::A),
            self.whom(Side::B),
            banked(Side::B),
        )?;
        Ok(())
    }

    fn game_ends(&mut self, game: &Game, outcome: Outcome) -> io::Result<()> {
        let Outcome { winner, double } = outcome;
        writeln!(self.out)?;
        let (reach, total) = (self.does(winner, "reach"), game.total(winner));
        let wins = if self.who(winner) == "You" {
            "win"
        } else {
            "wins"
        };
        writeln!(
            self.out,
            "{reach} {total} and {wins} the game, {total} to {}.",
            game.total(winner.other())
        )?;
        if double {
            writeln!(self.out, "A double game: the loser had not reached 500.")?;
        }
        Ok(())
    }
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
    Err(held.cards().find(|c| c.card_type() == card_type))
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
        assert!(said.contains("The stock is exhausted."));
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
