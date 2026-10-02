//! Twins cannot be told apart, so a player's view must not depend on which
//! copy of a twin is where, for the types that player has never held.
//!
//! The property: swap the two copies of a type everywhere (in the deal and
//! in every move) and the view of a player who has never held that type is
//! unchanged, at every position. Any difference would be information about
//! hidden cards that the real game, with indistinguishable twins, does not
//! give (review of 2026-10-02, finding O1).

use bezique_core::cards::{Card, CardSet, CardType, Suit};
use bezique_core::deal::{shuffled_pack, Deal, Seat, Stock};
use bezique_core::game::{Game, Side};
use bezique_core::hand::{Action, Hand};
use bezique_core::melds::{Announcement, Declared, Holdings, Meld};
use bezique_core::observation::View;
use bezique_core::rng::Rng;

fn swap_card(t: CardType, c: Card) -> Card {
    if c.card_type() == t {
        Card::new(t, 1 - c.copy())
    } else {
        c
    }
}

fn swap_set(t: CardType, s: CardSet) -> CardSet {
    s.cards()
        .fold(CardSet::EMPTY, |acc, c| acc.with(swap_card(t, c)))
}

fn swap_action(t: CardType, a: &Action) -> Action {
    let d = |x: Declared| Declared {
        meld: x.meld,
        cards: swap_set(t, x.cards),
    };
    match a {
        Action::Play(c) => Action::Play(swap_card(t, *c)),
        Action::Dix(c) => Action::Dix(swap_card(t, *c)),
        Action::Pass => Action::Pass,
        Action::Declare(x) => Action::Declare(Announcement {
            melds: x.melds.iter().map(|&m| d(m)).collect(),
            score: d(x.score),
        }),
    }
}

#[test]
fn swapping_twins_a_player_never_held_leaves_their_view_unchanged() {
    let mut checked = 0;
    for seed in 0..50u64 {
        let order = shuffled_pack(&mut Rng::stream(seed, 77));
        let mut choose = Rng::stream(seed, 78);
        let deal = |order: &[Card; 64]| {
            Game::arranged(seed, [0, 0], Side::A, Hand::new(Deal::from_order(order))).unwrap()
        };
        let mut game = deal(&order);
        // One game per type, its twins swapped, kept in step.
        let mut swapped: Vec<(CardType, Game)> = CardType::all()
            .map(|t| (t, deal(&std::array::from_fn(|i| swap_card(t, order[i])))))
            .collect();
        while game.to_move().is_some() && game.hand_number() == 1 {
            let legal = game.legal();
            let action = legal[choose.below(legal.len() as u64) as usize].clone();
            game.apply(action.clone()).unwrap();
            if game.hand_number() != 1 {
                break;
            }
            for (t, other) in &mut swapped {
                other.apply(swap_action(*t, &action)).unwrap();
                for side in Side::BOTH {
                    let seat = game.seat_of(side);
                    if !t.twins().intersect(game.hand().handled(seat)).is_empty() {
                        continue;
                    }
                    checked += 1;
                    let (v, w) = (View::of(&game, side), View::of(other, side));
                    assert_eq!(
                        format!("{v:?}"),
                        format!("{w:?}"),
                        "seed {seed}, {t} swapped, {side:?}'s view differs"
                    );
                }
            }
        }
    }
    assert!(checked > 40_000, "{checked}");
}

fn c(code: &str, copy: u8) -> Card {
    Card::new(CardType::parse(code).unwrap(), copy)
}

fn play(game: &mut Game, code: &str) {
    let t = CardType::parse(code).unwrap();
    let a = game
        .legal()
        .into_iter()
        .find(|a| matches!(a, Action::Play(c) if c.card_type() == t))
        .unwrap();
    game.apply(a).unwrap();
}

fn declare(game: &mut Game, meld: Meld) {
    let a = game
        .legal()
        .into_iter()
        .find(|a| matches!(a, Action::Declare(x) if x.score.meld == meld && x.melds.len() == 1))
        .unwrap();
    game.apply(a).unwrap();
}

/// The dealer declares bezique, then double bezique. Whether it held both
/// queens of spades at the first, or drew the second later, the pone sees the
/// same things happen; the record of the first bezique must read the same.
#[test]
fn a_double_bezique_does_not_tell_whether_both_queens_were_held_at_the_first() {
    let world = |both_at_first: bool| {
        let pone: Vec<Card> = ["7C", "8C", "9C", "7D", "8D", "9D", "8S", "9S"]
            .iter()
            .map(|s| c(s, 0))
            .collect();
        let filler = c("KD", 0);
        let (extra, first_draw) = if both_at_first {
            (c("QS", 0), filler)
        } else {
            (filler, c("QS", 0))
        };
        let dealer = [
            c("QS", 1),
            extra,
            c("JD", 0),
            c("AC", 0),
            c("AC", 1),
            c("AD", 0),
            c("TS", 0),
            c("TS", 1),
        ];
        let turn_up = c("KH", 0);
        let top = [first_draw, c("7S", 0), c("JD", 1), c("7S", 1)];
        let used = CardSet::of(&top)
            .union(CardSet::of(&pone))
            .union(CardSet::of(&dealer))
            .with(turn_up);
        let mut order = top.to_vec();
        order.extend(CardSet::PACK.minus(used).cards());
        let hand = Hand::arranged(
            Suit::HEARTS,
            Stock::new(&order, Some(turn_up)),
            [
                Holdings::new(CardSet::of(&dealer)),
                Holdings::new(CardSet::of(&pone)),
            ],
            Seat::Pone,
            0,
        )
        .unwrap();
        let mut game = Game::arranged(1, [0, 0], Side::A, hand).unwrap();
        play(&mut game, "7C");
        play(&mut game, "AC");
        declare(&mut game, Meld::Bezique);
        play(&mut game, "AC");
        play(&mut game, "8C");
        game.apply(Action::Pass).unwrap();
        play(&mut game, "AD");
        play(&mut game, "9C");
        declare(&mut game, Meld::DoubleBezique);
        View::of(&game, Side::B)
    };
    let (a, b) = (world(true), world(false));
    assert_eq!(
        a.theirs.announced[0].declared.cards,
        b.theirs.announced[0].declared.cards
    );
    assert_eq!(format!("{a:?}"), format!("{b:?}"));
}
