//! Empty input and bare scripts are `ParseError::Malformed`.
//!
//! `parse("")` and whitespace used to be an empty row. `\frac{}{}` used to
//! be a fraction of two empty rows. A bare prime used to be a superscript
//! on that empty row. `^2` and `^{2}` used to be a superscript on that empty
//! row. `_i` used to be a subscript on it. `'^2` is a prime with no base.
//! A bare `^` is already a missing argument.

use latex_rust::{parse, ParseError};

#[test]
fn empty_and_bare_scripts_are_malformed() {
    let inputs = [
        "",
        " ",
        "\t",
        "\n",
        r"\frac{}{}",
        r"\frac{ }{ }",
        "^",
        "'",
        "^2",
        "^{2}",
        "_i",
        "'^2",
    ];
    let mut problems = Vec::new();
    for input in inputs {
        match parse(input) {
            Err(ParseError::Malformed(_)) => {}
            Ok(node) => problems.push(format!("{input:?} parsed as {}", node.gold())),
            Err(err) => problems.push(format!("{input:?}: {err}")),
        }
    }
    assert!(
        problems.is_empty(),
        "expected Malformed:\n{}",
        problems.join("\n")
    );
}
