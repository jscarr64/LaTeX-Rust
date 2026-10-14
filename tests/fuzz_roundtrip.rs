//! Fuzz round-trip regressions. Each input is a minimized crash of the
//! `parse` fuzz target: parse → `to_latex` → parse must give the identical
//! tree and a stable print, or the input must be rejected outright.

use latex_rust::parse;

fn round_trip(input: &str) {
    let node = match parse(input) {
        Ok(node) => node,
        Err(err) => panic!("{input:?} did not parse: {err}"),
    };
    let printed = node.to_latex();
    let again = match parse(&printed) {
        Ok(again) => again,
        Err(err) => panic!("reprint {printed:?} of {input:?}: {err}"),
    };
    assert_eq!(again, node, "input {input:?} printed {printed:?}");
    assert_eq!(again.to_latex(), printed, "reprint of {input:?} not stable");
}

/// `$ & _ ^` in text must be escaped, or the reprint stops at the token.
#[test]
fn text_specials_are_escaped() {
    for input in [
        "\\mbox{\\$}",
        "\\text{\\&}",
        "\\mbox{\\$\0}",
        "\\text{\0\\&\0}",
        "\0\\mbox{\\$\0}",
        "\0\\mbox{t\\${}?}l(",
        "\0\\mbox{\0\0\0\0#Q\0\\$\0\0\0\0\0\0}oxs\u{10}",
        "\0k'6\\mbox{\\#Q\0\\$\0\0\0\0\0\0\0\0\0\0}oxs\u{10}",
        "\\text{\\_}",
        "\\text{\\^}",
        "\\operatorname{a\\_b}",
    ] {
        round_trip(input);
    }
}

/// `\` before a newline or tab is a control space, as in TeX.
#[test]
fn control_whitespace_in_text_is_a_space() {
    for input in [
        "\\text{\\\n}",
        "\\text{\\\t}",
        "\\text{a\\\nb}",
        "\\text{\\\r}",
    ] {
        round_trip(input);
    }
}
