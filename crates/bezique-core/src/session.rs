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
    /// The rung, 1 to `agents::TOP`; others are clamped into that range.
    pub level: u8,
    /// A persona by key, or `None` to draw one from the seed. A key that
    /// names no persona is drawn from the seed too.
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

/// The version of the engine's play that a record replays against. A record
/// is a list of commands, and replays faithfully only while the cards dealt,
/// the opponent's choices and the order of the declaring options are the
/// same: bump this whenever a change alters any of them, so that a saved
/// record from before is refused rather than replayed into a different
/// game. The test `the_golden_game_replays_unchanged` fails when play
/// changes, as the reminder to bump it.
pub const RECORD_VERSION: u32 = 3;

/// A sitting: one person against the opponent, from the cut to the end of
/// the game, advanced one human decision at a time.
pub struct Session {
    seed: u64,
    settings: Settings,
    cut: Cut,
    game: Option<Game>,
    opponent: Erratic,
    /// Every move the opponent has made, in order.
    moves: Vec<Action>,
    /// Moves known from before an undo, replayed rather than thought again:
    /// the replay passes through the very positions it passed through the
    /// first time, so the opponent would choose the same. Cleared once the
    /// replay is done.
    script: Vec<Action>,
    /// Watching two computer players: side A played too, by this agent, the
    /// game advanced by [`Session::step`], and the players named "A" and
    /// "B" in the narration rather than "you" and "your opponent".
    watched: Option<(Erratic, Persona)>,
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
        Session::with_script(seed, settings, Vec::new(), false)
    }

    /// Two computer players, both set up by `settings` (each with a persona
    /// of its own, drawn from the seed unless one is named), to be watched:
    /// advance it with [`Session::step`]; the narration names them "A" and
    /// "B". Whoever cuts higher chooses to deal.
    pub fn watch(seed: u64, settings: Settings) -> Session {
        Session::with_script(seed, settings, Vec::new(), true)
    }

    /// Plays one move in a watched game, and tells it. False once the game
    /// is over (or if this is not a watched game).
    pub fn step(&mut self) -> bool {
        let Some(game) = &self.game else { return false };
        let Some(side) = game.to_move() else {
            return false;
        };
        let view = View::of(game, side);
        let legal = game.legal();
        let action = match (side, self.watched.as_mut()) {
            (Side::A, Some((a, _))) => a.choose(&view, &legal),
            (Side::B, Some(_)) => self.opponent.choose(&view, &legal),
            (_, None) => return false,
        };
        self.apply(side, action).expect("the players move legally");
        true
    }

    /// Whether this is a watched game between two computer players.
    pub fn watching(&self) -> bool {
        self.watched.is_some()
    }

    /// How a player is named as the subject of a sentence.
    fn subject(&self, a: bool) -> &'static str {
        match (self.watching(), a) {
            (true, true) => "A",
            (true, false) => "B",
            (false, true) => "You",
            (false, false) => "Your opponent",
        }
    }

    /// How a player is named as the object of a sentence.
    fn object(&self, a: bool) -> &'static str {
        match (self.watching(), a) {
            (true, true) => "A",
            (true, false) => "B",
            (false, true) => "you",
            (false, false) => "your opponent",
        }
    }

    /// `verb` agreeing with the player as subject: "you take", "B takes".
    fn verb(&self, a: bool, verb: &str) -> String {
        if a && !self.watching() {
            verb.to_string()
        } else if verb.ends_with("ch") {
            format!("{verb}es")
        } else {
            format!("{verb}s")
        }
    }

    /// [`Session::new`], the opponent's first moves taken from `script`.
    fn with_script(seed: u64, settings: Settings, script: Vec<Action>, watch: bool) -> Session {
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
        let opponent = Erratic::new(
            settings.level,
            &persona,
            settings.erraticism,
            Rng::stream(seed, purpose::agent(Side::B.index())),
        );
        let watched = watch.then(|| {
            let all = opponent::personas();
            let persona = settings
                .persona
                .as_deref()
                .and_then(opponent::persona)
                .unwrap_or_else(|| {
                    let mut rng = Rng::stream(seed, purpose::persona(1));
                    all[rng.below(all.len() as u64) as usize].clone()
                });
            let agent = Erratic::new(
                settings.level,
                &persona,
                settings.erraticism,
                Rng::stream(seed, purpose::agent(Side::A.index())),
            );
            (agent, persona)
        });
        let mut session = Session {
            seed,
            settings,
            cut: cut.clone(),
            game: None,
            opponent,
            moves: Vec::new(),
            script,
            watched,
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
                    "{} {} the {}; {} the {}.{again}",
                    session.subject(true),
                    session.verb(true, "cut"),
                    yours.code(),
                    session.object(false),
                    theirs.code()
                ),
            );
        }
        if cut.chooser == Side::B || session.watching() {
            session.start(cut.chooser);
        }
        session
    }

    /// Rebuilds a sitting from its seed, settings and record: how a client
    /// reloads a game. Entries marked `*` were made by the table, and are
    /// made again as the record is replayed; if they come out differently,
    /// the record was made by a build that plays differently, and is
    /// refused rather than replayed into another game (a second guard
    /// beside `RECORD_VERSION`).
    pub fn replay(seed: u64, settings: Settings, record: &[String]) -> Result<Session, String> {
        let session = Session::rebuild(seed, settings, record, Vec::new())?;
        if session.record != record {
            return Err(
                "the record replays differently: it was made by another version".to_string(),
            );
        }
        Ok(session)
    }

    /// Replays the person's entries of `record`, the table making its own:
    /// undo's replay, whose record may lack the table's entries after it.
    fn rebuild(
        seed: u64,
        settings: Settings,
        record: &[String],
        script: Vec<Action>,
    ) -> Result<Session, String> {
        let mut session = Session::with_script(seed, settings, script, false);
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
        self.record.iter().any(|r| undoable(r))
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
        self.player(Side::B)
    }

    /// A computer player, in words: how strong, its style, how steady. In a
    /// person's sitting, side B is "your opponent"; watching, A or B.
    pub fn player(&self, side: Side) -> String {
        let persona = match (side, &self.watched) {
            (Side::A, Some((_, persona))) => persona,
            _ => &self.persona,
        };
        let level = self.settings.level.clamp(1, agents::TOP);
        let mut said = format!(
            "{} {}.",
            self.subject(side == Side::A),
            agents::describe(level)
        );
        if persona.key != "balanced" {
            said.push_str(&format!(" Its style: it {}.", persona.described));
        }
        if self.settings.erraticism > 0.0 {
            said.push_str(" It does not always play its best.");
        }
        said
    }

    /// With hints on, what the top rung would do in the person's place: the
    /// command to send, and in words.
    pub fn hint(&self) -> Option<(String, String)> {
        self.advice().map(|(c, t, _)| (c, t))
    }

    /// The hint with its reason, machine-readable (`hints::Reason`).
    pub fn advice(&self) -> Option<(String, String, crate::hints::Reason)> {
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
        let reason = crate::hints::classify(&view, &legal, &advice);
        let (command, said) = match advice {
            Action::Play(card) => (
                format!("play {}", card.0),
                format!("Play the {}.", card.code()),
            ),
            other => (
                format!("declare {index}"),
                self.option(game.hand(), &other).text,
            ),
        };
        Some((command, said, reason))
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
        let text = match (you, self.cut.chooser, self.watching()) {
            (true, _, false) => "You deal first.".to_string(),
            (_, chooser, true) | (false, chooser @ Side::B, false) => format!(
                "{} cut higher and {} to deal.",
                self.subject(chooser == Side::A),
                self.verb(chooser == Side::A, "choose")
            ),
            (false, Side::A, false) => "You give the deal to your opponent.".to_string(),
        };
        self.tell(EventKind::FirstDealer { you }, text);
        self.hand_begins();
        self.run();
    }

    /// Runs the opponent, and whatever the table does for the person, until
    /// the person has a decision to make or the game is over.
    fn run(&mut self) {
        if self.watching() {
            return;
        }
        loop {
            let Some(game) = &self.game else { return };
            let Some(side) = game.to_move() else { return };
            let legal = game.legal();
            if side == Side::B {
                let action = match self.script.get(self.moves.len()) {
                    Some(known) => {
                        self.opponent.replay();
                        known.clone()
                    }
                    None => self.opponent.choose(&View::of(game, Side::B), &legal),
                };
                self.moves.push(action.clone());
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
            .rposition(|r| undoable(r))
            .ok_or("there is nothing to undo")?;
        let mut record: Vec<String> = self.record[..last].to_vec();
        record.extend(
            self.record[last + 1..]
                .iter()
                .filter(|r| r.starts_with("set "))
                .cloned(),
        );
        let mut again = Session::rebuild(
            self.seed,
            self.settings.clone(),
            &record,
            self.moves.clone(),
        )?;
        again.script.clear();
        again.error = None;
        // An aid switched on since can make the table play the decision just
        // taken back (a forced card, with play_forced): then there was no
        // decision there, and undo goes back to the one before.
        if again.record.len() >= self.record.len() - 1
            && format!("{:?}", again.view()) == format!("{:?}", self.view())
        {
            if !again.can_undo() {
                return Err("there is nothing to undo".to_string());
            }
            again.undo()?;
        }
        *self = again;
        Ok(())
    }

    /// Tells an event of the hand in play.
    fn tell(&mut self, kind: EventKind, text: String) {
        let hand = self.game.as_ref().map_or(0, Game::hand_number);
        self.tell_of(hand, kind, text);
    }

    /// Tells an event of hand `hand`: a move's events belong to the hand it
    /// was made in, though the game may have dealt the next.
    fn tell_of(&mut self, hand: u32, kind: EventKind, text: String) {
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
            "Hand {}. {} {}. The {} is turned up: {} are trumps.",
            game.hand_number(),
            self.subject(you_deal),
            self.verb(you_deal, "deal"),
            turn_up.code(),
            hand.trump().name()
        );
        if hand.log().iter().any(|s| s.source == Source::TurnedSeven) {
            text.push_str(&format!(
                " {} {} ten for it.",
                self.subject(you_deal),
                self.verb(you_deal, "score")
            ));
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
        let who = self.subject(you);
        let does = |verb: &str| self.verb(you, verb);
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
                    format!("{who} {} the {}.", does(verb), card.code()),
                ));
                if before.phase() == Phase::Follow {
                    let trick = *after.tricks().last().expect("a trick was taken");
                    let won = trick.winner == mine;
                    let points: u32 = after.log()[before.log().len()..]
                        .iter()
                        .map(|x| x.points)
                        .sum();
                    let taker = format!("{} {}", self.subject(won), self.verb(won, "take"));
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
                // Both twins named by type would share an id: number them.
                let laid = laid
                    .iter()
                    .enumerate()
                    .map(|(i, &c)| {
                        if laid[..i].contains(&c) {
                            Card::new(c.card_type(), 1 - c.copy())
                        } else {
                            c
                        }
                    })
                    .collect();
                let points = a.score.meld.points(trump);
                told.push((
                    EventKind::Declared {
                        you,
                        cards: laid,
                        points,
                    },
                    format!("{who} {}: \"{said}\"", does("declare")),
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
                        format!("{who} {} the other seven of trumps: ten.", does("show")),
                    )
                } else {
                    (
                        Some(self.named(turned, &after, mine)),
                        format!(
                            "{who} {} the seven of trumps for the {}: ten.",
                            does("exchange"),
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
            let old_turn_up = before.stock().turn_up();
            // The turn-up when the draw comes: the seven, if one was just
            // exchanged for it.
            let exchanged = match action {
                Action::Dix(seven)
                    if old_turn_up.is_some_and(|t| {
                        t.rank() != crate::cards::Rank::SEVEN || t.suit() != trump
                    }) =>
                {
                    Some(*seven)
                }
                _ => None,
            };
            let turn_up = exchanged.or(old_turn_up);
            // Not draws: the card taken by the exchange, and the turn-up
            // taken when the stock runs out.
            let not_drawn = [old_turn_up, turn_up]
                .into_iter()
                .flatten()
                .fold(crate::cards::CardSet::EMPTY, |s, c| s.with(c));
            let drawn = after
                .holdings(mine)
                .cards()
                .minus(before.holdings(mine).cards())
                .minus(not_drawn);
            // A watched game's draws are not told: the watchers see the
            // hands, or not, as they chose.
            for card in drawn.cards().filter(|_| !self.watching()) {
                told.push((
                    EventKind::Drew { card },
                    format!("You draw the {}.", card.code()),
                ));
            }
            if last {
                let turned = turn_up.expect("the turn-up is the last card");
                let you_took_last = before.leader() == mine;
                let text = format!(
                    "{} {} the last card; {} the turn-up, the {}. The stock is exhausted: the cards on the table go back into the hands, and the second player must now follow suit and win the trick if able.",
                    self.subject(you_took_last),
                    self.verb(you_took_last, "take"),
                    self.object(!you_took_last),
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
                format!(
                    "End of the hand: {} {yours}, {} {theirs}.",
                    self.object(true),
                    self.object(false)
                ),
            ));
        }
        let mut game_ends = None;
        if let Some(outcome) = game.outcome() {
            let you_won = outcome.winner == Side::A;
            let (a, b) = (game.total(Side::A), game.total(Side::B));
            let (high, low) = if you_won { (a, b) } else { (b, a) };
            let mut text = format!(
                "{} {} {high} and {} the game, {high} to {low}.",
                self.subject(you_won),
                self.verb(you_won, "reach"),
                self.verb(you_won, "win")
            );
            if outcome.double {
                text.push_str(" A double game: the loser had not reached 500.");
            }
            game_ends = Some((
                EventKind::GameEnds {
                    you_won,
                    double: outcome.double,
                },
                text,
            ));
        }
        for (kind, text) in told {
            self.tell_of(number, kind, text);
        }
        // A new deal is told even when it ends the game: a turned seven can.
        if new_hand {
            self.hand_begins();
        }
        if let Some((kind, text)) = game_ends {
            self.tell(kind, text);
        }
    }
}

/// Whether undo may take back this entry of the record: a decision of the
/// person's own, not one the table made for them, not a change of aid, and
/// not the choice of dealer. The first deal comes from the seed, and the
/// choice decides only which of its hands is whose: taking it back would
/// show the person the hand the opponent then holds.
fn undoable(entry: &str) -> bool {
    !entry.starts_with('*') && !entry.starts_with("set ") && !entry.starts_with("dealer ")
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

    /// The first deal comes from the seed; choosing the dealer only decides
    /// which of its two hands is whose. Undoing the choice would show the
    /// person the hand the opponent then holds.
    #[test]
    fn the_choice_of_dealer_cannot_be_undone() {
        let seed = (0..50)
            .find(|&s| Session::new(s, Settings::default()).prompt() == Prompt::ChooseDealer)
            .expect("a seed where the person cuts higher");
        let mut s = Session::new(seed, Settings::default());
        assert!(s.send("dealer you"));
        assert!(!s.can_undo());
        assert!(!s.send("undo"));
        assert_ne!(s.prompt(), Prompt::ChooseDealer);
    }

    /// With play_forced switched on after a forced card was played by hand,
    /// undoing that card would only see the table play it again: undo goes
    /// back to the decision before.
    #[test]
    fn undo_always_takes_something_back() {
        let mut s = Session::new(11, Settings::default());
        // Play by hand until a card is the only one allowed, and play it.
        let mut forced = false;
        for _ in 0..400 {
            if let Prompt::Play { legal } = s.prompt() {
                if legal.len() == 1 && s.can_undo() {
                    assert!(s.send(&format!("play {}", legal[0].0)));
                    forced = true;
                    break;
                }
            }
            let command = first(&s).unwrap();
            assert!(s.send(&command));
        }
        assert!(forced);
        assert!(s.send("set play_forced on"));
        let before = format!("{:?}", s.view());
        assert!(s.send("undo"));
        assert_ne!(format!("{:?}", s.view()), before);
        assert!(!matches!(s.prompt(), Prompt::Play { legal } if legal.len() == 1));
    }

    /// Whole games at every level, each persona at two levels, the person
    /// choosing at random among what is offered (the dealer too, when it
    /// cuts higher), pinned by a fingerprint of every prompt (the legal
    /// cards, the declaring options' words in order), the record and the
    /// narration. If this fails, a change has altered play, what is offered
    /// or how it is told. If play or the options changed, bump
    /// `RECORD_VERSION`; then pin the new fingerprint. (A change of wording
    /// alone needs only the new pin: records replay the same.)
    #[test]
    fn the_golden_game_replays_unchanged() {
        let mut fingerprint = 0xcbf2_9ce4_8422_2325u64;
        let mut mix = |text: &str| {
            for b in text.bytes().chain([0xff]) {
                fingerprint = (fingerprint ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3);
            }
        };
        let personas = opponent::personas();
        let (mut dealer_choices, mut wide_declarations) = (0, 0);
        for level in 1..=agents::TOP {
            for k in 0..2 {
                let persona = &personas[(usize::from(level) + 2 * k) % personas.len()];
                let settings = Settings {
                    level,
                    persona: Some(persona.key.to_string()),
                    erraticism: 0.2,
                };
                let seed = 100 + 10 * u64::from(level) + k as u64;
                let mut s = Session::new(seed, settings);
                let mut choose = Rng::stream(seed, 999);
                loop {
                    let command = match s.prompt() {
                        Prompt::ChooseDealer => {
                            dealer_choices += 1;
                            ["dealer you", "dealer them"][choose.below(2) as usize].to_string()
                        }
                        Prompt::Play { legal } => {
                            mix(&format!("{legal:?}"));
                            format!(
                                "play {}",
                                legal[choose.below(legal.len() as u64) as usize].0
                            )
                        }
                        Prompt::Declare { options } => {
                            for o in &options {
                                mix(&o.text);
                            }
                            wide_declarations += usize::from(options.len() >= 3);
                            format!("declare {}", choose.below(options.len() as u64))
                        }
                        Prompt::Over => break,
                    };
                    assert!(s.send(&command), "{command}: {:?}", s.error());
                }
                for entry in s.record() {
                    mix(entry);
                }
                for e in s.events() {
                    mix(&e.text);
                }
            }
        }
        // The games must reach what they are meant to pin.
        assert!(
            dealer_choices > 0 && wide_declarations > 0,
            "{dealer_choices} {wide_declarations}"
        );
        assert_eq!(
            (RECORD_VERSION, fingerprint),
            (3, 6_109_118_772_721_476_042),
            "play, the options or the narration changed: see this test's comment"
        );
    }

    /// A record whose table-made entries come out differently is refused.
    #[test]
    fn a_record_from_another_version_is_refused() {
        let mut s = Session::new(11, Settings::default());
        play_out(&mut s);
        let mut record = s.record().to_vec();
        assert!(Session::replay(11, Settings::default(), &record).is_ok());
        let auto = record
            .iter()
            .position(|r| r.starts_with('*'))
            .expect("a table-made entry");
        record[auto] = "*declare 9".to_string();
        assert!(Session::replay(11, Settings::default(), &record).is_err());
    }

    /// Undo replays the opponent's known moves rather than thinking them
    /// again: at level 5, with slips, taking back and resending every
    /// decision gives exactly the sitting played straight through, and the
    /// one a full replay rebuilds; and an undo is quick.
    #[test]
    fn undo_reuses_the_opponents_moves_and_changes_nothing_else() {
        let settings = Settings {
            level: 5,
            persona: None,
            erraticism: 0.3,
        };
        let mut straight = Session::new(31, settings.clone());
        let mut undone = Session::new(31, settings.clone());
        let mut slowest = std::time::Duration::ZERO;
        while let Some(command) = first(&straight) {
            assert!(straight.send(&command));
            assert!(undone.send(&command));
            if undone.can_undo() {
                let start = std::time::Instant::now();
                assert!(undone.send("undo"));
                slowest = slowest.max(start.elapsed());
                assert!(undone.send(&command), "{command}: {:?}", undone.error());
            }
            assert_eq!(
                format!("{:?}", undone.view()),
                format!("{:?}", straight.view())
            );
            assert_eq!(undone.events(), straight.events());
            assert_eq!(undone.record(), straight.record());
        }
        let replayed = Session::replay(31, settings, straight.record()).unwrap();
        assert_eq!(replayed.events(), straight.events());
        // A full replay at level 5 takes most of a second; reusing the
        // opponent's moves, an undo is a small fraction of that.
        assert!(
            slowest < std::time::Duration::from_millis(200),
            "{slowest:?}"
        );
    }

    /// Two computer players, watched: the game is played to its end a move
    /// at a time, told with the players named A and B, and no draws told.
    #[test]
    fn a_watched_game_is_told_with_the_players_named() {
        let settings = Settings {
            level: 3,
            persona: None,
            erraticism: 0.0,
        };
        for seed in 0..4 {
            let mut s = Session::watch(seed, settings.clone());
            let mut moves = 0;
            while s.step() {
                moves += 1;
                assert!(moves < 2000);
            }
            assert!(s.outcome().is_some());
            assert_eq!(s.prompt(), Prompt::Over);
            assert!(s.record().is_empty());
            let texts: Vec<&str> = s.events().iter().map(|e| e.text.as_str()).collect();
            for text in &texts {
                assert!(
                    !text.contains("You") && !text.contains("you") && !text.contains("opponent"),
                    "{text}"
                );
            }
            assert!(!s
                .events()
                .iter()
                .any(|e| matches!(e.kind, EventKind::Drew { .. })));
            assert!(texts
                .iter()
                .any(|t| t.starts_with("A leads") || t.starts_with("B leads")));
            assert!(texts
                .iter()
                .any(|t| t.contains("cut higher and chooses to deal.")));
            assert!(matches!(
                s.events().last().unwrap().kind,
                EventKind::GameEnds { .. }
            ));
        }
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
    fn the_deal_is_told_in_good_english() {
        for seed in 0..10 {
            let mut s = Session::new(seed, Settings::default());
            if s.prompt() == Prompt::ChooseDealer {
                s.send("dealer you");
            }
            let dealt = s
                .events()
                .iter()
                .find(|e| matches!(e.kind, EventKind::Dealt { .. }))
                .unwrap();
            assert!(
                dealt.text.contains("You deal.") || dealt.text.contains("Your opponent deals."),
                "{}",
                dealt.text
            );
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

    // Narration, from the review of 2026-10-02.

    fn c(code: &str, copy: u8) -> Card {
        Card::new(crate::cards::CardType::parse(code).unwrap(), copy)
    }

    fn set(cards: &[(&str, u8)]) -> crate::melds::Holdings {
        let cards: Vec<Card> = cards.iter().map(|&(code, copy)| c(code, copy)).collect();
        crate::melds::Holdings::new(crate::cards::CardSet::of(&cards))
    }

    /// A sitting from a position part-way through a game, the opponent at
    /// rung 2.
    fn arranged(seed: u64, banked: [u32; 2], dealer: Side, hand: Hand) -> Session {
        let persona = opponent::persona("balanced").unwrap();
        let mut s = Session {
            seed,
            settings: Settings::default(),
            cut: cut_for_deal(&mut Rng::stream(seed, purpose::CUT)),
            game: Some(Game::arranged(seed, banked, dealer, hand).unwrap()),
            opponent: Erratic::new(2, &persona, 0.0, Rng::stream(seed, purpose::agent(1))),
            moves: vec![],
            script: vec![],
            watched: None,
            persona,
            record: vec![],
            events: vec![],
            aids: Aids::default(),
            error: None,
        };
        s.run();
        s
    }

    /// The last stock trick: the dealer leads the ace of spades and wins;
    /// the ten of clubs is the last face-down card, the king of hearts (hearts
    /// trumps) the turn-up. The winner holds the seven of trumps.
    fn last_stock_trick() -> (
        crate::melds::Holdings,
        crate::melds::Holdings,
        crate::deal::Stock,
    ) {
        let winner = set(&[
            ("AS", 0),
            ("7H", 0),
            ("8C", 0),
            ("9C", 0),
            ("JC", 0),
            ("8D", 0),
            ("9D", 0),
            ("QC", 0),
        ]);
        let loser = set(&[
            ("7S", 0),
            ("8S", 0),
            ("9S", 0),
            ("8S", 1),
            ("9S", 1),
            ("7C", 0),
            ("7D", 0),
            ("8D", 1),
        ]);
        let stock = crate::deal::Stock::new(&[c("TC", 0)], Some(c("KH", 0)));
        (winner, loser, stock)
    }

    #[test]
    fn after_your_dix_on_the_last_stock_trick_the_opponent_takes_the_seven() {
        let (winner, loser, stock) = last_stock_trick();
        let hand = Hand::arranged(
            crate::cards::Suit::HEARTS,
            stock,
            [winner, loser],
            Seat::Dealer,
            23,
        )
        .unwrap();
        let mut s = arranged(1, [0, 0], Side::A, hand);
        assert!(s.send(&format!("play {}", c("AS", 0).0)), "{:?}", s.error());
        let Prompt::Declare { options } = s.prompt() else {
            panic!("{:?}", s.prompt())
        };
        let dix = options
            .iter()
            .position(|o| o.text.contains("seven"))
            .unwrap();
        let from = s.events().len();
        assert!(s.send(&format!("declare {dix}")));
        let told = &s.events()[from..];
        let out = told.iter().find_map(|e| match e.kind {
            EventKind::StockOut { turn_up, .. } => Some((turn_up, e.text.clone())),
            _ => None,
        });
        let (turn_up, text) = out.expect("the stock runs out");
        assert_eq!(turn_up.card_type(), c("7H", 0).card_type(), "{text}");
        assert!(text.contains("the 7H"), "{text}");
        // The king taken by the exchange and the ten drawn: only the ten is a draw.
        let drew: Vec<Card> = told
            .iter()
            .filter_map(|e| match e.kind {
                EventKind::Drew { card } => Some(card),
                _ => None,
            })
            .collect();
        assert_eq!(drew, vec![c("TC", 0)]);
    }

    #[test]
    fn after_the_opponents_dix_on_the_last_stock_trick_you_take_the_seven() {
        let (winner, loser, stock) = last_stock_trick();
        let hand = Hand::arranged(
            crate::cards::Suit::HEARTS,
            stock,
            [winner, loser],
            Seat::Dealer,
            23,
        )
        .unwrap();
        // The opponent deals and leads; the person answers, and rung 2,
        // winning, takes the dix.
        let mut s = arranged(1, [0, 0], Side::B, hand);
        while !s.game().unwrap().hand().stock().is_empty() {
            let command = first(&s).unwrap();
            assert!(s.send(&command));
        }
        assert!(s
            .events()
            .iter()
            .any(|e| matches!(e.kind, EventKind::Dix { you: false, .. })));
        assert!(!s.events().iter().any(|e| matches!(e.kind, EventKind::Drew { card } if card.card_type() == c("7H", 0).card_type())),
            "the turn-up taken is told as a draw");
        let out = s.events().iter().find_map(|e| match e.kind {
            EventKind::StockOut { turn_up, .. } => Some(turn_up),
            _ => None,
        });
        assert_eq!(out.unwrap().card_type(), c("7H", 0).card_type());
    }

    #[test]
    fn every_event_carries_the_hand_it_belongs_to() {
        let mut s = Session::new(11, Settings::default());
        play_out(&mut s);
        let mut current = 0;
        for e in s.events() {
            if let EventKind::Dealt { .. } = e.kind {
                current = e.hand;
            }
            if current > 0 {
                assert_eq!(e.hand, current, "{:?}: {}", e.kind, e.text);
            }
        }
        assert!(current > 1);
    }

    #[test]
    fn a_turned_seven_that_wins_the_game_is_told() {
        let last_trick = || {
            Hand::arranged(
                crate::cards::Suit::HEARTS,
                crate::deal::Stock::new(&[], None),
                [set(&[("AS", 0)]), set(&[("7S", 0)])],
                Seat::Dealer,
                31,
            )
            .unwrap()
        };
        // A seed whose next deal turns up a seven, with the opponent dealing it.
        let seed = (0..500u64)
            .find(|&seed| {
                let mut g = Game::arranged(seed, [0, 0], Side::A, last_trick()).unwrap();
                g.apply(g.legal()[0].clone()).unwrap();
                g.apply(g.legal()[0].clone()).unwrap();
                g.hand()
                    .log()
                    .iter()
                    .any(|s| s.source == Source::TurnedSeven)
            })
            .unwrap();
        let mut s = arranged(seed, [0, 990], Side::A, last_trick());
        assert!(s.send(&format!("play {}", c("AS", 0).0)), "{:?}", s.error());
        assert_eq!(s.outcome().unwrap().winner, Side::B);
        let kinds: Vec<&EventKind> = s.events().iter().map(|e| &e.kind).collect();
        let dealt = kinds
            .iter()
            .position(|k| matches!(k, EventKind::Dealt { .. }));
        let ends = kinds
            .iter()
            .position(|k| matches!(k, EventKind::GameEnds { .. }));
        assert!(dealt.unwrap() < ends.unwrap(), "{kinds:?}");
        let text = &s.events()[dealt.unwrap()].text;
        assert!(text.contains("scores ten for it"), "{text}");
    }

    #[test]
    fn giving_away_the_deal_is_told_as_your_choice() {
        let seed = (0..50)
            .find(|&s| Session::new(s, Settings::default()).prompt() == Prompt::ChooseDealer)
            .unwrap();
        let mut s = Session::new(seed, Settings::default());
        assert!(s.send("dealer them"));
        let told = s
            .events()
            .iter()
            .find(|e| matches!(e.kind, EventKind::FirstDealer { .. }))
            .unwrap();
        assert!(!told.text.contains("cut higher"), "{}", told.text);
    }

    #[test]
    fn a_double_bezique_is_told_with_four_distinct_cards() {
        // The opponent (dealer) leads, wins and declares double bezique.
        let theirs = set(&[
            ("AS", 0),
            ("QS", 0),
            ("QS", 1),
            ("JD", 0),
            ("JD", 1),
            ("8C", 0),
            ("9C", 0),
            ("8D", 0),
        ]);
        let yours = set(&[
            ("7S", 0),
            ("8S", 0),
            ("9S", 0),
            ("8S", 1),
            ("9S", 1),
            ("7C", 0),
            ("7D", 0),
            ("8D", 1),
        ]);
        let stock =
            crate::deal::Stock::new(&[c("TC", 0), c("TC", 1), c("9D", 0)], Some(c("KH", 0)));
        let hand = Hand::arranged(
            crate::cards::Suit::HEARTS,
            stock,
            [theirs, yours],
            Seat::Dealer,
            22,
        )
        .unwrap();
        let mut s = arranged(1, [0, 0], Side::B, hand);
        let command = first(&s).unwrap();
        assert!(s.send(&command));
        let laid = s
            .events()
            .iter()
            .find_map(|e| match &e.kind {
                EventKind::Declared {
                    you: false, cards, ..
                } => Some(cards.clone()),
                _ => None,
            })
            .expect("the opponent declares");
        assert_eq!(laid.len(), 4);
        let distinct: std::collections::BTreeSet<u8> = laid.iter().map(|c| c.0).collect();
        assert_eq!(distinct.len(), 4, "{laid:?}");
    }
}
