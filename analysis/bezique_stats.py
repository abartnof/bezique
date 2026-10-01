#!/usr/bin/env python3
"""Exact odds for bezique variants, cross-checked by seeded Monte Carlo.

Every figure is COMPUTED here, not taken from a source. Model: a player ends the
drawing phase holding exactly half of the pack (he is dealt N cards and draws
once per stock trick), so the set of cards he receives is a uniform random
half. Figures are therefore UPPER BOUNDS for "can be declared" (cards drawn
late are unusable, cards may be played away, melds need a trick win).
"""
import random
from math import comb
from fractions import Fraction

def P(x): return float(x)

# ---- pack sizes ---------------------------------------------------------
GAMES = {
    "two-pack (standard)": dict(packs=2, hand=8, turnup=1),
    "four-pack (Rubicon)":  dict(packs=4, hand=9, turnup=0),
    "six-pack (Chinese)":   dict(packs=6, hand=12, turnup=0),
}

def structure(packs, hand, turnup):
    n = 32 * packs
    stock = n - 2 * hand - turnup            # undealt, excluding turn-up
    stock_tricks = (stock + turnup) // 2     # each trick draws 2 cards (last draw takes turn-up)
    return dict(cards=n, stock_cards=stock, stock_tricks=stock_tricks,
                endgame_tricks=hand, total_tricks=stock_tricks + hand,
                brisques=packs * 8, brisque_points=packs * 80,
                received=hand + stock_tricks)  # cards each player ends up holding

# ---- exact helpers -------------------------------------------------------
def at_least_one_of_each(n, k, types, copies):
    """P(hand of k from n contains >=1 of each of `types` distinct card types, each with `copies` copies)."""
    tot = comb(n, k); s = 0
    for j in range(types + 1):
        s += (-1) ** j * comb(types, j) * comb(n - j * copies, k)
    return Fraction(s, tot)

def at_least_a_of_two_types(n, k, c, a):
    """P(>=a of type1 and >=a of type2), each type has c copies among n; hand size k."""
    tot = comb(n, k); s = 0
    for i in range(a, c + 1):
        for j in range(a, c + 1):
            s += comb(c, i) * comb(c, j) * comb(n - 2 * c, k - i - j)
    return Fraction(s, tot)

def hyper_ge(n, m, k, t):
    """P(X>=t), X = number of the m special cards in a hand of k from n."""
    tot = comb(n, k)
    return Fraction(sum(comb(m, x) * comb(n - m, k - x) for x in range(t, min(m, k) + 1)), tot)

def carte_blanche(packs, hand):
    n = 32 * packs; non_court = 20 * packs     # A,10,9,8,7 x 4 suits = 20 per pack
    return Fraction(comb(non_court, hand), comb(n, hand))


def distinguishable_hands(mult, k, types=32):
    """Number of distinguishable k-card hands when each of `types` card types has `mult` identical copies:
    the coefficient of x^k in (1 + x + ... + x^mult)^types (the 'restricted combinations with repetitions'
    count discussed by Gorman, Kunkel and Vasko 2011 for Pinochle decks)."""
    p = [1]
    for _ in range(types):
        q = [0] * min(len(p) + mult, k + 1)
        for i, a in enumerate(p):
            for j in range(mult + 1):
                if i + j <= k: q[i + j] += a
        p = q
    return p[k]

def duplicate_stats(packs, hand):
    n = 32 * packs; m = packs   # copies of each card type = number of packs
    lab = comb(n, hand)
    p_none = Fraction(comb(32, hand) * m ** hand, lab)
    e_pairs = Fraction(comb(hand, 2) * (m - 1), n - 1)
    return distinguishable_hands(m, hand), lab, p_none, e_pairs

# ---- Monte Carlo cross-check --------------------------------------------
def mc_checks(seed=20260930, trials=200000):
    rnd = random.Random(seed)
    # standard 2-pack, trump = spades; ranks 0..7 = A,10,K,Q,J,9,8,7 ; suits S,H,D,C = 0..3
    deck = [(r, s) for r in range(8) for s in range(4)] * 2
    c = dict(bez=0, dbl=0, aces=0, seq=0, marr=0, cb9=0)
    for _ in range(trials):
        rnd.shuffle(deck)
        half = deck[:32]
        qs = sum(1 for (r, s) in half if r == 3 and s == 0)
        jd = sum(1 for (r, s) in half if r == 4 and s == 2)
        if qs >= 1 and jd >= 1: c["bez"] += 1
        if qs >= 2 and jd >= 2: c["dbl"] += 1
        if sum(1 for (r, s) in half if r == 0) >= 4: c["aces"] += 1
        if all(any(rr == r and s == 0 for (rr, s) in half) for r in range(5)): c["seq"] += 1
        if any(r == 2 and s == 1 for (r, s) in half) and any(r == 3 and s == 1 for (r, s) in half): c["marr"] += 1
    # carte blanche 9 cards of 128 (four packs)
    big = [(r, s) for r in range(8) for s in range(4)] * 4
    for _ in range(trials):
        h = rnd.sample(big, 9)
        if all(r not in (2, 3, 4) for (r, s) in h): c["cb9"] += 1
    return {k: v / trials for k, v in c.items()}

def carte_blanche_ev(packs, hand, points):
    """Exact expected carte-blanche points per hand for one player: P(no court card dealt)
    times points times (1 + expected extra awards). After each draw the player scores again while
    the drawn card is a non-court card; draws are modelled as sequential draws without replacement
    from the undealt cards, counting only the holder's draws (opponent's cards ignored)."""
    n = 32 * packs; nc = 20 * packs
    pcb = carte_blanche(packs, hand)
    left_n, left_nc = n - hand, nc - hand
    run = Fraction(0); prod = Fraction(1)
    for t in range(left_nc + 1):
        if t >= 1:
            prod *= Fraction(left_nc - (t - 1), left_n - (t - 1))
            run += prod
        if prod == 0: break
    return pcb * points * (1 + run), run

def dix_odds():
    """Two 7s of the trump suit among 64 cards; a player receives 32."""
    none = Fraction(comb(62, 32), comb(64, 32))
    both = Fraction(comb(62, 30), comb(64, 32))
    return 1 - none, both

if __name__ == "__main__":
    print("== structure ==")
    for name, g in GAMES.items():
        s = structure(**g)
        print(name, s)
    print("\n== exact odds, two-pack, player receives 32 of 64 ==")
    n, k = 64, 32
    print("bezique (>=1 Q-spades and >=1 J-diamonds):", P(at_least_a_of_two_types(n, k, 2, 1)))
    print("double bezique (both of each):", P(at_least_a_of_two_types(n, k, 2, 2)))
    print("four aces (any suits) (>=4 of 8 aces):", P(hyper_ge(n, 8, k, 4)))
    print("four kings/queens/jacks each same by symmetry")
    print("trump sequence A,10,K,Q,J of the trump suit:", P(at_least_one_of_each(n, k, 5, 2)))
    print("marriage in a named suit (K and Q of that suit):", P(at_least_a_of_two_types(n, k, 2, 1)))
    print("\n== exact odds, four-pack (Rubicon), receives 64 of 128 ==")
    n, k = 128, 64
    for a, name in [(1, "single bezique"), (2, "double"), (3, "triple"), (4, "quadruple")]:
        print(name, P(at_least_a_of_two_types(n, k, 4, a)))
    print("\n== exact odds, six-pack, receives 96 of 192 ==")
    n, k = 192, 96
    for a, name in [(1, "single bezique"), (2, "double"), (3, "triple"), (4, "quadruple"), (5, "quintuple"), (6, "sextuple")]:
        print(name, P(at_least_a_of_two_types(n, k, 6, a)))
    print("\n== carte blanche at the deal ==")
    print("four-pack, 9 cards:", P(carte_blanche(4, 9)))
    print("six-pack, 12 cards:", P(carte_blanche(6, 12)))
    print("two-pack, 8 cards (no such score in the standard game):", P(carte_blanche(2, 8)))
    print("\n== carte blanche expected points per hand (exact) ==")
    ev, run = carte_blanche_ev(4, 9, 50);  print("four-pack: EV", float(ev), "expected extra re-scores given CB", float(run))
    ev, run = carte_blanche_ev(6, 12, 250); print("six-pack: EV", float(ev), "expected extra re-scores given CB", float(run))
    at_least_one, both = dix_odds()
    print("\n== 7 of trumps (two copies), two-pack ==")
    print("receives at least one:", float(at_least_one), "receives both:", float(both), "expected number:", 1.0)
    print("\n== distinguishable dealt hands (duplicate cards) ==")
    for name,(packs,hand) in {"two-pack, 8 cards":(2,8),"four-pack, 9 cards":(4,9),"six-pack, 12 cards":(6,12)}.items():
        d,lab,pn,ep=duplicate_stats(packs,hand)
        print(name,"distinguishable",d,"labelled",lab,"P(no duplicate)",float(pn),"E[identical pairs]",float(ep))
    print("\n== Monte Carlo (seed 20260930, 200000 trials) vs exact ==")
    mc = mc_checks()
    for k_, v in mc.items(): print(k_, v)
