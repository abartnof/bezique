//! `bezique`: play bezique in the terminal.
//!
//! ```text
//! bezique [--seed N] [--level 1|2] [--watch [--reveal] [--fast]] [--no-colour]
//! ```

mod table;

use std::io::{self, IsTerminal};
use std::time::{SystemTime, UNIX_EPOCH};

use table::{Options, Table};

fn main() {
    let mut options = Options {
        seed: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(1, |d| d.as_secs() % 1_000_000),
        level: 2,
        watch: false,
        reveal: false,
        pause: true,
        colour: io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none(),
    };
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--seed" => options.seed = number(args.next(), "--seed"),
            "--level" => options.level = number(args.next(), "--level") as u8,
            "--watch" => options.watch = true,
            "--reveal" => options.reveal = true,
            "--fast" => options.pause = false,
            "--no-colour" | "--no-color" => options.colour = false,
            "-h" | "--help" => {
                println!(
                    "bezique [--seed N] [--level 1|2] [--watch [--reveal] [--fast]] [--no-colour]"
                );
                println!("  --level 1  your opponent plays legally and no more");
                println!("  --level 2  your opponent plays the period maxims (the default)");
                println!(
                    "  --watch    two computer players; Enter after each trick (--fast: no pauses)"
                );
                return;
            }
            other => {
                eprintln!("unknown option {other:?}; try --help");
                std::process::exit(2);
            }
        }
    }
    let stdin = io::stdin();
    let result = Table::new(stdin.lock(), io::stdout(), options).play();
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn number(value: Option<String>, flag: &str) -> u64 {
    value.and_then(|v| v.parse().ok()).unwrap_or_else(|| {
        eprintln!("{flag} needs a number");
        std::process::exit(2);
    })
}
