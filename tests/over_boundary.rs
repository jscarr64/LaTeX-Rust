//! `\over` and `\choose` are whole control words.
//!
//! Seven commands, six contexts: 42 asserts. `\choose` keeps its six
//! infix contexts. A `{` in front of `\overline` must stay an accent.

use latex_rust::{parse, ParseError};

struct Accent {
    body: &'static str,
    gold: &'static str,
}

const ACCENTS: &[Accent] = &[
    Accent {
        body: r"\overline{x}",
        gold: r#"(accent overline (atom Ord "x"))"#,
    },
    Accent {
        body: r"\overset{a}{b}",
        gold: r#"(overunder (atom Ord "b") (atom Ord "a") _)"#,
    },
    Accent {
        body: r"\overrightarrow{AB}",
        gold: r#"(accent overrightarrow (row (atom Ord "A") (atom Ord "B")))"#,
    },
    Accent {
        body: r"\overleftarrow{AB}",
        gold: r#"(accent overleftarrow (row (atom Ord "A") (atom Ord "B")))"#,
    },
    Accent {
        body: r"\overleftrightarrow{AB}",
        gold: r#"(accent overleftrightarrow (row (atom Ord "A") (atom Ord "B")))"#,
    },
    Accent {
        body: r"\overbrace{x}",
        gold: r#"(accent overbrace (atom Ord "x"))"#,
    },
];

fn assert_gold(input: &str, expect: &str) {
    match parse(input) {
        Ok(node) => {
            let got = node.gold();
            assert_eq!(got, expect, "input: {input}");
        }
        Err(err) => panic!("{input}: {err}"),
    }
}

fn assert_outside_group(input: &str, command: &str) {
    match parse(input) {
        Err(ParseError::Malformed(msg)) => {
            assert!(
                msg.contains(command) && msg.contains("outside a group"),
                "{input}: {msg}"
            );
        }
        Ok(node) => panic!("{input} parsed as {}", node.gold()),
        Err(err) => panic!("{input}: {err}"),
    }
}

#[test]
fn accent_commands_are_not_fractions() {
    let mut n = 0;
    for accent in ACCENTS {
        let cases = [
            (accent.body.to_string(), accent.gold.to_string()),
            (format!("{{{}}}", accent.body), accent.gold.to_string()),
            (
                format!(r"\frac{{{}}}{{2}}", accent.body),
                format!(r#"(frac {} (atom Ord "2"))"#, accent.gold),
            ),
            (
                format!(r"\sqrt{{{}}}", accent.body),
                format!(r"(sqrt {})", accent.gold),
            ),
            (
                format!("x_{{{}}}", accent.body),
                format!(r#"(sub (atom Ord "x") {})"#, accent.gold),
            ),
            (
                format!("x^{{{}}}", accent.body),
                format!(r#"(sup (atom Ord "x") {})"#, accent.gold),
            ),
        ];
        for (input, expect) in cases {
            assert_gold(&input, &expect);
            n += 1;
        }
    }
    assert_eq!(n, 36, "accent contexts");
}

#[test]
fn real_over_is_still_a_fraction() {
    let frac = r#"(frac (atom Ord "a") (atom Ord "b"))"#;
    assert_outside_group(r"a \over b", "\\over");
    assert_gold(r"{a \over b}", frac);
    assert_gold(
        r"\frac{a \over b}{2}",
        &format!(r#"(frac {frac} (atom Ord "2"))"#),
    );
    assert_gold(
        &format!(r"\sqrt{{{frac_src}}}", frac_src = r"a \over b"),
        &format!("(sqrt {frac})"),
    );
    assert_gold(
        &format!("x_{{{}}}", r"a \over b"),
        &format!(r#"(sub (atom Ord "x") {frac})"#),
    );
    assert_gold(
        &format!("x^{{{}}}", r"a \over b"),
        &format!(r#"(sup (atom Ord "x") {frac})"#),
    );
}

#[test]
fn choose_is_still_a_binom() {
    let binom = r#"(delim ( (frac (atom Ord "n") (atom Ord "k")) ))"#;
    assert_outside_group(r"n \choose k", "\\choose");
    assert_gold(r"{n \choose k}", binom);
    assert_gold(
        r"\frac{n \choose k}{2}",
        &format!(r#"(frac {binom} (atom Ord "2"))"#),
    );
    assert_gold(r"\sqrt{n \choose k}", &format!("(sqrt {binom})"));
    assert_gold(
        &format!("x_{{{}}}", r"n \choose k"),
        &format!(r#"(sub (atom Ord "x") {binom})"#),
    );
    assert_gold(
        &format!("x^{{{}}}", r"n \choose k"),
        &format!(r#"(sup (atom Ord "x") {binom})"#),
    );
}
