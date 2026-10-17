#!/usr/bin/env python3
"""The opponent's phrase bank: what is said at the table, by game moment.

    python3 tools/phrases.py           # rewrite docs/PHRASES.md and web3d/words.json
    python3 tools/phrases.py --check   # fail if either is out of date

The single source of truth (pattern after strohmandeln's tools/phrases.py).
A group is a moment; each wording in it is a tuple (text, source, locator).
`source` is a key of SOURCES; "T" marks an interpretation: a plain remark the
period sources do not spell out. web3d/test/words.test.js checks the result.

Slots in a text: {formula} (the engine's own declaring formula), {points}
(the points of a trick), {gap} (the points between the scores).
"""

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

SOURCES = {
    "F97": ("Foster, Foster's Complete Hoyle (1897), Bezique, pp. 337-345",
            "docs/sources/foster-1897-bezique.txt"),
    "C96": ("Cady, Bezique (1896), the rules of play (no page recorded in the text copy)",
            "docs/sources/cady-1896-bezique.txt"),
    "WP70": ("Westminster Papers (1870), a correspondent calling bezique the slowest of slow games; "
             "second hand, docs/LITERATURE.md section 3.4 [S16]", "docs/LITERATURE.md"),
    "T": ("Interpretation: table talk and card-room banter of the period are not recorded in the "
          "sources read, so these remarks are the table's own, kept few and plain", ""),
}

# Mode -> probability of a moment's line, by kind. A call is the game itself
# (the declaring formula, the cut, the end) and is always said.
MODES = {
    "quiet": {"call": 1.0, "remark": 0.0, "chatter": 0.0},
    "normal": {"call": 1.0, "remark": 0.6, "chatter": 0.12},
    "chatty": {"call": 1.0, "remark": 1.0, "chatter": 0.45},
}
DEFAULT_MODE = "normal"

# The personas (opponent::personas() keys): the same bank, weighted a little
# differently. A weight multiplies a group's probability (never a call's).
PERSONAS = {
    "balanced": {},
    "collector": {"decl_you_marriage": 1.5, "decl_them_lucky": 1.4, "decl_you_four": 1.3, "play_them_low": 1.3},
    "hunter": {"play_them_brisque": 1.8, "play_you_brisque": 1.6, "play_them_high": 1.4, "play_them_trump": 1.3},
    "cautious": {"play_them_brisque": 0.6, "play_them_high": 0.6, "pace": 1.5, "hand_ahead_them": 0.7, "hand_ahead_you": 1.3, "last_eight": 1.4},
    "gambler": {"decl_you_bezique": 1.8, "decl_you_double": 1.8, "decl_you_four": 1.5, "dix_you": 1.4, "decl_them_lucky": 1.5},
    "specialist": {"play_them_low": 1.6, "play_you_low": 1.5, "play_them_trump": 1.4, "stock_low": 1.5, "last_eight": 1.5},
}

# (group, kind, moment, [(text, source, locator)])
GROUPS = [
    ("start", "call", "the sitting begins", [
        ("Good evening. Two packs, sixty-four cards, a thousand up.", "F97", "p. 339"),
        ("Ace, ten, king, queen, jack, nine, eight, seven: that is the rank.", "F97", "p. 338"),
        ("Cut for the deal: the higher card chooses.", "F97", "p. 339"),
        ("Bezique: queen of spades, jack of diamonds, and some patience.", "F97", "p. 341"),
        ("A game for two. I shall explain as we go.", "T", ""),
    ]),
    ("cut_you_high", "call", "the cut: yours is higher", [
        ("You cut higher: the choice of dealing is yours.", "F97", "p. 339"),
        ("Your cut is the higher. Will you deal, or leave it to me?", "F97", "p. 339"),
    ]),
    ("cut_them_high", "call", "the cut: mine is higher", [
        ("The higher cut is mine, so I choose to deal.", "F97", "p. 339"),
        ("I cut higher: the first deal, then.", "F97", "p. 339"),
    ]),
    ("deal_them", "call", "your opponent deals", [
        ("My deal: eight cards each, the seventeenth turned for trumps.", "F97", "p. 339"),
        ("Three, two, three, and the trump turned up. You lead.", "F97", "pp. 339-340"),
        ("I deal; you, not dealing, lead any card you like.", "F97", "p. 339"),
    ]),
    ("deal_you", "call", "you deal", [
        ("Your deal: the dealing alternates, and I lead first.", "C96", ""),
        ("Deal away: three, two, three. I begin.", "F97", "p. 339"),
        ("The non-dealer leads, so I shall begin.", "F97", "pp. 339-340"),
    ]),
    ("seven_them", "remark", "a seven is turned up and the dealer (your opponent) scores ten", [
        ("A seven turned up: ten for the dealer, and that is I.", "F97", "p. 339"),
        ("Ten for the seven turned up. A fortunate start.", "C96", ""),
    ]),
    ("seven_you", "remark", "a seven is turned up and the dealer (you) scores ten", [
        ("A seven turned up: ten to you as dealer.", "F97", "p. 339"),
        ("Ten to you for the seven, and trumps are made. Lucky.", "C96", ""),
    ]),
    ("play_them_brisque", "chatter", "your opponent plays an ace or a ten", [
        ("A brisque: ten for whoever takes it.", "F97", "p. 340"),
        ("Aces and tens are the brisques, ten points apiece.", "F97", "p. 340"),
        ("Only a trick with a brisque in it is worth the winning, bar the lead.", "F97", "p. 340"),
    ]),
    ("play_them_trump", "chatter", "your opponent plays a trump", [
        ("A trump: it beats any plain suit.", "F97", "p. 340"),
        ("Trumps win plain suits, and I need not follow suit while the stock lasts.", "F97", "p. 340"),
    ]),
    ("play_them_high", "chatter", "your opponent plays a king or queen", [
        ("A high card, to keep the lead.", "F97", "p. 340"),
        ("The lead is worth keeping: it carries the right to declare.", "F97", "p. 340"),
    ]),
    ("play_them_low", "chatter", "your opponent plays a seven, eight, nine or jack", [
        ("A small card: nothing here worth winning.", "F97", "p. 340"),
        ("A small one. While the stock lasts, nobody need follow suit.", "F97", "p. 340"),
        ("Just a seven, to see what you hold.", "T", ""),
    ]),
    ("play_you_brisque", "chatter", "you play an ace or a ten", [
        ("A brisque on the table: ten to whoever wins it.", "F97", "p. 340"),
        ("You lead a brisque. I shall give that some thought.", "T", ""),
    ]),
    ("play_you_trump", "chatter", "you play a trump", [
        ("A trump: that wins unless I hold a higher one.", "F97", "p. 340"),
        ("Trumps already? Quite in order while the stock lasts.", "F97", "p. 340"),
    ]),
    ("play_you_high", "chatter", "you play a king or queen", [
        ("A high card: if it holds, the lead stays yours.", "F97", "p. 340"),
        ("Playing to keep the lead, I think.", "T", ""),
    ]),
    ("play_you_low", "chatter", "you play a small card", [
        ("A small card thrown: there is no need to follow suit yet.", "F97", "p. 340"),
        ("Prudent: nothing in this one worth giving away.", "T", ""),
    ]),
    ("trick_them", "remark", "your opponent takes a trick", [
        ("That trick is mine, and with it the right to declare.", "F97", "p. 340"),
        ("I take it and draw first: the winner always does.", "F97", "p. 340"),
        ("Mine, and the lead with it.", "F97", "p. 340"),
    ]),
    ("trick_you", "remark", "you take a trick", [
        ("Yours: declare before you draw, if you hold anything.", "F97", "p. 341"),
        ("That one is yours; the winner draws first.", "F97", "p. 340"),
        ("Well taken.", "T", ""),
    ]),
    ("trick_rich_them", "remark", "your opponent takes a trick worth twenty or more", [
        ("{points} in one trick: brisques are worth the taking.", "F97", "p. 340"),
        ("{points} to me there. The cards fell kindly.", "T", ""),
    ]),
    ("trick_rich_you", "remark", "you take a trick worth twenty or more", [
        ("{points} in that trick to you: brisques tell.", "F97", "p. 340"),
        ("{points} to you, and well earned.", "T", ""),
    ]),
    ("streak_you", "chatter", "you take three tricks running", [
        ("Three tricks running to you: the cards are with you.", "T", ""),
        ("You have the run of the cards just now.", "T", ""),
    ]),
    ("streak_them", "chatter", "your opponent takes three tricks running", [
        ("The run of the cards is mine for the moment. It will turn.", "T", ""),
        ("Three running: I cannot claim it as skill, only luck.", "T", ""),
    ]),
    ("decl_them", "call", "your opponent declares: the formula as the engine words it", [
        ("{formula}", "F97", "pp. 342-343"),
    ]),
    ("decl_them_lucky", "remark", "your opponent declares a sequence, double bezique or four aces", [
        ("The cards were kind to me there.", "T", ""),
        ("I could not have asked for a better draw.", "T", ""),
    ]),
    ("decl_you_marriage", "remark", "you declare a marriage in a plain suit", [
        ("A marriage: king and queen of one suit, twenty.", "F97", "p. 341"),
        ("Twenty for the marriage; it would be forty in trumps.", "F97", "p. 341"),
    ]),
    ("decl_you_royal", "remark", "you declare the royal marriage", [
        ("The royal marriage: king and queen of trumps, forty.", "F97", "p. 341"),
        ("Forty for the trump marriage, and the sequence may follow.", "F97", "p. 342"),
    ]),
    ("decl_you_sequence", "remark", "you declare the sequence in trumps", [
        ("Ace, ten, king, queen, jack of trumps: two-fifty!", "F97", "p. 341"),
        ("A sequence in trumps: two hundred and fifty. Splendid.", "F97", "p. 341"),
    ]),
    ("decl_you_bezique", "remark", "you declare bezique", [
        ("Queen of spades and jack of diamonds: forty for bezique.", "F97", "p. 341"),
        ("Bezique! A second pair would make it five hundred.", "F97", "p. 341"),
    ]),
    ("decl_you_double", "remark", "you declare double bezique", [
        ("Double bezique: five hundred, the great prize of the game.", "F97", "p. 341"),
        ("Both queens and both jacks: five hundred. My congratulations.", "F97", "p. 341"),
    ]),
    ("decl_you_four", "remark", "you declare four aces, kings, queens or jacks", [
        ("Four of a kind: aces a hundred, kings eighty, queens sixty, jacks forty.", "F97", "p. 341"),
        ("A fine four. Only one declaration to a trick, remember.", "F97", "p. 341"),
    ]),
    ("dix_them", "remark", "your opponent exchanges or shows the seven of trumps", [
        ("The seven of trumps, ten points, exchanged for the turn-up.", "F97", "p. 340"),
        ("Ten for the seven of trumps, once the trick is won.", "F97", "p. 340"),
    ]),
    ("dix_you", "remark", "you exchange or show the seven of trumps", [
        ("Ten for the seven of trumps: a fair exchange.", "F97", "p. 340"),
        ("The seven goes for the turn-up, and ten to you. Well spotted.", "F97", "p. 340"),
    ]),
    ("stock_low", "remark", "the stock is running low", [
        ("The stock runs low; only a few draws are left.", "F97", "p. 343"),
        ("Count the stock if you wish: a few tricks yet before it is spent.", "F97", "p. 343"),
    ]),
    ("stock_out", "call", "the stock is exhausted", [
        ("The stock is spent: the cards laid down go back into the hands.", "F97", "p. 343"),
        ("No more declarations now that the stock is gone.", "F97", "p. 341"),
    ]),
    ("stock_last_you", "remark", "you take the last card of the stock", [
        ("You take the last card, and the turn-up comes to me.", "F97", "p. 343"),
    ]),
    ("stock_last_them", "remark", "your opponent takes the last card of the stock", [
        ("I take the last card, and the turn-up is yours.", "F97", "p. 343"),
    ]),
    ("last_eight", "remark", "the last eight tricks begin", [
        ("Eight tricks left, and each must follow suit and win if it can.", "F97", "p. 343"),
        ("The last eight tricks: brisques still count ten each.", "C96", ""),
        ("The last trick of all is worth ten.", "F97", "p. 343"),
    ]),
    ("last_trick_you", "remark", "you take the last trick of all", [
        ("The last trick of all, and ten for it, to you.", "F97", "p. 343"),
    ]),
    ("last_trick_them", "remark", "your opponent takes the last trick of all", [
        ("The last trick of all is mine: ten for it.", "F97", "p. 343"),
    ]),
    ("hand_ahead_them", "remark", "the hand ends, your opponent ahead in the game", [
        ("That is the hand. I lead by {gap}, but a thousand is a long road.", "C96", ""),
        ("The cards have run my way this hand, and that is all it is.", "T", ""),
    ]),
    ("hand_ahead_you", "remark", "the hand ends, you ahead in the game", [
        ("That is the hand. You lead by {gap}, and the thousand is near.", "C96", ""),
        ("The cards are with you; I am trying to turn them.", "T", ""),
    ]),
    ("hand_level", "remark", "the hand ends, the game level", [
        ("Level at the end of the hand. A thousand is the game.", "C96", ""),
        ("Nothing between us yet.", "T", ""),
    ]),
    ("pace", "chatter", "a new hand, some way into the game", [
        ("They called bezique the slowest of slow games. We shall see.", "WP70", ""),
        ("No hurry: the cards will wait.", "T", ""),
    ]),
    ("double_you_threatened", "remark", "the hand ends, and you could still lose a double game", [
        ("Mind the double: a thousand before you reach five hundred counts twice.", "F97", "p. 339"),
        ("Five hundred is your safe harbour: reach it and the game cannot count double.", "F97", "p. 339"),
    ]),
    ("double_them_threatened", "remark", "the hand ends, and your opponent could still lose a double game", [
        ("I must reach five hundred, or the game counts double against me.", "F97", "p. 339"),
        ("The double is in the air: I need five hundred to be safe.", "F97", "p. 339"),
    ]),
    ("double_off_you", "remark", "you pass five hundred and the double is off", [
        ("Past five hundred: the double game is off the table for you.", "F97", "p. 339"),
    ]),
    ("double_off_them", "remark", "your opponent passes five hundred and the double is off", [
        ("Five hundred reached at last: I cannot be doubled now.", "F97", "p. 339"),
    ]),
    ("win_you", "call", "you win the game", [
        ("A thousand to you: congratulations, well played.", "C96", ""),
        ("The game is yours. Shall we have another?", "C96", ""),
        ("The cards ran for you tonight, and you used them well.", "T", ""),
    ]),
    ("win_you_double", "call", "you win a double game", [
        ("A thousand before I reached five hundred: a double game to you. Well played.", "F97", "p. 339"),
    ]),
    ("win_them", "call", "your opponent wins the game", [
        ("The game falls to me this time; the cards helped. Another?", "T", ""),
        ("A thousand to me, thanks to the run of the cards. Thank you for the game.", "T", ""),
        ("It was close in places. Shall we play again?", "T", ""),
    ]),
    ("win_them_double", "call", "your opponent wins a double game", [
        ("A thousand while you were short of five hundred: a double game. The cards were unkind to you.", "F97", "p. 339"),
    ]),
    ("undo", "call", "you take back a move", [
        ("Take it back, by all means.", "T", ""),
        ("Of course: try another card.", "T", ""),
        ("Nothing is lost between friends.", "T", ""),
    ]),
]


def build():
    """The bank as words.json holds it."""
    groups = {}
    for gid, kind, moment, lines in GROUPS:
        groups[gid] = {
            "kind": kind,
            "moment": moment,
            "lines": [{"t": t, "s": s, **({"l": loc} if loc else {})} for t, s, loc in lines],
        }
    return {
        "version": 1,
        "default_mode": DEFAULT_MODE,
        "modes": MODES,
        "personas": PERSONAS,
        "sources": {k: {"cite": v[0], "file": v[1]} for k, v in SOURCES.items()},
        "groups": groups,
    }


def markdown():
    n = sum(len(g[3]) for g in GROUPS)
    t = sum(1 for g in GROUPS for line in g[3] if line[1] == "T")
    out = [
        "# What the table says",
        "",
        "The opponent's phrase bank, by game moment. Generated from `tools/phrases.py` into `web3d/words.json`;",
        "edit the bank, not this file. `python3 tools/phrases.py --check` fails if either is stale.",
        "",
        "## Design",
        "",
        "- **Voice.** First person, plain, brief and kind: a card-room companion who explains as it plays. "
        "Luck, speed, the score and the run of the cards are fair game; looks, intelligence, age, gender, class, "
        "religion and nationality never are (`web3d/test/words.test.js` checks a word list).",
        "- **What the sources give.** Foster (1897) and Cady (1896) record the declaring formulae "
        "(\"Forty for bezique, and twenty to score\", \"Royal Marriage, forty\", \"Two-fifty in trumps\") "
        "and the rules of the game, but no banter. So the declaring is said in the period formulae "
        "(the engine's own wording, `words.rs`), and the remarks are paraphrases of the rules, each "
        f"with its page, so that they teach as they speak. {t} of {n} lines are the table's own "
        "interpretation (source T): luck, pace, congratulation and consolation, kept few and plain.",
        "- **Modes** (pref `talk`, URL `?talk=`): `quiet` says only calls (the cut, the deal, the declaring "
        "formulae, the end of the game, undo); `normal` (the default) adds a remark at most moments with "
        "probability 0.6 and the play-by-play at 0.12; `chatty` says every remark and the play-by-play at 0.45.",
        "- **Kinds.** A call carries the game and is always said. A remark follows an "
        "event. Chatter is a comment on a single card played; it yields to calls and is dropped if late.",
        "- **Personas.** The same bank, weighted: a weight multiplies a remark's probability. "
        "`concentrates` remarks on small cards, trumps and the stock; `close` is reserved about brisques "
        "and marriages and readier to speak of pace and the score; `bezique` dwells on bezique, fours "
        "and the seven of trumps. Calls are never weighted.",
        "- **Repeats.** Each group deals its lines from a shuffled bag (`kit/js/bag.js`): none comes round "
        "again before the rest, and never the same twice running. One line at a time: lines wait their "
        "turn on the table's clock and appear in a bubble by the opponent's hand, for a time scaled by "
        "the game speed and the text's length.",
        "- **Slots.** `{formula}` is the engine's declaring formula; `{points}` a trick's points; "
        "`{gap}` the points between the scores.",
        "",
        "## Sources",
        "",
    ]
    for k, (cite, f) in SOURCES.items():
        out.append(f"- **{k}**: {cite}" + (f" (`{f}`)" if f else ""))
    out += ["", "## Moments", ""]
    for gid, kind, moment, lines in GROUPS:
        out += [f"### `{gid}` ({kind})", "", f"{moment}.", "", "| Line | Source |", "|---|---|"]
        for text, src, loc in lines:
            out.append(f"| {text.replace('|', '/')} | {src}{' ' + loc if loc else ''} |")
        out.append("")
    return "\n".join(out)


def main():
    words = json.dumps(build(), indent=1, ensure_ascii=False) + "\n"
    md = markdown() + "\n"
    targets = {ROOT / "web3d" / "words.json": words, ROOT / "docs" / "PHRASES.md": md}
    if "--check" in sys.argv:
        stale = [str(p) for p, s in targets.items() if not p.exists() or p.read_text() != s]
        if stale:
            sys.exit("out of date: " + ", ".join(stale))
        return
    for p, s in targets.items():
        p.write_text(s)


if __name__ == "__main__":
    main()
