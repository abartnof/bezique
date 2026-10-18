//! Why the top rung would play what it plays: a reason a client can word.
//!
//! The rung chooses by weighing; this module classifies the choice after the
//! fact, against the alternatives, with cheap deterministic tests. The result
//! is a stable `key` plus the few numbers or cards the wording needs. A choice
//! that fits no case is `generic`. The wording and the rule behind each key
//! live in the client (`web3d/src/hinttext.js`) and in `docs/HINTS.md`.

use crate::cards::{Card, CardType, Rank, Suit};
use crate::hand::{Action, Phase};
use crate::melds::{Class, Meld};
use crate::observation::View;
use crate::tricks;

/// One value a reason carries.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Param {
    /// A card, by its code (`7D`).
    Card(String),
    Int(i64),
    /// A word: a meld's key or a suit's name.
    Word(String),
}

/// The reason for a hint.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Reason {
    pub key: &'static str,
    pub params: Vec<(&'static str, Param)>,
}

impl Reason {
    fn new(key: &'static str) -> Reason {
        Reason {
            key,
            params: vec![],
        }
    }
    fn card(mut self, name: &'static str, card: Card) -> Reason {
        self.params.push((name, Param::Card(card.code())));
        self
    }
    fn int(mut self, name: &'static str, n: i64) -> Reason {
        self.params.push((name, Param::Int(n)));
        self
    }
    fn word(mut self, name: &'static str, w: &str) -> Reason {
        self.params.push((name, Param::Word(w.to_string())));
        self
    }
}

/// Every key `classify` can return, for the documentation and the tests.
pub const KEYS: [&str; 31] = [
    "declare_now",
    "score_carried",
    "single_before_double",
    "royal_before_sequence",
    "exchange_dix",
    "show_dix",
    "pass",
    "take_brisque",
    "take_to_declare",
    "win_last_stock_trick",
    "take_with_lowest_winner",
    "take_sparing_pieces",
    "let_brisque_go",
    "throw_lowest",
    "lead_low_nonbrisque",
    "play_declared_card",
    "lead_master_brisque",
    "nothing_decisive",
    "keep_for_bezique",
    "keep_marriage",
    "keep_sequence",
    "keep_four",
    "save_seven_for_dix",
    "last_eight_must_win",
    "last_eight_give_up",
    "last_eight_lead_master",
    "last_eight_lead_low",
    "last_eight_lead_ace",
    "draw_out_trump",
    "solved_endgame",
    "generic",
];

fn meld_word(meld: Meld, trump: Suit) -> (String, Option<Suit>) {
    match meld {
        Meld::Marriage(s) if s == trump => ("royal_marriage".into(), Some(s)),
        Meld::Marriage(s) => ("marriage".into(), Some(s)),
        Meld::Sequence => ("sequence".into(), None),
        Meld::Bezique => ("bezique".into(), None),
        Meld::DoubleBezique => ("double_bezique".into(), None),
        Meld::Four(r) => (
            match r {
                Rank::ACE => "four_aces",
                Rank::KING => "four_kings",
                Rank::QUEEN => "four_queens",
                _ => "four_jacks",
            }
            .into(),
            None,
        ),
    }
}

/// What a card is being kept for, if anything: the combination it can still
/// serve in, reckoned from the player's own hand and the cards played.
fn work(view: &View, card: Card) -> Option<Reason> {
    let gone = view.played();
    let available = |t: CardType| 2 - gone.count_of(t).min(2) as i32;
    let mine = &view.mine;
    let held = mine.cards();
    let holds = |rank: Rank, suit: Suit| held.count_of(CardType::new(rank, suit)) > 0;
    let trump = view.trump;
    let t = card.card_type();
    let qs = CardType::new(Rank::QUEEN, Suit::SPADES);
    let jd = CardType::new(Rank::JACK, Suit::DIAMONDS);
    if (t == qs || t == jd)
        && mine.level(card, Class::Bezique) < 2
        && available(qs) >= 1
        && available(jd) >= 1
    {
        return Some(Reason::new("keep_for_bezique").card("kept", card));
    }
    let seq = [Rank::ACE, Rank::TEN, Rank::KING, Rank::QUEEN, Rank::JACK];
    if card.suit() == trump
        && seq.contains(&card.rank())
        && mine.level(card, Class::Marriages) < 2
        && seq.iter().filter(|&&r| holds(r, trump)).count() >= 3
        && seq.iter().all(|&r| available(CardType::new(r, trump)) >= 1)
    {
        return Some(Reason::new("keep_sequence").card("kept", card));
    }
    let partner = match card.rank() {
        Rank::KING => Some(Rank::QUEEN),
        Rank::QUEEN => Some(Rank::KING),
        _ => None,
    };
    if let Some(p) = partner {
        if holds(p, card.suit()) && mine.level(card, Class::Marriages) == 0 {
            let points = if card.suit() == trump { 40 } else { 20 };
            return Some(
                Reason::new("keep_marriage")
                    .card("kept", card)
                    .card("partner", Card::new(CardType::new(p, card.suit()), 0))
                    .int("points", points),
            );
        }
    }
    if [Rank::ACE, Rank::KING, Rank::QUEEN, Rank::JACK].contains(&card.rank())
        && mine.level(card, Class::Fours) == 0
        && held.cards().filter(|c| c.rank() == card.rank()).count() >= 3
        && Suit::ALL
            .iter()
            .map(|&s| available(CardType::new(card.rank(), s)))
            .sum::<i32>()
            >= 4
    {
        return Some(Reason::new("keep_four").card("kept", card));
    }
    if card.rank() == Rank::SEVEN && card.suit() == trump {
        return Some(Reason::new("save_seven_for_dix").card("kept", card));
    }
    None
}

/// Classify `advice`, the top rung's choice among `legal` in `view`.
pub fn classify(view: &View, legal: &[Action], advice: &Action) -> Reason {
    let trump = view.trump;
    match advice {
        Action::Pass => Reason::new("pass"),
        Action::Dix(seven) => match view.turn_up {
            Some(t) if t.rank() == Rank::SEVEN && t.suit() == trump => Reason::new("show_dix"),
            Some(t) => Reason::new("exchange_dix")
                .card("card", *seven)
                .card("turn_up", t),
            None => Reason::new("show_dix"),
        },
        Action::Declare(x) => {
            let has = |m: Meld| {
                legal
                    .iter()
                    .any(|a| matches!(a, Action::Declare(y) if y.score.meld == m))
            };
            let meld = x.score.meld;
            let points = meld.points(trump) as i64;
            let (word, suit) = meld_word(meld, trump);
            let base = if meld == Meld::Bezique && has(Meld::DoubleBezique) {
                Reason::new("single_before_double")
            } else if meld == Meld::Marriage(trump) && has(Meld::Sequence) {
                Reason::new("royal_before_sequence")
            } else if x.melds.is_empty() {
                Reason::new("score_carried")
            } else {
                Reason::new("declare_now")
            };
            let mut r = base.word("meld", &word).int("points", points);
            if let Some(s) = suit {
                r = r.word("suit", s.name());
            }
            r
        }
        Action::Play(card) => classify_play(view, legal, *card),
    }
}

fn plays(legal: &[Action]) -> Vec<Card> {
    legal
        .iter()
        .filter_map(|a| match a {
            Action::Play(c) => Some(*c),
            _ => None,
        })
        .collect()
}

/// The cost of giving a card up: a trump costs most, then an ace or ten, then
/// by rank.
fn cost(trump: Suit, c: Card) -> (bool, bool, Rank) {
    (c.suit() == trump, c.is_brisque(), c.rank())
}

/// `key` for a card that is the cheapest of the legal plays, with the dearest
/// it spares; `None` if some other card is cheaper.
fn endgame_cheapest(view: &View, all: &[Card], card: Card, key: &'static str) -> Option<Reason> {
    let trump = view.trump;
    if all.iter().any(|&c| cost(trump, c) < cost(trump, card)) {
        return None;
    }
    let r = Reason::new(key).card("card", card);
    let spared = all
        .iter()
        .copied()
        .filter(|&c| c.card_type() != card.card_type() && cost(trump, c) > cost(trump, card))
        .max_by_key(|&c| (cost(trump, c), c.0));
    Some(match spared {
        Some(c) => r.card("spared", c),
        None => r,
    })
}

/// The last eight tricks, where the opponent's hand is exactly the cards not
/// yet seen: the top rung plays them as an open-hand puzzle, so when no
/// simpler case fits and the exact solution is the card chosen, say so.
fn solved_endgame(view: &View, card: Card) -> Reason {
    let theirs = view.theirs.cards().union(view.unseen());
    match crate::agents::solve(view) {
        Some(best) if best.card_type() == card.card_type() => Reason::new("solved_endgame")
            .card("card", card)
            .int("count", i64::from(theirs.len())),
        _ => Reason::new("generic"),
    }
}

/// A card worth holding on to: a trump, an ace or a ten, or one a
/// combination in hand or in prospect still needs.
fn valuable(view: &View, c: Card) -> bool {
    c.suit() == view.trump || c.is_brisque() || work(view, c).is_some()
}

/// The card to name when nothing separates the plays: the dearest of the
/// other legal cards that is worth keeping.
fn dearest_kept(view: &View, all: &[Card], card: Card) -> Option<Card> {
    all.iter()
        .copied()
        .filter(|c| c.card_type() != card.card_type() && valuable(view, *c))
        .max_by_key(|&c| (cost(view.trump, c), c.0))
}

fn classify_play(view: &View, legal: &[Action], card: Card) -> Reason {
    let trump = view.trump;
    let all = plays(legal);
    let in_endgame = view.stock == 0 && view.turn_up.is_none();
    let led = if view.phase == Phase::Follow {
        view.led
    } else {
        None
    };
    if in_endgame {
        if let Some(led) = led {
            let forced = all.iter().all(|&c| tricks::second_wins(led, c, trump));
            if forced && tricks::second_wins(led, card, trump) {
                return Reason::new("last_eight_must_win")
                    .card("card", card)
                    .card("led", led);
            }
            // No card wins: the choice is what to give up, and what to keep.
            return endgame_cheapest(view, &all, card, "last_eight_give_up")
                .map(|r| r.card("led", led))
                .unwrap_or_else(|| solved_endgame(view, card));
        }
        if card.rank() == Rank::ACE && card.suit() != trump {
            return Reason::new("last_eight_lead_ace").card("card", card);
        }
        let trumps = all.iter().filter(|c| c.suit() == trump).count();
        if card.suit() == trump && trumps >= 3 {
            return Reason::new("draw_out_trump")
                .card("card", card)
                .int("trumps", trumps as i64);
        }
        // A lead nothing still out can beat in its suit.
        let out = view.unseen().union(view.theirs.cards());
        if !out
            .cards()
            .any(|c| c.suit() == card.suit() && c.rank() > card.rank())
        {
            let kind = if card.suit() == trump {
                "trump"
            } else {
                "plain"
            };
            return Reason::new("last_eight_lead_master")
                .card("card", card)
                .word("kind", kind);
        }
        return endgame_cheapest(view, &all, card, "last_eight_lead_low")
            .unwrap_or_else(|| solved_endgame(view, card));
    }
    if let Some(led) = led {
        let winners: Vec<Card> = all
            .iter()
            .copied()
            .filter(|&c| tricks::second_wins(led, c, trump))
            .collect();
        if winners.contains(&card) {
            if led.is_brisque() {
                return Reason::new("take_brisque")
                    .card("card", card)
                    .card("led", led);
            }
            let waiting = view
                .mine
                .candidates(trump)
                .into_iter()
                .chain(view.mine.carried())
                .map(|d| d.meld.points(trump) as i64)
                .max()
                .unwrap_or(0);
            if waiting > 0 {
                return Reason::new("take_to_declare")
                    .card("card", card)
                    .card("led", led)
                    .int("points", waiting);
            }
            if view.stock == 1 {
                return Reason::new("win_last_stock_trick")
                    .card("card", card)
                    .card("led", led);
            }
            // Cheapest by the order of play: a plain-suit winner before a trump.
            let cost = |c: Card| (c.suit() == trump, c.rank());
            if winners.iter().all(|&w| cost(w) >= cost(card)) {
                return Reason::new("take_with_lowest_winner")
                    .card("card", card)
                    .card("led", led);
            }
            // A cheaper winner exists, but every one is a piece of a
            // combination: the card played is the cheapest that is free.
            let cheaper: Vec<Card> = winners
                .iter()
                .copied()
                .filter(|&w| cost(w) < cost(card))
                .collect();
            if let Some(kept) = cheaper
                .iter()
                .copied()
                .filter(|&w| work(view, w).is_some())
                .max_by_key(|&w| (cost(w), w.0))
            {
                if cheaper.iter().all(|&w| work(view, w).is_some()) {
                    return Reason::new("take_sparing_pieces")
                        .card("card", card)
                        .card("led", led)
                        .card("kept", kept);
                }
            }
            return Reason::new("generic");
        }
        if led.is_brisque() && !winners.is_empty() {
            return Reason::new("let_brisque_go")
                .card("card", card)
                .card("led", led)
                .card("winner", winners[0]);
        }
    }
    // A card given up or led: what was spared, if the choice shows it.
    if let Some(r) = all
        .iter()
        .filter(|c| c.card_type() != card.card_type() && c.rank() <= card.rank())
        .find_map(|&c| work(view, c))
    {
        return r.card("card", card);
    }
    if view.mine.shown.contains(card) && work(view, card).is_none() {
        return Reason::new("play_declared_card").card("card", card);
    }
    let spared: Option<Card> = all
        .iter()
        .copied()
        .filter(|c| c.is_brisque() && c.card_type() != card.card_type())
        .max_by_key(|c| (c.rank(), c.0));
    if !card.is_brisque() {
        // Cheap by the order of play: no legal card costs less.
        if all.iter().all(|&c| cost(trump, c) >= cost(trump, card)) {
            let key = if led.is_some() {
                "throw_lowest"
            } else {
                "lead_low_nonbrisque"
            };
            let r = Reason::new(key).card("card", card);
            return match spared {
                Some(s) => r.card("spared", s),
                None => r,
            };
        }
        // No clear case: a cheap card, and the dear one to hold on to.
        if !valuable(view, card) {
            if let Some(kept) = dearest_kept(view, &all, card) {
                return Reason::new("nothing_decisive")
                    .card("card", card)
                    .card("kept", kept);
            }
        }
        return Reason::new("generic");
    }
    if led.is_none() && card.suit() != trump {
        // An ace or ten no higher card of its suit can beat, bar a trump.
        let out = view.unseen().union(view.theirs.cards());
        if !out
            .cards()
            .any(|c| c.suit() == card.suit() && c.rank() > card.rank())
        {
            let r = Reason::new("lead_master_brisque").card("card", card);
            return match view
                .mine
                .cards()
                .cards()
                .filter(|c| c.suit() == card.suit() && c.rank() > card.rank())
                .max_by_key(|c| (c.rank(), c.0))
            {
                Some(k) => r.card("kept", k),
                None => r,
            };
        }
    }
    Reason::new("generic")
}
