//! ASCII digraphs and unicode relations, at top level and inside `cases`.

mod common;

use common::{flag, gold, gold_with};
use latex_rust::ParseError;

fn cases(cond: &str) -> String {
    format!(r"\begin{{cases}} 1 & {cond} \end{{cases}}")
}

#[test]
fn twelve_relation_asserts() -> Result<(), ParseError> {
    let ascii = flag("ascii");
    let unicode = flag("unicode");
    // Top-level ASCII digraphs.
    assert_eq!(gold_with("x <= 0", &ascii)?, gold(r"x \leq 0")?);
    assert_eq!(gold_with("x >= 0", &ascii)?, gold(r"x \geq 0")?);
    assert_eq!(gold_with("x != 0", &ascii)?, gold(r"x \neq 0")?);
    // The same three inside cases.
    assert_eq!(
        gold_with(&cases("x <= 0"), &ascii)?,
        gold(&cases(r"x \leq 0"))?
    );
    assert_eq!(
        gold_with(&cases("x >= 0"), &ascii)?,
        gold(&cases(r"x \geq 0"))?
    );
    assert_eq!(
        gold_with(&cases("x != 0"), &ascii)?,
        gold(&cases(r"x \neq 0"))?
    );
    // Unicode relations, top level and inside cases.
    assert_eq!(gold_with("x ≤ 0", &unicode)?, gold(r"x \leq 0")?);
    assert_eq!(gold_with("x ≥ 0", &unicode)?, gold(r"x \geq 0")?);
    assert_eq!(gold_with("x ≠ 0", &unicode)?, gold(r"x \neq 0")?);
    assert_eq!(
        gold_with(&cases("x ≤ 0"), &unicode)?,
        gold(&cases(r"x \leq 0"))?
    );
    assert_eq!(
        gold_with(&cases("x ≥ 0"), &unicode)?,
        gold(&cases(r"x \geq 0"))?
    );
    assert_eq!(
        gold_with(&cases("x ≠ 0"), &unicode)?,
        gold(&cases(r"x \neq 0"))?
    );
    Ok(())
}

#[test]
fn command_aliases_already_match() -> Result<(), ParseError> {
    assert_eq!(gold(r"\le")?, gold(r"\leq")?);
    assert_eq!(gold(r"\ge")?, gold(r"\geq")?);
    assert_eq!(gold(r"\ne")?, gold(r"\neq")?);
    Ok(())
}

#[test]
fn flags_off_keep_todays_relation_atoms() -> Result<(), ParseError> {
    assert_eq!(gold("<=")?, r#"(row (atom Rel "<") (atom Rel "="))"#);
    assert_eq!(gold(">=")?, r#"(row (atom Rel ">") (atom Rel "="))"#);
    assert_eq!(gold("!=")?, r#"(row (atom Punct "!") (atom Rel "="))"#);
    assert_eq!(gold("≤")?, r#"(atom Rel "≤")"#);
    assert_eq!(gold("≥")?, r#"(atom Rel "≥")"#);
    assert_eq!(gold("≠")?, r#"(atom Rel "≠")"#);
    Ok(())
}

#[test]
fn spaced_digraph_stays_two_atoms() -> Result<(), ParseError> {
    assert_eq!(gold_with("x < = 0", &flag("ascii"))?, gold("x < = 0")?);
    Ok(())
}
