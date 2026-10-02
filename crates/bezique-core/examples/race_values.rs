use bezique_core::race::value;
fn main() {
    for (m, t) in [
        (0, 0),
        (500, 500),
        (480, 600),
        (520, 600),
        (800, 800),
        (900, 700),
        (700, 900),
        (950, 450),
        (990, 990),
    ] {
        println!(
            "{m:>4} vs {t:>4}: dealing {:+.3}, not dealing {:+.3}",
            value(m, t, true),
            value(m, t, false)
        );
    }
    let start = std::time::Instant::now();
    for i in 0..100 {
        std::hint::black_box(value(i, 0, true));
    }
    println!(
        "{:.2} ms a value from the start",
        start.elapsed().as_secs_f64() * 10.0
    );
}
