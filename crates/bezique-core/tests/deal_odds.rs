//! What the deal and the draw deliver, against the exact odds.
//!
//! Over a hand each player is dealt eight cards and draws once after each of
//! the 24 stock tricks, so ends up having received exactly half the pack. As
//! long as who wins each trick does not depend on the face-down cards (it
//! cannot: nobody sees them), the half a player receives is a uniformly random
//! half. `analysis/bezique_stats.py` computes the chance of receiving each
//! combination's cards exactly, and these tests hold the shuffle, the deal and
//! the draw to those figures.
//!
//! These tests check the deal and the draw: that the pone's positions in the
//! pack, whatever the play, add up to a uniformly random half. They have
//! little power against a subtly biased shuffle (review finding 10): that is
//! the job of the permutation test in `rng.rs`, which is shown to reject the
//! classic wrong shuffles.
//!
//! The winners here are coin flips from a stream of their own. Each figure may
//! stray 4.5 standard errors, which a correct implementation does about once
//! in 150,000 tries; the seeds are fixed, so the test is deterministic anyway.

use bezique_core::cards::{Card, CardSet, CardType, Rank, Suit};
use bezique_core::deal::{Deal, Seat};
use bezique_core::rng::Rng;

const HANDS: u32 = 40_000;

/// Plays out the drawing with random trick winners and returns what each seat
/// received: its eight dealt cards and its 24 draws.
fn received(rng: &mut Rng, coin: &mut Rng) -> [CardSet; 2] {
    let deal = Deal::shuffled(rng);
    let mut got = [deal.hand(Seat::Dealer), deal.hand(Seat::Pone)];
    let (_, _, mut stock) = deal.into_parts();
    while !stock.is_empty() {
        let winner = if coin.below(2) == 0 {
            Seat::Dealer
        } else {
            Seat::Pone
        };
        for seat in [winner, winner.other()] {
            let card = stock
                .draw()
                .expect("the stock holds an even number of cards");
            got[seat.index()] = got[seat.index()].with(card);
        }
    }
    got
}

fn of(code: &str) -> CardType {
    CardType::parse(code).unwrap()
}

fn check(name: &str, hits: u32, exact: f64) {
    let observed = f64::from(hits) / f64::from(HANDS);
    let se = (exact * (1.0 - exact) / f64::from(HANDS)).sqrt();
    assert!(
        (observed - exact).abs() < 4.5 * se,
        "{name}: received in {observed:.5} of hands, exactly {exact:.5} expected (se {se:.5})"
    );
}

#[test]
fn each_player_receives_exactly_half_the_pack() {
    let (mut rng, mut coin) = (Rng::stream(1, 0), Rng::stream(1, 1));
    for _ in 0..1000 {
        let [dealer, pone] = received(&mut rng, &mut coin);
        assert_eq!(dealer.len(), 32);
        assert_eq!(pone.len(), 32);
        assert_eq!(dealer.union(pone), CardSet::PACK);
    }
}

#[test]
fn the_pone_receives_each_combination_as_often_as_the_exact_odds_say() {
    let (mut rng, mut coin) = (Rng::stream(20261001, 0), Rng::stream(20261001, 1));
    let (queen, jack) = (of("QS"), of("JD"));
    let sequence = [Rank::ACE, Rank::TEN, Rank::KING, Rank::QUEEN, Rank::JACK]
        .map(|rank| CardType::new(rank, Suit::SPADES));
    let mut hits = [0u32; 4];
    for _ in 0..HANDS {
        let hand = received(&mut rng, &mut coin)[Seat::Pone.index()];
        let has = |t: CardType, n: u32| hand.count_of(t) >= n;
        hits[0] += u32::from(has(queen, 1) && has(jack, 1));
        hits[1] += u32::from(has(queen, 2) && has(jack, 2));
        let aces = hand
            .cards()
            .filter(|c: &Card| c.rank() == Rank::ACE)
            .count();
        hits[2] += u32::from(aces >= 4);
        hits[3] += u32::from(sequence.iter().all(|&t| has(t, 1)));
    }
    // analysis/bezique_stats.py, "exact odds, two-pack".
    check("bezique", hits[0], 0.564_532_916_991_933_4);
    check("double bezique", hits[1], 0.056_596_409_055_425_45);
    check("four aces", hits[2], 0.646_076_964_198_957_1);
    check("sequence in spades", hits[3], 0.226_394_509_770_008_8);
}
