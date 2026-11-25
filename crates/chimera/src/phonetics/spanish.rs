//! Spanish text → phoneme segments. Spanish spelling is close to phonemic, so letters map to
//! sounds by rule and no dictionary is needed (Latin American Spanish: seseo, yeísmo):
//! - c before e i, z and s are [s]; c elsewhere, k and qu are [k]; g before e i and j are [x];
//!   gu before e i is [g] (the u is silent) and gü is [gw]; h is silent; ch is [tʃ]; ñ is [ɲ];
//!   ll and consonant y are [ʝ]; x is [ks]; v is [b].
//! - b d g are stops at the start of a word and after a nasal (d also after l), and the
//!   approximants [β ð ɣ] everywhere else ("lado" [laðo], "agua" [aɣwa]).
//! - r is a trill [r] at the start of a word, after n l s and when written rr, a tap [ɾ] elsewhere.
//! - n takes the place of the consonant after it: [m] before b v p m, [ŋ] before k g j q x.
//! - i and u beside another vowel are glides [j w] in the same syllable ("canción", "agua",
//!   "hoy"), unless the vowel is accented (í ú) or both vowels are strong (a e o): "país", "leo".
//!
//! Stress: an accent mark decides; else the next-to-last syllable of a word ending in a vowel,
//! n or s, and the last syllable of the rest. One-syllable function words ("el", "de", "que")
//! take none.
//!
//! Spanish vowels don't reduce, so there are no weak forms, and a phrase takes statement
//! intonation, or a rising one when it ends in "?".

use super::english::{Phrase, is_phrase_punct, split_keeping};
use super::mandarin::MandarinLexicon;
use super::syllables::{FrontEnd, Lang, lyrics_in};
use super::{Intonation, Segment, Syllable};

fn is_letter(c: char) -> bool {
    c.is_ascii_alphabetic() || "áéíóúüñÁÉÍÓÚÜÑ".contains(c)
}

fn es_lower(c: char) -> char {
    c.to_lowercase().next().unwrap_or(c)
}

fn is_vowel_letter(c: char) -> bool {
    "aeiouáéíóúü".contains(c)
}

/// A letter that fronts a c or g before it (e, i)
fn fronts(c: Option<char>) -> bool {
    c.is_some_and(|c| "eiéí".contains(c))
}

fn vowel_ipa(base: char) -> &'static str {
    match base {
        'a' => "a",
        'e' => "e",
        'i' => "i",
        'o' => "o",
        _ => "u",
    }
}

/// Words of one syllable that take no stress of their own.
const CLITICS: &[&str] = &[
    "el", "la", "los", "las", "un", "de", "en", "y", "e", "o", "u", "a", "que", "con", "por", "mi", "tu", "su", "se", "lo", "le", "les", "me", "te", "nos", "al", "del", "si", "ni", "sin",
];

/// One letter or digraph of a word: a vowel, or a consonant with the sounds it makes (none for
/// a silent h).
struct Elem {
    vowel: bool,
    /// a, e, o and any accented vowel; i and u are weak, and form a diphthong with another vowel
    strong: bool,
    accent: bool,
    /// the vowel letter without its mark
    base: char,
    ipa: Vec<&'static str>,
    /// the letters it is written with: `a..b`
    a: usize,
    b: usize,
}

/// What n sounds like before the letter at `j`.
fn n_before(w: &[char], j: usize) -> &'static str {
    match w.get(j).copied() {
        Some('b' | 'v' | 'p' | 'm') => "m",
        Some('g' | 'j' | 'k' | 'q' | 'x') => "ŋ",
        Some('c') if !fronts(w.get(j + 1).copied()) && w.get(j + 1) != Some(&'h') => "ŋ",
        _ => "n",
    }
}

fn elems(w: &[char]) -> Vec<Elem> {
    let n = w.len();
    let mut out: Vec<Elem> = Vec::with_capacity(n);
    let mut i = 0;
    while i < n {
        let c = w[i];
        let next = w.get(i + 1).copied();
        // y is a vowel at the end of a word and before a consonant ("hoy", "y"), else a consonant
        let y_vowel = c == 'y' && !next.is_some_and(is_vowel_letter);
        if is_vowel_letter(c) || y_vowel {
            let (base, accent) = match c {
                'á' => ('a', true),
                'é' => ('e', true),
                'í' => ('i', true),
                'ó' => ('o', true),
                'ú' => ('u', true),
                'ü' => ('u', false),
                'y' => ('i', false),
                c => (c, false),
            };
            let strong = accent || "aeo".contains(base);
            out.push(Elem { vowel: true, strong, accent, base, ipa: vec![vowel_ipa(base)], a: i, b: i + 1 });
            i += 1;
            continue;
        }
        let prev_c: Option<&str> = out.last().filter(|e| !e.vowel).and_then(|e| e.ipa.last().copied());
        let initial = out.is_empty();
        let nasal = prev_c.is_some_and(|p| matches!(p, "m" | "n" | "ŋ" | "ɲ"));
        let mut len = 1;
        let ipa: Vec<&'static str> = match c {
            'b' | 'v' => vec![if initial || nasal { "b" } else { "β" }],
            'c' if next == Some('h') => {
                len = 2;
                vec!["tʃ"]
            }
            'c' if fronts(next) => vec!["s"],
            'c' | 'k' | 'q' => {
                if c == 'q' && next == Some('u') {
                    len = 2;
                }
                vec!["k"]
            }
            'd' => vec![if initial || nasal || prev_c == Some("l") { "d" } else { "ð" }],
            'f' => vec!["f"],
            'g' => {
                if fronts(next) {
                    vec!["x"]
                } else {
                    // gue, gui: the u only keeps the g hard
                    if next == Some('u') && fronts(w.get(i + 2).copied()) {
                        len = 2;
                    }
                    vec![if initial || nasal { "g" } else { "ɣ" }]
                }
            }
            'h' => vec![],
            'j' => vec!["x"],
            'l' if next == Some('l') => {
                len = 2;
                vec!["ʝ"]
            }
            'l' => vec!["l"],
            'm' => vec!["m"],
            'n' => vec![n_before(w, i + 1)],
            'ñ' => vec!["ɲ"],
            'p' => vec!["p"],
            'r' if next == Some('r') => {
                len = 2;
                vec!["r"]
            }
            'r' => vec![if initial || matches!(prev_c, Some("n" | "l" | "s")) { "r" } else { "ɾ" }],
            's' | 'z' => vec!["s"],
            't' => vec!["t"],
            'w' => vec!["w"],
            'x' => {
                if initial {
                    vec!["s"]
                } else {
                    vec!["k", "s"]
                }
            }
            'y' => vec!["ʝ"],
            _ => vec![],
        };
        out.push(Elem { vowel: false, strong: false, accent: false, base: c, ipa, a: i, b: i + len });
        i += len;
    }
    out
}

/// A run of vowels in one syllable, or one consonant.
enum Unit {
    C(usize),
    N(Vec<usize>),
}

/// How many of the consonants between two vowels close the syllable before: all but the longest
/// tail that can open one (a stop or f, then l or r: "a-bril", "com-pra", "ins-tru").
fn coda_share(w: &[char], els: &[Elem], run: &[usize]) -> usize {
    let n = run.len();
    let cluster = |a: usize, b: usize| -> bool {
        let (ea, eb) = (&els[a], &els[b]);
        if ea.b - ea.a != 1 || eb.b - eb.a != 1 {
            return false;
        }
        match w[eb.a] {
            'l' => "pbfgck".contains(w[ea.a]),
            'r' => "pbfgcktd".contains(w[ea.a]),
            _ => false,
        }
    };
    match n {
        0 | 1 => 0,
        2 => usize::from(!cluster(run[0], run[1])),
        _ => {
            if cluster(run[n - 2], run[n - 1]) {
                n - 2
            } else {
                n - 1
            }
        }
    }
}

/// One syllable: its sounds, and where its letters start.
struct Built {
    onset: Vec<&'static str>,
    nuc: &'static str,
    coda: Vec<&'static str>,
    accent: bool,
    start: usize,
}

struct Parsed {
    typed: Vec<char>,
    syls: Vec<Built>,
    stressed: Option<usize>,
}

fn glide(e: &Elem) -> &'static str {
    if e.base == 'i' { "j" } else { "w" }
}

fn parse(typed: &str) -> Parsed {
    let mut letters: Vec<char> = Vec::with_capacity(typed.len());
    let mut kept: Vec<char> = Vec::with_capacity(typed.len());
    for c in typed.chars() {
        if is_letter(c) {
            letters.push(es_lower(c));
            kept.push(c);
        }
    }
    let els = elems(&letters);
    // vowels beside each other share a syllable unless both are strong ("le-o", "pa-ís")
    let mut units: Vec<Unit> = Vec::new();
    for (k, e) in els.iter().enumerate() {
        if e.vowel {
            if let Some(Unit::N(g)) = units.last_mut() {
                let strong = g.iter().filter(|&&x| els[x].strong).count();
                if g.len() < 3 && !(e.strong && strong > 0) {
                    g.push(k);
                    continue;
                }
            }
            units.push(Unit::N(vec![k]));
        } else {
            units.push(Unit::C(k));
        }
    }
    // consonants between two nuclei: the tail that can open a syllable goes to the next
    struct Syl {
        onset: Vec<usize>,
        group: Vec<usize>,
        coda: Vec<usize>,
    }
    let mut syls: Vec<Syl> = Vec::new();
    let mut pending: Vec<usize> = Vec::new();
    for u in &units {
        match u {
            Unit::C(k) => pending.push(*k),
            Unit::N(g) => {
                let onset = if syls.is_empty() {
                    std::mem::take(&mut pending)
                } else {
                    let cut = coda_share(&letters, &els, &pending);
                    let tail = pending.split_off(cut);
                    let head = std::mem::take(&mut pending);
                    if let Some(last) = syls.last_mut() {
                        last.coda.extend(head);
                    }
                    tail
                };
                syls.push(Syl { onset, group: g.clone(), coda: Vec::new() });
            }
        }
    }
    if let Some(last) = syls.last_mut() {
        last.coda.extend(pending);
    }
    let built: Vec<Built> = syls
        .iter()
        .map(|s| {
            // the nucleus is the strong vowel, else the last of a run of weak ones ("ui", "iu"); the
            // others are glides, rising before it and falling after it
            let at = s.group.iter().position(|&x| els[x].strong).unwrap_or(s.group.len() - 1);
            let mut onset: Vec<&'static str> = s.onset.iter().flat_map(|&x| els[x].ipa.iter().copied()).collect();
            onset.extend(s.group[..at].iter().map(|&x| glide(&els[x])));
            let mut coda: Vec<&'static str> = s.group[at + 1..].iter().map(|&x| glide(&els[x])).collect();
            coda.extend(s.coda.iter().flat_map(|&x| els[x].ipa.iter().copied()));
            let first = s.onset.first().copied().unwrap_or(s.group[0]);
            Built { onset, nuc: els[s.group[at]].ipa[0], coda, accent: s.group.iter().any(|&x| els[x].accent), start: els[first].a }
        })
        .collect();
    let stressed = stressed_syllable(&letters, &built);
    Parsed { typed: kept, syls: built, stressed }
}

fn stressed_syllable(letters: &[char], syls: &[Built]) -> Option<usize> {
    let n = syls.len();
    if n == 0 {
        return None;
    }
    if let Some(k) = syls.iter().position(|s| s.accent) {
        return Some(k);
    }
    if n == 1 {
        let word: String = letters.iter().collect();
        return if CLITICS.contains(&word.as_str()) { None } else { Some(0) };
    }
    Some(if matches!(letters.last(), Some('a' | 'e' | 'i' | 'o' | 'u' | 'n' | 's')) { n - 2 } else { n - 1 })
}

fn seg(ipa: &str, stress: u8) -> Segment {
    Segment { ipa: ipa.to_string(), stress: Some(stress), ..Default::default() }
}

/// One word, as typed, to segments with stress.
pub fn word(typed: &str) -> Vec<Segment> {
    let p = parse(typed);
    let mut out = Vec::new();
    for (k, s) in p.syls.iter().enumerate() {
        out.extend(s.onset.iter().map(|i| seg(i, 0)));
        out.push(seg(s.nuc, u8::from(p.stressed == Some(k))));
        out.extend(s.coda.iter().map(|i| seg(i, 0)));
    }
    out
}

/// One word's sung syllables: a vowel (with its glides) is a note, and the consonants between two
/// vowels are split as Spanish spells them ("can-ción", "a-gua", "com-pra").
fn sung_syllables(typed: &str) -> Vec<Syllable> {
    let p = parse(typed);
    p.syls
        .iter()
        .enumerate()
        .map(|(k, s)| {
            let end = p.syls.get(k + 1).map_or(p.typed.len(), |n| n.start);
            Syllable {
                text: p.typed[s.start..end].iter().collect(),
                onset: s.onset.iter().map(|i| seg(i, 0)).collect(),
                nucleus: seg(s.nuc, u8::from(p.stressed == Some(k))),
                coda: s.coda.iter().map(|i| seg(i, 0)).collect(),
                ..Default::default()
            }
        })
        .collect()
}

/// The Spanish front end for the lyrics router ([`super::syllables::read_lyrics`]). Spanish is
/// written in Latin letters, so the router can't tell it from English by script: the player
/// chooses it ([`Lang`]). Typed hyphens split a word your way, as in English.
pub struct Spanish;

impl FrontEnd for Spanish {
    fn starts(&self, c: char) -> bool {
        is_letter(c)
    }
    fn continues(&self, c: char) -> bool {
        is_letter(c) || c == '-'
    }
    fn word(&self, tok: &str) -> Vec<Syllable> {
        let mut out = Vec::new();
        for part in tok.split('-').filter(|p| p.chars().any(is_letter)) {
            out.append(&mut sung_syllables(part));
        }
        out
    }
}

/// Lyrics → sung syllables with Latin script read as Spanish ([`Lang::Es`]); other scripts are
/// routed as usual, and no dictionary is needed.
pub fn lyrics_to_syllables(text: &str) -> Vec<Syllable> {
    lyrics_in(text, &|_: &str, _: bool| Vec::<Segment>::new(), &MandarinLexicon::new(), Lang::Es)
}

/// A letter and the combining mark after it (as decomposed text has them) as one letter.
fn compose(t: &str) -> String {
    let mut out = String::with_capacity(t.len());
    for c in t.chars() {
        let merged = match (out.chars().last(), c) {
            (Some('a'), '\u{301}') => Some('á'),
            (Some('e'), '\u{301}') => Some('é'),
            (Some('i'), '\u{301}') => Some('í'),
            (Some('o'), '\u{301}') => Some('ó'),
            (Some('u'), '\u{301}') => Some('ú'),
            (Some('u'), '\u{308}') => Some('ü'),
            (Some('n'), '\u{303}') => Some('ñ'),
            (Some('A'), '\u{301}') => Some('Á'),
            (Some('E'), '\u{301}') => Some('É'),
            (Some('I'), '\u{301}') => Some('Í'),
            (Some('O'), '\u{301}') => Some('Ó'),
            (Some('U'), '\u{301}') => Some('Ú'),
            (Some('U'), '\u{308}') => Some('Ü'),
            (Some('N'), '\u{303}') => Some('Ñ'),
            _ => None,
        };
        match merged {
            Some(m) => {
                out.pop();
                out.push(m);
            }
            None => out.push(c),
        }
    }
    out
}

/// Spanish text → phrases of segments. Phrase punctuation breaks phrases as in English, a phrase
/// ending in "?" rises, and numbers are read out ("1.923,5" → "mil novecientos veintitrés coma cinco").
pub fn to_phrases(text: &str) -> Vec<Phrase> {
    let normalized = expand_numbers(&compose(text));
    let mut phrases = Vec::new();
    let mut words: Vec<String> = Vec::new();
    let mut flush = |delim: &str, words: &mut Vec<String>| {
        let mut segs: Vec<Segment> = Vec::new();
        let mut said: Vec<String> = Vec::new();
        for w in words.drain(..) {
            let mut ws = word(&w);
            let Some(last) = ws.last_mut() else { continue };
            last.word_end = Some(true);
            segs.append(&mut ws);
            said.push(w.chars().filter(|c| is_letter(*c)).map(es_lower).collect());
        }
        if segs.is_empty() {
            return;
        }
        let pause = if delim.contains(['.', '!', '?']) {
            0.35
        } else if !delim.is_empty() {
            0.18
        } else {
            0.3
        };
        let intonation = if delim.contains('?') { Intonation::Question } else { Intonation::Statement };
        phrases.push(Phrase { segments: segs, intonation, pause, words: said });
    };
    for part in split_keeping(&normalized, is_phrase_punct) {
        if !part.is_empty() && part.chars().all(is_phrase_punct) {
            flush(&part, &mut words);
        } else {
            words.extend(part.split(|c: char| !is_letter(c)).filter(|w| !w.is_empty()).map(str::to_string));
        }
    }
    flush("", &mut words);
    phrases
}

// ---------------------------------------------------------------- numbers
const SMALL: [&str; 30] = [
    "cero", "uno", "dos", "tres", "cuatro", "cinco", "seis", "siete", "ocho", "nueve", "diez", "once", "doce", "trece", "catorce", "quince", "dieciséis", "diecisiete", "dieciocho", "diecinueve", "veinte", "veintiuno",
    "veintidós", "veintitrés", "veinticuatro", "veinticinco", "veintiséis", "veintisiete", "veintiocho", "veintinueve",
];
const TENS: [&str; 10] = ["", "", "", "treinta", "cuarenta", "cincuenta", "sesenta", "setenta", "ochenta", "noventa"];
const HUNDREDS: [&str; 10] = ["", "ciento", "doscientos", "trescientos", "cuatrocientos", "quinientos", "seiscientos", "setecientos", "ochocientos", "novecientos"];

/// 1..999 in words.
fn below_thousand(n: u64) -> String {
    if n < 30 {
        SMALL[n as usize].to_string()
    } else if n < 100 {
        let (t, o) = ((n / 10) as usize, (n % 10) as usize);
        if o == 0 { TENS[t].to_string() } else { format!("{} y {}", TENS[t], SMALL[o]) }
    } else if n == 100 {
        "cien".to_string()
    } else {
        let h = HUNDREDS[(n / 100) as usize];
        if n % 100 == 0 { h.to_string() } else { format!("{h} {}", below_thousand(n % 100)) }
    }
}

/// "uno" before a thousand or a million is "un" ("veintiún mil").
fn apocope(s: String) -> String {
    if let Some(p) = s.strip_suffix("veintiuno") {
        format!("{p}veintiún")
    } else if let Some(p) = s.strip_suffix("uno") {
        format!("{p}un")
    } else {
        s
    }
}

fn words_of(n: u64) -> String {
    if n < 1_000 {
        below_thousand(n)
    } else if n < 1_000_000 {
        let (q, r) = (n / 1_000, n % 1_000);
        let head = if q == 1 { "mil".to_string() } else { format!("{} mil", apocope(below_thousand(q))) };
        if r == 0 { head } else { format!("{head} {}", below_thousand(r)) }
    } else {
        let (q, r) = (n / 1_000_000, n % 1_000_000);
        let head = if q == 1 { "un millón".to_string() } else { format!("{} millones", apocope(words_of(q))) };
        if r == 0 { head } else { format!("{head} {}", words_of(r)) }
    }
}

/// Digits → Spanish number words (digit by digit past the quadrillions).
fn number_words(digits: &str) -> String {
    let d = digits.trim_start_matches('0');
    if d.is_empty() {
        return SMALL[0].into();
    }
    match d.parse::<u64>().ok().filter(|n| *n < 1_000_000_000_000_000) {
        Some(n) => words_of(n),
        None => digits.bytes().map(|b| SMALL[(b - b'0') as usize]).collect::<Vec<_>>().join(" "),
    }
}

/// Numbers in text → words. A dot before three digits groups thousands ("1.000"), a comma
/// between digits is the decimal comma ("3,05" → "tres coma cero cinco").
fn expand_numbers(t: &str) -> String {
    let c: Vec<char> = t.chars().collect();
    let digit = |i: usize| c.get(i).is_some_and(char::is_ascii_digit);
    let mut out = String::with_capacity(t.len());
    let mut i = 0;
    while i < c.len() {
        if !digit(i) {
            out.push(c[i]);
            i += 1;
            continue;
        }
        let mut int = String::new();
        loop {
            while digit(i) {
                int.push(c[i]);
                i += 1;
            }
            if c.get(i) == Some(&'.') && digit(i + 1) && digit(i + 2) && digit(i + 3) && !digit(i + 4) {
                i += 1;
                continue;
            }
            break;
        }
        out.push(' ');
        out.push_str(&number_words(&int));
        if c.get(i) == Some(&',') && digit(i + 1) {
            i += 1;
            let mut frac = String::new();
            while digit(i) {
                frac.push(c[i]);
                i += 1;
            }
            out.push_str(" coma");
            let zeros = frac.len() - frac.trim_start_matches('0').len();
            for _ in 0..zeros {
                out.push_str(" cero");
            }
            if zeros < frac.len() {
                out.push(' ');
                out.push_str(&number_words(&frac[zeros..]));
            }
        }
        out.push(' ');
    }
    out
}
