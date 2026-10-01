//! The endgame solver's speed on full endgames (eight cards each), a fresh
//! solver for each, as rungs 3 and 4 use it.
//!
//! ```text
//! cargo run --release -p bezique-core --example solver_bench
//! ```
use bezique_core::cards::{Card, CardSet, Suit};
use bezique_core::rng::Rng;
use bezique_core::solver::{Endgame, Solver};
fn main() {
    let mut rng = Rng::seeded(5);
    let mut positions = Vec::new();
    for _ in 0..400 {
        let mut pack: Vec<Card> = CardSet::PACK.cards().collect();
        rng.shuffle(&mut pack);
        positions.push(Endgame { mover: CardSet::of(&pack[..8]), other: CardSet::of(&pack[8..16]), led: None });
    }
    let start = std::time::Instant::now();
    let mut sum = 0i64;
    for p in &positions { sum += i64::from(Solver::new(Suit::HEARTS).value(*p)); }
    println!("fresh solver: {:.3} ms a position (checksum {sum})", start.elapsed().as_secs_f64() * 1000.0 / positions.len() as f64);
}
