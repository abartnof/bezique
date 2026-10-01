# Bezique

Bezique is a card game for two players and two piquet packs shuffled together:
64 cards, every one of them twice. It came out of France in the nineteenth
century, swept Britain from 1869, and stayed a fixture of club and
country-house play into the 1930s. It was Winston Churchill's favourite game.
Tricks are played for the aces and tens they hold, and won tricks earn the
right to lay down marriages, fours, the trump sequence and, best of all, the
queen of spades with the jack of diamonds: bezique.

This project is building a version you can play against the computer, and
learn as you play, as one self-contained web page.

**Status:** in development. The rules engine, a first computer opponent and a
terminal table are written; the stronger opponents and the 3D table are next.

## Playing it now, in a terminal

The browser table is still to come; the game is playable in a terminal today.
With Rust installed:

```
cargo run --release -p bezique-cli                   # you against the computer
cargo run --release -p bezique-cli -- --level 1      # an opponent that plays legally and no more
cargo run --release -p bezique-cli -- --watch        # two computer players; Enter after each trick
cargo run --release -p bezique-cli -- --seed 42      # the same cards every time
```

Type a card as it is shown (`AS`, `10D`; `*` for the copy on the table) or by
its number; declarations are offered as a numbered list; `q` leaves.

## Learning the game

- **[pagat.com](https://www.pagat.com/marriage/bezique.html)**: John McLeod's
  card game site, with the modern rules.
- **R. F. Foster, *Foster's Complete Hoyle* (1897)**, the authority this
  version follows, and **A. Howard Cady, *Bézique* (1896)**. Both are on the
  [Internet Archive](https://archive.org), and their bezique chapters are kept
  in `docs/sources/`.

## The documents

- `docs/RULES.md`: the rules as implemented, with a source for every rule
  and a recorded decision on every point the old books dispute.
- `docs/DESIGN.md`: how the engine, the opponent and the table are built, and
  why.
- `docs/LITERATURE.md`: the literature review behind both: sixty-eight
  sources on the game's history, rules, variants and strategy.
- `PLAN.md`: where the work stands and what comes next.

## How it will work

The rules engine and the computer opponent are written in Rust and compiled
to WebAssembly, and the table is drawn with three.js and Material Design 3. The
whole game will ship as a single HTML page that needs no installation and no
network. It shares its architecture, its table and its card art with a sibling
project, a version of piquet.

## Thanks

- The period writers: **R. F. Foster**, **A. Howard Cady**, **"Cavendish"**
  (Henry Jones), **Louis Hoffmann** and **William Pole**, among many others
  listed in `docs/LITERATURE.md`.
- **John McLeod's pagat.com** and **Howard Fosdick's** bezique rules archived
  there.
- The **Internet Archive** and **Wikisource**, which hold the texts.

## Licence

© 2026 Andrew Bartnof, under the MIT License (`LICENSE`). The period texts in
`docs/sources/` are in the public domain. Third-party pieces will keep their own
licences, listed in `CREDITS.md` as they arrive.
