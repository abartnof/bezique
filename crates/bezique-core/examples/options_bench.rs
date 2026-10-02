//! The declaring options' speed, on positions from real games (the turn of
//! declaring, between two rung-4 players).
//!
//! ```text
//! cargo run --release -p bezique-core --example options_bench
//! ```
use bezique_core::agents::rung;
use bezique_core::game::{Game, Side};
use bezique_core::hand::Phase;
use bezique_core::observation::View;
fn main() {
    let mut positions = Vec::new();
    for seed in 0..200 {
        let mut game = Game::new(seed, Side::A);
        let mut agents = [rung(4), rung(4)];
        while let Some(side) = game.to_move() {
            if game.hand().phase() == Phase::Declare {
                positions.push(game.clone());
            }
            let view = View::of(&game, side);
            let action = agents[side.index()].choose(&view, &game.legal());
            game.apply(action).unwrap();
        }
    }
    let start = std::time::Instant::now();
    let mut n = 0;
    for _ in 0..5 {
        for g in &positions {
            n += g.legal().len();
        }
    }
    let each = start.elapsed().as_secs_f64() * 1e6 / (5 * positions.len()) as f64;
    println!(
        "{} declaring positions: {each:.1} µs for the options of each ({n} offered)",
        positions.len()
    );
}
