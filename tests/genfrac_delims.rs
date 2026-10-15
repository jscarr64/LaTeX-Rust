//! `\genfrac` delimiters are the ones `\left` and `\right` read, or a typed error.
//!
//! `\genfrac{}{a}{}{}|a` parsed to `Delimited(Empty, …, Char('a'))`, which
//! prints `\left .\frac {|}{a}\right a`, and `\right a` is not a delimiter, so
//! the reprint did not parse (fuzz r18long, `roundtrip-genfrac-delim`). The
//! `\genfrac` reader took any one character, or any word, as a delimiter.
//!
//! - a table of the crash inputs and the cases around them;
//! - a sweep over every character and a list of control words as the left or
//!   the right delimiter of `\genfrac`, and of `\left`, `\right`, `\middle`
//!   and `\big`, which must each give a stable round trip or an error.

use latex_rust::parse;

fn check(src: &str) -> Result<(), String> {
    let Ok(tree) = parse(src) else {
        return Ok(());
    };
    let printed = tree.to_latex();
    let again = parse(&printed).map_err(|e| format!("{src:?} printed {printed:?}: {e}"))?;
    if again != tree {
        return Err(format!(
            "{src:?} printed {printed:?}, which parses to another tree"
        ));
    }
    if again.to_latex() != printed {
        return Err(format!("{src:?} does not print twice the same"));
    }
    Ok(())
}

fn seed(name: &str) -> String {
    let path = format!("{}/fuzz/corpus/parse/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

#[test]
fn the_genfrac_seeds_round_trip_or_reject() {
    for name in ["roundtrip-genfrac-delim", "roundtrip-genfrac-delim-orig"] {
        if let Err(msg) = check(&seed(name)) {
            panic!("{name}: {msg}");
        }
    }
}

#[test]
fn genfrac_takes_only_real_delimiters() {
    for src in [
        "\\genfrac{}{a}{}{}|a",
        "\\genfrac{a}{}{}{}|a",
        "\\genfrac{ab}{}{}{}xy",
        "\\genfrac{\\lbrace}{\\rbrace}{}{}xy",
        "\\genfrac{x}{y}{}{}1 2",
        "\\genfrac{é}{}{}{}1 2",
    ] {
        assert!(parse(src).is_err(), "{src:?} must be a typed error");
    }
    for src in [
        "\\genfrac{(}{)}{}{}12",
        "\\genfrac{[}{]}{}{}12",
        "\\genfrac{\\{}{\\}}{}{}12",
        "\\genfrac{\\langle}{\\rangle}{}{}12",
        "\\genfrac{|}{.}{}{}12",
        "\\genfrac{}{}{}{}12",
        "\\genfrac{.}{.}{}{}12",
        "\\genfrac{ ( }{ ) }{}{}12",
    ] {
        if let Err(msg) = check(src) {
            panic!("{msg}");
        }
        assert!(parse(src).is_ok(), "{src:?} must parse");
    }
}

#[test]
fn every_character_and_word_as_a_delimiter_round_trips_or_is_rejected() {
    let mut delims: Vec<String> = (0u32..0x300)
        .filter_map(char::from_u32)
        .filter(|c| {
            !matches!(
                c,
                '{' | '}' | '\\' | '%' | '#' | '$' | '&' | '^' | '_' | '~'
            )
        })
        .map(String::from)
        .collect();
    for name in [
        "\\{",
        "\\}",
        "\\|",
        "\\.",
        "\\langle",
        "\\rangle",
        "\\lfloor",
        "\\rfloor",
        "\\lceil",
        "\\rceil",
        "\\lvert",
        "\\rvert",
        "\\lVert",
        "\\rVert",
        "\\vert",
        "\\Vert",
        "\\uparrow",
        "\\downarrow",
        "\\Uparrow",
        "\\Downarrow",
        "\\updownarrow",
        "\\Updownarrow",
        "\\backslash",
        "\\lgroup",
        "\\rgroup",
        "\\lmoustache",
        "\\rmoustache",
        "\\lbrace",
        "\\rbrace",
        "\\lbrack",
        "\\rbrack",
        "\\bigl",
        "\\foo",
        "ab",
        "..",
        "()",
        "\\\\",
        "\\left",
        "\\ ",
        "\\,",
        "\\alpha",
    ] {
        delims.push(name.to_string());
    }
    for d in &delims {
        for src in [
            format!("\\genfrac{{{d}}}{{}}{{}}{{}}ab"),
            format!("\\genfrac{{}}{{{d}}}{{}}{{}}ab"),
            format!("\\genfrac{{{d}}}{{{d}}}{{}}{{}}ab"),
            format!("\\left{d} a \\right{d}"),
            format!("\\left( a \\middle{d} b \\right)"),
            format!("\\big{d} a"),
            format!("\\Bigl{d} a \\Bigr{d}"),
        ] {
            if let Err(msg) = check(&src) {
                panic!("{msg}");
            }
        }
    }
}
