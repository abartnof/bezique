//! `bezique`: play bezique in the terminal.
//!
//! ```text
//! bezique [--seed N] [--level 1-4] [--persona KEY|random] [--erratic 0-1]
//!         [--watch [--reveal] [--fast]] [--no-colour]
//! ```

mod table;

use std::io::{self, IsTerminal};
use std::time::{SystemTime, UNIX_EPOCH};

use bezique_core::{agents, opponent};
use table::{Options, Table};

fn main() {
    let mut options = Options {
        seed: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(1, |d| d.as_secs() % 1_000_000),
        level: 2,
        persona: None,
        erraticism: 0.0,
        watch: false,
        reveal: false,
        pause: true,
        colour: io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none(),
    };
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--seed" => options.seed = number(args.next(), "--seed"),
            "--level" => {
                let level = number(args.next(), "--level");
                if !(1..=u64::from(agents::TOP)).contains(&level) {
                    fail(&format!("--level is 1 to {}", agents::TOP));
                }
                options.level = level as u8;
            }
            "--persona" => {
                let key = args.next().unwrap_or_default();
                if key == "random" {
                    options.persona = None;
                } else if opponent::persona(&key).is_some() {
                    options.persona = Some(key);
                } else {
                    let keys: Vec<&str> = opponent::personas().iter().map(|p| p.key).collect();
                    fail(&format!(
                        "--persona is random or one of: {}",
                        keys.join(", ")
                    ));
                }
            }
            "--erratic" => {
                options.erraticism = args
                    .next()
                    .and_then(|v| v.parse().ok())
                    .filter(|e: &f64| (0.0..=1.0).contains(e))
                    .unwrap_or_else(|| fail("--erratic needs a number from 0 to 1"));
            }
            "--watch" => options.watch = true,
            "--reveal" => options.reveal = true,
            "--fast" => options.pause = false,
            "--no-colour" | "--no-color" => options.colour = false,
            "-h" | "--help" => {
                help();
                return;
            }
            other => fail(&format!("unknown option {other:?}; try --help")),
        }
    }
    let stdin = io::stdin();
    let result = Table::new(stdin.lock(), io::stdout(), options).play();
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn help() {
    println!(
        "bezique [--seed N] [--level 1-{}] [--persona KEY|random] [--erratic 0-1] [--watch [--reveal] [--fast]] [--no-colour]",
        agents::TOP
    );
    for level in 1..=agents::TOP {
        let default = if level == 2 { " (the default)" } else { "" };
        println!(
            "  --level {level}  your opponent {}{default}",
            agents::describe(level)
        );
    }
    println!("  --persona  how it plays; drawn at random unless chosen:");
    for p in opponent::personas() {
        println!("               {:<10} {}", p.key, p.described);
        println!("               {:<10} (after {})", "", p.after);
    }
    println!("  --erratic  how often it slips below its level, from 0 (never, the default) to 1");
    println!("  --watch    two computer players; Enter after each trick (--fast: no pauses)");
}

fn number(value: Option<String>, flag: &str) -> u64 {
    value
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| fail(&format!("{flag} needs a number")))
}

fn fail(message: &str) -> ! {
    eprintln!("{message}");
    std::process::exit(2);
}
