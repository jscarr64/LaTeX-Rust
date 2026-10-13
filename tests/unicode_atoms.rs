//! Item 6 is on by default. Prime symbols stay behind `prime_symbol`.

mod common;

use common::{flag, gold, gold_with};
use latex_rust::{parse, ParseError};

#[test]
fn codepoints() {
    assert_eq!('√', '\u{221A}');
    assert_eq!('²', '\u{00B2}');
    assert_eq!('³', '\u{00B3}');
    assert_eq!('ⁿ', '\u{207F}');
    assert_eq!('⁻', '\u{207B}');
    assert_eq!('¹', '\u{00B9}');
    assert_eq!('−', '\u{2212}');
}

#[test]
fn radical_matches_sqrt() -> Result<(), ParseError> {
    assert_eq!(gold("√x")?, gold(r"\sqrt{x}")?);
    assert_eq!(gold("√{x}")?, gold(r"\sqrt{x}")?);
    assert_eq!(gold("√x")?, r#"(sqrt (atom Ord "x"))"#);
    assert_eq!(gold("√x^2")?, gold(r"\sqrt{x}^2")?);
    assert_eq!(gold("√{x^2}")?, gold(r"\sqrt{x^2}")?);
    Ok(())
}

#[test]
fn radical_without_argument_is_malformed() {
    assert!(matches!(
        parse("√"),
        Err(ParseError::Malformed(m)) if m == "missing argument"
    ));
}

#[test]
fn superscript_digits_letters_and_runs() -> Result<(), ParseError> {
    assert_eq!(gold("x²")?, gold("x^{2}")?);
    assert_eq!(gold("x³")?, gold("x^{3}")?);
    assert_eq!(gold("xⁿ")?, gold("x^{n}")?);
    assert_eq!(gold("x⁻¹")?, gold("x^{-1}")?);
    assert_eq!(gold("x²³")?, gold("x^{23}")?);
    assert_eq!(gold("x²⁻¹")?, gold("x^{2-1}")?);
    assert_eq!(gold("x²³ⁿ⁻¹")?, gold("x^{23n-1}")?);
    assert_eq!(gold("f'²")?, gold("f'^2")?);
    Ok(())
}

#[test]
fn superscript_without_base_is_malformed() {
    assert!(matches!(
        parse("²"),
        Err(ParseError::Malformed(m)) if m == "script has no base"
    ));
}

#[test]
fn unicode_minus_is_bin() -> Result<(), ParseError> {
    let minus = "\u{2212}";
    assert_eq!(gold(minus)?, gold("-")?);
    assert_eq!(gold(minus)?, r#"(atom Bin "-")"#);
    assert_eq!(gold(&format!("{minus}b"))?, gold("-b")?);
    Ok(())
}

#[test]
fn prime_flag_matches_prime_command_and_keeps_the_nest() -> Result<(), ParseError> {
    let on = flag("prime");
    assert_eq!(gold_with("f'", &on)?, gold(r"f^{\prime}")?);
    assert_eq!(gold_with("f''", &on)?, gold(r"f^{\prime\prime}")?);
    let nested = r#"(sup (atom Ord "f") (row (row (symbol prime) (symbol prime)) (symbol prime)))"#;
    let flat = r#"(sup (atom Ord "f") (row (symbol prime) (symbol prime) (symbol prime)))"#;
    assert_eq!(gold_with("f'''", &on)?, nested);
    assert_ne!(gold_with("f'''", &on)?, flat);
    assert_eq!(
        gold("f'''")?,
        r#"(sup (atom Ord "f") (row (row (atom Ord "′") (atom Ord "′")) (atom Ord "′")))"#
    );
    Ok(())
}
