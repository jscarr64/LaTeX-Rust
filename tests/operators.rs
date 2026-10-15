//! Bare operators are off unless `bare_operators` is set.

mod common;

use common::{flag, gold, gold_with};
use latex_rust::ParseError;

/// Names that `\sin`-style commands already produce, and whether the node
/// carries the limits flag.
const OPERATORS: &[(&str, bool)] = &[
    ("sin", false),
    ("cos", false),
    ("tan", false),
    ("cot", false),
    ("sec", false),
    ("csc", false),
    ("arcsin", false),
    ("arccos", false),
    ("arctan", false),
    ("sinh", false),
    ("cosh", false),
    ("tanh", false),
    ("coth", false),
    ("log", false),
    ("ln", false),
    ("lg", false),
    ("exp", false),
    ("limsup", false),
    ("liminf", false),
    ("sup", false),
    ("inf", false),
    ("max", false),
    ("min", false),
    ("det", false),
    ("dim", false),
    ("ker", false),
    ("deg", false),
    ("gcd", false),
    ("lcm", false),
    ("Pr", false),
    ("arg", false),
    ("coprod", true),
    ("bigcup", true),
    ("bigcap", true),
    ("bigsqcup", true),
    ("bigvee", true),
    ("bigwedge", true),
    ("bigoplus", true),
    ("bigotimes", true),
    ("biguplus", true),
];

#[test]
fn bare_words_match_their_commands() -> Result<(), ParseError> {
    let on = flag("bare");
    assert_eq!(gold_with("sin(x)", &on)?, gold(r"\sin(x)")?);
    assert_eq!(gold_with("sin x", &on)?, gold(r"\sin x")?);
    assert_eq!(gold_with("zeta(s,a)", &on)?, gold(r"\zeta(s,a)")?);
    assert_eq!(gold_with("zeta(s, a)", &on)?, gold(r"\zeta(s, a)")?);
    let cases_bare = r"\begin{cases} x & x < 0 \\ 0 & otherwise \end{cases}";
    let cases_text = r"\begin{cases} x & x < 0 \\ 0 & \text{otherwise} \end{cases}";
    assert_eq!(gold_with(cases_bare, &on)?, gold(cases_text)?);
    assert_eq!(gold_with("sinh", &on)?, gold(r"\sinh")?);
    assert_ne!(gold_with("sinh", &on)?, gold_with("sin h", &on)?);
    assert_eq!(gold_with("xsin", &on)?, gold("xsin")?);
    assert_eq!(gold_with("sinx", &on)?, gold("sinx")?);
    assert_eq!(gold_with("sinω", &on)?, gold(r"\sin ω")?);
    assert_eq!(gold_with("ω", &on)?, gold("ω")?);
    assert_eq!(gold_with("Pr", &on)?, gold(r"\Pr")?);
    assert_eq!(gold_with("bigcup", &on)?, gold(r"\bigcup")?);
    Ok(())
}

#[test]
fn operator_table_matches_commands() -> Result<(), ParseError> {
    let on = flag("bare");
    for (name, limits) in OPERATORS {
        let bare_gold = gold_with(name, &on)?;
        let command = format!("\\{name}");
        assert_eq!(bare_gold, gold(&command)?, "{name}");
        if *limits {
            assert!(bare_gold.contains("limits"), "{name}");
        }
    }
    assert_eq!(gold_with("zeta", &on)?, gold(r"\zeta")?);
    Ok(())
}

#[test]
fn longest_match_does_not_split_a_word() -> Result<(), ParseError> {
    let on = flag("bare");
    assert_eq!(gold_with("infty", &on)?, gold("infty")?);
    assert_eq!(gold_with("inf", &on)?, gold(r"\inf")?);
    assert_eq!(gold_with("pr", &on)?, gold("pr")?);
    assert_eq!(
        gold("otherwise")?,
        gold_with("otherwise", &on)?,
        "otherwise outside cases stays letters"
    );
    Ok(())
}

#[test]
fn flags_off_keep_todays_letter_rows() -> Result<(), ParseError> {
    assert_eq!(
        gold("sin(x)")?,
        r#"(row (atom Ord "s") (atom Ord "i") (atom Ord "n") (atom Open "(") (atom Ord "x") (atom Close ")"))"#
    );
    assert_eq!(
        gold("zeta(s,a)")?,
        r#"(row (atom Ord "z") (atom Ord "e") (atom Ord "t") (atom Ord "a") (atom Open "(") (atom Ord "s") (atom Punct ",") (atom Ord "a") (atom Close ")"))"#
    );
    assert_eq!(
        gold("sinh")?,
        r#"(row (atom Ord "s") (atom Ord "i") (atom Ord "n") (atom Ord "h"))"#
    );
    assert_eq!(
        gold(r"\begin{cases} 0 & otherwise \end{cases}")?,
        r#"(matrix cases ((atom Ord "0") (row (atom Ord "o") (atom Ord "t") (atom Ord "h") (atom Ord "e") (atom Ord "r") (atom Ord "w") (atom Ord "i") (atom Ord "s") (atom Ord "e"))))"#
    );
    Ok(())
}
