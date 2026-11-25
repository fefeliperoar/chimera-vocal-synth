# Spanish

Speak and Sing read Spanish once you choose it: **ES**, beside EN and TR. It is one setting for
both tabs, kept on the device with Sing's words. Spanish is written in Latin letters, so the
lyrics router can't tell it from English by script ([language-front-ends.md](language-front-ends.md));
English stays the default.

`crates/chimera/src/phonetics/spanish.rs` reads the spelling by rule, with no dictionary, onto the
existing inventory (no new sounds). It reads Latin American Spanish: seseo (c before e i, z, s are
all [s]) and yeísmo (ll and y are [ʝ]).

## Writing it

- Type it as you'd write it: á é í ó ú ü ñ, and ¿ ¡ if you like. Accents also mark the stress.
- In Sing, a vowel is a note: words split where Spanish spelling splits them (can-ción, a-gua,
  cons-truir). `-` alone holds the previous vowel onto the next note, and hyphens split a word
  your way.
- Numbers are read out in Speak ("1.923,5" → "mil novecientos veintitrés coma cinco").

## What it does

| | |
|---|---|
| c g | [s] and [x] before e i; [k] and [g] elsewhere; qu [k], gu before e i [g], gü [gw] |
| h j ll ñ ch x | h silent, j [x], ll [ʝ], ñ [ɲ], ch [tʃ], x [ks] |
| b v d g | stops [b d g] at the start of a word and after a nasal (d also after l); approximants [β ð ɣ] elsewhere (lado [laðo], agua [aɣwa]) |
| r | trill [r] at the start of a word, after n l s and as rr; a tap [ɾ] elsewhere (pero [peɾo], perro [pero]) |
| n | [m] before b v p m and [ŋ] before k g j q x |
| i u y | glides [j w] beside another vowel in the same syllable (canción, agua, hoy); a syllable of their own when accented (país) or between strong vowels (le-o) |
| Stress | the accent mark; else the next-to-last syllable of a word ending in a vowel, n or s, the last of the rest; one-syllable function words (el, de, que) take none |
| Intonation | statement; a phrase ending in "?" rises |

## Not yet

- Allophones are decided inside a word: a word's first b d g is a stop even after a vowel
  ("la dama" is [la dama], not [la ðama]).
- No s-aspiration or other dialects; no Castilian θ.
- Diphthongs are a glide segment beside a plain vowel, not a gliding nucleus, so a sung "ai" is the
  vowel then a short [j] in the coda.
- A word spelled in a foreign way (names, loans) is read by the same rules.
- Written without a compiler to hand: run `cargo test -p chimera --test spanish`, re-bless the
  goldens if a shared table moved, and listen before trusting it.
