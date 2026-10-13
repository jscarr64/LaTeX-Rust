//! `\dfrac`, `\tfrac`, and `\cfrac` keep the command that built them.
//!
//! `\frac` and `{a \over b}` stay a plain fraction. The styled commands
//! print back as themselves and lay out in that style.

use latex_rust::{layout, parse, MathFont, MathStyle};

fn gold_of(src: &str) -> String {
    parse(src)
        .unwrap_or_else(|err| panic!("{src}: {err}"))
        .gold()
}

#[test]
fn styled_fractions_keep_their_command() {
    let plain = gold_of(r"\frac{1}{2}");
    assert_eq!(plain, r#"(frac (atom Ord "1") (atom Ord "2"))"#);
    assert_eq!(gold_of(r"{1 \over 2}"), plain);

    let d = gold_of(r"\dfrac{1}{2}");
    let t = gold_of(r"\tfrac{1}{2}");
    let c = gold_of(r"\cfrac{1}{2}");
    assert_eq!(d, r#"(dfrac (atom Ord "1") (atom Ord "2"))"#);
    assert_eq!(t, r#"(tfrac (atom Ord "1") (atom Ord "2"))"#);
    assert_eq!(c, r#"(cfrac (atom Ord "1") (atom Ord "2"))"#);
    assert_ne!(d, plain);
    assert_ne!(t, plain);
    assert_ne!(c, plain);
}

#[test]
fn styled_fractions_round_trip() {
    for src in [
        r"\dfrac{1}{2}",
        r"\tfrac{a}{b}",
        r"\cfrac{1}{1+x}",
        r"\frac{1}{2}",
    ] {
        let node = parse(src).unwrap_or_else(|err| panic!("{src}: {err}"));
        let printed = node.to_latex();
        let again = parse(&printed).unwrap_or_else(|err| panic!("{printed}: {err}"));
        assert_eq!(again.gold(), node.gold(), "{src} -> {printed}");
        assert_eq!(again.to_latex(), printed, "{src}");
    }
    let over = parse(r"{1 \over 2}").unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(over.to_latex(), r"\frac {1}{2}");
    assert!(parse(r"\dfrac{1}{2}")
        .unwrap_or_else(|err| panic!("{err}"))
        .to_latex()
        .contains(r"\dfrac"));
    assert!(parse(r"\tfrac{1}{2}")
        .unwrap_or_else(|err| panic!("{err}"))
        .to_latex()
        .contains(r"\tfrac"));
    assert!(parse(r"\cfrac{1}{2}")
        .unwrap_or_else(|err| panic!("{err}"))
        .to_latex()
        .contains(r"\cfrac"));
}

#[test]
fn styled_fractions_use_their_math_style() {
    let font = MathFont::stix_two_math().expect("STIX Two Math");
    let dims = |src, style| {
        let ast = parse(src).unwrap_or_else(|err| panic!("{src}: {err}"));
        layout(&ast, &font, style)
            .unwrap_or_else(|err| panic!("{src}: {err}"))
            .dim_gold()
    };
    let display_frac = dims(r"\frac{1}{2}", MathStyle::Display);
    let text_frac = dims(r"\frac{1}{2}", MathStyle::Text);
    assert_ne!(display_frac, text_frac);
    assert_eq!(dims(r"\dfrac{1}{2}", MathStyle::Text), display_frac);
    assert_eq!(dims(r"\tfrac{1}{2}", MathStyle::Display), text_frac);
    assert_eq!(dims(r"\cfrac{1}{2}", MathStyle::Text), display_frac);
}
