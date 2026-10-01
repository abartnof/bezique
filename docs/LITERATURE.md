# Bezique literature review (working file)

*Purpose: foundation for a bezique video game. Baseline rules: Pagat (S1, S2). Everything is cited by source ID, e.g. [S3]; the registry in §2 has URLs. Nothing here is quoted at length; it is all paraphrase, so go to the source for exact wording.*

**Conventions**
- **Tiers:** A = authoritative or period primary; B = reputable secondary; C = usable with care; D = known errors, avoid as a rules source.
- **[inference]** marks the reviewer's own reasoning, not a source's claim.
- **Status** in the registry: FULL = read in full (or the bezique chapter/entry in full); EXCERPT = search-result or partial read; blocked items are listed at the end of §2.

---

## 0. Status

- **What this is:** a working literature review for a bezique video game. Every claim is cited by source ID (registry in §2); the reviewer's own reasoning is marked [inference]; figures in §10b are computed, not sourced. §14 is the changelog; the search history lives there and in §13, not in the body.
- **Coverage:** 68 registered sources (counts by tier and status in §10b). The strongest are the period primaries (S45–S48, S55–S56 and Cady S4, Hoffmann S3), the period bibliographies (S51, S64), Pagat (S1, S2) and the Larousse entries (S26, S54). Many web pages are thin or derivative and are flagged.
- **Main result:** the "standard" two-player game is stable only from the 1890s; before that, sources differ systematically on who deals, the meaning of the last trick, when brisques are counted, how many melds a trick may score, the bezique pair, card reuse and hand size (§6). Conflicts are reported by source, date and edition, never settled by counting sources.
- **Still open:** see §13 (unfound primaries such as Méry 1847, Van Tenac 1858, Cavendish's own booklets and Camden's text; originals unreachable from the sandbox).
- **Access limits:** see the end of §2.

---

## 1. Headline findings

1. **The core two-player game is stable across sources**: 64 cards (two 32-card piquet packs), rank A-10-K-Q-J-9-8-7, 8-card hands (usually dealt 3-2-3), turn-up trump, free play (no obligation to follow suit) while the stock lasts, melds only by the trick winner, a strict endgame, and a 1000-point target in most texts (Pole 1861 counts to 100 units; some texts use 1500 or 2000). [S2, S3, S4, S9, S48]
2. **At least 20 details are contested** and the engine has to choose. See §6.
3. **Several popular pages are wrong**: CardGameHeaven scores four-of-a-kind at 20 [S21]; h2g2 invents a 100-point plain sequence in the standard game [S20]; card-games.ca discloses AI-assisted content and contradicts itself [S22]. Prefer S2, S3, S4, S5.
4. **The "Gustave Flaker, Swedish schoolmaster" origin is fiction.** It is the bezique preamble of the 1864 *American Hoyle* (4th ed.) [S45]; the 1868 *Modern Pocket Hoyle* drops it [S46]; the French manual behind the American text does not contain it [S49]; Foster (1897) does not repeat it [S47]. A blogger reports no trace in Swedish archives and Spanish Wikipedia says specialists reject it; it is still repeated by S20, S23 and others. [S9, S16, S32]
5. **Living traditions exist**: Haiti (four packs + jokers), Turkey ("Bezik", 96 cards to 3000), French Polynesia. [S10, S19, S32]
6. **No bezique-specific academic paper (AI, solver, probability or history) was found.** Adjacent scholarship exists: a peer-reviewed combinatorics article on Pinochle [S65], AI work on the related games Schnapsen, Skat and Jass [S66, S67], and the games-history literature (Parlett, Depaulis) that the registry cites only at second hand (§10, §13). Apps: three indie/commercial (§10).
7. **Period texts before the 1880s differ from the 1890s standard** on who deals, the last trick, brisque timing, several melds per trick, the bezique pair, card reuse and endgame penalties (§6 rows 1, 4, 7–9, 12–13, 16, 20). Foster (1897) says the English rules were disputed in 1868–69 and by his day were practically uniform. [S45, S46, S47, S4, S55, S56]
8. **The trump-dependent bezique pair is not only a six-pack feature:** Cavendish (1870) and Pardon apply Q♣+J♥ (when spades or diamonds are trumps) to the ordinary game. [S55, S56]
9. **Six-pack play is older than the codified "Chinese" game**: mentioned c. 1870s [S56, S57]; defined by Foster 1897 [S47]; laws first in Jessel 1921 [S7].
10. **Odds and a small simulation are in §10b.** Two results matter for the engine: declaring before or after drawing changes a bot's meld points by about 20%, while allowing several melds per trick changes them by under 1%. (Computed.)

---

## 2. Source registry

One table for all sources, sorted by ID (tier and status codes as in Conventions above). "Relevant chapter" means the bezique chapter or entry was read in full and the rest of the work was not. Other status values: CATALOGUE (metadata only) and POINTER (a mirror of another entry).

| ID | Source | URL | Tier | Status | Notes |
|---|---|---|---|---|---|
| S1 | Pagat, Bezique hub | https://www.pagat.com/marriage/bezique.html | A | FULL |  |
| S2 | Pagat, "Bezique and Variations" (Howard Fosdick archive, with John McLeod's corrections) | https://www.pagat.com/marriage/Bezique_and_Variations.html | A- | FULL |  |
| S3 | Louis Hoffmann, *Hoyle's Games Modernized*, Bézique chapter incl. 66 Portland Club Rubicon laws (Wikisource text of the 1909 edition, revised by Ernest Bergholt; it carries the 1887 Portland Club laws, so treat the *hints* as Hoffmann-era, c. 1890s–1909, and the laws as 1887) | https://en.wikisource.org/wiki/Hoyle's_Games_Modernized/B%C3%A9zique | A (period primary) | FULL |  |
| S4 | A. Howard Cady, *Bézique: A Treatise on the Game in Its Several Varieties* (American Sports Publishing Co., copyright 1896; draws on Cavendish and Berkeley) | https://archive.org/stream/beziquetreatiseo00adya/beziquetreatiseo00adya_djvu.txt | A- | FULL |  |
| S5 | Philippe Lalanne, "Le Bésigue," Académie des jeux oubliés (1847 single-pack game, citing Méry) | https://salondesjeux.fr/besigue.htm | A- | FULL |  |
| S6 | Tony Hall, "Bezique Markers, 1860–1960," World of Playing Cards | https://www.wopc.co.uk/games/bezique-markers-1860-1960 | A- | FULL |  |
| S7 | Tony Hall, "De La Rue Pocket Guides," World of Playing Cards | https://www.wopc.co.uk/delarue/de-la-rue-pocket-guides | A- | FULL | Re-read Sept 2026 (Hall's page): it gives the first edition of *Pocket Guide to Bezique* as 1869 (18 pp; 2nd ed. 1869, 4th 1870, 16th 1916 and 1925), *Rubicon Bezique* 1887 (3rd ed. 1898), *Polish Bezique* 1873, and, in his numbered list of other Cavendish guides, "The Pocket Guide to Russian Bezique, 1887" (item 10). Hall notes that 1881 adverts listed sixteen Cavendish guides including Bezique and Polish Bezique, and that Jessel's six-pack laws were first issued by Mudie in 1921. |
| S8 | Pagat, Two-player Pinochle (John Hay) | https://www.pagat.com/marriage/pin2hand.html | A- | FULL |  |
| S9 | Wikipedia, Bezique | https://en.wikipedia.org/wiki/Bezique | B | FULL | Reference list read (Sept 2026): it cites Méry 1847 (p. 65), Anton 1879 (p. 43), Parlett 1996 (p. 16) and 2008 (pp. 287, 293), *Transactions of the Philological Society* 1910 (p. 289), Brownsmith 1869 (*Westminster Papers*, pp. 117–119), *Cassell's Household Guide* 1881 (p. 40), a Camden booklet (12th ed., 1872) and S65 (see §10, §13). |
| S10 | Wikipédia FR, Bésigue | https://fr.wikipedia.org/wiki/B%C3%A9sigue | B | FULL |  |
| S11 | Serge Jodra, cosmovisions.com (reproduces an older French dictionary entry, signed "(B.)") | https://cosmovisions.com/jeux/besigue.htm | B | FULL |  |
| S12 | Denexa Games, Bezique (six-pack rules) | https://www.denexa.com/blog/bezique/ | B | FULL |  |
| S13 | CatsAtCards, "How To Play Bezique" (many variants; some errors) | https://catsatcards.com/Games/Bezique.htm | C+ | FULL |  |
| S14 | AdPoker.fr, Règles du Bésigue | https://www.adpoker.fr/besigue.html | C+ | FULL |  |
| S15 | WhiteKnuckleCards, Fildinsky (derived from Fosdick) | https://whiteknucklecards.com/games/fildinski.html | C+ | FULL |  |
| S16 | "Past Times" blog (toucana), Bezique, 2017 | http://toucana.blogspot.com/2017/08/bezique-la-partie-de-besigue-gustave.html | C+ | FULL |  |
| S17 | "Past Times" blog, "The Curious History of Fildinski," 2021 (speculative) | http://toucana.blogspot.com/2021/08/the-curious-history-of-fildinski-card.html | C | FULL |  |
| S18 | dudun.ru, Безик (Russian; unusual penalty scale) | https://dudun.ru/g_bezik/ | C | FULL |  |
| S19 | Turkish Bezik rules (reposted "alıntıdır" quotation; original unclear) | https://abdullahabdurrahman.wordpress.com/2019/10/25/bezik-alintidir/ | C+ | FULL |  |
| S20 | h2g2 (Sharon Clarke) | https://h2g2.com/edited_entry/A646724 | C | FULL |  |
| S21 | CardGameHeaven | https://cardgameheaven.com/bezique/ | C/D | FULL |  |
| S22 | card-games.ca (AI-assisted per its own disclosure) | https://card-games.ca/traditional-card-games/bezique/ | D | FULL |  |
| S23 | PokerStarsNews.it (Italian; repeats Flaker legend) | https://www.pokerstarsnews.it/poker-sportivo/piglio-o-non-piglio-giochi-di-carte-alternativi-machiavelli-bazzica-e-misere/53392/ | C- | FULL |  |
| S24 | Britannica, "bezique" (author David Parlett; text truncated on fetch, used excerpts) | https://www.britannica.com/topic/bezique | A- (partial) | FULL | Listed for completeness; no claim in the body rests on it. |
| S25 | Spectator, review of Cavendish's *Rubicon Bezique*, 10 Dec 1887 | https://archive.spectator.co.uk/article/10th-december-1887/6/rubicon-bezique-by-cavendish-de-la-rue-and-cc--car | A- | EXCERPT |  |
| S26 | Larousse, *Grand dictionnaire universel du XIXe siècle*, "Bésigue" (fr.wikisource pages 214 and 222) | https://fr.wikisource.org/wiki/Page:Larousse_-_Grand_dictionnaire_universel_du_XIXe_si%C3%A8cle_-_Tome_2,_part._2,_Be-Bi.djvu/214 | A | FULL for the Bésigue entry (BnF scan); earlier Wikisource pages were excerpts | Whole entry read from the BnF scan on archive.org (https://archive.org/stream/LarousGrdictionnXIX02bnf/LarousGrdictionnXIX02bnf_djvu.txt; the item is catalogued 1866, the volume is dated 1867 in the Wikisource pages); OCR text, so numerals were cross-read. |
| S27 | New International Encyclopædia, 1st ed., "Bezique" (Wikisource) | https://en.wikisource.org/wiki/Page:The_New_International_Encyclop%C3%A6dia_1st_ed._v._03.djvu/27 | B+ | EXCERPT |  |
| S28 | V&A objects: De La Rue "Bezique and Polish Bezique" (O26683); Goodall "Royal Game of Bezique" (O26651) | https://collections.vam.ac.uk/item/O26683/bezique-and-polish-bezique-card-game/ and https://collections.vam.ac.uk/item/O26651 | A | EXCERPT |  |
| S29 | Wiktionary, "rubicon" (a Cavendish *piquet* quote; not bezique) | https://en.wiktionary.org/wiki/rubicon | n/a | EXCERPT | Listed for completeness; no claim in the body rests on it. |
| S30 | Russian Wikipedia; Belovinsky dictionary (ponjatija.ru); lightst.ru | https://ru.wikipedia.org/wiki/%D0%91%D0%B5%D0%B7%D0%B8%D0%BA_(%D0%BA%D0%B0%D1%80%D1%82%D0%BE%D1%87%D0%BD%D0%B0%D1%8F_%D0%B8%D0%B3%D1%80%D0%B0) ; http://ponjatija.ru/node/16986 ; http://www.lightst.ru/card/Games/bezik.htm | B/C | EXCERPT | Listed for completeness; no claim in the body rests on it. |
| S31 | German Wikipedia; Binokel Palast; BSFF Binokel | https://de.wikipedia.org/wiki/B%C3%A9zique ; https://www.binokel-palast.de/binokel-geschichte/ ; http://binokelsportfreundeflacht.de/binokel-regeln/ | B | EXCERPT |  |
| S32 | Spanish Wikipedia, Bésigue | https://es.wikipedia.org/wiki/B%C3%A9sigue | B | EXCERPT |  |
| S33 | French Wiktionnaire "bésigue"; Linternaute; dicocitations (first attestations, idioms) | https://fr.wiktionary.org/wiki/b%C3%A9sigue | B | EXCERPT |  |
| S34 | Riverton (NJ) Historical Society, "The Ladies of Riverton's Bezique Club" | https://rivertonhistory.com/2018/02/the-ladies-of-rivertons-bezique-club/ | C+ | EXCERPT |  |
| S35 | Red Hot Pawn forum, double-bezique probability | https://www.redhotpawn.com/forum/posers-and-puzzles/bezique.188155 | C | EXCERPT |  |
| S36 | Apps: Glowing Eye Games "Bezique!" (https://apps.apple.com/app/id1555727230); William Hale "Bezique" (https://apps.apple.com/cd/app/bezique/id1186935667); DiD (https://jimmy-smith.itch.io/bezique) | see left | n/a | EXCERPT |  |
| S37 | Churchill Archive catalogue item on a "Bezique debt" | https://www.churchillarchive.com/catalogue-item?docid=CHAR1_271_108 | A | EXCERPT |  |
| S38 | *American Hoyle* (1864) catalogue pages, LoC and Internet Archive (metadata only) | https://archive.org/details/americanhoyleorg00dick ; https://www.loc.gov/item/05024784/ | catalogue | CATALOGUE only | Full text since read: see S45. |
| S39 | HowStuffWorks, Rubicon/Chinese pages | https://entertainment.howstuffworks.com/how-to-play-bezique1.htm | C+ | EXCERPT |  |
| S40 | Hall, "Evolution of Bezique boxed sets 1869–1990" (https://www.wopc.co.uk/members/tony-hall/evolution-of-bezique-boxed-sets-1869-to-1990); Goodall booklets blog (https://rulebooklets.wordpress.com/2012/11/07/frederic-jessel-makes-another-appearance-the-laws-of-six-pack-bezique-or-chinese-bezique-1921-published-by-goodall-i-think-also-a-brief-criticism-of-the-bodleian-library-catalog/) | see left | B+ | EXCERPT |  |
| S41 | Wikipedia: Marjolet, Briscan, Binokel, Pinochle, Marriage group, Brisca | https://en.wikipedia.org/wiki/Marjolet ; https://en.wikipedia.org/wiki/Briscan | B | EXCERPT |  |
| S42 | Wikipedia, "The Bezique Game" (Caillebotte, 1880) | https://en.wikipedia.org/wiki/The_Bezique_Game | B | EXCERPT |  |
| S43 | Gutenberg full text of Hoffmann (pointer to S3) | https://www.gutenberg.org/files/39445/39445-h/39445-h.htm | A | POINTER only |  |
| S44 | Italian Bazzica page (Cavallo Re) | https://www.cavallore.it/vecchio_sito/giochi/bazzica.html | C | EXCERPT | Listed for completeness; no claim in the body rests on it. |
| S45 | Dick & Fitzgerald, *The American Hoyle* (editor "Trumps"), **4th edition, copyright 1864**, New York; Bezique chapter pp. 137–145 (index also lists Panaché and no-trump variants at p. 144) | https://archive.org/stream/americanhoyleorg00dick/americanhoyleorg00dick_djvu.txt | A (period primary) | FULL | for the title page, preface mention and pp. 137–145; rest of the book not read. OCR text: ditto marks print as "44", meld values reconstructed (see §14 OCR flags). The copy is the 4th edition; whether earlier editions carried bezique is unknown. |
| S46 | Dick & Fitzgerald, *The Modern Pocket Hoyle*, catalogue year 1868 (archive.org metadata; title page not inspected), Bezique pp. 167–172 | https://archive.org/stream/modernpockethoy00dickgoog/modernpockethoy00dickgoog_djvu.txt | A- (period primary; OCR poor) | FULL | for the bezique section; rest not read. Laws are numbered but OCR garbles the numerals, so cite by content. |
| S47 | *Foster's Complete Hoyle* (copyright 1897, F. A. Stokes), Bezique family pp. ~337–357 (ordinary, four- and three-handed, Polish, Cinq-Cents, Penchant, Rubicon, Chinese, Chouette) and the closing "Suggestions for Good Play" of the Binocle chapter pp. 366–368 | https://archive.org/stream/fosterscomplete00fostgoog/fosterscomplete00fostgoog_djvu.txt (Google scan of the 1897 edition). Gutenberg #53881 (https://www.gutenberg.org/files/53881/53881-h/53881-h.htm) was **not** read: the host is blocked from the sandbox, so it is unverified that it matches this scan. | A- (period compilation; systematic) | FULL | for the Bezique chapters; Binocle rules and the rest of the book not read. |
| S48 | W. Pole (signature OCR-garbled; attribution from S53), "Games at Cards for the Coming Winter" (running head as OCR gives it), *Macmillan's Magazine* vol. V, Dec 1861, Bazique section pp. ~138–139 | https://archive.org/stream/macmillansmagazi0005davi/macmillansmagazi0005davi_djvu.txt (a second scan, macmillansmagazi05macmuoft, also has it) | A (first English description) | FULL | for the Bazique text; the rest of the article (Quadrille, Piquet) not read Brownsmith (1869, cited in S9's references) also reads the signature as W.P. and assumes Professor Pole. |
| S49 | "Bonneveine" (pseudonym of Jules Rostaing per archive.org metadata), *Académie des jeux*, Delarue, 1865, chapter "Jeu de Bésigue" (OCR pages ~105–113) | https://archive.org/stream/bub_gb_EyRTAAAAcAAJ/bub_gb_EyRTAAAAcAAJ_djvu.txt | A- (period French manual; OCR poor) | FULL | for the chapter up to the start of Piquet |
| S50 | F. Dumesnil, *Académie des jeux*, Bernardin-Béchet, 1865, "Le Besigue ou Besi" (Google scan of the Harvard copy) | https://archive.org/stream/acadmiedesjeuxc00dumegoog/acadmiedesjeuxc00dumegoog_djvu.txt | A- (period French manual) | FULL | for the rules and the first four maxims; later hints not read |
| S51 | Frederic Jessel, *A Bibliography of Works in English on Playing Cards and Gaming* (1905): every entry mentioning Bezique/Besique/Basique, the Jones ("Cavendish"), Peel ("Berkeley"), Lewis and Camden entries, and the index | https://archive.org/stream/abibliographywo00jessgoog/abibliographywo00jessgoog_djvu.txt | A (period bibliography) | FULL | for those entries (keyword read, not a page-by-page read of the whole book) |
| S52 | *Notes and Queries*, 3rd ser. ix: query of 17 Feb 1866 (index: "Bexig, or Besique," p. 138) and the replies of 24 Mar 1866 (index p. 246); one reply signed "N. F.", one pointing to Macmillan's vol. v | https://archive.org/stream/sim_notes-and-queries_1866-03-24_9_221/sim_notes-and-queries_1866-03-24_9_221_djvu.txt (and ..._1866-02-17_9_216 for the query) | B+ (anonymous reader notes) | FULL | for the notes |
| S53 | F. Jessel, reply "Bezique" (9th ser. xi, p. 115), *Notes and Queries*, 7 Feb 1903 | https://archive.org/stream/sim_notes-and-queries_1903-02-07_11_267/sim_notes-and-queries_1903-02-07_11_267_djvu.txt | A- (expert letter) | FULL | for the note; the earlier note it answers (9 S. xi, p. 26) NOT read |
| S54 | *Nouveau Larousse illustré*, vol. 2, "Bésigue" entry (edition year not checked; the series runs c. 1897–1904) | https://archive.org/stream/nouveaularoussei02laro/nouveaularoussei02laro_djvu.txt | B+ (encyclopedia; OCR heavily damaged in the value list) | FULL | for the entry |
| S55 | "Cavendish" (Henry Jones), "Bézique," *The Gentleman's Magazine*, N.S. vol. IV, Jan 1870, pp. ~169–178 (signed; the same article is quoted by name in N&Q 1874, S59) | https://archive.org/stream/sim_gentlemans-magazine_1870-01_228/sim_gentlemans-magazine_1870-01_228_djvu.txt | A (the leading English authority, early text) | FULL | for the article |
| S56 | George Frederick Pardon, *The Card Player's Manual* (Ward, Lock), bezique chapter pp. ~149–154; the scan has no title page and is catalogued 1876; Jessel lists a Pardon edition "with the addition of Bezique" in 1870 (S51) | https://archive.org/stream/cardplayersmanua00pard/cardplayersmanua00pard_djvu.txt | A- (English handbook) | FULL | for the chapter. [inference] The text postdates Sept 1870: another chapter alludes to Bret Harte's "Heathen Chinee" poem |
| S57 | Barnet Phillips (ed.), *The Standard Hoyle*: "Guide to Bezique, by Cavendish," Polish and Rubicon (Portland Club laws of 1887); read in the copies catalogued 1904 and 1909 (edition of first publication not verified); also Fox's revised *Hoyle* (c. 1912), which carries the same guide | https://archive.org/stream/standardhoylecom00phil/standardhoylecom00phil_djvu.txt ; https://archive.org/stream/foxsrevisedediti00newy/foxsrevisedediti00newy_djvu.txt | A- (Cavendish's text as reprinted) | EXCERPT | selected passages and keyword checks, not a full read |
| S58 | *The American Hoyle* (Dick & Fitzgerald), the edition marked copyright 1864, 1880, 1886, 1892: bezique chapter pp. ~186–198, now Cavendish's description "made to harmonise with Rubicon Bézique" plus the Portland Rubicon laws | https://archive.org/stream/americanhoyle0000unse/americanhoyle0000unse_djvu.txt | A- | EXCERPT | ordinary-game description, endgame, scoring, rubicon settlement and Laws 1–17 read; later laws only skimmed |
| S59 | Period notices, 1869–74: *The Athenaeum*, 11 Sep 1869 (review of Ware's *Handbook of Besique*, Warne); *The Examiner*, 4 Dec 1869 (advert: Baldwin's *Rules of Besique* to appear 11 Dec); *Notes and Queries* 5th ser. i (21 Mar and 2 May 1874: etymology exchange, which also quotes Cavendish 1870 and *Once a Week* 13 Feb 1869); also *The Englishwoman's Domestic Magazine* essay on Besique (1869) and 10 Jun 1865 adverts for "Trumps's" *American Hoyle* in *Harper's Weekly* and *Frank Leslie's* | https://archive.org/stream/sim_athenaeum-uk_1869-09-11_2185/sim_athenaeum-uk_1869-09-11_2185_djvu.txt ; sim_examiner-a-weekly-paper-on-politics-literature-music_1869-12-04_3227 ; sim_notes-and-queries_1874-03-21_1_12 ; sim_notes-and-queries_1874-05-02_1_18 (all archive.org) ; sim_englishwomans-domestic-magazine_1868-1872_8 ; sim_harpers-weekly_1865-06-10_9_441 ; sim_leslies-weekly_1865-06-10_20_506 | B+ | EXCERPT | (the relevant paragraphs) |
| S60 | "Le Bésigue, jeu de cartes," zpag.net (Québec French; undated modern compilation, not attributed): ordinary game, no-turn-up, three- and four-hand, Chinese (six-pack), eight-pack, Rubicon, chouette | http://www.zpag.net/Jeux/Cartes/le_Besigue.html | C+ (unsourced but detailed; matches S12/S13/S14 in its six-pack tables) | FULL |  |
| S61 | Thomas A. Sawyer, blog on Goodall card-game booklets (2010–13 posts on "The Royal Game of Bezique" / Camden), citing Jessel and Mike Goodall's *Goodall Card Game Booklets 1868–1922* (2016); Hall's Feb 2024 note that bezique "evolved into Polish and Chinese" bezique with up to six packs | https://rulebooklets.wordpress.com/2010/12/28/comments-on-my-earliest-goodall-card-game-rule-booklet-and-on-a-cool-and-instructive-bezique-set/ ; https://www.wopc.co.uk/games/the-search-for-new-games-in-the-late-19th-century | B (collector's blog; images not text) | EXCERPT |  |
| S62 | William Christensen (Frederiksberggade 26), *Bezique* rules booklet in Danish, undated, packed with a pair of score markers; archive.org item bezique-danish (a scan; 2024 upload) | https://archive.org/stream/bezique-danish/Bezique_Danish_Old_djvu.txt | C+ (undated, shop booklet; OCR fair) | FULL |  |
| S63 | Modern digests, not period sources: Polish Wikipedia "Bezik" (https://pl.wikipedia.org/wiki/Bezik), the PAN dictionary entry (wsjp.pl), grykarciane.com and maluchy.pl Polish rule pages; Japanese "gamefarm.jp" Bezique/Pinochle page (https://gamefarm.jp/rule/bezique.html); Dutch Wikipedia "Bésigue" and Wikibooks "Kaartspel/Bezique" (nl.wikipedia.org, nl.wikibooks.org) and two Dutch rule blogs; Portuguese Wikipedia "Bezique (jogo)" and Ludopedia; a Czech search returned nothing on bezique | see left | C- (derivative) | EXCERPT | search snippets |
| S64 | Norton T. Horr, *A Bibliography of Card-Games and of the History of Playing-Cards* (Cleveland, 1892): the Jones ("Cavendish") entries 759–800, and entries mentioning Bezique | https://archive.org/stream/bibliographyofca00horruoft/bibliographyofca00horruoft_djvu.txt | A- (period bibliography, US-oriented) | EXCERPT | the Cavendish/Pocket Series entries and the bezique hits (4) only |
| S65 | P. S. Gorman, J. D. Kunkel and F. J. Vasko, "Using Pinochle to motivate the restricted combinations with repetitions problem," *International Journal of Mathematical Education in Science and Technology* 42(5): 679–684 (2011) | https://eric.ed.gov/?id=EJ929289 ; https://ui.adsabs.harvard.edu/abs/2011IJMES..42..679G/abstract | A- (peer-reviewed; about Pinochle, not bezique) | EXCERPT | Abstract read only (ERIC, NASA ADS). Counts distinguishable hands from decks with two copies (single deck) or four (double deck) of each card type and gives a general counting formula via generating functions. S9 cites it with the date October 2009 (presumably online-first [inference]); the issue is 2011. |
| S66 | "From Imitation to Interaction: Mastering Game of Schnapsen with Shallow Reinforcement Learning," arXiv:2605.17162 | https://arxiv.org/pdf/2605.17162 | B (preprint; a related game) | EXCERPT | Introduction read via a search snippet. Schnapsen (Sixty-Six) is bezique's nearest well-studied relative; the paper builds on the Schnapsen Game Engine and its RdeepBot (Monte Carlo sampling with fixed lookahead). Not about bezique. |
| S67 | Trick-taking AI literature seen only as citations, in the reference lists of arXiv:2205.13834 (a Wizard DQN paper) and arXiv:2411.09089 (set-based retrograde analysis): Sturtevant, "Improving state evaluation, inference, and search in trick-based card games" (IJCAI 2009); Buro et al., "Improving search with supervised learning in trick-based card games" (AAAI); Whitehouse et al., "Information Set Monte Carlo Tree Search" (IEEE Trans.); Koller et al. on Jass; Furtak (2013) on Skat null endgames | https://arxiv.org/pdf/2205.13834 ; https://arxiv.org/html/2411.09089v1 | B | EXCERPT | Citations only; none of these papers was read. Methodology relevant to a bezique AI: imperfect information, sampling-based search, endgame solving once the stock is gone. |
| S68 | Alexander Palace Time Machine, "Bezique, an Imperial Pastime" | https://www.alexanderpalace.org/palace/Bezique.php | C+ | EXCERPT | Says Nicholas and Alexandra played bezique often and cites the annotated *Last Diary of Tsaritsa Alexandra* (Yale University Press, 1997); supports the S16-only claim in §3.4. The annotated edition itself was not read. |

Also examined at keyword level only (EXCERPT, not fully read): the *American Hoyle* editions of 1868, 1874, 1875 and 1892 (archive.org ids americanhoyleorg00dick_2 / _0 / _1 and americanhoyle0000unse) and Dick's *American Card Player* (1866, americancardplay00dick): the Flaker passage and the Panaché, no-trump and fixed-point variants recur in the first four; the 1892 book (copyright 1864, 1880, 1886, 1892) has no Flaker match and a separate Rubicon Bezique chapter (not read). My keyword comparison of these editions was crude; no rule difference between editions is asserted here.

**Do not use as rules sources:** Grokipedia (AI-generated), CardRules+ (unverified literary claims), gamerules.com, Bridge in the Box and QuizzClub (copy Wikipedia and repeat the Flaker legend).

### Blocked / not reached

- Chris Herring's six-pack rules, Jean-François Bustarret's *Bésigue* page (jeuxdecartes.net, robots disallowed) and the Planet Haiti *Bésigue haïtien* page (all linked from Pagat, S1): originals not reached; substitutes read (S10, S14, S60).
- Boussac, *Le Bésigue et ses variétés* (1896), on casinoenligne-fr.fr (robots disallowed).
- Méry, *L'Arbitre des jeux* (1847); Van Tenac (1858); Cavendish's own 1868–70 booklets; the text of Camden (1883); Anton (1879); Parlett's books; Mike Goodall's 2016 book: not reached (§13).
- Network limits from the code sandbox: web.archive.org, www.gutenberg.org and archive.org `/download/` links are blocked; archive.org `/stream/…_djvu.txt`, `advancedsearch.php`, `/metadata/` and the full-text search API work. French Wikisource is cache-only; several sites disallow automated access.

---

## 3. History and lineage

### 3.1 Lineage
- Ancestors named in the period literature: Mariage → Brusquembille → Briscan/Brisque → Cinq-cents → Bézique. [S4, S5, S26]. Jessel (1903) disputes the Cinq-cents step: it was a cousin, not an ancestor [S53].
- Cady's summary of ancestors [S4]:
  - **Mariage:** 32 cards, six each, 13th card trump, plus an "amour" (ace+ten) score; sixty-six style card values (A 11, 10 10, K 4, Q 3, J 2).
  - **Brusquembille:** three cards each, seventh card turned up; 7 of trumps exchangeable; aces and tens are the counting cards.
  - **Briscan/Brisque:** six cards, marriages, fours (aces 150, tens 100, kings 80, queens 60, knaves 40), 600 up; dealer's turn-up of an ace, ten or court card scored ten; five court cards scored 20 and could re-score; carte blanche scored 10; the last trick did not score.
  - **Cinq-cents:** 500 up; quint major 250 trump / 125 other; fours 100/80/60/40/30; marriage 40/20; the Q♠–J♦ pairing scored 40 (Cady says it was called "brisque" there).
- Cavendish's account of how bezique arose (via Cady): shuffle two packs together, keep only the quint-major sequence, replace the "amour" with the Q♠–J♦ pairing, score aces and tens at ten each instead of 11 and 10, score in tens, play 1000 up. [S4]
- Larousse (1867) [S26]: born in France at an unknown date; long known in Poitou, Saintonge and Angoumois; introduced about forty years earlier into Paris cafés and then salons. The same entry proposes that the name comes from Latin *bijugum* (a pair or yoke), alluding to the double pack.
- Other etymology theories: Spanish *besico* ("little kiss"), Italian *bazzica*, Murray's "unknown." [S4] Lalanne: Occitan *besi*, "to see," since Q♠ and J♦ are the only queen and jack in profile facing each other; speculative. [S5]
- First rules in Paris in 1847, Méry calling it new. [S5, S9] First French attestations 1859 (Ponson du Terrail), 1861 ("bezi," Goncourt), 1864 ("bésigue"). [S33] These are dictionary dates: literary uses are recorded earlier (Paul de Kock 1846, Offenbach/Moinaux 1855, §3.4) and Larousse places the game in Paris cafés from about 1827 (below), so 1859 is a lexicographers' first attestation, not the age of the game.
- Wikipedia notes it is a member of the Marriage group and relates to Binokel and Pinochle; two-handed versions are almost identical. [S9, S31, S41]
- **1864 US account** [S45]: bezique was introduced in Paris only a few years earlier and is now popular in every kind of French company; it is a variation of Cinq-Cents, played for a long time in the south of France, with borrowings from Mariage; it is becoming fashionable in the US; German players call it Penuchle or *Peanukle*. The preamble carries the Flaker/Flakernuhle tale (§1 item 4). Note the south-of-France placement differs from Larousse's Poitou/Saintonge/Angoumois (western France) [S26]. The 1868 *Modern Pocket Hoyle* drops the Flaker tale and says instead that the game is a favourite of the Germans under the name Penuchle, differing chiefly in using one pack, Sixty-Six extended (the OCR breaks off mid-sentence) [S46].
- **Foster 1897** [S47]: bezique, Binocle and Sixty-Six form a family; named ancestors are Marriage, Matrimony and Cinq-Cents; he repeats the *basico* (little kiss) and *besaigne* (double-headed axe) etymologies. He speculates, as opinion, that bezique began as an attempt to play Binocle with a piquet pack (the card ranking being German) and that Binocle was first a 52-card game; one German writer, he says, calls the game Swiss, from Spain. [inference] Treat all of this as Foster's conjecture, not evidence.
- **Where and when: sources disagree.** Region: Poitou, Saintonge, Angoumois [S26]; the south of France [S45, S49]; Limousin, specifically Haute-Vienne, with the patois name *Besit/Bési/Bézy* (Jessel, who says all writers on the origin agree) [S53]; Dumesnil also says Limousin [S50]. Cinq-Cents: "south of France" [S45, S49] vs "north of France" [S53]. Arrival in Paris: about forty years before 1867, i.e. c. 1827 [S26] vs "only a few years" before 1864 [S45, S49] vs about 1850 [S53] vs "new" in 1847 [S5]. Not settled here; no majority vote. Jessel also says bezique is a blend of Briscan and Mariage and that Cinq-Cents is a cousin, not an ancestor, contradicting Cavendish [S53] (compare the ancestry list at the top of this section, which follows S4).
- **Spellings** [S48, S53]: Pole 1861 "Bazique"; Goodall, Reynolds and English (1868–69) "Besique/Bézigue"; Van Tenac (1858) "Bézigue"; Baldwin 1870 lists Bésigue, Bésit and Bézique; Cavendish 1870 "anglicised into Bézique." Jessel argues that Bézique is an English form.
- **Early literary hits:** Gaboriau, *Le 13e hussards*, a copy catalogued 1861 on archive.org, has a sergeant torn between a bésigue game and an assignation [keyword hit; title page not checked]. An undated Rouff-era printing of Paul de Kock's *Friquette* has players arguing over eight versus nine cards, four packs, 2000 up and a third bésigue worth 1500. That is 1860s–80s practice (four packs, triple bezique), not the single-pack game of Méry's 1847 rules, so either the novel's date (1846 per S10) is wrong or this printing revised the passage; weight: low. [both EXCERPT-level]
- **Cavendish's own ancestry account, Jan 1870** [S55]: brusquembille (3 cards, aces and tens best), *homme de brou* (four players, 300 up, marriages paid from a box of counters), briscan/brisque (5 or 6 cards; quint major won the game; tens counted in fours; last stock trick 10; blaze and carte blanche scores dropped from the modern game) and cinq-cents (500 up, quints 250/125) are the sources; he thinks brisque and cinq-cents were the direct parents of bezique, made by someone shuffling two piquet packs and playing brisque with them. He records that the invention had been ascribed to the Swiss, the French and the Americans without adequate grounds, and says the name has no clear reason behind it. [S55] Compare Cady's summary of the same ancestors (S4). The Athenaeum (Sep 1869) doubts whether the game came from France or from the Southern States [S59].
- **Etymology debate in N&Q (1874)** [S59]: derivations offered were Spanish *baza* (a trick) and Italian *bazzica* (company, conversation); Cavendish said no one knew the origin; a correspondent quotes *Once a Week* (13 Feb 1869) as saying Pole's Dec 1861 rules were the first published. Jessel (1903) argues the French forms are older and "Bézique" an English one [S53].

### 3.2 Britain and publishing
- Dr. William Pole introduced it to England in 1861 (Macmillan's, December 1861, spelling "Bazique") but it only took off in 1869. [S6, S16, S48] S6/S16 add that *Hoyle's Games Modernised* documented it in 1863; [unverified] this sits badly with Pole's remark that the game had not been described in print [S48], with a reader's request for its rules in February 1866 [S52], and with Jessel's bezique entries, which list no 1863 item [S51].
- **Pole 1861 read (S48):** he calls it an unwritten game, probably of recent invention, and says he has never seen it described in print. Two piquet packs; rank as in whist but the ten between ace and king; game 100, though some players make it 1,000 (all scores then in tens). Deal in turn, 3-3-2, 17th card turned. Values in units of ten: four aces 1 (10), kings 8 (80), queens 6, knaves 4, marriage 2, trump marriage 4, sequence 25 (250), bazique 4, double 50 (500), seven of trumps 1. Declaration only right after winning a trick and before drawing; declared cards stay face up and in hand; same-class re-use barred, different class allowed, **including sequence cards later married**. When the stock is down to one card plus the turn-up, the trick winner takes the stock card and the other player the turn-up; declarations stop, table cards return to hand, and the second player must follow suit or trump. Last trick 1 (10); brisques 1 (10) each, counted after the last trick; if both reach the target only by brisques, only the majority is scored and another hand may follow. Forfeit: one point (10) for each card taken at the wrong time. Winner draws first. [S48]
- **Bibliographic map of English-language bezique publishing.** Source: Jessel 1905 [S51] unless another is named; the entries are bibliographic records, not texts read.

| Year | Item | Publisher / note |
|---|---|---|
| 1861 | Pole, "Bazique," *Macmillan's Magazine* | first English description [S48] |
| 1865 | A. B. Swift, *Bézique* | New York (cited from the Westminster Papers) |
| 1868 | *The Royal Game of Bezique* (8 pp) | Goodall; the first Goodall rule booklet |
| 1868 | *Besique* | Reynolds |
| 1868 or 1869 | Jones ("Cavendish"), *Pocket Guide to Bezique* (18 pp) | De La Rue; Jessel 1868, Hall 1869 [S7] |
| 1869 | *Bézique* | English & Co. |
| 1869 | *Rules for the Game of Bésique* (16 pp), "translated from the most approved French methods" | Leuchars & Son |
| 1869 | De La Rue rules card (2 pp) | De La Rue |
| 1869 | Ware, *Handbook of Bezique* (96 pp) | Warne |
| 1870 | Baldwin, *Rules of Besique* (12 pp), adapted from Bonneveine's *Nouvel almanach des jeux* and Van Tenac's *Académie des jeux* | Harrison |
| 1870 | "Battle-Axe," *Besique* (16 pp) | Tiver |
| 1870 | *Feldinski, or Besique Royal: A Polish Game* | Willis |
| 1870 | Ware, *The Modern Hoyle* (adds Besique); Pardon, *Card Player's Manual* (adds Bezique) | Warne; Ward, Lock |
| 1873 | Jones, *Pocket Guide to Polish Bezique* (18 pp); De La Rue Polish rules card (2 pp) | De La Rue |
| 1873–74 | *The Popular Recreator* (Bezique by George Tindall) | Cassell |
| 1874 | *Bézique without a Marking-board* (2nd ed.); *Complete Guide to the Game of Bézique* | Cornish (London); Scribner (New York) |
| 1878 | *Rules and Directions for Playing the New Game of Bésique* (3rd ed., 12 pp) | Willis |
| 1883 | Camden, *The Standard Rules of the Royal Game of Bézique* (11 pp) | Goodall |
| 1887 | Jones, *The Laws of Rubicon Bézique adopted by the Portland Club* (19 pp, private) and the 49-pp edition with his guide; *Pocket Guide to Rubicon Bezique* (21 pp) | De La Rue; Hall also lists a *Pocket Guide to Russian Bezique* (1887) [S7] |
| 1889 | *Die Salonspiele. Besique…* (32 pp, German) | Vienna: Wenedikt [S64] |
| 1890 | Berkeley (W. H. Peel), *Bezique and Cribbage* (96 pp; first edition suppressed as largely pirated from Cavendish; 63-pp edition also 1890) | Bell |
| by 1892 | Jones, *Pocket Guide to Japanese Bezique* (undated) | De La Rue [S64] |
| 1895 | Lewis, *Rubicon Bezique* (28 pp) | Goodall |
| 1901 | Stewart, *How to Play Bizique and Euchre* | Gaskill & Webb |
| 1902 | *The Laws of Bezique* (15 pp) | U.S. Playing Card Co., London and Cincinnati |
| 1903 | "Persicus" (Baron de Reuter), *Laws of Four-Handed Bezique adopted by the Portland Club* (96 pp) | De La Rue |
| 1903, 1905 | *Bézique Rules* (17 pp); *Bezique and Rubicon Bézique* (21 pp) | International Card Co., Glasgow |
| 1921 | Jessel (ed.), *The Laws of Six-Pack Bezique or Chinese Bezique* | Mudie; De La Rue reprints 1934, 1937 [S7] |
| undated | Hunt & Co., *Rules of Bézique*; J. Jaques & Son, *The Games of Bezique and Barrule* (31 pp); Pardon, *Bézique, Euchre, Écarté…* (97 pp) | London |

- **Date conflict:** Cavendish's *Pocket Guide to Bezique*: 1868 in Jessel [S51], 1869 in Hall [S7]. Not settled. [inference] The Pall Mall Gazette praise of Cavendish on 23 Jan 1869 (S6) makes a late-1868 first appearance plausible.
- **Cavendish, Jan 1870 [S55]:** an early signed statement by Cavendish, in *The Gentleman's Magazine*, says several points were then disputed: which trick is "the last trick" and whether a higher card of any suit or only of the suit led wins. He follows the briscan precedent: the "last trick" (10 points) is the last *open* trick, the one made when two cards, the trump and one other, remain in the stock, though eight tricks still remain; the last of all is ignored. He has the lowest cut deal; the second player is free to follow, trump or discard; brisques scored as won (the best and simplest method, in his view); the winner of the last stock trick may still declare, then declarations cease; last eight tricks: the second player must follow suit if able and win the trick if able (with a trump if void); game 1000, double if the loser is under 500, a match is best of three of five. [S55] (Foster says Cavendish's first *Pocket Guide* erred on the deal and the last trick; see the Foster bullet below [S47].)
- **Cavendish on declaring, Jan 1870 [S55]:** he says the older rule allowed one declaration per trick but that now a player may declare everything he holds after winning a trick, provided one meld is not part of another; bezique and four aces may be declared together and 140 marked, but K, Q♠, J♦ cannot all be declared to make marriage plus bezique at once (the queen would have to wait). A meld whose cards are all already on the table cannot be declared. [S55] Contrast Pardon (one combination per trick) [S56] and Foster (announce several, score one) [S47].
- **Pardon [S56], c. 1870–76:** a handbook that gives the lowest cut the deal, a forfeit table (drawing out of turn 10, playing out of turn 10, playing without drawing 10, overdrawing 100, a revoke in the last eight tricks forfeits all eight tricks), scores for playing, turning up or exchanging the 7 of trumps (10 each; not in the last eight tricks when played), the last trick 10 (with a note that reading it as the 32nd trick is, in his view, a mistake of nomenclature), brisques sometimes counted at the end, reduced scores by agreement (double bezique 300, sequence 200), and three-handed play with an extra pack per player (or two packs less an Eight). [S56]
- The game was slow to catch on until 1868, when Goodall and Reynolds both re-launched it. The Stationers' Hall registration race was in **September–October 1869** (Reynolds first, Goodall four days later); Goodall's marker design was registered 15 April 1869. Reynolds' booklet claimed rules translated from "L'Académie des Jeux" as published in *The Times* of 24 December 1868. [S6, S40]
- S6/S16 date a 10th edition of Goodall's bezique booklet to 1870 and count 50 editions between 1870 and 1920; the booklet was sold as *The Royal Game of Bezique* and the Duke of Edinburgh's patronage was used in marketing [S6, S16, S28]. The "Camden" authorship label is attested by 1872: Wikipedia cites a Camden *Standard Rules of the Royal Game of Bezique*, 12th edition, 1872 (Goodall) [S9]; Jessel lists an 11-page Camden booklet of 1883 [S51]. A 10th edition of 1870 under the same label is therefore plausible (editions were frequent); not verified.
- Pall Mall Gazette, 23 Jan 1869, complained that publishers' rules were unintelligible, with Cavendish's the exception. [S6]
- De La Rue bibliography (Hall) [S7]: Cavendish's "Pocket Guide to Bezique" 1869 (18 pp; 16th ed. 1916 and 1925); "Polish Bezique" 1873; **"Russian Bezique" 1887 (game unidentified)**; "Rubicon Bezique" 1887 (3rd ed. 1898); later replaced by Angelo Lewis's *Rubicon Bezique* and "Camden"'s *Bezique*. Jessel's *Laws of Six-Pack Bezique or Chinese Bezique* (Mudie, 1921; De La Rue reprints 1934 and 1937).
- Heyday in Britain 1870s to mid-1930s; a bezique set was among the 1933 Wills miniature-card gifts; decline from the 1960s. [S6]
- Spectator 1887 review: four packs, carte blanche, first marriage sets trump, plain sequences, nine cards; the reviewer thought the rubicon scoring only increased the gambling element. [S25]
- **Foster on the English arrival** [S47]: bezique reached England in the winter of 1868–69; Dr. Pole's 1861 rules (the OCR prints "i86t"; read as 1861, matching S6/S16) came first; many publishers then rushed out their own versions. Points then in dispute: whether the last trick meant the absolute last or the last before the stock ran out; whether the highest or lowest cut dealt (Cavendish erred on both in his first *Pocket Guide*, silently fixed in the second); whether trumps had any force before the stock was exhausted (many players gave trumps no value until the last eight tricks); and whether a double marriage should equal a double bezique. Foster says time settled all of these and the rules were by his day practically uniform everywhere. [inference] The lowest-cut-deals rule found in the US Hoyles (1864–68), Cavendish (1870) and Pardon is therefore probably early practice, later dropped.

### 3.3 Rubicon, Chinese, Polish
- Paris clubs codified a four-pack game in 1880 as *Règle du Bézique Japonais*; the Portland Club's 1887 code renamed it Rubicon. [S4, S6, S3]
- Six-pack / "Chinese": the "1899" date in S16 is unsourced [S16]. Foster (copyright 1897) defines Chinese Bezique as Rubicon with six packs, scores often reaching 6000 [S47]; the name alone appears in Daudet's *Les Rois en exil* (1879) [S10, S33]; six-pack play is mentioned c. 1870s (below); codified laws first appear in Jessel 1921 [S7]. Churchill's 1896 letter (quoted in S16) asks for Rubicon (four-pack) cards.
- **"Bésigue chinois" in French:** the Nouveau Larousse defines it as an ordinary two-hand bésigue in which one card can be re-scored in like combinations (the same king in as many marriages as there are queens of its suit), which is *not* the six-pack game [S54]. Jessel (1903) treats the Chinese, Japanese and Polish labels as fancy names [S53]. Foster (1897) is still the earliest dated definition of Chinese Bezique as six-pack Rubicon [S47]. Keyword search of Hong Kong and Shanghai papers (1870–97, archive.org) found bezique cards and Berkeley's book advertised, but nothing on Chinese Bezique.
- **Six-pack and four-pack play attested earlier than Foster:** Pardon (catalogued 1876, text after Sept 1870) says the game may be played with four or six prepared packs, nine cards each [S56]. The paragraph on "Number of Packs" in Cavendish's guide, as reprinted in Phillips's *Standard Hoyle* (and Fox), says two players sometimes use four or six packs, nine cards may be dealt, the game is 2000 up and triple bezique scores, and adds that so many packs make the game too complicated [S57]. Hall (2024) describes bezique developing into Polish and Chinese bezique, with the pack growing from 64 cards to six packs [S61]. [inference] Pardon's text cannot be dated closer than 1870–76 and the reprinted Cavendish paragraph could come from any edition of his guide, so the earliest *securely dated* mention of six-pack play remains Foster 1897 [S47]; the undated evidence points to the 1870s. A named, codified six-pack game ("Chinese") is first firm in Foster (1897) and Jessel (1921).
- **"Japanese Bezique":** Horr's 1892 bibliography lists, in the Cavendish Pocket Series, "Pocket Guide to Japanese Bezique" (undated), beside "Pocket Guide to Bezique" (De La Rue, 1890, 11th ed., 21 pp) and the separate *Laws of Rubicon Bezique* (1887, 49 pp); it lists no Polish, Rubicon or Russian pocket guide [S64]. Cady says the Paris clubs codified the four-pack game as *Règle du Bézique Japonais* in 1880 and the Portland Club renamed it Rubicon in 1887 (§3.3, first bullet; S4), and Jessel lists a Jones "Pocket Guide to Rubicon Bezique" of 1887 (21 pp) [S51]. [inference, unverified] The Cavendish pocket guide titled "Japanese Bezique" may be the same four-pack game under its earlier name, later reissued as "Rubicon Bezique" (1887); this would also explain why Jessel and Horr never list a "Russian Bezique." Hall's "Russian Bezique, 1887" (S7) remains unexplained: a slip for Japanese, or a real but lost title.
- **Precursor of Rubicon's trump rule** [S45, S46]: the 1864 and 1868 US Hoyles already describe an ordinary two-pack game with no turn-up card, the first marriage declared setting trump (§5). Cady says the ordinary game is sometimes played this way too [S4]. So trump set by the first marriage predates the 1880 Paris code and the 1887 Portland laws (§3.3, first bullet). Foster says Rubicon seems to have originated in France [S47].
- Polish Bezique / Fildinski: a 1870 British Library record for "Royal Besique by Fildinski" and Cavendish's 1873 Polish guide; "Fildinski" is not a meaningful word; a blogger hypothesises Cavendish invented it (unproven). [S15, S17, S28 (V&A box 1901–1910)] Jessel lists *Feldinski, or Besique Royal: A Polish Game* (Willis, 1870), with a note that Lord Sherbrooke said in N&Q 1903 that a member of the Feilding family wrote it; that N&Q note was not found (the 7 Feb 1903 issue holds only Jessel's reply, S53, which calls the Polish, Chinese and Japanese labels fancy names). [inference, unverified] Sherbrooke (Robert Lowe) died in 1892, so any such statement in 1903 would be a repeat of an earlier one. Cavendish's Polish guide and the De La Rue Polish rules card are both 1873 [S51]. [inference] Since a Feldinski booklet exists in 1870, the blogger hypothesis (S17) that Cavendish invented the game in 1873 looks weaker; it is not disproved.

### 3.4 Famous players and cultural appearances
- Churchill: associated with six-pack in Wikipedia and S12; letters to 1895–96 mention bezique; the Churchill Archive catalogues a letter enclosing a cheque for a "Bezique debt." Anecdotes (Sicily night, "winning the war was easier") are only in S16; unverified. [S9, S16, S37]
- French literature: Paul de Kock, *Friquette* (1846; date per S10, but see §3.1 on the passage in the printing examined); Offenbach/Moinaux, *Les Deux Aveugles* (1855); Daudet, *Tartarin* (1872) and *Les Rois en exil* (1879, calls Chinese bezique a game where a bad player can lose a fortune without effort); Zola, *Nana* (1880); Maupassant, *Une vie* (1883); George Sand's letters; Becque, *La Navette* (opens with a character scoring forty for bésigue); Huysmans (a partner announces "le 250"). [S10, S33]
- Caillebotte's 1880 *La Partie de bésigue* shows a small wooden marker; now in the Louvre Abu Dhabi. [S10, S16, S42]
- Lewis Carroll's 1869 poem "The Three Voices" and Tsaritsa Alexandra's last diary entry (17 July 1918) mention it, per S16 only. Christina Rossetti and Wilkie Collins are said to have played. [S9, S16] The Alexandra diary claim is echoed by a site citing the annotated *Last Diary of Tsaritsa Alexandra* (Yale University Press, 1997) [S68; edition not read]. A scholarly edition of Lawrence's *Lady Chatterley* novels (Cambridge University Press, 2002) annotates bezique's Victorian popularity [S9, reference list; not read].
- The *Westminster Papers* (1870) has a correspondent dismissing bezique as the slowest of slow games; S16 attributes the remark to Lord Aldenham [S16; the passage itself read in the volume catalogued 1871].
- US: box set by Albert Swift 1865; Dick & Fitzgerald's *Modern Pocket Hoyle* (1868); a ladies' Bezique Club in Riverton, NJ, appears in society columns in 1897–98. [S16, S34] The *American Hoyle* 4th edition (copyright 1864) already has a full bezique chapter [S45]; whether earlier editions did is unknown. *Harper's Weekly* and *Frank Leslie's* of 10 June 1865 advertise "Trumps's" *American Hoyle* (507 pp) as including Besique, so the book was on sale in the US by June 1865 [S59]. An *Englishwoman's Domestic Magazine* essay of 1869 calls the origin unsettled between France and America and jokes about Basque origin; it is not a rules source. [S59]
- Countries still playing: Turkey, Haiti, French Polynesia (Spanish Wikipedia). [S32]
- **Denmark:** an undated Copenhagen-shop booklet in Danish exists (S62). Its terms: *Matador* for the trump sequence, *kongeligt/borgerligt ægteskab* for royal/common marriage. Polish- and Japanese-language pages found are modern digests (S63). Dutch, Czech and Portuguese searches found only modern digests (S63); no period source in those languages was found. [S62, S63]
- **Austria:** Horr lists a German-language booklet, *Die Salonspiele. Besique. Regeln und Vorsichten dabei, dann Schnapsen…* (Vienna: Wenedikt, 1889, 32 pp), so a Vienna printing of Besique rules exists by 1889 [S64, bibliographic record only, not read].

---

## 4. Core rules (2-player, consensus)

*"Consensus" means the practice of the 1890s and later standard texts. Where earlier texts differ, the difference is noted and cross-referenced to §6.*

- **Cards:** two 32-card piquet packs = 64; suits equal; rank A, 10, K, Q, J, 9, 8, 7 in cutting and play. [S2, S3, S4]
- **Deal:** cut for deal. Who deals depends on date and country: the lowest cut deals in the US Hoyles (1864–68), Cavendish (1870) and Pardon; the highest cut deals or chooses in the French manuals (1865–67), Cady, Foster and the Rubicon laws; Hoffmann and Wikipedia are recorded as giving the highest cut [S3, S9; not re-read] (§6 row 12). Eight cards each, usually 3-2-3 (row 15); the 17th card is turned up for trump; a turned 7 scores the dealer 10. [S3, S4, S26, S45–S50, S55, S56]
- **Trick:** non-dealer leads any card; second player may play any card; higher card of the led suit or any trump wins; identical cards go to the leader. [S3, S4] (One 1866 note and a modern digest let any higher card win: row 19.)
- **Melds:** only the trick winner declares. In the 1890s standard texts several melds may be laid down but only one scores per trick, the rest announced "and N to score" [S3, S4, S47]; earlier and local practice was one meld per trick, or all disjoint melds at once (Cavendish 1870) (row 4). Declared cards stay face up but remain part of the hand. Declare **before drawing**; then the winner draws first (rows 2, 17). Combination cards must all be in hand at the same time, so cards played to a trick can't count. [S3, S4, S5, S9]
- **Reuse:** Cady and Foster sort melds into three classes: (A) marriages and sequences, (B) bezique, (C) fours. A declared card may join a meld of a *different* class; within one class it can be re-declared only in a *superior* meld, not an equal or inferior one. So marriage → sequence works; sequence → marriage of the same king and queen does not; fours ↔ marriage works (different classes); a queen of spades used in a bezique can later join a marriage, sequence or four queens. [S4, S47] The 1860s texts differ (row 16).
- **End of stock:** the winner of the trick that leaves one card in the stock takes it; the loser takes the turn-up; melds go back into hands; no more declarations. [S2, S3, S4]
- **Endgame:** the second player must follow suit, win the trick if able, and trump if unable to follow (see §6 row 1; Pole 1861 and Dumesnil 1865 omit the duty to win). [S2, S4, S45–S47, S55]
- **Scoring extras:** 10 for the last trick of the hand (from the 1890s the last of all; earlier disputed, row 9); brisques (aces and tens won in tricks) 10 each; game 1000 (row 10). [S2, S3, S4, S47, S55]

### Meld values (standard)

| Meld | Points |
|---|---|
| Common marriage (K+Q, plain suit) | 20 |
| Royal marriage (K+Q, trump) | 40 |
| Bezique (Q♠ + J♦) | 40 |
| Double bezique (both Q♠ + both J♦) | 500 |
| Four jacks / queens / kings / aces (any suits) | 40 / 60 / 80 / 100 |
| Trump sequence (A-10-K-Q-J) | 250 |
| 7 of trumps (dix): turned up, exchanged or played | 10 |
| Last trick; each brisque | 10; 10 |

- Double bezique declared all at once scores only 500; adding an undeclared single to a declared one scores 40 + 500 [S4, S47]. Whether two separate singles each score 40 before the double is disputed: Cady says yes (40 + 40 + 500), Foster says the second single cannot score (row 20).
- There is no score for four tens in Bezique (Marjolet has it). [S2, S5]

---

## 5. Variants (summary)

### 5.1 Ordinary two-pack game: options found in period texts

- **Bezique without a turn-up (1864/1868 US Hoyles):** ordinary game, but no trump card is turned. Trump is the suit of the first marriage declared by either player. The trump sequence cannot be declared until then, the 7 of trumps scores nothing, and bezique, fours and other melds count as usual and may be declared before trump exists. [S45, S46] Cady describes the same option (also no dix) [S4].
- **Bezique Panaché (1864/1868 US Hoyles):** a four (aces, kings, queens or knaves) counts only if it holds one card of each of the four suits; K♠ K♠ K♥ K♦ scores nothing. It is to be specially agreed beforehand. The books' closing sentence says other combinations follow ordinary rules; its wording is odd and may be garbled. No different point values are given. The wording is identical in both books. [S45, S46] [inference] Ordinary fours are therefore any-suit, as Foster says explicitly [S47].
- **"Limited to a fixed point" (1864):** by prior agreement, the first player to reach the target may stop and claim game without playing the hand out, adding his brisques; if the claim falls short (example: 1,490 against a 1,500 target) the opponent wins. [S45]
- **Hand size and packs, two players (1864/1868):** eight cards normally; nine or ten now and then, with the trump card then the 19th or 21st; two *or three* packs of 32 [S45, S46]. Cady also lists a two-player three-pack version as a variety some players prefer [S4]; nine cards otherwise appear in Rubicon [S4, S47].
- **Bezique pair depends on trump, in the ordinary game (Cavendish, Pardon):** when clubs or hearts are trumps the pair is Q♠+J♦; when spades or diamonds are trumps it is Q♣+J♥ (Cavendish 1870 and later reprints; Pardon); in Polish bezique, Cavendish gives Q♥+J♣ instead [S55, S56, S57]. Cady agrees for Polish [S4]. The Québec six-pack page maps each trump to its own pair (spades: Q♠J♦, diamonds: Q♦J♠, hearts: Q♥J♣, clubs: Q♣J♥) and notes that many players accept only Q♠+J♦ [S60]. Foster, Hoffmann and the 1860s texts give a fixed Q♠+J♦ [S45, S47].
- **Trick winner: any higher card vs same suit only (a disputed early point):** N&Q 1866 describes a higher card of any suit winning [S52]; Cavendish 1870 lists the point as disputed [S55]; the Québec page states as its main rule that the highest value or a trump wins and suit does not matter, giving suit-based winning as a variant [S60]. All other sources read here use same-suit-or-trump. [S52, S55, S60]
- **American Hoyle 1892 [S58]:** the chapter is now Cavendish's, with these choices: deal 8 cards one at a time or 3-2-3; the second player is not bound to follow suit or to win before the stock ends; the winner draws the top card first; brisques scored as won, with an option to count at the end (if both then score out and tie, the game is void); no-turn-up option where the first marriage (or, failing that, the first sequence) sets trump and no 7 scores; the 7 is only shown, not laid on the table, when declared; last eight tricks strictly follow-and-win; game 1000 up, players deal alternately, double if 1000 before the opponent has 500, a match is best of three games of five with doubles counting two. Rubicon: higher cut chooses deal, seats and markers; carte blanche 50; rubicon settlement as in Foster; a player may not count brisques in his tricks while more than twelve cards remain in the stock; the stock may be counted. [S58]

### 5.2 Multi-pack and multi-hand games

*Period evidence for four-, six- and eight-pack play is thin: the Spectator review of 1887 (S25), the remarks of Cavendish and Pardon (S56, S57), Foster's definition of Chinese Bezique (S47) and Jessel's 1921 laws (not read). The detailed six- and eight-pack rules below come from modern web sources [S12, S13, S14, S39, S60] and describe post-1921 practice, not period rules.*

- **Rubicon (4 packs):** 9 cards, no turn-up, first marriage (or sequence) sets trump, no dix, carte blanche 50 (shown before playing; re-scores each draw until a court card comes), plain sequence 150, triple bezique 1500, quadruple 4500, last trick 50, brisques counted only for ties or to save the rubicon, one hand = one game. Tricks are left face up until a brisque is played. Melds can be re-scored after replacing a played card. Game value: winner minus loser (to nearest 100) + 500; if loser under 1000 ("rubiconed"), winner adds both scores + 1000 + 300 for brisques (32 brisques = 320, but the odd 20 aren't reckoned). Hoffmann's Rubicon Laws (S3) run to 66 numbered laws. [S2, S3, S4]
- **Rubicon details added by Foster (1897):** nine cards dealt 3-3-3; carte blanche 50 only if dealt, re-scored 50 on each draw until a K, Q or J comes; plain sequence 150, trump sequence 250; bezique 40/500/1500/4500; last trick 50; no dix; the first marriage scored sets trump (its holder need not declare it). Declared melds may be broken up and re-formed indefinitely by playing a card and replacing it (a trump sequence re-scored six times = 1500), but double, triple and quadruple bezique need all their cards on the table at once and are built cumulatively (single 40, then double 500, triple 1500, quadruple 4500: 6,540 in all). Tricks stay face up until an ace or ten is played; the winner then gathers them. Brisques count only to break a near-tie or save a rubicon: if the scores are within a hundred or two, brisques are counted; if still within 100 after that the winner adds a 100 bonus. Game value = winner minus loser (fractions of 100 dropped) + 500. Loser under 1000 = rubiconed: his score is added to the winner's, plus 1000 and 300 for brisques whoever won them; +100 more if the loser scored under 100; a rubiconed loser is saved if his brisques lift him to 1000, otherwise brisques are ignored. Last nine tricks: the second player must follow and win if able. Players may count the stock (English law; French do not). [S47]
- **Six-pack / Chinese (192 cards):** 12 cards each; the dealer cuts about 24 cards and the opponent guesses the count (exact 24 scores the dealer 250, correct guess 150); carte blanche and last trick 250; trump fours 1000/900/800/600/400; rubicon 3000; game bonus 1000; the same suit can't set trump on consecutive hands; bezique is trump-dependent (S13). [S12, S13, S14, S39]
- **Chinese Bezique (Foster 1897):** Rubicon with six packs; scores of 6000 not unusual; no other rules given. [S47]
- **Eight-pack:** 15 cards, rubicon 5000, quintuple bezique 9000 (modern sources). [S13, S20]
- **French "bésigue chinois" (Nouveau Larousse):** see §3.3; a card may serve for several like combinations. Its ordinary game there uses nine cards, and the turned-up card is the top of the stock; three-hand is played with three packs, three Q♠+three J♦ = 1500. Card-point values in the OCR are damaged (a Cinq-Cents-style scale, ten 10, king 4, queen 3, appears). [S54]
- **Three-hand:** three packs, triple bezique 1500, 2000 or 3000 up. **Four-hand:** partnerships, partners may combine declared cards for double/triple bezique. [S4, S13]
- **Three-handed:** 1864/1868: with two packs, one Eight is removed first [S45, S46] ([inference] 63 cards; how the odd stock divides is not addressed). Foster: two full packs plus a third pack lacking an Eight, triple bezique 1500, 2000 up [S47].
- **Four-handed:** 1864: two against two, partners opposite, scores and brisques pooled, last trick 10, or more by agreement [S45]. Foster: four packs, triple bezique 1500, 2000 up; on a trick win either partner may declare everything in the hand but only one meld scores per trick, and partners may build on each other's melds; for the last eight tricks players re-pair with a former opponent, seats unchanged [S47].
- **Chouette bezique:** a lone player against advising others, with rotation rules. [S4]

### 5.3 Neighbouring games and ancestors

- **Marjolet** (32 cards, 6-card hands, 500 up): the *loser* of the 10th trick scores 10 (McLeod's correction of an error in Parlett's *Oxford A–Z*, fixed in the *Penguin Book*); four tens score, four jacks don't; 50 for the last six tricks. [S2]
- **1847 single-pack Bésigue / "Cinq Cents":** 6-card hands, 13th-card turn-up, dix exchange only before your first card, meld from the five remaining cards before drawing, sequences 250 plain / 500 trump, fours 100/80/60/40 (no tens), card points A 11, 10 10, K 4, Q 3, J 2 (total 120), 500 up; trick points counted only after a player reaches 400. [S5, S11]
- **Foster's Cinq-Cents (1897):** one pack; extra 120 for a plain-suit sequence of the five highest; brisques not scored as taken but card points counted at the end (A 11, 10 10, K 4, Q 3, J 2; total 120); a player who knows he has enough may "knock" to stop play and count, and a wrong knock loses; 500 up, raised by 100 if both qualify without a knock. [S47] Same scale as S5/S11.
- **Foster's Penchant (1897):** a single-pack six-card game with pairs, triplets and flushes, twelve brisques (the sevens count), trump set by the first "penchant" (Q+J of different suits), four deals to a game. Not bezique proper. [S47]
- **Brac (Larousse 1867):** a derivative of bésigue where the trump quint scores 150 (not 250) and the double bésigue 80 (not 500); three players, two holding the cards, the one with fewer points giving way to the third; 60 or 80 counters of agreed value; each hand settled by the difference in scores. Larousse says it dates only from a few years earlier. [S26]
- **Binocle (French, per S14):** 5-card, four-deal game with no turn-up. **Two-player Pinochle:** 48 cards, 12-card hands, run 150. [S8, S14]
- **Polish / Fildinski:** melds built only from cards won in tricks; at least one new card in each meld; up to two melds per trick (sources vary); brisques scored as won; melds continue in the endgame; 2000 up; Cady says bezique cards become Q♥+J♣ when spades or diamonds are trumps; the 7 of trumps is called "the Deece" in S13 and S15. [S2, S4, S13, S15]
- **Foster's "Polish Bezique" (1897)** is a different description from Cady's: the trick winner sets aside the court cards and the trump ten from tricks he wins, face up, to form melds with cards from hand or from later tricks; those won cards can never be led or played or taken into hand. Nothing else is said to differ from the ordinary game; no length, brisque or endgame rule is given. [S47] (Compare the Polish bullet above: S2, S4, S13, S15.)

### 5.4 National and local traditions (modern or undated sources)

- **Bésigue haïtien:** four packs + four jokers, a scorer called a "marque"; declared jokers can't be reused. [S10, and a French snippet]
- **Russian "no-trump" (бескозырный):** first marriage sets the dominant suit for 40 (50 if double); Russian sources also describe a three-deck game to 1500. [S18]
- **Turkish Bezik:** 96 cards (9–A × 4 packs), 9 cards dealt in threes, to 3000, first K+Q shown sets trump; sequence ("tablo") 250 trump / 150 plain; trump fours 1000 (aces), 800, 600, 400; mixed-suit fours 100/80/60/40; bezik 50/500/1500 for first/second/third; last trick 250; you must call the number aloud when laying a meld; **"vido"** doubling (2, 4, 8; the opponent accepts or concedes; can't redouble twice in a row); **"şuvet"** (chouette-like lone player with advisers); loser under 1500 pays double ("rubikon"); scoring frame called a **"marköz"** with pegs 1, 5, 10, 50, 100, 500, 1000, 5000; "brisk" = ace/ten in tricks, 20 each if the stock runs out before game. [S19]
- **Danish booklet [S62]:** two-hand bezique with fixed Q♠+J♦; one meld at a time; sequence 250 (*Matador*), fours, marriages 40/20, 7 of trumps 10 when declared (exchangeable for the turn-up), last trick 10, brisques 10; a card used in a meld cannot join a like meld (a king married may join four kings or the sequence); after a single bezique its cards **cannot** be used for a double bezique; cards laid on the table cannot alone build a new meld (after declaring the sequence you cannot then declare the royal marriage); once the stock is gone the led suit must be followed or trumped (a duty to win is not stated). Game 1000: the first player to reach it by melds wins even if the other could count more in aces and tens; otherwise aces and tens are counted and the taker of the last trick counts first. Three-hand: three packs, 2000 up, triple bezique 1500; four-hand: four packs, partners as at whist, the trick winner declares or says pass and then the partner may declare; 2000 up. [S62]
- **Québec compilation [S60]:** ordinary game, lowest card drawn deals; deal 3-2-3, or 3-3-3 for nine, 3-3-3-1 for ten; only one combination per trick; the same card may serve in several combinations except twice in the same four; endgame: opponent must follow and beat, else trump, else discard; 1500 up (or 1000 or 2000); no-turn-up variant; three-hand ("trifouille") with three packs, triple bezique 1500, 2000 up; four-hand with partners, who may finish each other's melds but not build a double or triple from a partner's cards; six-pack: 12 cards, dealer's cut of 24 (250) or a correct guess (150), carte blanche 250 re-scored per draw, plain-suit quint 150, last trick 250, trump fours 1000/900/800/600/400, the same suit cannot be trump twice in a row, one hand per game, 1000 bonus, rubicon under 3000; eight-pack: 15 cards, rubicon under 5000, quintuple bezique 9000; Rubicon (four packs): nine cards, "portes arrière" (plain-suit quints) 150, carte blanche 50, 500 game bonus, rubicon under 1000 with 1000 bonus plus both scores plus 320 for brisques; chouette rotation. No sources are given; treat as a modern digest. [S60]

---

## 6. Rule-conflict matrix (engine decision points)

The table gives the range of positions and an engine note; §6.1 lists each source's position with its date and edition. Conflicts are not settled by counting sources.

| # | Question | Range of positions | Engine note |
|---|---|---|---|
| 1 | Endgame: must you trump, and must you win? | Once the stock is gone, the second player must follow suit and (in most sources) win if able, else trump. Strict in every source read from 1864 on; Pole 1861 and Dumesnil 1865 omit the duty to win; looser only in S3's standard passage and S20. Penalty varies: 40 points (US 1864–68), replay or forfeited scores (Foster, French), all eight tricks (Pardon). | Default strict; the leader is unbound; an engine can simply prevent illegal moves. |
| 2 | Meld before or after drawing | Before drawing (after winning the trick) in every period source, 1861–1897; after drawing appears only in a Pinochle rulebook (S8). | Default before. Simulation: after-drawing adds about 20% to meld points (§10b). |
| 3 | Dix timing | Highly variable: exchange the 7 of trumps after winning a trick, before drawing (Cavendish, Foster, S4); 10 also for a 7 turned up or played (US 1864–68, Pardon); Larousse's editor: only when it wins a trick; Foster: not on the same trick as a meld. | Expose as options; default: the exchange scores 10 at a trick win. |
| 4 | One meld per turn vs several | One meld per trick (US 1864–68, Larousse, Dumesnil, Pardon, Danish, Québec); several announced but one scored per trick, the rest "to score" (Foster, Hoffmann, Cady); all disjoint melds scored at once (Cavendish 1870). | Simulation: all-disjoint changes meld points by under 1%, so a minor decision (§10b). |
| 5 | Four-of-a-kind composition | Any suits in every source. "Panaché" (one of each suit) appears only as a special-agreement variant (S45, S46, S49; S16 mentions it). | Option. |
| 6 | Plain sequence | None in the ordinary game (S2, S3, S45–S47); 150 in Rubicon; 250/500 in the 1847 single-pack game; S20's 100 is unsupported. |  |
| 7 | Which cards make the bezique | Fixed Q♠+J♦ (Hoffmann, Foster, Cady's ordinary game, the 1860s texts, Danish) or trump-dependent: Q♣+J♥ when spades or diamonds are trumps (Cavendish's ordinary game, Pardon); Q♥+J♣ in Cavendish's and Cady's Polish game; a full four-way mapping in modern six-pack. | Expose as an option. |
| 8 | Brisque timing | At the end of the hand in every source of 1861–68 (Pole, the French manuals, Larousse, the US Hoyles); as won in Cavendish 1870, Cady, Foster and the 1892 Hoyle (which allows end-counting as an option). Rubicon: brisques only to break ties or save a rubicon. | Simulation: the game winner changes in 3.7% of games (§10b). |
| 9 | "Last trick" bonus | Last *open* trick (the trick that leaves one stock card; Cavendish 1870; Pardon calls the 32nd-trick reading a mistake) versus the last of all (Cady and Foster by the 1890s; the 1892 Hoyle). Disputed until the 1880s–90s. | Default: last of all; period option: last open trick. |
| 10 | Game length | 1000 typical; 1500 or 2000 by agreement (US 1864, Larousse, Dumesnil); Pole 100 units; 500 in the 1847 and Marjolet games; 600 in four-hand French play; double game if the loser is under 500 (Cavendish, Cady, Foster). |  |
| 11 | Ten in the endgame | Some circles revert the ten to whist rank (S3, note 8); not recommended. |  |
| 12 | Dealer selection | Lowest cut deals: US 1864–68, Cavendish 1870, Pardon, the Québec ordinary game. Highest cut deals or chooses: French manuals 1865–67, Cady, Foster, the Rubicon laws. By lot: S49. Players deal in turn: Pole. | Option; a date-based preset. |
| 13 | Penalties | Very variable: 10 to 100 points, 40 (US 1864–68), replay from the error, forfeited tricks; the English Rubicon extra-card penalty (opponent adds 1300 plus up to 900). | An engine that prevents illegal play can skip. |
| 14 | Rubicon settlement | Winner adds both scores + 1000 + 300 for brisques (S3, S4, Foster, the 1892 Hoyle); modern sources vary (+320 in S18, S20; "1000 + sum" in S2). |  |
| 15 | Deal packets | 3-2-3 most common; 3-3-2 (Pole, S46, an alternative in Cavendish); two at a time or two-or-three (French manuals, N&Q 1866); one at a time (Cady, 1892 Hoyle); never singly (S14, Dumesnil). | Cosmetic. |
| 16 | Reuse of declared cards | Three regimes: the class rule (Cady, Foster, Cavendish, Dumesnil: a card may join another class or a superior meld of its own class); Pole 1861 (same type barred, sequence → marriage allowed); US 1864–68 (bezique cards cannot join other melds). Pardon and the Danish booklet bar sequence → marriage. | Expose as options. |
| 17 | Who draws first after a trick | The winner draws first in every English, US and French period text read; Foster reports that in France the pone always draws first. | Cheap fairness toggle. |
| 18 | Hand size and packs, two players | Two packs, eight cards standard; nine or ten cards by agreement (US 1864, Larousse); two or three packs (Bonneveine, US 1864); four or six packs with nine cards (Pardon, Cavendish's reprinted guide). |  |
| 19 | Trick-winning rule before the stock ends | Same suit or trump wins in every period source except N&Q 1866; any higher card wins in N&Q 1866 and as the main rule of the modern Québec page; Cavendish (1870) lists the point as disputed. | A real variant to expose. |
| 20 | Two single beziques before a double | Two single beziques before a double: 40 + 40 + 500 (Cady) versus 40 + 500 (Foster, US 1864, the Cavendish reprint); the Danish booklet bars a double after a single. | Option; the simulation and baseline use Foster's version. |

### 6.1 Positions by source

**Row 1. Endgame: must you trump, and must you win?**

- Strict (follow, win, trump): S2 (Marjolet), S4, Rubicon Law 55 in S3, Larousse "like écarté" (S26), S9. Looser: S3 (standard text: "at liberty to trump"), S20 (may trump but not obliged).
- 1864 (S45, 4th ed.): once the stock is gone, follow suit (the text specifies the highest card of the suit), trump if void, and take the trick if able; revoke = 40 points off. 1868 (S46): same, with a bracketed gloss: take the trick if you can, otherwise play your best card of the suit, otherwise your best trump. Foster 1897 (S47): only the non-leading player is bound: follow suit and win with a higher card or a trump if able; likewise Rubicon (last nine tricks); penalty is a replay from the error (French: nothing scores from that point on).
- Pole 1861 [S48]: second player must follow suit, else trump; "win if able" not stated. N&Q 1866 [S52]: follow suit, else win with a trump if able. Bonneveine 1865 [S49]: follow, else force or trump. Dumesnil 1865 [S50]: follow suit, else trump, else any card. Larousse 1867 [S26]: as at écarté.
- Cavendish Jan 1870 [S55] and the 1892 American Hoyle [S58]: second player must follow suit and win if able, else trump and win if able. Pardon [S56]: a revoke in the last eight tricks forfeits all eight tricks. Québec page [S60]: follow and beat, else trump, else discard; a revoke gives the opponent the last trick.
- Danish booklet [S62]: follow suit or trump once the stock is gone; no duty to win stated.
- *Note:* Dated positions, not a tally. Open [inference]: S45/S46 do not say whether the *leader* is bound; the highest-card wording reads as "best card of the suit if you cannot win," per S46's gloss.

**Row 2. Meld before or after drawing**

- Before: S3, S4, S5, S11, S8 (book version). After: a rule in John Hay's rulebook (S8, a two-player Pinochle source, so an imperfect witness for bezique).
- Foster 1897 (S47): before drawing, immediately after winning the trick; if you draw without announcing, the right lapses until you win another trick. 1864/1868 (S45, S46): a meld requires taking a trick first (rule 21), but the order relative to drawing is not stated for ordinary melds; the last-stock-trick rule (declare, then take the last card) implies before drawing [inference]. The trick winner draws first in both.
- Pole [S48], N&Q 1866 [S52], Dumesnil [S50]: declare after winning a trick and before drawing.
- Cavendish 1870 [S55], Pardon [S56], 1892 Hoyle [S58]: declare after winning and before drawing.
- *Note:* Before is the standard; "after" lets drawn cards complete melds at once. Simulation (§10b Table 4): after-drawing declaring adds about 20% to a bot's meld points, so this row matters for balance. [inference: computed]

**Row 3. Dix timing**

- Standard: exchange or declare only on a trick win, before drawing (S4); on lead (S9); before your first card in the 1847 game (S5); Polish: on the trick containing it (S4); second dix scores when drawn or played (S20). Some sources say the player who trumps the played 7 scores (S10 snippet).
- Foster 1897 (S47): the exchange must be made right after winning a trick and before drawing; it may still be scored if the turn-up is a 7 or an exchange was already made; a dix and a meld cannot be scored on the same trick. 1864/1868 (S45, S46): the exchange is allowed when the turn-up is not a 7; the score table and hint 7 give 10 for each 7 of trumps, whether played or turned up (the player must call for it); no timing rule for the exchange.
- Larousse 1867 [S26]: exchange only after a trick; the editor's opinion is that a 7 scores only when it wins a trick (on the principle that any card scored must be a card laid down). Dumesnil [S50]: the trick winner may score 10 for a held 7 (a 7 may be scored together with another meld, an exception to one-at-a-time) and exchanges it unless already exchanged; by custom, a meld completed by the card obtained at the exchange may be counted at once. N&Q 1866 [S52]: whoever takes the last turn-up card also scores 10. Pole [S48]: 7 scores 10 and can be exchanged.
- Cavendish 1870 [S55]: exchange after winning a trick; if both 7s are in play the second is declared like any card; 10 each. Pardon [S56]: 10 for turning up, exchanging, or playing a 7 of trumps (not in the last eight tricks). 1892 Hoyle [S58]: exchange scores 10; the other 7 is declared by showing it to the opponent only.
- *Note:* Local variation is wide.

**Row 4. One meld per turn vs several**

- Lay several, score one, claim rest later (S3, S4); "local rules" (S9).
- 1864/1868 (S45, S46): only one combination may be declared at a time; declaring all held combinations on one trick win is the habit of some circles (S46 calls it an innovation needing prior agreement). Foster 1897 (S47): several may be announced and shown but only one scores per trick; unscored ones must still be on the table when claimed and are lost if the stock ends first.
- Larousse [S26] and Dumesnil [S50]: one at a time, each after its own trick (Dumesnil excepts the 7 of trumps). N&Q 1866 [S52]: the same announcement can be made only once by each player.
- **Cavendish Jan 1870 [S55]: all unrelated held melds may be declared and scored on one trick (bezique + four aces = 140); melds sharing a card need separate tricks; he says the older rule was one per trick.** Pardon [S56] and the Québec page [S60]: one combination per trick. Chronology is unclear: Pardon's text is undated (1870–76).
- Danish booklet [S62]: one meld at a time.

**Row 5. Four-of-a-kind composition**

- Any suits (S3, S4, S2); one per suit ("Pinochle style" S13; "Panaché" per S16, 1864 Hoyle). S21's 20-point table is wrong.
- Foster 1897 (S47): fours may be of different suits or any two of the same suit; Panaché is not mentioned. 1864 and 1868 (S45, S46): Panaché (one card of each suit per four), described as needing special agreement. [inference] So plain fours are any-suit in those books.
- Bonneveine 1865 [S49]: Panaché appears as a variant needing special agreement, so ordinary fours are any-suit; the odd English sentence in S45/S46 about combinations of the four points reads as a mistranslation of the French wording about uniting the four suits [inference from S49 vs S45].

**Row 6. Plain sequence**

- None in standard (S2, S3); 100 in S20 (unsupported); 250/500 in 1847 game (S5); 150 in Rubicon.
- Foster 1897 (S47): none in the ordinary game; 150 in Rubicon. 1864/1868 (S45, S46): none.

**Row 7. Which cards make the bezique**

- Fixed Q♠+J♦: S2, S3, S4, S9. Trump-dependent: six-pack S13 (cross-mapping); S4 Polish (Q♥+J♣ when spades/diamonds trump); S27 (varies with trump). Current Wikipedia (S9, via Wikiwand mirror https://www.wikiwand.com/en/articles/Bezique) says a variant uses **Q♣+J♥** when spades or diamonds are trumps, whereas Cady (S4) says Q♥+J♣.
- 1864, 1868, Foster (S45, S46, S47): fixed Q♠+J♦.
- Pole, Bonneveine, Dumesnil, Larousse, N&Q 1866, Nouveau Larousse [S48, S49, S50, S26, S52, S54]: fixed Q♠+J♦.
- **Trump-dependent in the ordinary game too:** Cavendish 1870 [S55], Pardon [S56], Cavendish as reprinted in Phillips/Fox [S57]: Q♠+J♦ unless spades or diamonds are trumps, then Q♣+J♥; Polish: Q♥+J♣ [S57, S4]. Québec page [S60]: full four-way mapping in six-pack, fixed Q♠+J♦ as a common option.
- *Note:* English period practice splits between Cavendish and Pardon (trump-dependent) and Hoffmann, Foster, Cady and the 1860s texts (fixed). Cady's ordinary game was not re-checked for the alternate pair. [inference: expose as an option]

**Row 8. Brisque timing**

- As taken (S3 footnote "the better rule," S4 "as a rule"); at end (S2); French thresholds (S10); after 400 (S5).
- 1864/1868 (S45, S46): counted at the end of the hand; not counted at all if a player has already reached game through melds; if neither has, the higher total over the target wins, ties to the taker of the last trick. Foster 1897 (S47): scored at once when taken (20 for two in a trick); end-counting is reported and discouraged; in Rubicon brisques count only to break a near-tie or save a rubicon.
- **Every source from 1861–67 counts brisques at the end** (Pole [S48], Bonneveine [S49], Dumesnil [S50], Larousse [S26, explicit: not as made; only the last-trick 10 is scored at once], N&Q 1866 [S52]); Foster (1897) and Cady prefer as-taken. Larousse [S26] also adds "figures" (court cards) to the final count; the scale is not given in that entry [S26]. Dumesnil/Bonneveine: whoever reaches the target by melds first wins even if the other leads on brisques; if both need brisques, the higher total wins, ties to the last trick [S49, S50].
- Cavendish 1870 [S55]: as won (his preferred method). 1892 Hoyle [S58]: as won by default, end-counting as an option, a tie after end-counting is void. Pardon [S56]: sometimes at the end.
- Danish booklet [S62]: counted after the last trick; a player who reaches the target by melds wins first; otherwise the taker of the last trick counts first and wins on reaching the target even if the other would count more.

**Row 9. "Last trick" bonus**

- Absolute last trick (S4, citing club ruling; S3 says 32nd). S9 table ambiguous.
- Foster (S47): the winner of the last trick of all scores it (10 ordinary, 50 Rubicon); he reports the absolute-versus-last-before-stock question as an early dispute now settled. 1864/1868 (S45, S46): 10 points; it also breaks a tie in the brisque count.
- Pole, Bonneveine, Dumesnil, Larousse: 10 [S48, S49, S50, S26]; a convention may vary it [S49].
- **Cavendish 1870 [S55]: the "last trick" (10) is the last *open* trick (the one made when the trump and one stock card remain), not the last of all; the 1892 Hoyle [S58] says the winner of the last of the eight tricks scores 10; Pardon [S56] (and the Cavendish reprint [S57]) call the 32nd-trick reading a mistake, i.e. side with Cavendish 1870.** Foster reports this as an early dispute, Cavendish having got it wrong in his first edition [S47]. So row 9 is "resolved" only from the 1890s.
- *Note:* Resolved only from the 1890s.

**Row 10. Game length**

- 1000 (most); 500 (1847, Marjolet); 1500; 2000; 600 four-player (S10); tie at 1000 raises to 1500 (S13). Cady: doubles if 1000 reached before opponent has 500.
- 1864 (S45): agreed beforehand: 1,000, 1,500, 2,000 or more. 1868 (S46): usually 1,000. Foster 1897 (S47): 1,000 (2,000 in three- and four-hand); a double game if the loser is under 500.
- Pole 1861: 100 units, or 1,000 optionally [S48]; Larousse: 1,000 or 1,500, later 1,500 or 2,000; one-pack game 500 [S26]; Dumesnil: 1,500 [S50]; N&Q 1866: 1,000, further hands if nobody reaches it [S52].
- Cavendish 1870 [S55] and 1892 Hoyle [S58]: 1000 up, double if the loser has under 500, match = best of 3 of 5. Québec page [S60]: 1500 up (1000 or 2000 optional).

**Row 11. Ten in the endgame**

- Some circles revert it to whist rank (S3 note 8), not recommended.

**Row 12. Dealer selection**

- Highest cut deals (S3, S9, as logged; Cady gives the highest cut the choice of seats and markers, S4); lowest (S13).
- 1864 and 1868 (S45 rule 2, S46 law 2): the lowest cut deals. Foster 1897 (S47): the highest cut has first choice of seat and decides whether to deal himself; he records highest/lowest as an early dispute. Cady 1896 (S4, re-read): the highest cut has the choice of seats and markers in the two-pack game and the choice of the deal in Rubicon (so "highest deals" in the S3/S4 entry above is a simplification).
- French: highest cut deals [S26, S50] or by lot [S49]; Pole: deal in turn [S48]. The lowest-cut-deals rule appears in the 1860s US texts (and Foster reports highest/lowest as an early English dispute); the French 1865 manuals read here do not state it [S49, S50, S26].
- Cavendish 1870 [S55]: the lowest cut deals. Pardon [S56]: lowest cut wins the deal. 1892 Hoyle Rubicon law [S58]: higher cut has choice of deal, seats and markers. Québec page [S60]: lowest draws first deal in the ordinary game; highest chooses in the six-pack game. Foster: Cavendish erred in his first Pocket Guide [S47]. Lowest-deals is therefore the early English (Cavendish 1870, Pardon) and US (1864–68) practice, revised by the 1880s–1890s (Rubicon Law, Foster, Cady).
- *Note:* S13's "lowest" is not an outlier: it matches the 1860s US texts and Cavendish/Pardon. [inference] Probably an early practice.

**Row 13. Penalties**

- S20: >8 cards = 100; failing to draw = 10; revoke = lose endgame. Portland laws in S3, S4. Russian scale (S18).
- 1864/1868 (S45, S46): revoke = 40 points off; misdeal: hand passes or re-deal at the opponent's option; forgetting to draw: lose the play or later draw two; a card too many: void at the opponent's choice, or play twice without drawing; announcing four with only three cards: take the score down and play one of the three. Foster 1897 (S47): similar rules, plus the English Rubicon extra-card penalty (game abandoned; opponent adds 1300 plus the offender's points, the latter capped at 900; numbers as printed, not cross-checked against S3) against French leniency; Foster complains that English laws presume the opponent a rogue and are absurdly severe.
- Athenaeum 1869 on Ware's handbook [S59]: more than eight cards costs 500, or the opponent may add 500. Pardon [S56]: forfeits of 10/10/10/100 and eight tricks for a last-eight revoke (see §3.2). 1892 Hoyle Rubicon laws [S58]: more than nine cards = rubiconed (opponent adds up to 900 plus 1300). Québec page [S60]: erroneous four-of-a-kind announcement: take down the score; a misdeal or exposed-card rules; a revoke after the stock gives the last trick.
- *Note:* An engine that enforces rules can skip.

**Row 14. Rubicon settlement**

- Winner adds both scores + 1000 + 300 (S3, S4); "+320" (S20, S18); "+1000 + sum" (S2).
- Foster 1897 [S47] and 1892 Hoyle [S58] agree: winner adds both scores + 1000 + 300 for brisques; loser saved if brisques lift him to 1000; +100 if the loser scored under 100; game value = difference + 500 otherwise. Québec page [S60]: same for Rubicon (500 bonus, 1000 if rubiconed).

**Row 15. Deal packets**

- 3-2-3 (S3, S4, S9); 2-3-2 (S18); French: two or three at a time, never one (S14).
- 1864 (S45): two at a time, or 3 then 2, until eight. 1868 (S46): 3, 3, 2. Foster 1897 (S47): 3-2-3 (Rubicon 3-3-3). Cady (S4): 3-2-3, or one at a time.
- Pole: 3-3-2 [S48]. Larousse: two by two or three by three [S26]. Dumesnil: 2 and 3, or 4 and 2, never singly [S50]. Bonneveine: by two, or three and two [S49]. N&Q 1866: two at a time [S52].
- Cavendish (Phillips/Fox reprint) [S57]: three, then two, then three to each player, alternately (3-2-3), or sometimes 3-3-2; 1892 Hoyle [S58]: one at a time or 3-2-3; Québec page [S60]: 3-2-3.

**Row 16. Reuse of declared cards**

- **Class rule** (marriage+sequence / bezique / fours; different class free, same class only upward): S4 (1896), S47 (1897). So fours ↔ marriage allowed, sequence → marriage barred, bezique cards may later join marriage, sequence, fours. **1864 (S45), main text (French practice):** a king in a marriage may join four kings and the trump sequence; a trump ace from four aces may join the sequence; after a bezique, Q♠+J♦ may join a sequence and Q♠ a marriage. **1864 footnote (US practice):** a married K+Q cannot marry again but may join fours, the sequence or a bezique; any card except one used in a bezique may join any meld it has not been in. **1868 (S46):** the US practice is now the main text, and it explicitly lets sequence or fours cards form a marriage or bezique; bezique cards are barred unless agreed beforehand (French practice bracketed).
- Pole 1861 [S48]: different class allowed, including sequence → marriage; same class barred. Dumesnil [S50]: same-class barred; a king from a marriage may join four kings. Bonneveine [S49]: same as the S45 main text. Nouveau Larousse "chinois" [S54]: like combinations may re-use a card freely.
- Pardon [S56]: K+Q used in a sequence cannot afterwards be declared a royal marriage; four knaves may be followed by J♦ in a bezique or sequence; four aces by the trump ace in a sequence; a married Q♠ may join bezique and four queens. Cavendish 1870 [S55]: class rule; a bezique queen may be declared in bezique, marriage and four queens; a card cannot be declared twice in one class; a meld whose cards are all already on the table is not declarable. Québec page [S60]: marriage → sequence allowed; sequence → marriage barred; a four can be re-formed only by breaking it first.
- Danish booklet [S62]: like-class barred; a single bezique's cards cannot make a double; cards on the table cannot alone form a new meld (a third practice, stricter than the class rule).
- *Note:* S45 holds both practices; S46 reverses which is main. Sequence → marriage: barred in S4/S47, **allowed** in S46. Bezique cards reusable: yes in S4, S47, S45 main text; no in S45 footnote and S46.

**Row 17. Who draws first after a trick**

- Winner first: S45, S46, S4. Foster 1897 (S47): the winner first under the English laws; in France the pone always draws first, which Foster says guards against marked cards revealing the next draw to the winner.
- Winner draws first: Pole [S48], Larousse [S26], Dumesnil [S50]; N&Q 1866 [S52] says each draws after every trick.
- Pardon [S56] and 1892 Hoyle [S58]: the winner draws the top card first.
- *Note:* Engine option; [inference] a cheap fairness toggle.

**Row 18. Hand size and packs, two players**

- Two packs and eight cards standard. 1864/1868: nine or ten cards by agreement (turn-up = 19th or 21st card) [S45]; two or three packs [S45, S46]. Cady: a three-pack version as a variety [S4]; nine cards only in Rubicon [S4, S47].
- Bonneveine 1865 [S49]: two or three packs; eight cards almost always. Larousse 1867 [S26]: eight or nine (17th or 19th card is trump). Nouveau Larousse [S54]: nine cards ordinarily, two packs; three-hand with three packs. Dumesnil [S50]: two packs, three-hand removes one Eight.
- Pardon [S56]: two packs; four or six packs with nine cards; three-hand with three packs or two packs less an Eight; four-hand partners. Cavendish reprint [S57]: four or six packs, nine cards, 2000 up, triple bezique.

**Row 19. Trick-winning rule before the stock ends**

- Same suit or trump: Pole, Bonneveine, Dumesnil, Larousse, Cavendish, Cady, Foster and the 1860s US texts [S48, S49, S50, S26, S55, S4, S47, S45]. Higher card of any suit (suit irrelevant): N&Q 1866 [S52]; the modern Québec compilation as its main rule [S60]; Cavendish lists the point as disputed in 1870 [S55].
- *Note:* A real variant to expose, not just an old dispute.

**Row 20. Two single beziques before a double**

- Cady (1896): two single beziques may each score 40 and then the double 500 (40 + 40 + 500); adding an undeclared single to a declared one scores 40 + 500; declared all at once, 500 only [S4].
- Foster (1897): the second single cannot score 40; adding the second pair to a declared single scores the double, so 40 + 500; the double needs at least one new card [S47].
- US 1864 (rules 22–23): the double counts 500 in addition to the 40 already scored [S45]. Cavendish as reprinted: the 500 is in addition to the 40 if a single was scored [S57, S58].
- Danish booklet: after a single, its cards cannot be used for a double [S62] (see row 16). Hoffmann (S3) not re-checked for this point.

---

## 7. Strategy compendium

**Hoffmann's hints** [S3]
- Early tricks matter only for brisques or to declare. Pick rival melds by value and prospects; watch the opponent's declarations.
- With Q♠ and two J♦, Double Bezique dies if the opponent declares a spade marriage. The opponent should keep the Q♠ as long as possible.
- When a brisque is led, win it if it is not too costly: capturing a brisque is a 20-point swing.
- Otherwise throw sevens, eights, nines and plain jacks; risk an ace before a king or queen; hold three aces hoping for the fourth.
- Hold sequence and bezique cards; late in the hand, consider declaring Double Bezique or the sequence directly, since declaring the smaller meld and not winning another trick loses the larger.
- Near the end take tricks to deny the opponent's remaining declarations; note his exposed cards; reserve a good trump for the last trick.

**Cady's hints** [S4] (also drawing on Cavendish and Berkeley)
- Look for carte blanche first; lead your lowest card; discard worthless cards unless you have a reason to win.
- Before trumps are made, discard small cards, preferably from "unfashionable" suits (hearts, clubs) since the bezique cards are spades and diamonds.
- Play declared cards to show the opponent less; be careful about making trumps early.
- Watch the opponent's discards to infer his suit; if three of a sequence or bezique card are known and you hold the fourth, hide it: the opponent will sacrifice other melds chasing it.
- Don't lead a ten if avoidable; keep it to win tricks.
- If you can't save the rubicon, score as little as possible. If the opponent is under the rubicon, it may suit you to lose the last trick, because whoever wins it must score it.
- Last nine tricks: remember trumps and bezique cards; lead a high plain-suit card to force a trump; if reduced to trumps, lead your lowest.
- Shuffle well before a second game, since the last cards of a hand tend to be trumps and bezique cards.
- Polish section hints: don't be too anxious to take tricks (leading is a disadvantage); don't lead small cards of the bezique suits; lead from a dead suit; keep aces of the bezique suits for the endgame.

**Others**
- Pagat: brisques versus melds is the central tension; keep flexible; no mistake is worse than chasing a card that's gone. [S2]
- h2g2: hide that you hold the card the opponent needs; if he shows Bezique with one J♦ and Four Jacks with another, he'll keep both jacks and the queen chasing Double Bezique; endgame: lead aces and tens if you're sure he holds a lower card of the suit; lead trumps to draw his. [S20]
- CardGameHeaven: don't rush to declare; play from the table; discard low; a brisque is worth as much as a marriage. [S21]
- Russian advice (S18): lead low, win high; keep queens and kings for marriages; guard the Q♠; play four aces at once so you don't lose the tricks.
- Old French dictionary advice: shed low cards, take only with cards that count. [S11]
- card-games.ca (D): contradictory advice on declaring marriage before sequence; not reliable. [S22]
- **1864 and 1868 US Hoyles** (identical maxims) [S45, S46]: note when the opponent shows enough bezique or sequence cards that your own chances are dead, and release those cards; do not throw away bezique or sequence cards while a real chance remains; avoid showing cards that tell the opponent his double bezique or sequence is impossible; keep kings and queens to marry rather than aces if you must choose (win the trick with the ace, so you count it, and save the king); it is very hard to declare four aces and keep them all, so take aces and tens as brisques when you can (each ace or ten you let the opponent take is a 20-point swing); exchange the 7 of trumps, especially for a bezique or sequence card; keep aces and tens of trumps for the last eight tricks and win the trick before the stock runs out to get the lead and the last declaration; near the end, study the opponent's exposed cards. US practice (no reuse of bezique cards) leads to a further maxim: keep Q♠ and J♦ in hand so they can join other melds first, particularly when spades or diamonds are trumps. [S46 maxim 6; S45 bracket to rule 25]
- **Foster 1897** (hints under Binocle, pp. 366–368) [S47]: giving the opponent the deal is usually best; the lead is a disadvantage unless you can declare, a brisque is in the trick, or you can bring home a plain-suit ten; lead small cards when nothing matters; aces are better leads than kings or queens (court cards can marry), kings beat queens (especially spade queens), jacks are safer still but keep the J♦; with duplicates, play the one already on the table; pick one or two suits or melds and discard the rest; read the opponent's discards (in bezique, three kings shown means he has no fourth to hold); it is seldom right to show bezique cards in other melds, and four jacks is a poor meld because it tells the opponent double bezique is dead; keep a small trump to regain the lead; do not trump in for minor melds unless time is short; seldom lead the trump ace except at the end or with duplicates; study the exposed cards before the last trick. Rubicon: declare the trump sequence before a double bezique (so a sequence card is free to win the lead); with a plain sequence and four aces declare the aces first; when the opponent is already rubiconed it may pay to give him the 50-point last trick if that lifts him over a hundred (+100 to you). Foster also lists cheating risks (marked cards, seconds dealing) and the French rule that the pone always draws first as the counter.
- **Pole 1861** [S48] (his own seven maxims; note the overlap with the hints in S45/S46, which are 1864–68 US copies of a French set, so the direction of influence is unclear [inference]): keep sequence and bazique cards while any chance remains; note when the opponent shows two cards of a kind that kill your sequence or bazique; take aces and tens (each you lose is 20 against you); prefer to keep kings rather than aces if you must choose; do not tell the opponent his sequence or double is dead; be careful in the last eight tricks; keep your temper.
- **Dumesnil 1865** [S50]: keep Q♠ and J♦; play a run of like cards to force the opponent to shed trumps and counters; hold trumps to declare and to build the trump quint; usual to shed low cards and win with counting cards such as plain-suit tens (later maxims not read).
- **Pardon** [S56] (c. 1870–76; fourteen hints): the lead is no advantage unless you can declare; the cards to part with are sevens, eights and nines, then knaves; lead a ten or ace rather than a king or queen; go for four aces only with three in hand and no other difficulty; if a led ace or ten is not taken, lead another; keep small trumps and trump-sequence cards; keep bezique cards until near the end, even after declaring bezique; when choosing, play a declared card so as not to expose the hand; do not show the opponent that his sequence or double is dead; take a plain-suit ten-lead with your own ten; win the last trick; in the last eight tricks save your aces and tens and win his. [S56]
- **No odds found:** neither Foster's bezique chapters nor his Binocle hints contain probability or odds material (searched the range for probability/odds wording); the 1864 book's appended chances treatise has no bezique entry (its only bezique hits are the chapter itself and the index). [S45, S47] Odds for double bezique etc. still have to be computed for the engine [inference].

**[inference] AI implications**
- Imperfect information with heavy card-counting value: eight copies of each rank, four Q♠ and four J♦ tracked.
- Key decision points: meld timing (marriage first or hold for sequence), when to chase Double Bezique, how to spend trumps before the stock runs out, when to deny the opponent's tricks late in the stock.
- The endgame is close to perfect information once the stock is gone.

---

## 8. Terminology and voice

- **English:** brisque, dix (older "deece"), marriage (royal/common), sequence, bezique, carte blanche, rubiconed, "Forty, and twenty to score" (Cady), the older "Goodall Register" (markers). [S4, S6, S13]
- **French:** bésigue, retourne, talon, levée, atout, quinte majeure, carré, mariage, en bas table, "marque" (marker, Haiti), "quarante de bésigue," "tailler un bésigue," "annoncer le 250." [S10, S14, S33]
- **Turkish:** marköz (scoring frame), tablo (sequence), vido (double), şuvet (chouette), brisk. Calling the number aloud when laying a meld is part of the Turkish rules. [S19]
- **Russian:** безик, марьяж, козырь, бескозырный безик. [S18]
- **German/Binokel:** Binokel/Binocle; "bin oculi" folk etymology. [S31]
- **1864–1897 usage:** *talon* and *eldest hand* (S45, S46); *stock or talon*, *dealer*, *pone* (S47); *quint-major* = sequence; *brisques*; players *declare* (S45/46) or *announce* (S47) and *score*; players *call for* the ten for a played 7 of trumps (S45 hint 7); *carte blanche*; *rubiconed*; *penchant* (Foster's game); Foster's hints use the word *dix*, so it was in English use by 1897, while S45/S46 say only Seven of trumps. Foster gives the same announcement formula as Cady (forty for bezique, twenty to score). The 1864/1868 books name fours by their value (a hundred of aces, an eighty of kings, and so on). [S45, S46, S47, S4]
- **French period terms:** *retourne* (turn-up), *talon*, *levée*, *brisque*, *besigue*, *quinte majeure*, *bési/bezy* (Larousse variants), patois *Besit/Bési/Bézy* [S53], *bésigue sans retourne*, *bésigue panaché*, *bésigue arrêté au point*, *bésigue chinois* [S54]; Larousse calls bésigue a game of calculation and states, as a general rule, that any card scored must be a card laid down. [S26, S49, S50, S53, S54]
- **More terms:** *porte arrière* (a plain-suit quint at 150 in Rubicon; the English "backdoor" in S39 is presumably this, [inference]), *trifouille* (three-hand), *dans la boîte* (the chouette player) [S60, S14]; *dix* used in English hints [S47]; "last open trick" (Cavendish 1870) [S55]; *rubiconed*, *carte blanche* [S47, S58]; *treble bezique* (three-hand pack with a pack per player) [S56].

---

## 9. Material culture and visual reference

- Scoring devices: leather-covered and wooden pointer markers with dials to 1000, later card markers; Goodall's marker "Register" designs, De La Rue's two-circle 0–90 / 0–900 dials; Hall's article documents many types and dates (registered designs from 1869; Goodall three-circle marker). [S6]
- Early cards were square-cut with no corner indices, until about 1870. [S6]
- Boxed sets: red boxes, "The Royal Game of Bezique" (Goodall), Reynolds's "Besique" spelling, Willis "Patent Self-scoring markers." [S6, S28]
- A Goodall set of c. 1868–69 held two bezique packs (courts unturned), two markers (pink and green, matching the pack backs) and a second-edition booklet; Goodall's bezique booklet was later reissued as "Camden," *The Royal Game of Bezique*, 11 pp in 1883 (S51), and the two-pack Goodall set is dated "1868 or thereabouts" by a collector [S61, S51].
- Turkish marköz: a peg frame with spring pegs; Haitian "marque"; French "marqueur." [S10, S19]
- Caillebotte, *La Partie de bésigue* (1880). [S10, S42]
- Copyright status of images is **not checked** here.

---

## 10. Digital implementations and academic literature

### 10.1 Digital implementations

- Glowing Eye Games iOS "Bezique!" (tutorial, three difficulty levels, backgrounds); William Hale iOS "Bezique" (learning mode, rated games); DiD Windows game on itch.io (champions table, online scores). [S36]
- Pagat lists Cardmaster (four-pack Rubicon program); h2g2 lists Card Parade. [S1, S20]
- Forum thread attempting double-bezique odds: concluded too many unknowns. [S35]
- No open-source engine found in one GitHub search (weak evidence).

### 10.2 Academic and scholarly literature

Searches (web, arXiv-style keyword, and archive.org full text) found **no bezique-specific paper** on AI, solving, probability or game theory, and no journal article on its history. What exists is adjacent:

| Work | Relevance | Status |
|---|---|---|
| Gorman, Kunkel & Vasko, "Using Pinochle to motivate the restricted combinations with repetitions problem," *IJMEST* 42(5), 2011 [S65] | Peer-reviewed combinatorics on counting distinguishable hands from decks with duplicate cards (two copies per type in a single Pinochle deck, four in a double deck): the same structure as the 64-card bezique pack. Applied in §10b Table 5. Pinochle is bezique's American descendant | abstract read |
| "From Imitation to Interaction: Mastering Game of Schnapsen with Shallow Reinforcement Learning," arXiv:2605.17162 [S66] | Schnapsen (Sixty-Six) is the nearest well-studied relative (marriage melds, two-phase play with a stock, strict endgame). Its Schnapsen Game Engine and sampling-based RdeepBot are a template for a bezique bot | snippet read |
| Trick-taking AI citations: Sturtevant 2009; Buro et al.; Whitehouse et al. (ISMCTS); Koller et al. (Jass); Furtak 2013 (Skat null endgames) [S67] | Methods for imperfect-information trick-taking games: sampling-based search, information-set MCTS, endgame solving. The bezique endgame (last eight tricks, stock gone) is near-perfect information, so retrograde or exact search is plausible [inference] | citations only |
| Parlett, *A History of Card Games* (1990/91), *Oxford Dictionary of Card Games* (1996, p. 16) and *Penguin Book of Card Games* (2008, pp. 287, 293); Depaulis in *The Playing-Card* | The standard games-history scholarship; Parlett's pages on bezique are cited by S9, and no Depaulis article specifically on bezique was found | not read (books; §13) |
| Anton, *Encyklopädie der Spiele*, 3rd ed. (Leipzig, 1879, p. 43); *Transactions of the Philological Society* (1910, p. 289); Brownsmith, "Bézique," *Westminster Papers* (1869, pp. 117–119) | Period scholarship on the game's origin, etymology and English arrival, all cited by S9 | not located in the archive.org scans searched (§13) |

No published statistics on bezique (meld frequencies, game length, first-mover edge) were found, so the figures in §10b are original computations.

---

## 10b. Stats and odds (computed, not sourced)

*Everything in this section is **computed** ([inference: computed]); none of it is cited to a source, except where a source is named for a qualitative cross-check. Script: `bezique_stats.py` (delivered with this file; seeded, reproducible). Method: a player is dealt N cards and draws once per stock trick, so by the end of the drawing phase he has received exactly half the pack, and the set he received is a uniformly random half. The odds below are the exact probabilities (hypergeometric counts) that a given player **receives** the cards a meld needs at some point during the drawing phase. They are **upper bounds** for actually scoring the meld: late cards cannot be declared, cards may be played away or captured, and a trick must be won to declare. Bezique is taken as Q♠+J♦ (any trump-dependent pair has the same probability for its own cards). Cross-check: a 200,000-trial seeded simulation matches the exact figures within sampling noise.*

**Table 1: structure of a hand, by variant**

| Variant | Cards | Dealt each | Turn-up | Stock tricks | Endgame tricks | Total tricks | Brisques (A+10) | Brisque points |
|---|---|---|---|---|---|---|---|---|
| Two-pack | 64 | 8 | yes | 24 | 8 | 32 | 16 | 160 |
| Four-pack (Rubicon) | 128 | 9 | no | 55 | 9 | 64 | 32 | 320 |
| Six-pack (Chinese) | 192 | 12 | no | 84 | 12 | 96 | 48 | 480 |

Cross-check with sources: Rubicon's 32 brisques = 320 matches S3/S4/S47; "last eight" and "last nine" tricks match S4/S47/S58. The "last open trick" of Cavendish 1870 [S55] is trick 24 of 32 in the two-pack game (§6 row 9).

**Table 2: chance a player receives the needed cards over the drawing phase**

| Meld | Two-pack (32 of 64) | Four-pack (64 of 128) | Six-pack (96 of 192) |
|---|---|---|---|
| Single bezique (one Q♠ and one J♦) | 56.5% | 88.4% | 97.1% |
| Double bezique | 5.66% | 47.2% | 79.9% |
| Triple bezique | – | 9.13% | 42.9% |
| Quadruple bezique | – | 0.31% | 11.2% |
| Quintuple bezique | – | – | 1.01% |
| Sextuple bezique | – | – | 0.017% |
| Marriage in a named suit | 56.5% | – | – |
| Four aces (any suits); the same for kings, queens, knaves | 64.6% | – | – |
| Trump sequence (A, 10, K, Q, J of the trump suit) | 22.6% | – | – |
| At least one 7 of trumps / both 7s of trumps | 75.4% / 24.6% | – | – |

Qualitative cross-check: Foster says that in Rubicon double combinations are much more frequent and triples not uncommon [S47]; the figures above (47% and 9%) fit. The rarity of quadruple (0.3%) and quintuple (1%) matches how seldom sources dwell on them.

**Table 3: carte blanche**

| Variant | P(no K, Q or J in the deal) | Award | Expected extra re-scores if dealt | Expected carte-blanche points per hand |
|---|---|---|---|---|
| Four-pack, 9 cards | 1.22% | 50 | 1.45 | 1.49 |
| Six-pack, 12 cards | 0.286% | 250 | 1.48 | 1.77 |

The expected extra re-scores follow the rule that a carte blanche is scored again after each draw until a K, Q or J arrives (S47, S58, S60); draws are modelled without replacement and only the holder's draws are counted, which slightly overstates a running pool. (The two-pack game has no carte blanche score; for reference, 8 cards with no court card occurs 1.74% of the time.)

**Table 4: self-play simulation of the two-pack game (computed; `bezique_sim.py`, seed 20260930)**

*Method: a heuristic bot plays 40,000 hands (80,000 player-hands) against itself. Card play does not depend on which melds have been declared and dix exchanges are not modelled, so each recorded hand is re-scored under several rule sets: the rule effects are measured on **identical hands**. The bot is crude (it always leads its lowest-value card and takes cheap tricks); the figures describe this bot, and the differences between rule sets are the useful output. Sanity checks that pass: brisque points always total exactly 160 per hand; each player wins 12.00 of the 24 stock tricks on average; bezique cards received (56.4%) match the exact 56.5%; double bezique received is 5.85% against an exact 5.66% (about 2.3 standard errors high, probably chance; the independent 200,000-trial check in `bezique_stats.py` gives 5.66%).*

| Rule set (row of §6) | Mean meld points per player-hand | Median | 90th pct | 99th pct | vs baseline |
|---|---|---|---|---|---|
| Baseline: one meld per trick, declare before drawing, class reuse (rows 2, 4, 16) | 133.0 | 100 | 290 | 640 | – |
| All disjoint melds per trick (Cavendish 1870; row 4) | 133.6 | 100 | 290 | 640 | +0.4% |
| Declare **after** drawing (row 2) | 159.1 | 120 | 350 | 660 | **+19.6%** |
| Reuse per Pole 1861 (marriage after sequence allowed; row 16) | 134.4 | 100 | 310 | 640 | +1.1% |
| Reuse per US 1864 (bezique cards cannot join other melds; row 16) | 126.7 | 100 | 290 | 600 | −4.7% |

- **Where a hand's points come from (baseline, both players together):** melds 266, brisques 160, last trick 10, dix 5.6 (total about 442 per hand). So melds are about 60% of all points and brisques about 36%.
- **Received versus declared (baseline, per player-hand):** single or double bezique received 56.4%, declared 44.1% (78% conversion); double bezique received 5.85%, declared 3.32% (57%); trump sequence received (exact, Table 2) 22.6%, declared 9.6% (42%). The low sequence conversion reflects the bot's poor sequence-building, so treat it as a floor.
- **Brisque timing and game outcome (row 8):** playing 8,000 games to 1000 on identical hand streams, scoring brisques as won versus withholding them to the end of the hand (the 1861–67 practice, S45/S49) changes the *winner* in **3.7%** of games. The number of hands per game is identical by construction (either rule ends the game in the hand where someone first reaches 1000), mean 4.45 hands. [inference: computed]
- **First-mover fairness:** the player who leads hand 1 wins 50.2% of games, so no visible first-mover edge at 1000 up with alternating dealers.
- **Reading the rule effects:** letting a player declare all disjoint melds on one trick is almost irrelevant for this bot (0.4%), because pending melds simply wait for the next won trick; **declaring after drawing is the big one (about a fifth more meld points)** because the freshly drawn card can complete a meld at once and the last stock trick gets an extra card; the 1864 US no-reuse rule costs about 5%. No period bezique source supports declaring after drawing (it appears only in a Pinochle rulebook, S8), so it should be an explicit option, not a default. [inference: computed]

**Table 5: distinguishable dealt hands (duplicate cards)**

*Because each card type has several identical copies, many labelled hands look the same. The count of distinguishable hands is the coefficient of x^k in (1 + x + … + x^m)^32 (m copies per type, hand size k), the "restricted combinations with repetitions" problem treated for Pinochle decks by Gorman, Kunkel and Vasko [S65]. Computed exactly (`bezique_stats.py`); the formula is checked against a small brute-force case.*

| Variant | Labelled k-card hands | Distinguishable hands | P(no two identical cards in the hand) | Expected identical pairs in a hand |
|---|---|---|---|---|
| Two-pack, 8 cards | 4,426,165,368 | 49,721,892 | 60.8% | 0.44 |
| Four-pack, 9 cards | 1.906 × 10^13 | 271,763,360 | 38.6% | 0.85 |
| Six-pack, 12 cards | 3.689 × 10^18 | 15,326,614,520 | 13.3% | 1.73 |

- A two-pack player is dealt at least one duplicated card about 39% of the time, so duplicate-tracking (four Q♠ and four J♦ in the pack, two of each other card) matters from the first trick. The expected number of identical pairs in an 8-card hand is exactly 28/63. [inference: computed]
- These counts are also the size of the space a hand-evaluation table or opening book would have to cover: about 5 × 10^7 hands for the two-pack game. [inference]

**Observations for the engine [inference]**
- In the two-pack game a **double bezique is at most a 1-in-18 hand per player** (5.66%), so the 500 is a rare swing; four aces and a marriage are the routine scores. This is an upper bound: a forum attempt to price it (S35) stopped at "too many unknowns," and the true figure depends on play.
- In Rubicon and six-pack, doubles are the norm, so game balance rests on triple and quadruple frequencies (9%/0.3% and 43%/11%), not on the doubles.
- Carte blanche adds only about 1.5–1.8 points per hand on average despite its headline value; it matters as a rare event, not for the mean.
- **Configuration space:** the disputed points in §6 give at least twelve independent options (dealer rule; brisque timing; meaning of the last trick; one or several melds per trick; fixed or trump-dependent bezique pair; reuse of bezique cards; trick winner by suit or by rank; dix scoring; strictness of the endgame; target; hand size; number of packs). Treat rule sets as named presets (e.g. Pagat baseline, Cavendish 1870, Rubicon 1887, six-pack) rather than hard-coding one.

**Source inventory (computed from the registry in §2)**

| Tier (first letter of the tier cell) | Count | Status (first word of the status cell) | Count |
|---|---|---|---|
| A (A, A-) | 28 | FULL | 39 |
| B (B+, B) | 18 | EXCERPT | 27 |
| C (C+, C, C-) | 18 | CATALOGUE / POINTER | 2 |
| D | 1 | | |
| n/a | 3 | | |

The counts cover S1–S68. S26 and S7 are counted by the first word of their status cell. Blocked or unreached items are listed at the end of §2 and in §13, not counted. [inference: computed by script from the file]

---

## 11. Corrections log

Corrections to earlier statements in this file, and tensions in the sources that remain unresolved. Ordered by topic, not by date of discovery.

**Corrected**

1. **Stationers' Hall race:** 1869 (not 1868); 1868 is the re-launch. [S6, S40]
2. **Six-pack / Chinese dates:** the "1899" in S16 is unsupported. Foster (1897) already defines Chinese Bezique as six-pack Rubicon [S47]; the name appears in Daudet (1879) [S10, S33]; six-pack play is mentioned c. 1870s [S56, S57]; the first codified laws are Jessel 1921 [S7]. "Chinese" also names a different game (free re-use of cards) in the Nouveau Larousse c. 1900 [S54], and Jessel (1903) treats the label as fanciful [S53]. See §3.3.
3. **Polish Bezique first date:** 1870 (*Feldinski*, Willis; also a British Library record) versus 1873 (Cavendish's guide and De La Rue's card); both are real items. [S51, S7, S17]
4. **Churchill's 1896 letter** concerns Rubicon cards, not six-pack. [S16]
5. **Reuse of cards (§4):** an earlier text said four kings → marriage does not work. Cady and Foster classify melds; fours and marriages are different classes, so it does work. S3 was not re-read for this point. [S4, S47]
6. **Deal (§4, row 12):** "lowest cut deals" (S13) was called an outlier. It is the rule in the 1864–68 US Hoyles, Cavendish 1870 and Pardon; the French manuals do not state it; Cady gives the highest cut the *choice* of seats and markers (and, in Rubicon, of the deal); Foster reports the point as an early dispute. [S13, S45, S46, S55, S56, S49, S50, S26, S4, S47]
7. **Row 1 wording:** "Majority strict" was replaced by dated positions; conflicts are reported by source, date and edition, not by counting sources.
8. **Pole (1861):** the article spells the game "Bazique" (not "Basique") and counts to 100 units (1,000 optional). [S48, S53]
9. **Bezique pair (§5, row 7):** the alternate pair was treated as a six-pack or Polish feature and as disputed between Wikipedia (Q♣+J♥) and Cady (Q♥+J♣). Cavendish applies Q♣+J♥ to the *ordinary* game and Q♥+J♣ to *Polish*, reconciling the two; Pardon agrees with Cavendish for the ordinary game. [S55, S56, S57, S4, S9]
10. **Last trick (row 9):** "resolved: absolute last trick" holds only from the 1890s; Cavendish (1870) used the last open trick. [S55, S56, S58, S47]
11. **Several melds per trick (row 4, §12):** "several may be laid down, one scored per trick" is the late majority practice, not the only English one: Cavendish (1870) reports scoring all unrelated held melds on one trick. [S55]
12. **Two single beziques before a double (§4, row 20):** the earlier text gave Cady's 40 + 40 + 500 as consensus; Foster (1897) and the 1864 US text allow only 40 + 500. [S4, S47, S45]

**Unresolved tensions**

13. **Cavendish's first *Pocket Guide to Bezique*:** 1868 in Jessel, 1869 in Hall. [S7, S51]
14. **"Russian Bezique" (Hall, 1887):** Jessel (1905) and Horr (1892) list no such title; Horr lists a Cavendish "Pocket Guide to Japanese Bezique." It may be the four-pack game under its earlier name; unverified. [S7, S51, S64] §3.3.
15. **First French attestation:** dictionary sources give 1859, but literary uses are recorded in 1846 (Kock) and 1855 (Offenbach/Moinaux), and Larousse dates the game in Paris cafés to c. 1827. The 1846 date for Kock's *Friquette* is itself uncertain: the passage examined describes four packs and a third bésigue (1860s–80s practice). [S33, S10, S26, S45]
16. **"Hoyle's Games Modernised documented it in 1863" (§3.2):** unverified; it conflicts with Pole's 1861 remark, a reader's 1866 request for rules and Jessel's entries. [S6, S16, S48, S52, S51]
17. **A "Camden" edition of 1870 (§3.2):** flagged in v1.8 as possibly retrospective because the label first appeared in Jessel's 1883 entry. That flag was too strong: Wikipedia cites a Camden 12th edition of 1872 [S9], so the label existed by 1872 and an 1870 10th edition is plausible; still unverified. [S6, S16, S51, S9]

---

## 12. Baseline ruleset for the engine [inference]

**Baseline:** the Pagat/Fosdick scoring table (§4); several melds may be laid down but one scores per trick, the rest announced "to score" (Hoffmann, Cady, Foster); melds declared before drawing, winner draws first; fixed Q♠+J♦ bezique; strict endgame (follow, win, trump); the last trick of all scores 10; brisques scored as taken; two single beziques score 40 then 500 (Foster); 1000 up (optional doubling); optional variants: Marjolet, single-pack Bésigue, Rubicon, six-pack, Polish.

**Options worth exposing, with the evidence for each:**

- No turn-up (first marriage sets trump): S45, S46, S4, S58.
- Panaché: S45, S46, S49.
- Claim game early ("limited to a fixed point"): S45, S49.
- Reuse of bezique cards: barred in the 1864–68 US texts, allowed in the French text, Cady and Foster (row 16).
- Brisque timing: as taken (Cavendish, Cady, Foster) or at the end (every 1861–68 source); simulation: the winner changes in 3.7% of games (§10b).
- Declare after drawing: simulation: about +20% meld points (§10b).
- All disjoint melds per trick (Cavendish 1870): simulation: under 1%.
- Trump-dependent bezique pair (row 7); last open trick versus last of all (row 9); lowest cut deals (row 12); any higher card wins (row 19); two singles each scoring 40 (row 20); nine-card hands and pone-draws-first (rows 17, 18).

**Suggested presets:** "Pagat baseline," "Cavendish 1870," "US Hoyle 1864," "Rubicon 1887," "six-pack (modern)." [inference] The pre-1870 sources all count brisques at the end, so a period mode would use end-counting; the baseline keeps as-taken because it is the later, more uniform practice.

---

## 13. Open items and access notes

**Access notes.** (1) archive.org `/stream/<id>/<id>_djvu.txt` works from the code sandbox (the text arrives wrapped in HTML; grep for "bezique" and print only the relevant line ranges); `/download/` links, www.gutenberg.org and web.archive.org are blocked. (2) archive.org's `advancedsearch.php`, `/metadata/<id>` and full-text search API work, but the search API returns intermittent 502s. (3) The fetch tool truncates only from the start (no paging) and archive.org details pages add about 6k tokens of boilerplate. (4) Bustarret, casinoenligne-fr.fr, durbetsel.ru and several other sites disallow automated access. (5) Where these routes fail, the fallback is to upload the pages.

| Item | Status | Notes |
|---|---|---|
| 1864 *American Hoyle*; 1868 *Modern Pocket Hoyle*; Foster 1897 | done (S45, S46, S47) | Later *American Hoyle* editions 1868–1892 checked by keyword (S58 for 1892); Gutenberg #53881 itself not read (S47 is the archive.org scan of the 1897 edition) |
| Pole 1861; Larousse GDU; Bonneveine and Dumesnil 1865 | done (S48, S26/S54, S49, S50) | |
| Cavendish 1870; Pardon; Phillips/Fox; 1892 Hoyle | done (S55–S58) | |
| Stats section | done (§10b) | Scripts `bezique_stats.py` (exact odds) and `bezique_sim.py` (seeded self-play). Not covered: human-level play, the chance a held meld is completed given the cards seen, Polish-variant odds, the effect of dix exchanges |
| Méry, *L'Arbitre des jeux* (1847) | not found | Not in archive.org title or full-text search; only a Gallica author page turned up by web search; Gallica, HathiTrust and Google Books are not reachable from the sandbox. S5 is the only summary |
| Van Tenac (1858) | not found | Known via Jessel's Baldwin entry and N&Q 1903 |
| Cavendish's own 1868–70 booklets; Camden's text | not found | Bibliographic records only [S51, S64]; Cavendish's text survives in reprints [S55–S58] |
| "Russian Bezique" | unresolved | See §11 item 14 and §3.3 |
| Six-pack origin | partly resolved | §3.3, §11 item 2 |
| Herring, Bustarret, Planet Haiti originals | unreachable | Substitutes: S10, S14, S60 |
| Lord Sherbrooke's N&Q 1903 note on *Feldinski* | not found | Cited by Jessel [S51] |
| Mike Goodall, *Goodall Card Game Booklets 1868–1922* (2016) and "The Great Bezique race of 1869" | not read | Known only via Hall and Sawyer [S6, S40, S61] |
| Berkeley (1901) | no item found | Jessel lists only the 1890 *Bezique and Cribbage*; S16 says Berkeley "writing in 1901" dated the craze to 1869 and mentions a Cavendish monograph of 1870 going to a second printing (source not identified) |
| Dutch, Czech, Portuguese, Polish, Japanese sources | only modern digests | S63; the Danish booklet is S62; the Vienna 1889 booklet is a bibliographic record only [S64] |
| Scholarly leads cited by S9 (§10.2) | not reached | Parlett, *A History of Card Games* (1990/91), *Oxford Dictionary of Card Games* (1996, p. 16), *Penguin Book of Card Games* (2008, pp. 287, 293); Anton, *Encyklopädie der Spiele*, 3rd ed. (1879, p. 43; the 1853 Alvensleben edition in archive.org has no bezique text); *Transactions of the Philological Society* (1910, p. 289); Brownsmith, "Bézique," *Westminster Papers* (1869, pp. 117–119; not found in archive.org volumes catalogued 1871–79, searched by keyword); *Cassell's Household Guide* (1881, p. 40); Méry 1847 (p. 65 suggests two packs); *The Last Diary of Tsaritsa Alexandra* (Yale UP, 1997) |
| Academic literature | searched | No bezique-specific paper found; adjacent works in §10.2 (S65–S67). Not read beyond abstract or citation: Gorman et al. (2011), the Schnapsen preprint, and the Skat/Jass/ISMCTS citations |
| Not yet read | | *Illustrated London News* 1870s; *Once a Week*, 13 Feb 1869; *Sphinx* 1869; the N&Q note at 9 S. xi p. 26; Girardi (1905), the German *Encyklopädie der Spiele* and Vanderheid (searched, no bezique text found) |

---

## 14. Changelog

- **v1 (rounds 1–2):** first consolidated review; Cady, Turkish and De La Rue additions.
- **v1.1:** Hoffmann edition corrected to 1909 (Bergholt revision); Wikipedia's Q♣+J♥ variant added; heavy-lift texts logged.
- **v1.2:** read S45–S47 (1864 *American Hoyle*, 1868 *Modern Pocket Hoyle*, Foster 1897); extended §3–§8 and matrix rows 1–5, 8.
- **v1.3:** added S48–S54 and upgraded S26 (Pole 1861, French manuals 1865, Jessel's bibliography, N&Q notes, Nouveau Larousse).
- **v1.4:** added S55–S61 (Cavendish 1870, Pardon, Phillips/Fox, the 1892 *American Hoyle*, periodicals, the Québec page, the Goodall blog); matrix row 19.
- **v1.5:** added S62–S63; new §10b (exact odds, `bezique_stats.py`).
- **v1.6:** `bezique_sim.py` and §10b Table 4; Dutch, Czech and Portuguese searches.
- **v1.7:** added S64 (Horr 1892); "Japanese Bezique" and a Vienna 1889 record.
- **v1.8 (review for anachronism and cruft), 2026-09-30:**
  - **Structure:** registry merged into one table (the S7, S26, S38 and S43 rows were malformed); §0–§1 rewritten as current-state summaries; §5 regrouped into period options, multi-pack games, neighbouring games and regional traditions, with modern-source variants labelled; §6 split into a short table plus per-source detail (§6.1); §11–§14 consolidated; all "round 3x" tags, first-person phrasing and stale "not yet read" lists removed from the body.
  - **New conflict:** row 20 (two single beziques before a double): Cady 40 + 40 + 500 versus Foster 40 + 500; §4 no longer presents Cady's version as consensus.
  - **Anachronism and reliability flags added:** the "1863 Hoyle" claim, the "Camden 1870" edition, dictionary first-attestation dates versus earlier literary uses, and Paul de Kock's *Friquette* (date versus the four-pack game its text describes) (§11 items 15–17); six-pack and eight-pack details marked as modern-source (§5.2); the Pinochle rulebook in row 2 labelled as such; "bijugum" corrected; a Jessel-based table replaces a wall of bibliographic text (§3.2).
  - **Not changed:** every source citation and every computed figure (the S3/Hoffmann positions were not re-read).

- **v1.9 (academic-literature search), 2026-09-30:**
  - **Added:** S65–S68 (Pinochle combinatorics paper; Schnapsen and trick-taking AI literature; Alexander Palace page); §10 split into digital implementations and academic literature (§10.2); §10b Table 5 (distinguishable hands, exact) and its function in `bezique_stats.py`; §1 item 6 rewritten; scholarly leads and Berkeley 1901 note in §13.
  - **Correction of the v1.8 review:** the "Camden edition of 1870 may be retrospective" flag was too strong: Wikipedia cites a Camden 12th edition of 1872 (S9); §3.2 and §11 item 17 corrected.
  - **Found nothing:** no bezique-specific paper on AI, solving, probability, game theory or history. The 2009-dated citation of Gorman et al. in S9 is the 2011 journal issue.
  - **Also:** registry rows S24, S29, S30 and S44 had a malformed notes cell; repaired.