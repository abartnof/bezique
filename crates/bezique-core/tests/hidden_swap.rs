//! A view must not depend on where the cards it cannot see lie (review of
//! 2026-10-02). `twin_symmetry.rs` checks that for twins; this checks it for
//! hidden cards of different types: swapping two of them, in the deal, and
//! playing the same moves, leaves the view the same until the viewer has
//! held, or can see, either card.

use bezique_core::cards::{Card, CardSet};
use bezique_core::deal::{shuffled_pack, Deal};
use bezique_core::game::{Game, Side};
use bezique_core::hand::{Action, Hand};
use bezique_core::observation::View;
use bezique_core::rng::Rng;

fn touches(a: &Action, types: CardSet) -> bool {
    match a {
        Action::Play(c) | Action::Dix(c) => types.contains(*c),
        Action::Pass => false,
        Action::Declare(x) => x
            .melds
            .iter()
            .chain([&x.score])
            .any(|d| !d.cards.intersect(types).is_empty()),
    }
}

fn game_of(order: &[Card; 64]) -> Game {
    Game::arranged(1, [0, 0], Side::A, Hand::new(Deal::from_order(order))).unwrap()
}

/// Swap positions i and j of the pack; play identical moves; the views of
/// `viewer` must agree until either card can be seen by it.
fn run(seed: u64, i: usize, j: usize, viewer: Side) -> usize {
    let order = shuffled_pack(&mut Rng::stream(seed, 501));
    let (x, y) = (order[i], order[j]);
    if x.card_type() == y.card_type() {
        return 0;
    }
    let mut other = order;
    other.swap(i, j);
    let mut a = game_of(&order);
    let mut b = game_of(&other);
    let types = x.card_type().twins().union(y.card_type().twins());
    let mut choose = Rng::stream(seed, 502);
    let mut compared = 0;
    loop {
        // Stop once the viewer has held, or can see, either card.
        let seat = a.seat_of(viewer);
        let hand = a.hand();
        if !hand.handled(seat).intersect(types).is_empty() {
            break;
        }
        if hand.stock().turn_up().is_some_and(|c| types.contains(c)) {
            break;
        }
        let va = format!("{:?}", View::of(&a, viewer));
        let vb = format!("{:?}", View::of(&b, viewer));
        assert_eq!(va, vb, "seed {seed} swap {i}<->{j} viewer {viewer:?}");
        compared += 1;
        if a.to_move().is_none() || a.hand_number() != 1 {
            break;
        }
        let legal = a.legal();
        let action = legal[choose.below(legal.len() as u64) as usize].clone();
        if touches(&action, types) {
            break;
        }
        a.apply(action.clone()).unwrap();
        if b.apply(action.clone()).is_err() {
            panic!("seed {seed}: {action:?} refused in the twin world");
        }
        if a.hand_number() != 1 {
            break;
        }
    }
    compared
}

/// A card in one player's hand swapped with a stock card, or two stock
/// cards swapped: the other player cannot tell, so its view must not change
/// until it can see either card.
#[test]
fn moving_a_hidden_card_leaves_the_view_unchanged() {
    let pone_at = [0, 1, 2, 6, 7, 10, 11, 12];
    let dealer_at = [3, 4, 5, 8, 9, 13, 14, 15];
    let mut total = 0;
    for seed in 0..600u64 {
        let k = (seed % 8) as usize;
        let j = 17 + (seed as usize * 7) % 47;
        // Viewer A is the dealer: swap a pone card with a stock card.
        total += run(seed, pone_at[k], j, Side::A);
        // Viewer B is the pone: swap a dealer card with a stock card.
        total += run(seed, dealer_at[k], j, Side::B);
        // Two stock cards swapped, both viewers.
        let j2 = 17 + (seed as usize * 13 + 5) % 47;
        if j2 != j {
            total += run(seed, j, j2, Side::A);
            total += run(seed, j, j2, Side::B);
        }
    }
    assert!(total > 20_000, "{total}");
}
