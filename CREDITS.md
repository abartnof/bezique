# Credits

**Bezique** is © 2026 Andrew Bartnof and released under the **MIT License**
(`LICENSE`). Everything listed below is someone else's and keeps its own
licence, notably the card back, which is CC BY-SA 3.0.

Credit is given where it is due, including where no licence demands it. Every
third-party asset is recorded here, in the commit that brings it in: art,
libraries and build tools.

## Card art

### The faces

**"Public domain complete playing card deck"**, by **AustinGabriel64**, from
Wikimedia Commons (uploaded 27 July 2026).
<https://commons.wikimedia.org/wiki/File:Public_domain_complete_playing_card_deck.svg>

Licence: **CC0 1.0 Universal**, a public-domain dedication
(<https://creativecommons.org/publicdomain/zero/1.0/>). No attribution is
required; it is given anyway. The game uses the 32 bezique cards (seven to ace
in each suit, twice over in the pack), rasterised from the vector original.

### The back

**"Reverso baraja española"**, by **Germarquezm**, from Wikimedia Commons (2013).
<https://commons.wikimedia.org/wiki/File:Reverso_baraja_espa%C3%B1ola.svg>

It includes elements from **"Baraja española.svg"**, also by Germarquezm
(<https://commons.wikimedia.org/wiki/File:Baraja_espa%C3%B1ola.svg>).

Licence: **CC BY-SA 3.0** (<https://creativecommons.org/licenses/by-sa/3.0/>).
Attribution is **required**, and ShareAlike applies to adaptations.

Changes: rasterised, and re-framed from its Spanish proportions (about 1:1.53)
to the 5:7 of the face cards. The back the game shows is therefore an
adaptation, so **that image is itself licensed CC BY-SA 3.0**.

## Libraries in the page

| Package | Version | Licence | Holder |
|---|---|---|---|
| three.js | 0.186.1 | MIT | three.js authors |

The licence notice travels with the copy: the build keeps three.js's licence
comment in the page's script.

## Build tools (used, not shipped)

| Tool | Version | Licence | Used for |
|---|---|---|---|
| esbuild | 0.28.2 | MIT | Bundling the page's JavaScript |

## The kit

`kit/` holds the card table's game-independent code (scene, cards, motion,
director, score HUD, page builder). It was vendored from the `kit/` of
**Strohmandeln**, which in turn took it from **Cassino**'s `web3d`. Both are
MIT-licensed and by the same maintainer as this project. `kit/STAMP` records
the Cassino commit and, file by file, where each came from and how it differs.

The rules engine and everything else in this repository are the project's own.
