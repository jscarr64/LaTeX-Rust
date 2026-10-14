//! Grammar-built property test for text mode (documents/TEXT_MODE_SPEC.md,
//! section 3 "Property tests"). Properties:
//!
//! (a) `parse` never panics.
//! (b) accepted `s`: `parse(print(parse(s))) == parse(s)`.
//! (c) `print` is idempotent.
//! (d) a generated run in canonical form (spec 2.5) parses and prints back
//!     byte for byte.
//! (e) an error construct injected into a valid body gives its typed error.
//!
//! The RNG seed is fixed. 4,096 cases by default;
//! `LATEX_RUST_TEXT_CASES` overrides the count.

use latex_rust::{parse, ParseError};
use proptest::prelude::*;
use proptest::test_runner::{
    Config, FileFailurePersistence, RngAlgorithm, TestCaseError, TestRng, TestRunner,
};

#[derive(Clone, Debug)]
enum Item {
    Str(String),
    Space(&'static str),
    Esc(char),
    Ctrl(&'static str),
    Tilde,
    /// Source name, canonical print.
    Sym(&'static str, &'static str),
    Group(Vec<Item>),
    /// Source command, canonical command.
    Nested(&'static str, &'static str, Vec<Item>),
    /// Math source, `$` form when true.
    Math(&'static str, bool),
}

const STR_CHARS: &[char] = &[
    'a', 'b', 'x', 'Z', '0', '9', '(', ')', '[', ']', '\'', '?', '!', '.', ',', ';', ':', '+', '-',
    '=', '<', '>', '|', '/', '@', '"', '*', '\u{e9}', '\u{3b1}', '\0',
];

const MATH: &[&str] = &[
    "x",
    "(a)'",
    "(x)^2",
    "a_1",
    "\\frac{1}{2}",
    "(\\text{)})'",
    "\\{a\\}",
    "\\text{b c}",
];

fn leaf() -> impl Strategy<Value = Item> {
    prop_oneof![
        4 => proptest::collection::vec(proptest::sample::select(STR_CHARS), 1..4)
            .prop_map(|v| Item::Str(v.into_iter().collect())),
        2 => proptest::sample::select(&[" ", "  ", "\t", "\n", " \n "][..]).prop_map(Item::Space),
        2 => proptest::sample::select(&['{', '}', '$', '%', '&', '#', '_'][..]).prop_map(Item::Esc),
        1 => proptest::sample::select(&["\\ ", "\\\t", "\\\n"][..]).prop_map(Item::Ctrl),
        1 => Just(Item::Tilde),
        1 => proptest::sample::select(
            &[
                ("ldots", "\\ldots{}"),
                ("dots", "\\ldots{}"),
                ("quad", "\\quad{}"),
                ("qquad", "\\qquad{}"),
                ("textbackslash", "\\textbackslash{}"),
                ("textasciicircum", "\\textasciicircum{}"),
                ("textasciitilde", "\\textasciitilde{}"),
            ][..]
        )
        .prop_map(|(a, b)| Item::Sym(a, b)),
        1 => (proptest::sample::select(MATH), any::<bool>()).prop_map(|(m, d)| Item::Math(m, d)),
    ]
}

fn body() -> impl Strategy<Value = Vec<Item>> {
    let item = leaf().prop_recursive(3, 24, 4, |inner| {
        prop_oneof![
            3 => inner.clone(),
            1 => proptest::collection::vec(inner.clone(), 0..4).prop_map(Item::Group),
            1 => (
                proptest::sample::select(
                    &[
                        ("text", "text"),
                        ("mbox", "text"),
                        ("textrm", "textrm"),
                        ("textbf", "textbf"),
                        ("textit", "textit"),
                    ][..]
                ),
                proptest::collection::vec(inner, 0..4),
            )
                .prop_map(|((s, c), v)| Item::Nested(s, c, v)),
        ]
    });
    proptest::collection::vec(item, 0..6)
}

fn source(items: &[Item], out: &mut String) {
    for it in items {
        match it {
            Item::Str(s) => out.push_str(s),
            Item::Space(s) | Item::Ctrl(s) => out.push_str(s),
            Item::Esc(c) => {
                out.push('\\');
                out.push(*c);
            }
            Item::Tilde => out.push('~'),
            Item::Sym(name, _) => {
                out.push('\\');
                out.push_str(name);
                out.push_str("{}");
            }
            Item::Group(v) => {
                out.push('{');
                source(v, out);
                out.push('}');
            }
            Item::Nested(cmd, _, v) => {
                out.push('\\');
                out.push_str(cmd);
                out.push('{');
                source(v, out);
                out.push('}');
            }
            Item::Math(m, true) => {
                out.push('$');
                out.push_str(m);
                out.push('$');
            }
            Item::Math(m, false) => {
                out.push_str("\\(");
                out.push_str(m);
                out.push_str("\\)");
            }
        }
    }
}

/// Spec 2.4/2.5: groups spliced, space runs collapsed, chars escaped.
#[derive(PartialEq)]
enum Canon {
    Chars(String),
    Space,
    Other(String),
}

fn canon_items(items: &[Item], out: &mut Vec<Canon>) {
    for it in items {
        match it {
            Item::Str(s) => push_chars(out, s),
            Item::Esc(c) => push_chars(out, &c.to_string()),
            Item::Sym("textbackslash", _) => push_chars(out, "\\"),
            Item::Sym("textasciicircum", _) => push_chars(out, "^"),
            Item::Sym("textasciitilde", _) => push_chars(out, "~"),
            Item::Space(_) => {
                if out.last() != Some(&Canon::Space) {
                    out.push(Canon::Space);
                }
            }
            Item::Ctrl(_) => out.push(Canon::Other("\\ ".into())),
            Item::Tilde => out.push(Canon::Other("~".into())),
            Item::Sym(_, print) => out.push(Canon::Other((*print).into())),
            Item::Group(v) => canon_items(v, out),
            Item::Nested(_, cmd, v) => out.push(Canon::Other(format!(
                "\\{cmd} {{{}}}",
                canon_run(v)
            ))),
            Item::Math(m, _) => {
                let printed = parse(m).map(|n| n.to_latex()).unwrap_or_default();
                out.push(Canon::Other(format!("\\({printed}\\)")));
            }
        }
    }
}

fn push_chars(out: &mut Vec<Canon>, s: &str) {
    if let Some(Canon::Chars(prev)) = out.last_mut() {
        prev.push_str(s);
    } else {
        out.push(Canon::Chars(s.into()));
    }
}

fn canon_run(items: &[Item]) -> String {
    let mut v = Vec::new();
    canon_items(items, &mut v);
    let mut out = String::new();
    for c in v {
        match c {
            Canon::Chars(s) => {
                for ch in s.chars() {
                    match ch {
                        '\\' => out.push_str("\\textbackslash{}"),
                        '^' => out.push_str("\\textasciicircum{}"),
                        '~' => out.push_str("\\textasciitilde{}"),
                        '{' | '}' | '$' | '%' | '&' | '#' | '_' => {
                            out.push('\\');
                            out.push(ch);
                        }
                        _ => out.push(ch),
                    }
                }
            }
            Canon::Space => out.push(' '),
            Canon::Other(s) => out.push_str(&s),
        }
    }
    out
}

fn wrap(ctx: u8, x: &str) -> String {
    match ctx {
        0 => x.to_string(),
        1 => format!("({x})"),
        2 => format!("({x})'"),
        3 => format!("{x}^2"),
        4 => format!("{{(}}{x}{{)}}"),
        _ => format!("{x}\\ "),
    }
}

fn runner() -> TestRunner {
    let cases = std::env::var("LATEX_RUST_TEXT_CASES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(4096);
    let mut config = Config::with_cases(cases);
    config.failure_persistence = Some(Box::new(FileFailurePersistence::Off));
    TestRunner::new_with_rng(config, TestRng::from_seed(RngAlgorithm::ChaCha, &[42; 32]))
}

fn text_input(items: &[Item]) -> String {
    let mut s = String::from("\\text{");
    source(items, &mut s);
    s.push('}');
    s
}

#[test]
fn prop_a_b_c_round_trip() {
    let result = runner().run(&(body(), 0u8..6), |(items, ctx)| {
        let input = wrap(ctx, &text_input(&items));
        let Ok(node) = parse(&input) else {
            return Ok(());
        };
        let printed = node.to_latex();
        let again = parse(&printed)
            .map_err(|e| TestCaseError::fail(format!("{input:?} printed {printed:?}: {e}")))?;
        prop_assert_eq!(&again, &node, "{:?} printed {:?}", input, printed);
        prop_assert_eq!(again.to_latex(), printed);
        Ok(())
    });
    if let Err(e) = result {
        panic!("{e}");
    }
}

#[test]
fn prop_b_valid_bodies_are_accepted() {
    let result = runner().run(&body(), |items| {
        let input = text_input(&items);
        parse(&input).map_err(|e| TestCaseError::fail(format!("{input:?}: {e}")))?;
        Ok(())
    });
    if let Err(e) = result {
        panic!("{e}");
    }
}

#[test]
fn prop_d_canonical_form_is_a_fixed_point() {
    let result = runner().run(&body(), |items| {
        let canon = format!("\\text {{{}}}", canon_run(&items));
        let printed = parse(&canon)
            .map(|n| n.to_latex())
            .map_err(|e| TestCaseError::fail(format!("{canon:?}: {e}")))?;
        prop_assert_eq!(&printed, &canon);
        // The source spelling parses to the same tree as its canonical form.
        prop_assert_eq!(parse(&text_input(&items)), parse(&canon));
        Ok(())
    });
    if let Err(e) = result {
        panic!("{e}");
    }
}

#[test]
fn prop_e_injected_errors_are_typed() {
    let bad = proptest::sample::select(
        &[
            ("\\\\", "M:line break in text"),
            ("\\foo{}", "U:\\foo"),
            ("^", "M:unexpected token ^ in text"),
            ("_", "M:unexpected token _ in text"),
            ("&", "M:unexpected token & in text"),
            ("#", "M:unexpected token # in text"),
            ("\\)", "M:unmatched math shift in text"),
        ][..],
    );
    let result = runner().run(&(body(), any::<prop::sample::Index>(), bad), |(items, at, (inj, want))| {
        let at = at.index(items.len() + 1);
        let mut s = String::from("\\text{");
        source(&items[..at], &mut s);
        s.push_str(inj);
        source(&items[at..], &mut s);
        s.push('}');
        let want = match want.split_once(':') {
            Some(("U", n)) => ParseError::Unknown(n.into()),
            Some((_, m)) => ParseError::Malformed(m.into()),
            None => unreachable!(),
        };
        prop_assert_eq!(parse(&s), Err(want), "{:?}", s);
        Ok(())
    });
    if let Err(e) = result {
        panic!("{e}");
    }
    // Unclosed math at the end of a body.
    let result = runner().run(&(body(), any::<bool>()), |(items, dollar)| {
        let mut s = String::from("\\text{");
        source(&items, &mut s);
        s.push_str(if dollar { "$x" } else { "\\(x" });
        s.push('}');
        prop_assert_eq!(
            parse(&s),
            Err(ParseError::Malformed("unmatched math shift in text".into())),
            "{:?}",
            s
        );
        Ok(())
    });
    if let Err(e) = result {
        panic!("{e}");
    }
}
