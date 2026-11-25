//! Spanish G2P: letters to sounds, allophones, diphthongs, stress, numbers, phrases, and the
//! lyrics front end. (Written without a compiler at hand: run `cargo test -p chimera --test spanish`.)
use chimera::phonetics::spanish::{lyrics_to_syllables, to_phrases, word};
use chimera::phonetics::{Intonation, Segment};

/// Segments as IPA, a stress mark before the stressed vowel and a space after each word.
fn shown(segs: &[Segment]) -> String {
    segs.iter()
        .map(|s| format!("{}{}{}", if s.stress == Some(1) { "ˈ" } else { "" }, s.ipa, if s.word_end == Some(true) { " " } else { "" }))
        .collect::<String>()
        .trim_end()
        .to_string()
}
fn w(typed: &str) -> String {
    shown(&word(typed))
}
fn es(text: &str) -> String {
    to_phrases(text).iter().map(|p| shown(&p.segments)).collect::<Vec<_>>().join(" | ")
}
fn texts(lyrics: &str) -> Vec<String> {
    lyrics_to_syllables(lyrics).into_iter().map(|s| s.text).collect()
}

#[test]
fn letters_map_to_sounds() {
    assert_eq!(w("casa"), "kˈasa");
    assert_eq!(w("hola"), "ˈola");
    assert_eq!(w("cena"), "sˈena");
    assert_eq!(w("queso"), "kˈeso");
    assert_eq!(w("rojo"), "rˈoxo");
    assert_eq!(w("ñandú"), "ɲandˈu");
    assert_eq!(w("chico"), "tʃˈiko");
    assert_eq!(w("calle"), "kˈaʝe");
}

#[test]
fn tap_and_trill_differ() {
    assert_eq!(w("pero"), "pˈeɾo");
    assert_eq!(w("perro"), "pˈero");
    assert_eq!(w("enrique"), "enrˈike");
}

#[test]
fn b_d_g_are_stops_only_after_a_pause_or_nasal() {
    assert_eq!(w("lado"), "lˈaðo");
    assert_eq!(w("donde"), "dˈonde");
    assert_eq!(w("agua"), "ˈaɣwa");
    assert_eq!(w("inglés"), "iŋglˈes");
}

#[test]
fn diphthongs_are_glides_and_hiatus_is_not() {
    assert_eq!(w("canción"), "kansjˈon");
    assert_eq!(w("hoy"), "ˈoj");
    assert_eq!(w("país"), "paˈis");
}

#[test]
fn clitics_take_no_stress() {
    assert_eq!(w("que"), "ke");
    assert_eq!(w("el"), "el");
}

#[test]
fn a_question_rises() {
    let p = to_phrases("¿Cómo estás?");
    assert_eq!(p.len(), 1);
    assert_eq!(p[0].intonation, Intonation::Question);
    assert_eq!(es("¿Cómo estás?"), "kˈomo estˈas");
}

#[test]
fn numbers_are_read_out() {
    assert_eq!(es("5"), es("cinco"));
    assert_eq!(es("21.000"), es("veintiún mil"));
    assert_eq!(es("1.000.000"), es("un millón"));
    assert_eq!(es("3,05"), es("tres coma cero cinco"));
}

#[test]
fn lyrics_split_where_spanish_spelling_splits() {
    assert_eq!(texts("canción"), ["can", "ción"]);
    assert_eq!(texts("estrella"), ["es", "tre", "lla"]);
    assert_eq!(texts("construir"), ["cons", "truir"]);
    assert_eq!(texts("ca-sa"), ["ca", "sa"]);
}
