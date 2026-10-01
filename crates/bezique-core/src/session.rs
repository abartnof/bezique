//! A sitting at the table: one person (side A) against the opponent, advanced
//! one human decision at a time.
//!
//! The session cuts for the deal, lets the person choose the dealer if they
//! cut higher, runs the opponent between the person's decisions, keeps a
//! record from which the whole sitting can be replayed (and so undone), and
//! tells what happened as a list of events. Everything a client shows should
//! come from [`Session::view`], which holds only what the person may know.
//!
//! Commands are one line each:
//!
//! - `dealer you` / `dealer them`: having cut higher, choose who deals first;
//! - `play <id>`: lead or answer with your card numbered `id`;
//! - `declare <n>`: choose option `n` of the turn of declaring (0 is to draw
//!   without declaring);
//! - `undo`: take back the last decision;
//! - `set <aid> on|off`: switch an aid (`hints`, `play_forced`, `tracker`).

use crate::agents::{self, rung_with};
use crate::cards::Card;
use crate::deal::Seat;
use crate::game::{cut_for_deal, Cut, Game, Outcome, Side};
use crate::hand::{Action, Hand, Phase, Source};
use crate::melds::Declared;
use crate::observation::{Agent, View};
use crate::opponent::{self, Erratic, Persona};
use crate::rng::{purpose, Rng};
use crate::words;

/// How the opponent is set up.
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    /// The rung, 1 to `agents::TOP`.
    pub level: u8,
    /// A persona by key, or `None` to draw one from the seed.
    pub persona: Option<String>,
    /// How often it slips below its level, 0 to 1.
    pub erraticism: f64,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            level: 2,
            persona: None,
            erraticism: 0.0,
        }
    }
}

/// Aids change what the person is asked, never what happens.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Aids {
    /// Offer the top rung's choice as a hint.
    pub hints: bool,
    /// Play a card when it is the only one allowed.
    pub play_forced: bool,
    /// Show every trick played this hand. Off, only the last trick shows:
    /// a quitted trick may not be looked at again (Foster, p. 343), so
    /// remembering them is the player's skill.
    pub tracker: bool,
}

/// What the person must decide now.
#[derive(Clone, Debug, PartialEq)]
pub enum Prompt {
    /// Having cut higher, choose who deals first.
    ChooseDealer,
    /// Lead or answer: the cards allowed.
    Play { legal: Vec<Card> },
    /// The turn of declaring: the options, in words, the first being to draw
    /// without declaring.
    Declare { options: Vec<DeclareOption> },
    /// The game is over.
    Over,
}

/// One option of the turn of declaring.
#[derive(Clone, Debug, PartialEq)]
pub struct DeclareOption {
    pub text: String,
    /// The cards it lays on the table (or the seven shown or exchanged).
    pub cards: Vec<Card>,
    /// The points it scores now.
    pub points: u32,
}

/// Something that happened, to be told or animated. Cards are as the
/// person's view numbers them.
#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    pub kind: EventKind,
    /// The hand it happened in.
    pub hand: u32,
    /// One sentence, ready to show.
    pub text: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EventKind {
    /// A pair of cards shown in the cut for deal.
    Cut { yours: Card, theirs: Card },
    /// Who deals first.
    FirstDealer { you: bool },
    /// A hand is dealt; the turn-up shows trumps.
    Dealt { you_deal: bool, turn_up: Card },
    /// A card is played.
    Played { you: bool, card: Card },
    /// A trick is taken.
    TookTrick { you: bool, points: u32 },
    /// A combination is announced and one scored.
    Declared {
        you: bool,
        cards: Vec<Card>,
        points: u32,
    },
    /// The seven of trumps scored, exchanged for `taken` if it was.
    Dix { you: bool, taken: Option<Card> },
    /// The person drew this card.
    Drew { card: Card },
    /// The last card and the turn-up are taken; the stock is gone.
    StockOut { you_took_last: bool, turn_up: Card },
    /// A hand ends.
    HandEnds { yours: u32, theirs: u32 },
    /// The game ends.
    GameEnds { you_won: bool, double: bool },
}

/// A sitting.
pub struct Session {
    seed: u64,
    settings: Settings,
    cut: Cut,
    game: Option<Game>,
    opponent: Box<dyn Agent>,
    persona: Persona,
    record: Vec<String>,
    events: Vec<Event>,
    aids: Aids,
    error: Option<String>,
}

impl Session {
    /// Sits down: the cut is made, and if the opponent cut higher it deals
    /// first and the game begins.
    pub fn new(seed: u64, settings: Settings) -> Session {
        let cut = cut_for_deal(&mut Rng::stream(seed, purpose::CUT));
        let persona = settings
            .persona
            .as_deref()
            .and_then(opponent::persona)
            .unwrap_or_else(|| {
                let all = opponent::personas();
                let mut rng = Rng::stream(seed, purpose::PERSONA);
                all[rng.below(all.len() as u64) as usize].clone()
            });
        let opponent = Box::new(Erratic::new(
            settings.level,
            &persona,
            settings.erraticism,
            Rng::stream(seed, purpose::agent(Side::B.index())),
        ));
        let mut session = Session {
            seed,
            settings,
            cut: cut.clone(),
            game: None,
            opponent,
            persona,
            record: Vec::new(),
            events: Vec::new(),
            aids: Aids::default(),
            error: None,
        };
        for (i, &[yours, theirs]) in cut.rounds.iter().enumerate() {
            let again = if i + 1 < cut.rounds.len() {
                " Equal: cut again."
            } else {
                ""
            };
            session.tell(
                EventKind::Cut { yours, theirs },
                format!(
                    "You cut the {}; your opponent the {}.{again}",
                    yours.code(),
                    theirs.code()
                ),
            );
        }
        if cut.chooser == Side::B {
            session.start(Side::B);
        }
        session
    }

    /// Rebuilds a sitting from its seed, settings and record: how a client
    /// reloads a game, and how undo works. Entries marked `*` were made by
    /// the table, and are made again as the record is replayed.
    pub fn replay(seed: u64, settings: Settings, record: &[String]) -> Result<Session, String> {
        let mut session = Session::new(seed, settings);
        for entry in record.iter().filter(|e| !e.starts_with('*')) {
            if !session.send(entry) {
                return Err(format!(
                    "the record does not fit at {entry:?}: {}",
                    session.error().unwrap_or("refused")
                ));
            }
        }
        Ok(session)
    }

    /// Carries out one command. A command the rules forbid is refused, the
    /// sitting is left as it was, and `error` says why.
    pub fn send(&mut self, command: &str) -> bool {
        self.error = None;
        match self.execute(command.trim()) {
            Ok(()) => true,
            Err(why) => {
                self.error = Some(why);
                false
            }
        }
    }

    pub fn prompt(&self) -> Prompt {
        let Some(game) = &self.game else {
            return Prompt::ChooseDealer;
        };
        if game.to_move() != Some(Side::A) {
            return Prompt::Over;
        }
        let hand = game.hand();
        let legal = game.legal();
        match hand.phase() {
            Phase::Declare => Prompt::Declare {
                options: legal.iter().map(|a| self.option(hand, a)).collect(),
            },
            _ => Prompt::Play {
                legal: legal
                    .iter()
                    .filter_map(|a| match a {
                        Action::Play(c) => Some(*c),
                        _ => None,
                    })
                    .collect(),
            },
        }
    }

    /// The person's view, once the game has begun.
    pub fn view(&self) -> Option<View> {
        self.game.as_ref().map(|g| View::of(g, Side::A))
    }

    pub fn game(&self) -> Option<&Game> {
        self.game.as_ref()
    }

    pub fn events(&self) -> &[Event] {
        &self.events
    }

    /// Every command taken from the person's seat; `*` marks what the table
    /// did for them.
    pub fn record(&self) -> &[String] {
        &self.record
    }

    pub fn aids(&self) -> Aids {
        self.aids
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn can_undo(&self) -> bool {
        self.record
            .iter()
            .any(|r| !r.starts_with('*') && !r.starts_with("set "))
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// The cut for the first deal.
    pub fn cut(&self) -> &Cut {
        &self.cut
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// The opponent, in a player's words.
    pub fn opponent(&self) -> String {
        let level = self.settings.level.clamp(1, agents::TOP);
        let mut said = format!("Your opponent {}.", agents::describe(level));
        if self.persona.key != "balanced" {
            said.push_str(&format!(" Its style: it {}.", self.persona.described));
        }
        if self.settings.erraticism > 0.0 {
            said.push_str(" It does not always play its best.");
        }
        said
    }

    /// With hints on, what the top rung would do in the person's place: the
    /// command to send, and in words.
    pub fn hint(&self) -> Option<(String, String)> {
        if !self.aids.hints {
            return None;
        }
        let game = self.game.as_ref()?;
        if game.to_move() != Some(Side::A) {
            return None;
        }
        let legal = game.legal();
        let view = View::of(game, Side::A);
        let advice = rung_with(agents::TOP, &crate::agents::Style::default()).choose(&view, &legal);
        let index = legal.iter().position(|a| *a == advice)?;
        Some(match advice {
            Action::Play(card) => (
                format!("play {}", card.0),
                format!("Play the {}.", card.code()),
            ),
            other => (
                format!("declare {index}"),
                self.option(game.hand(), &other).text,
            ),
        })
    }

    pub fn outcome(&self) -> Option<Outcome> {
        self.game.as_ref().and_then(Game::outcome)
    }
}

impl Session {
    fn execute(&mut self, command: &str) -> Result<(), String> {
        let words: Vec<&str> = command.split_whitespace().collect();
        match words.as_slice() {
            ["undo"] => self.undo(),
            ["set", aid, setting] => {
                let on = match *setting {
                    "on" => true,
                    "off" => false,
                    _ => return Err("set an aid on or off".to_string()),
                };
                match *aid {
                    "hints" => self.aids.hints = on,
                    "play_forced" => self.aids.play_forced = on,
                    "tracker" => self.aids.tracker = on,
                    _ => return Err(format!("there is no aid called {aid}")),
                }
                self.record.push(format!("set {aid} {setting}"));
                self.run();
                Ok(())
            }
            ["dealer", who] => {
                if self.game.is_some() {
                    return Err("the dealer is already chosen".to_string());
                }
                let dealer = match *who {
                    "you" => Side::A,
                    "them" => Side::B,
                    _ => return Err("say dealer you or dealer them".to_string()),
                };
                self.record.push(format!("dealer {who}"));
                self.start(dealer);
                Ok(())
            }
            ["play", id] => {
                if !matches!(self.prompt(), Prompt::Play { .. }) {
                    return Err("it is not your turn to play".to_string());
                }
                let card = Card(
                    id.parse::<u8>()
                        .map_err(|_| format!("{id} is not a card number"))?,
                );
                self.act(Action::Play(card), format!("play {}", card.0))
            }
            ["declare", n] => {
                let Prompt::Declare { .. } = self.prompt() else {
                    return Err(
                        "you may declare only after winning a trick, while the stock lasts"
                            .to_string(),
                    );
                };
                let n: usize = n
                    .parse()
                    .map_err(|_| format!("{n} is not an option number"))?;
                let legal = self.game.as_ref().expect("declaring").legal();
                let action = legal
                    .get(n)
                    .cloned()
                    .ok_or_else(|| format!("there is no option {n}"))?;
                self.act(action, format!("declare {n}"))
            }
            _ => Err(format!("{command:?} is not a command")),
        }
    }

    /// The person's move: made, recorded, told; then the table runs on.
    fn act(&mut self, action: Action, entry: String) -> Result<(), String> {
        self.apply(Side::A, action)?;
        self.record.push(entry);
        self.run();
        Ok(())
    }

    /// Makes a move in the game and tells what came of it.
    fn apply(&mut self, side: Side, action: Action) -> Result<(), String> {
        let game = self.game.as_mut().expect("a game in progress");
        let before = game.hand().clone();
        let number = game.hand_number();
        game.apply(action.clone())?;
        self.narrate(&before, number, side, &action);
        Ok(())
    }

    /// Deals the first hand.
    fn start(&mut self, dealer: Side) {
        self.game = Some(Game::new(self.seed, dealer));
        let you = dealer == Side::A;
        let text = if you {
            "You deal first."
        } else {
            "Your opponent cut higher and chooses to deal."
        };
        self.tell(EventKind::FirstDealer { you }, text.to_string());
        self.hand_begins();
        self.run();
    }

    /// Runs the opponent, and whatever the table does for the person, until
    /// the person has a decision to make or the game is over.
    fn run(&mut self) {
        loop {
            let Some(game) = &self.game else { return };
            let Some(side) = game.to_move() else { return };
            let legal = game.legal();
            if side == Side::B {
                let view = View::of(game, Side::B);
                let action = self.opponent.choose(&view, &legal);
                self.apply(Side::B, action)
                    .expect("the opponent moves legally");
                continue;
            }
            let automatic = match (game.hand().phase(), legal.as_slice()) {
                // Nothing to declare is never put to the person.
                (Phase::Declare, [Action::Pass]) => Some((Action::Pass, "*declare 0".to_string())),
                (Phase::Lead | Phase::Follow, [Action::Play(card)]) if self.aids.play_forced => {
                    Some((Action::Play(*card), format!("*play {}", card.0)))
                }
                _ => None,
            };
            let Some((action, entry)) = automatic else {
                return;
            };
            self.apply(Side::A, action).expect("a forced move is legal");
            self.record.push(entry);
        }
    }

    fn undo(&mut self) -> Result<(), String> {
        let last = self
            .record
            .iter()
            .rposition(|r| !r.starts_with('*') && !r.starts_with("set "))
            .ok_or("there is nothing to undo")?;
        let mut record: Vec<String> = self.record[..last].to_vec();
        record.extend(
            self.record[last + 1..]
                .iter()
                .filter(|r| r.starts_with("set "))
                .cloned(),
        );
        let mut again = Session::replay(self.seed, self.settings.clone(), &record)?;
        again.error = None;
        *self = again;
        Ok(())
    }

    fn tell(&mut self, kind: EventKind, text: String) {
        let hand = self.game.as_ref().map_or(0, Game::hand_number);
        self.events.push(Event { kind, hand, text });
    }

    /// How a card is named to the person: their own cards exactly; another's
    /// only by type (copy 0), since which twin it was is not theirs to know.
    fn named(&self, card: Card, hand: &Hand, seat: Seat) -> Card {
        if hand
            .handled(seat)
            .intersect(card.card_type().twins())
            .is_empty()
        {
            Card::new(card.card_type(), 0)
        } else {
            card
        }
    }

    fn option(&self, hand: &Hand, action: &Action) -> DeclareOption {
        let trump = hand.trump();
        let seat = hand.to_move().expect("declaring");
        match action {
            Action::Pass => DeclareOption {
                text: "Nothing: draw.".to_string(),
                cards: vec![],
                points: 0,
            },
            Action::Dix(seven) => {
                let text = match hand.stock().turn_up() {
                    Some(t) if t.rank() == crate::cards::Rank::SEVEN && t.suit() == trump => {
                        "Show the other seven of trumps: ten.".to_string()
                    }
                    Some(t) => format!("Exchange the seven of trumps for the {}: ten.", t.code()),
                    None => "The seven of trumps: ten.".to_string(),
                };
                DeclareOption {
                    text,
                    cards: vec![*seven],
                    points: 10,
                }
            }
            Action::Declare(a) => {
                let mut after = hand.holdings(seat).clone();
                after.apply(trump, a).expect("offered, so legal");
                let laid = a
                    .melds
                    .iter()
                    .fold(crate::cards::CardSet::EMPTY, |s, d: &Declared| {
                        s.union(d.cards)
                    })
                    .intersect(hand.holdings(seat).concealed);
                DeclareOption {
                    text: words::announcement(a, trump, &after.carried()),
                    cards: laid.cards().collect(),
                    points: a.score.meld.points(trump),
                }
            }
            Action::Play(card) => DeclareOption {
                text: format!("Play the {}.", card.code()),
                cards: vec![*card],
                points: 0,
            },
        }
    }

    fn hand_begins(&mut self) {
        let game = self.game.as_ref().expect("a game");
        let you_deal = game.dealer() == Side::A;
        let hand = game.hand();
        let turn_up = hand.stock().turn_up().expect("a fresh stock");
        let shown = self.named(turn_up, hand, game.seat_of(Side::A));
        let mut text = format!(
            "Hand {}. {} deal. The {} is turned up: {} are trumps.",
            game.hand_number(),
            if you_deal { "You" } else { "Your opponent" },
            turn_up.code(),
            hand.trump().name()
        );
        if hand.log().iter().any(|s| s.source == Source::TurnedSeven) {
            text.push_str(if you_deal {
                " You score ten for it."
            } else {
                " Your opponent scores ten for it."
            });
        }
        self.tell(
            EventKind::Dealt {
                you_deal,
                turn_up: shown,
            },
            text,
        );
    }

    fn narrate(&mut self, before: &Hand, number: u32, side: Side, action: &Action) {
        let game = self.game.as_ref().expect("a game");
        let mut told: Vec<(EventKind, String)> = Vec::new();
        let new_hand = game.hand_number() > number;
        let after = if new_hand {
            game.previous_hand()
                .expect("the hand just finished")
                .clone()
        } else {
            game.hand().clone()
        };
        // The person's seat in the hand the move was made in.
        let dealer = if new_hand {
            game.dealer().other()
        } else {
            game.dealer()
        };
        let mine = if dealer == Side::A {
            Seat::Dealer
        } else {
            Seat::Pone
        };
        let you = side == Side::A;
        let who = if you { "You" } else { "Your opponent" };
        let s = if you { "" } else { "s" };
        let trump = before.trump();
        match (before.phase(), action) {
            (Phase::Lead | Phase::Follow, Action::Play(card)) => {
                let verb = if before.phase() == Phase::Lead {
                    "lead"
                } else {
                    "play"
                };
                let named = self.named(*card, &after, mine);
                told.push((
                    EventKind::Played { you, card: named },
                    format!("{who} {verb}{s} the {}.", card.code()),
                ));
                if before.phase() == Phase::Follow {
                    let trick = *after.tricks().last().expect("a trick was taken");
                    let won = trick.winner == mine;
                    let points: u32 = after.log()[before.log().len()..]
                        .iter()
                        .map(|x| x.points)
                        .sum();
                    let taker = if won {
                        "You take"
                    } else {
                        "Your opponent takes"
                    };
                    let worth = if points > 0 {
                        format!(": {}", words::points(points))
                    } else {
                        String::new()
                    };
                    told.push((
                        EventKind::TookTrick { you: won, points },
                        format!("{taker} the trick{worth}."),
                    ));
                }
            }
            (Phase::Declare, Action::Declare(a)) => {
                let seat = before.leader();
                let said = words::announcement(a, trump, &after.holdings(seat).carried());
                let laid: Vec<Card> = a
                    .melds
                    .iter()
                    .fold(crate::cards::CardSet::EMPTY, |acc, d| acc.union(d.cards))
                    .intersect(before.holdings(seat).concealed)
                    .cards()
                    .map(|c| self.named(c, &after, mine))
                    .collect();
                let points = a.score.meld.points(trump);
                told.push((
                    EventKind::Declared {
                        you,
                        cards: laid,
                        points,
                    },
                    format!("{who} declare{s}: \"{said}\""),
                ));
            }
            (Phase::Declare, Action::Dix(_)) => {
                let turned = before
                    .stock()
                    .turn_up()
                    .expect("there while declaring lasts");
                let shown_seven =
                    turned.rank() == crate::cards::Rank::SEVEN && turned.suit() == trump;
                let (taken, text) = if shown_seven {
                    (
                        None,
                        format!("{who} show{s} the other seven of trumps: ten."),
                    )
                } else {
                    (
                        Some(self.named(turned, &after, mine)),
                        format!(
                            "{who} exchange{s} the seven of trumps for the {}: ten.",
                            turned.code()
                        ),
                    )
                };
                told.push((EventKind::Dix { you, taken }, text));
            }
            _ => {}
        }
        if before.phase() == Phase::Declare {
            let last = before.stock().face_down() == 1;
            let turn_up = before.stock().turn_up();
            let drawn = after
                .holdings(mine)
                .cards()
                .minus(before.holdings(mine).cards())
                .minus(turn_up.map_or(crate::cards::CardSet::EMPTY, |c| {
                    crate::cards::CardSet::EMPTY.with(c)
                }));
            for card in drawn.cards() {
                told.push((
                    EventKind::Drew { card },
                    format!("You draw the {}.", card.code()),
                ));
            }
            if last {
                let turned = turn_up.expect("the turn-up is the last card");
                let you_took_last = before.leader() == mine;
                let text = format!(
                    "{} the last card; {} the turn-up, the {}. The stock is exhausted: the cards on the table go back into the hands, and the second player must now follow suit and win the trick if able.",
                    if you_took_last { "You take" } else { "Your opponent takes" },
                    if you_took_last { "your opponent" } else { "you" },
                    turned.code()
                );
                let named = self.named(turned, &after, mine);
                told.push((
                    EventKind::StockOut {
                        you_took_last,
                        turn_up: named,
                    },
                    text,
                ));
            }
        }
        if after.phase() == Phase::Over {
            let (yours, theirs) = (after.score(mine), after.score(mine.other()));
            told.push((
                EventKind::HandEnds { yours, theirs },
                format!("End of the hand: you {yours}, your opponent {theirs}."),
            ));
        }
        if let Some(outcome) = game.outcome() {
            let you_won = outcome.winner == Side::A;
            let (a, b) = (game.total(Side::A), game.total(Side::B));
            let mut text = if you_won {
                format!("You reach {a} and win the game, {a} to {b}.")
            } else {
                format!("Your opponent reaches {b} and wins the game, {b} to {a}.")
            };
            if outcome.double {
                text.push_str(" A double game: the loser had not reached 500.");
            }
            told.push((
                EventKind::GameEnds {
                    you_won,
                    double: outcome.double,
                },
                text,
            ));
        }
        let game_over = game.outcome().is_some();
        for (kind, text) in told {
            self.tell(kind, text);
        }
        if new_hand && !game_over {
            self.hand_begins();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The command a scripted person sends: the first option offered.
    fn first(session: &Session) -> Option<String> {
        match session.prompt() {
            Prompt::ChooseDealer => Some("dealer them".to_string()),
            Prompt::Play { legal } => Some(format!("play {}", legal[0].0)),
            Prompt::Declare { options } => Some(format!("declare {}", options.len() - 1)),
            Prompt::Over => None,
        }
    }

    fn play_out(session: &mut Session) -> usize {
        let mut sent = 0;
        while let Some(command) = first(session) {
            assert!(session.send(&command), "{command}: {:?}", session.error());
            sent += 1;
            assert!(sent < 5000);
        }
        sent
    }

    #[test]
    fn a_whole_game_can_be_played_through_commands() {
        let mut s = Session::new(11, Settings::default());
        play_out(&mut s);
        assert_eq!(s.prompt(), Prompt::Over);
        assert!(s.outcome().is_some());
        assert!(s
            .events()
            .iter()
            .any(|e| matches!(e.kind, EventKind::GameEnds { .. })));
    }

    #[test]
    fn the_cut_is_told_and_the_higher_card_chooses() {
        for seed in 0..20 {
            let s = Session::new(seed, Settings::default());
            assert!(matches!(s.events()[0].kind, EventKind::Cut { .. }));
            match s.prompt() {
                Prompt::ChooseDealer => assert!(s.game().is_none()),
                _ => assert!(s
                    .events()
                    .iter()
                    .any(|e| matches!(e.kind, EventKind::FirstDealer { you: false }))),
            }
        }
    }

    #[test]
    fn a_record_replays_to_the_same_sitting() {
        let mut s = Session::new(
            12,
            Settings {
                level: 3,
                ..Settings::default()
            },
        );
        for _ in 0..60 {
            if let Some(c) = first(&s) {
                s.send(&c);
            }
        }
        let again = Session::replay(12, s.settings().clone(), s.record()).unwrap();
        assert_eq!(again.record(), s.record());
        assert_eq!(format!("{:?}", again.view()), format!("{:?}", s.view()));
        assert_eq!(again.events(), s.events());
    }

    #[test]
    fn undo_takes_back_the_last_decision() {
        let mut s = Session::new(13, Settings::default());
        while !matches!(s.prompt(), Prompt::Play { .. }) {
            let c = first(&s).unwrap();
            s.send(&c);
        }
        let before = format!("{:?}", s.view());
        let c = first(&s).unwrap();
        assert!(s.send(&c));
        assert!(s.can_undo());
        assert!(s.send("undo"));
        assert_eq!(format!("{:?}", s.view()), before);
    }

    #[test]
    fn a_refused_command_changes_nothing_and_says_why() {
        let mut s = Session::new(14, Settings::default());
        let before = (format!("{:?}", s.view()), s.record().to_vec());
        assert!(!s.send("play 999"));
        assert!(!s.send("nonsense"));
        assert!(s.error().is_some());
        assert_eq!((format!("{:?}", s.view()), s.record().to_vec()), before);
    }

    #[test]
    fn forced_cards_are_played_for_you_when_asked() {
        let mut s = Session::new(15, Settings::default());
        assert!(s.send("set play_forced on"));
        play_out(&mut s);
        assert!(
            s.record().iter().any(|r| r.starts_with("*play")),
            "the endgame forces some cards"
        );
        let again = Session::replay(15, s.settings().clone(), s.record()).unwrap();
        assert_eq!(again.outcome(), s.outcome());
    }

    #[test]
    fn hints_name_a_legal_command() {
        let mut s = Session::new(
            16,
            Settings {
                level: 4,
                ..Settings::default()
            },
        );
        assert!(s.send("set hints on"));
        for _ in 0..80 {
            if let Some((command, text)) = s.hint() {
                assert!(!text.is_empty());
                let mut probe = Session::replay(16, s.settings().clone(), s.record()).unwrap();
                assert!(probe.send(&command), "{command}");
            }
            match first(&s) {
                Some(c) => {
                    s.send(&c);
                }
                None => break,
            }
        }
    }

    #[test]
    fn the_opponent_is_described_without_a_name() {
        let s = Session::new(
            17,
            Settings {
                level: 4,
                persona: Some("close".to_string()),
                erraticism: 0.0,
            },
        );
        let said = s.opponent();
        assert!(said.contains("remembers every card"), "{said}");
        assert!(said.contains("table first"), "{said}");
    }
}
