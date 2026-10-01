# Play-testing in the terminal

## Run it

```
cd ~/bezique
cargo build --release -p bezique-cli     # once, after pulling
./target/release/bezique
```

If `cargo` is not found, run `source ~/.cargo/env` first.

Options (all listed by `--help`):

- `--level 1` to `--level 5`: how strong your opponent is (2 is the default).
  1 plays legally and no more; 2 plays by the maxims of the old books; 3 also
  remembers every card and plays the last eight tricks perfectly; 4 also
  fights for the last trick before the stock runs out; 5 also, near the end
  of the stock, tries each card against guesses at your hand and the stock,
  and plays the one that does best.
- `--persona balanced|concentrates|close|bezique`: its style, drawn at random
  unless you choose. Each plays about as well as the others; they differ in
  how they look.
- `--erratic 0.2`: how often it slips below its level (0 to 1; 0 is the
  default).
- `--seed 42` deals the same cards again (the seed is printed at the top).
- `--watch` lets two computer players play while you look on (Enter after
  each trick; `--reveal` shows both hands; `--fast` skips the pauses).

## Play it

- Type a card as shown (`AS`, `TD`; `10D` works too) or by its number in the list.
- `*` marks a card on the table; type `QS*` for the table copy when you hold
  both.
- After winning a trick, you are offered a numbered list of declarations, but
  only when you have one; `0` draws without declaring.
- Forced cards are played for you.
- `u` takes back your last decision, `h` turns hints on and off, `t` shows
  every trick of the hand (by the old rules a quitted trick may not be looked
  at again, so it is off by default), `q` leaves.

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
5. **The opponent.** Anything silly? Does each level feel stronger than the
   one below? Can you tell the personas apart?
6. **Pace.** A game is about four or five hands. Does it drag?
