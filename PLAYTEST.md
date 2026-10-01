# Play-testing in the terminal

## Run it

```
cd ~/bezique
./target/release/bezique              # or: cargo run --release -p bezique-cli
```

If `cargo` is not found, run `source ~/.cargo/env` first.

- `--seed 42` deals the same cards again (the seed is printed at the top).
- `--level 1` is a weaker opponent (rung 1); the default is rung 2.
- `--watch` lets two computer players play while you look on (Enter after
  each trick; `--reveal` shows both hands; `--fast` skips the pauses).

## Play it

- Type a card as shown (`AS`, `10D`) or by its number in the list.
- `*` marks a card on the table; type `QS*` for the table copy when you hold
  both.
- After winning a trick, you are offered a numbered list of declarations, but
  only when you have one; `0` draws without declaring.
- Forced cards are played for you. `q` leaves.

## What to look out for

Note the seed and the hand number with anything odd, so it can be replayed.

1. **Declaring.** Are the options clear? Is "and twenty to score" (a
   combination carried to a later trick) understandable? Too many prompts, or
   too few?
2. **Choosing cards.** Is typing codes or numbers comfortable? Is the `*`
   for table cards clear?
3. **The table display and narration.** Is anything missing, too wordy, or
   hard to find?
4. **The rules.** Anything that looks wrong: who takes a trick, the last
   eight tricks (follow and win), the seven of trumps, who gets the last card
   and the turn-up, scores, the game ending the moment someone reaches 1,000,
   a double game.
5. **The opponent.** Anything silly? Too easy?
6. **Pace.** A game is about five hands. Does it drag?
