//! Text mode per documents/TEXT_MODE_SPEC.md: crash-seed regressions, the
//! typed errors of section 2.3, and the silent wrong results of section 1.

use latex_rust::{parse, ParseError};

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

fn malformed(msg: &str) -> ParseError {
    ParseError::Malformed(msg.into())
}

fn unknown(name: &str) -> ParseError {
    ParseError::Unknown(name.into())
}

const SEEDS: &[&str] = &[
    "control-space-comment",
    "prime-run-34",
    "roundtrip-g1-41ad61cb",
    "roundtrip-g1-a83b1d95",
    "roundtrip-g1-b71856d2",
    "roundtrip-g1-ba56776f",
    "roundtrip-g1-c23638fb",
    "roundtrip-g1-db72c70e",
    "roundtrip-g1-e13c26ab",
    "roundtrip-g1-f7ab6a10",
    "roundtrip-g2-6da362ae",
    "roundtrip-g3-203b89fe",
    "roundtrip-g3-51652d7b",
    "roundtrip-g4-3d55a54a",
    "roundtrip-g4-3d80a1bd",
    "roundtrip-g4-63b40237",
    "roundtrip-g4-84931cd8",
    "roundtrip-g4-9d223dd5",
    "roundtrip-g4-b6f7760a",
    "roundtrip-g4-b9ede5d3",
    "roundtrip-g4-d3dde77a",
    "roundtrip-g4-d807c16e",
    "roundtrip-g4-fdf2f41c",
    "roundtrip-g5-64e3983c",
    "roundtrip-g5-d3f2c752",
    "roundtrip-g5-f091efdf",
    "roundtrip-g6-5c1df81b",
    "roundtrip-g6-75154f11",
    "roundtrip-g6-85a23c0a",
    "roundtrip-g6-bb32f5b3",
    "roundtrip-g7-2c53e50e",
    "roundtrip-g7-62a0555f",
    "roundtrip-g7-8569dea9",
    "roundtrip-g8-7b95ab68",
    "roundtrip-g8-f2ab310f",
    "roundtrip-g9-1410dba0",
    "roundtrip-canon-big-paren",
    "roundtrip-canon-fence-open",
    "roundtrip-canon-option-bracket",
    "roundtrip-canon-option-inner-paren",
    "roundtrip-canon-option-paren",
    "roundtrip-canon-option-paren-open",
    "roundtrip-canon-prime-symbol",
    "roundtrip-canon-primed-close",
    "roundtrip-canon-primed-close-accent",
    "roundtrip-canon-primed-open",
    "roundtrip-canon-primed-subsup",
    "roundtrip-canon-sqrt-bracket",
    "roundtrip-canon-text-math-close",
    "roundtrip-canon-text-math-fence",
    "roundtrip-depth-03a818ac",
    "roundtrip-depth-1d3eba56",
    "roundtrip-depth-2831ee77",
    "roundtrip-depth-2ab0f7cc",
    "roundtrip-depth-32fcc59b",
    "roundtrip-depth-3896821d",
    "roundtrip-depth-426209ab",
    "roundtrip-depth-46a6374d",
    "roundtrip-depth-46dc6a92",
    "roundtrip-depth-7225d264",
    "roundtrip-depth-8a7759eb",
    "roundtrip-depth-a66a4405",
    "roundtrip-depth-aa588734",
    "roundtrip-depth-cbe83c33",
    "roundtrip-depth-fa46a27f",
    "roundtrip-dim-dot",
    "roundtrip-dim-overflow",
    "roundtrip-perf-paren-row",
];

/// Every crash seed either round-trips (property b and c) or is rejected
/// with a typed error. The seed list is the corpus's named files: the 36
/// text-mode seeds, the 15 depth seeds, the 14 canonical-print seeds and
/// `roundtrip-dim-dot`.
#[test]
fn crash_seeds_round_trip_or_reject() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/fuzz/corpus/parse");
    let mut named = std::fs::read_dir(dir)
        .map(|it| {
            it.filter_map(Result::ok)
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .filter(|n| n.contains('-'))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    named.sort();
    let mut listed = SEEDS.to_vec();
    listed.sort_unstable();
    assert_eq!(named, listed, "seed table out of date");
    for name in SEEDS {
        let bytes = std::fs::read(format!("{dir}/{name}")).unwrap_or_default();
        let Ok(input) = std::str::from_utf8(&bytes) else {
            continue;
        };
        if parse(input).is_ok() {
            round_trip(input);
        }
    }
}

/// Seeds whose verdict the spec fixes.
#[test]
fn crash_seed_verdicts() {
    // G9: `\)` with no `\(` inside text (R5).
    assert_eq!(
        parse("(???????\\text {\\text {\\ }\\)') t{Lambda}Bb) }\\ "),
        Err(malformed("unmatched math shift in text"))
    );
    // G3, G8: same rule.
    assert_eq!(
        parse("(\\text{\\)}\0(BB"),
        Err(malformed("unmatched math shift in text"))
    );
    assert_eq!(
        parse("\\text{\\{\\)}"),
        Err(malformed("unmatched math shift in text"))
    );
    // G1: a bare `#` in text (R8).
    assert_eq!(
        parse("\0\\mbox{\0\0\0\0#Q\0\\$\0\0\0\0\0\0}oxs\u{10}"),
        Err(malformed("unexpected token # in text"))
    );
    // G7: `\over` is not a text command (R3).
    assert_eq!(parse(")\\text{\\over\0{\\{ }}"), Err(unknown("\\over")));
}

/// Section 2.3, one row each.
#[test]
fn typed_errors() {
    let table: &[(&str, ParseError)] = &[
        ("\\text", malformed("expected '{'")),
        ("\\text x", malformed("expected '{'")),
        ("\\text{a\\text x}", malformed("expected '{'")),
        ("\\text{a", malformed("unmatched '{'")),
        ("\\text{a{b}", malformed("unmatched '{'")),
        ("\\text{\\", ParseError::TrailingBackslash),
        ("\\text{a\\\\b}", malformed("line break in text")),
        ("\\text{a\\\\over b}", malformed("line break in text")),
        ("\\text{\\\\{ )}}", malformed("line break in text")),
        ("\\intertext{a\\\\b}", malformed("line break in text")),
        ("\\text{\\foo}", unknown("\\foo")),
        ("\\text{a\\foo b}", unknown("\\foo")),
        ("\\text{{\\rm a}}", unknown("\\rm")),
        ("\\text{{a \\over b}}", unknown("\\over")),
        ("\\text{\\,}", unknown("\\,")),
        ("\\text{\\^}", unknown("\\^")),
        ("\\text{\\[}", unknown("\\[")),
        ("\\text{\\]}", unknown("\\]")),
        ("\\text{a^b}", malformed("unexpected token ^ in text")),
        ("\\text{a_b}", malformed("unexpected token _ in text")),
        ("\\text{a&b}", malformed("unexpected token & in text")),
        ("\\text{a#b}", malformed("unexpected token # in text")),
        ("\\text{\\(x}", malformed("unmatched math shift in text")),
        ("\\text{x\\)}", malformed("unmatched math shift in text")),
        ("\\text{$x}", malformed("unmatched math shift in text")),
        ("\\text{$x\\)}", malformed("unmatched math shift in text")),
    ];
    for (input, want) in table {
        assert_eq!(&parse(input), &Err(want.clone()), "{input:?}");
    }
}

/// Nesting inside text counts toward `max_depth`.
#[test]
fn text_nesting_is_bounded() {
    let deep = format!("{}x{}", "\\text{".repeat(64), "}".repeat(64));
    assert_eq!(
        parse(&deep),
        Err(malformed("input nests deeper than 32 levels"))
    );
    let math = format!("{}x{}", "\\text{\\(".repeat(32), "\\)}".repeat(32));
    assert_eq!(
        parse(&math),
        Err(malformed("input nests deeper than 32 levels"))
    );
    round_trip(&format!("{}x{}", "\\text{".repeat(8), "}".repeat(8)));
}

/// The silent wrong results listed in spec section 1.
#[test]
fn silent_wrong_results_are_fixed() {
    // P3: a group in text is spliced, not rewritten to `)`.
    assert_eq!(parse("\\text{{ )}}"), parse("\\text{ )}"));
    // P11: `\mbox` in text is a nested text run.
    assert_eq!(
        parse("\\text{\\mbox{x}}").map(|n| n.to_latex()),
        Ok("\\text {\\text {x}}".to_string())
    );
    // L2: a tab in text is a space.
    assert_eq!(parse("\\text{a\tb}"), parse("\\text{a b}"));
    assert_eq!(parse("\\text{a\n b}"), parse("\\text{a b}"));
    // D4: `{a}` is a group, `\{a\}` is braces.
    assert_eq!(parse("\\text{{a}}"), parse("\\text{a}"));
    assert_ne!(parse("\\text{{a}}"), parse("\\text{\\{a\\}}"));
    // D2: a control space stays distinct from a space.
    assert_ne!(parse("\\text{a\\ b}"), parse("\\text{a b}"));
    // P8, P9, L3, D1 (`\\`) are typed errors: see `typed_errors`.
    // P8, P9 still apply in math.
    assert_eq!(parse("{\\rm a}"), parse("\\mathrm{a}"));
    assert_eq!(parse("{a \\over b}"), parse("\\frac{a}{b}"));
}

/// New in 3.0.0: math, `~` and text symbols inside text.
#[test]
fn accepted_text_forms() {
    for input in [
        "\\text{$x$}",
        "\\text{a \\(x^2\\) b}",
        "\\text{\\(\\text{y}\\)}",
        "\\text{$\\text{$x$}$}",
        "\\text{a~b}",
        "\\text{\\ldots}",
        "\\text{\\dots x}",
        "\\text{\\textbackslash}",
        "\\text{\\textasciicircum\\textasciitilde}",
        "\\text{\\quad a\\qquad}",
        "\\text{\\textbf{b} \\textit{i}}",
        "\\text{a%comment\nb}",
        "\\text{\\{\\}\\$\\%\\&\\#\\_}",
        "(\\text{)'})",
        "(\\text{(})'",
        "\\intertext{a \\(x\\)}",
    ] {
        round_trip(input);
    }
    assert_eq!(parse("\\text{\\dots}"), parse("\\text{\\ldots}"));
    assert_eq!(parse("\\text{\\ldots   x}"), parse("\\text{\\ldots{}x}"));
    assert_eq!(
        parse("\\text{\\(x\\)}").map(|n| n.to_latex()),
        Ok("\\text {\\(x\\)}".to_string())
    );
    assert_eq!(
        parse("\\text{a\\textbackslash b}").map(|n| n.to_latex()),
        Ok("\\text {a\\textbackslash{}b}".to_string())
    );
}

/// The `(` script lookahead skips brace groups (spec 3.3).
#[test]
fn paren_lookahead_skips_groups() {
    for input in ["({a)'}", "(\\mathrm{a)'})", "(\\text{a)'})"] {
        round_trip(input);
    }
}
