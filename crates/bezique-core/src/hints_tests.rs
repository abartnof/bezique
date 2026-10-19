//! Positions built by hand, where a hint's claim would be false.

use super::*;
use crate::cards::CardSet;
use crate::deal::Seat;
use crate::game::Side;
use crate::hand::Trick;
use crate::melds::Holdings;

const HEARTS: Suit = Suit::HEARTS;

fn set(codes: &str) -> CardSet {
    CardSet::parse(codes).unwrap()
}

fn card(code: &str, copy: u8) -> Card {
    Card::new(CardType::parse(code).unwrap(), copy)
}

/// A view with the hands given; the played pile is filled so that exactly
/// `unseen` is unaccounted for.
fn view(mine: CardSet, theirs: CardSet, unseen: CardSet, stock: usize, phase: Phase) -> View {
    let mut rest = CardSet::PACK
        .minus(mine)
        .minus(theirs)
        .minus(unseen)
        .cards();
    let mut tricks = vec![];
    while let (Some(led), Some(second)) = (rest.next(), rest.next()) {
        tricks.push(Trick {
            leader: Seat::Dealer,
            led,
            second,
            winner: Seat::Dealer,
        });
    }
    View {
        side: Side::A,
        seat: Seat::Dealer,
        trump: HEARTS,
        turn_up: None,
        stock,
        phase,
        to_move: Some(Seat::Dealer),
        leader: Seat::Dealer,
        led: None,
        mine: Holdings::new(mine),
        theirs: Holdings::new(CardSet::EMPTY),
        their_hidden: theirs.len(),
        tricks,
        log: vec![],
        hand_scores: (0, 0),
        totals: (0, 0),
        hand_number: 1,
    }
}

/// Mid-hand, the stock lasting: every card not mine is unseen.
fn stock_view(mine: &str) -> View {
    let mine = set(mine);
    view(mine, CardSet::EMPTY, CardSet::PACK.minus(mine), 12, Phase::Lead)
}

fn following(mine: &str, led: Card) -> View {
    let mine = set(mine);
    let mut v = view(
        mine,
        CardSet::EMPTY,
        CardSet::PACK.minus(mine).without(led),
        12,
        Phase::Follow,
    );
    v.led = Some(led);
    v
}

fn legal(v: &View) -> Vec<Action> {
    v.mine.cards().cards().map(Action::Play).collect()
}

fn reason(v: &View, code: &str) -> Reason {
    let c = v
        .mine
        .cards()
        .cards()
        .find(|c| c.code() == code)
        .unwrap_or_else(|| panic!("{code} not held"));
    classify(v, &legal(v), &Action::Play(c))
}

fn named(r: &Reason, name: &str) -> Option<String> {
    r.params.iter().find_map(|(k, p)| match p {
        Param::Card(c) if *k == name => Some(c.clone()),
        _ => None,
    })
}

#[test]
fn keep_marriage_never_keeps_the_partner_of_the_card_played() {
    // K and Q of clubs together; playing the king cannot keep "the pair".
    let v = stock_view("QC KC 8D 9D 8S 9S 7C 8C");
    let r = reason(&v, "KC");
    assert_ne!(r.key, "keep_marriage", "{r:?}");
}

#[test]
fn keep_four_does_not_count_the_ace_played() {
    let v = stock_view("AS AC AD 7D 8D 9D 8S 9S");
    let r = reason(&v, "AS");
    assert_ne!(r.key, "keep_four", "{r:?}");
    // Three kings kept while an ace is played: the claim holds.
    let v = stock_view("KS KC KD AD 7D 8D 9D 8S");
    let r = reason(&v, "AD");
    assert_eq!(r.key, "keep_four", "{r:?}");
}

#[test]
fn keep_for_bezique_does_not_keep_the_other_half_of_the_card_played() {
    let v = stock_view("QS JD 8D 9D 8C 9C 7C KC");
    let r = reason(&v, "QS");
    assert_ne!(r.key, "keep_for_bezique", "{r:?}");
    let r = reason(&v, "KC");
    assert_eq!(r.key, "keep_for_bezique", "{r:?}");
}

#[test]
fn a_combination_with_no_copy_left_to_come_is_not_kept_for() {
    // Both jacks of diamonds lie on the opponent's table: no bezique can
    // come to the queen of spades held.
    let mine = set("QS KC 8D 9D 8C 9C 7C 8S");
    let jds = set("JD JD");
    let mut v = view(mine, jds, CardSet::PACK.minus(mine).minus(jds), 12, Phase::Lead);
    v.theirs.shown = jds;
    let r = reason(&v, "KC");
    assert_ne!(r.key, "keep_for_bezique", "{r:?}");
    // With one jack of diamonds on their table and one still out, it is kept.
    let mut w = stock_view("QS KC 8D 9D 8C 9C 7C 8S");
    w.theirs.shown = set("JD");
    let r = reason(&w, "KC");
    assert_eq!(r.key, "keep_for_bezique", "{r:?}");
}

#[test]
fn draw_out_trump_names_the_highest_trump() {
    let theirs = set("7C 8C 9C JC QC");
    let mine = set("9H KH AH 7S 8S");
    let v = view(mine, theirs, CardSet::EMPTY, 0, Phase::Lead);
    assert_ne!(reason(&v, "9H").key, "draw_out_trump");
    assert_ne!(reason(&v, "KH").key, "draw_out_trump");
    assert_eq!(reason(&v, "AH").key, "draw_out_trump");
}

#[test]
fn a_plain_master_lead_is_not_one_when_a_trump_can_take_it() {
    // I lead the ten of spades, both aces mine; they hold trumps only.
    let mine = set("TS AS AS 7C 8C");
    let void = set("7H 8H 9H JH QH");
    let v = view(mine, void, CardSet::EMPTY, 0, Phase::Lead);
    assert_ne!(reason(&v, "TS").key, "last_eight_lead_master");
    // With a spade in their hand they must follow suit: nothing beats it.
    let follow = set("7H 8H 9H JH 7S");
    let v = view(mine, follow, CardSet::EMPTY, 0, Phase::Lead);
    let r = reason(&v, "TS");
    assert_eq!(r.key, "last_eight_lead_master", "{r:?}");
}

#[test]
fn let_brisque_go_names_the_cheapest_winner() {
    // A ten of clubs is led; the ace of clubs, 7H and the ace of hearts win.
    let v = following("7H AH AC 8D 9D 8S 9S 7S", card("TC", 0));
    let r = reason(&v, "8D");
    assert_eq!(r.key, "let_brisque_go", "{r:?}");
    assert_eq!(named(&r, "winner").as_deref(), Some("AC"), "{r:?}");
}

#[test]
fn throw_lowest_may_fire_when_the_trick_could_be_won() {
    // A king is led and the ace of its suit is held: the wording must not say
    // the trick cannot be won (web3d/test/hinttext.test.js checks it).
    let v = following("AC 7D 8D 9D 8S 9S 7S 8H", card("KC", 0));
    let r = reason(&v, "7D");
    assert_eq!(r.key, "throw_lowest", "{r:?}");
}

#[test]
fn cheapest_card_that_is_a_combination_piece_is_not_called_least_useful() {
    // Every card but the queen of clubs is a trump, ace or ten, or the king
    // that marries it: the queen is "cheapest" but is half a pair.
    let v = stock_view("QC KC AS TS AD TD AH TH");
    let r = reason(&v, "QC");
    assert_ne!(r.key, "lead_low_nonbrisque", "{r:?}");
    assert_ne!(r.key, "throw_lowest", "{r:?}");
}

#[test]
fn a_twin_in_both_sets_is_counted_twice() {
    // `union` is a bitwise or over physical cards, one bit per copy: the
    // opponent's seen seven and its unseen twin are two cards.
    let theirs = CardSet::EMPTY.with(card("7S", 0));
    let unseen = CardSet::EMPTY.with(card("7S", 1));
    assert_eq!(theirs.union(unseen).len(), 2);
}
