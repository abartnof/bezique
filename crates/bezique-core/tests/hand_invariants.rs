//! Whole hands of random legal play, held to what must be true of every hand.

use bezique_core::cards::{CardSet, Rank};
use bezique_core::deal::{Deal, Seat};
use bezique_core::hand::{Action, Hand, Phase, Source};
use bezique_core::rng::Rng;

/// Plays a hand out, choosing uniformly among the legal moves.
fn random_hand(seed: u64) -> Hand {
    let mut deal_rng = Rng::stream(seed, 0);
    let mut choose = Rng::stream(seed, 1);
    let mut hand = Hand::new(Deal::shuffled(&mut deal_rng));
    let mut moves = 0;
    while let Some(_seat) = hand.to_move() {
        let legal = hand.legal();
        assert!(!legal.is_empty(), "a seat to move must have a move");
        let pick = legal[choose.below(legal.len() as u64) as usize].clone();
        hand.apply(pick.clone())
            .unwrap_or_else(|why| panic!("legal move {pick:?} refused: {why}"));
        moves += 1;
        assert!(moves < 200, "the hand must end");
    }
    hand
}

#[test]
fn every_hand_ends_after_thirty_two_tricks_with_every_card_taken() {
    for seed in 0..2000 {
        let hand = random_hand(seed);
        assert_eq!(hand.phase(), Phase::Over);
        assert_eq!(hand.tricks().len(), 32);
        assert!(hand.stock().is_empty());
        let taken = hand.won(Seat::Dealer).union(hand.won(Seat::Pone));
        assert_eq!(taken, CardSet::PACK, "seed {seed}");
        assert!(hand
            .won(Seat::Dealer)
            .intersect(hand.won(Seat::Pone))
            .is_empty());
        for seat in Seat::BOTH {
            assert!(hand.holdings(seat).cards().is_empty());
            assert_eq!(hand.won(seat).len() % 2, 0);
        }
    }
}

#[test]
fn the_log_reconciles_brisques_make_160_and_the_last_trick_10() {
    for seed in 2000..4000 {
        let hand = random_hand(seed);
        // The score rebuilt without the log: brisques from the cards taken,
        // combinations from what was announced and scored, the last trick
        // from the tricks. Only the sevens are counted from the log.
        let rebuilt = |seat: Seat| -> u32 {
            let brisques = 10 * hand.won(seat).cards().filter(|c| c.is_brisque()).count() as u32;
            let melds: u32 = hand
                .holdings(seat)
                .announced
                .iter()
                .filter(|a| a.scored)
                .map(|a| a.declared.meld.points(hand.trump()))
                .sum();
            let last = if hand.tricks()[31].winner == seat {
                10
            } else {
                0
            };
            let sevens = 10
                * hand
                    .log()
                    .iter()
                    .filter(|s| {
                        s.seat == seat && matches!(s.source, Source::Dix | Source::TurnedSeven)
                    })
                    .count() as u32;
            brisques + melds + last + sevens
        };
        for seat in Seat::BOTH {
            assert_eq!(hand.score(seat), rebuilt(seat), "seed {seed}");
            let brisques: u32 = hand
                .log()
                .iter()
                .filter(|s| s.seat == seat && s.source == Source::Brisques)
                .map(|s| s.points)
                .sum();
            let held = hand.won(seat).cards().filter(|c| c.is_brisque()).count() as u32;
            assert_eq!(brisques, 10 * held, "seed {seed}: brisques scored as taken");
        }
        let brisques: u32 = hand
            .log()
            .iter()
            .filter(|s| s.source == Source::Brisques)
            .map(|s| s.points)
            .sum();
        assert_eq!(brisques, 160);
        let last: Vec<_> = hand
            .log()
            .iter()
            .filter(|s| s.source == Source::LastTrick)
            .collect();
        assert_eq!(last.len(), 1);
        assert_eq!(last[0].points, 10);
        assert_eq!(last[0].seat, hand.tricks()[31].winner);
        let sevens = hand
            .log()
            .iter()
            .filter(|s| matches!(s.source, Source::Dix | Source::TurnedSeven))
            .count();
        assert!(
            sevens <= 2,
            "seed {seed}: there are only two sevens of trumps"
        );
    }
}

#[test]
fn declaring_happens_only_while_the_stock_lasts_and_play_follows_the_rules() {
    for seed in 4000..4500 {
        let mut deal_rng = Rng::stream(seed, 0);
        let mut choose = Rng::stream(seed, 1);
        let mut hand = Hand::new(Deal::shuffled(&mut deal_rng));
        while hand.to_move().is_some() {
            if hand.phase() == Phase::Declare {
                assert!(!hand.in_endgame());
                assert!(hand.tricks().len() <= 24);
            }
            if hand.in_endgame() {
                assert!(hand.tricks().len() >= 24);
                for seat in Seat::BOTH {
                    assert!(hand.holdings(seat).shown.is_empty());
                }
            }
            let legal = hand.legal();
            let pick = legal[choose.below(legal.len() as u64) as usize].clone();
            if let (Phase::Follow, true, Action::Play(card)) =
                (hand.phase(), hand.in_endgame(), &pick)
            {
                let led = hand.led().unwrap();
                let seat = hand.to_move().unwrap();
                let suit = hand.holdings(seat).cards().in_suit(led.suit());
                assert!(
                    suit.is_empty() || card.suit() == led.suit(),
                    "seed {seed}: must follow suit"
                );
            }
            hand.apply(pick).unwrap();
        }
        let sevens_in_tricks = hand
            .won(Seat::Dealer)
            .union(hand.won(Seat::Pone))
            .cards()
            .filter(|c| c.rank() == Rank::SEVEN)
            .count();
        assert_eq!(sevens_in_tricks, 8);
    }
}
