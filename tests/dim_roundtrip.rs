//! A dimension prints so that it reads back as the identical `Dim`.
//!
//! `\hspace{…}` and `\rule{…}{…}` print their `Dim` as text. The print cut a
//! decimal off after 24 digits, so a longer decimal (25 digits, or a mu value
//! such as `1mu`, which is 1/18 em and never ends) reparsed to another number
//! and `parse(print(parse(x))) != parse(x)` (fuzz r16long and r17long,
//! `roundtrip-dim-long-a` and `-b`).
//!
//! - a table of crash inputs (the two fuzz seeds, their minimized forms, and
//!   the by-hand cases around them), each checked for the round trip, a stable
//!   print, and where given the exact print;
//! - a property over the whole dimension grammar: 1 to 60 digits, leading and
//!   trailing zeros, with and without a point, every unit the parser accepts.
//!   4,096 cases by default; `LATEX_RUST_DIM_CASES` overrides the count.

use std::cell::Cell;

use latex_rust::parse;
use proptest::prelude::*;
use proptest::test_runner::{
    Config, FileFailurePersistence, RngAlgorithm, TestCaseError, TestRng, TestRunner,
};

/// `Err` names the first broken promise for an accepted `src`; a rejected
/// `src` passes (a typed error is allowed for a number that does not fit).
fn check(src: &str) -> Result<(), String> {
    let Ok(tree) = parse(src) else {
        return Ok(());
    };
    let printed = tree.to_latex();
    let again = parse(&printed).map_err(|e| format!("{src:?} printed {printed:?}: {e}"))?;
    if again != tree {
        return Err(format!(
            "{src:?} printed {printed:?}, which parses to {again:?} instead of {tree:?}"
        ));
    }
    let second = again.to_latex();
    if second != printed {
        return Err(format!("{src:?} prints {printed:?} then {second:?}"));
    }
    Ok(())
}

fn seed(name: &str) -> String {
    let path = format!("{}/fuzz/corpus/parse/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

#[test]
fn the_long_dim_seeds_round_trip() {
    for name in [
        "roundtrip-dim-long-a",
        "roundtrip-dim-long-b",
        "roundtrip-dim-long-a-min",
        "roundtrip-dim-long-b-min",
    ] {
        let src = seed(name);
        assert!(parse(&src).is_ok(), "{name} must parse");
        if let Err(msg) = check(&src) {
            panic!("{name}: {msg}");
        }
    }
}

/// (input, exact print). An empty print means only the round trip is checked.
const TABLE: &[(&str, &str)] = &[
    // 25 digits: the first length the old print cut short.
    (
        "\\hspace{0.1234567890123456789012345em}",
        "\\hspace{0.1234567890123456789012345em}",
    ),
    (
        "\\rule{2}{.1662322688380342688380341}",
        "\\rule{0em}{0.1662322688380342688380341em}",
    ),
    (
        "{\\rule{2}{00000.166232268034181866232268034181}}",
        "\\rule{0em}{0.166232268034181866232268034181em}",
    ),
    // 38 digits after the point is the longest a Dim holds.
    (
        "\\hspace{0.12345678901234567890123456789012345678em}",
        "\\hspace{0.12345678901234567890123456789012345678em}",
    ),
    (
        "\\hspace{170141183460469231731687303715884105727em}",
        // `.0` after 39 digits would overflow the reader, so the print has none.
        "\\hspace{170141183460469231731687303715884105727em}",
    ),
    // Trailing zeros past the point do not change the value or the print.
    (
        "\\hspace{0.50000000000000000000000000000em}",
        "\\hspace{0.5em}",
    ),
    ("\\hspace{-0.00000000000000000000000000001}", ""),
    ("\\hspace{+007.250}", "\\hspace{7.25em}"),
    // mu is 1/18 em: not a finite decimal in em, so it prints in mu.
    ("\\hspace{1mu}", "\\hspace{1.0mu}"),
    ("\\hspace{3mu}", "\\hspace{3.0mu}"),
    ("\\hspace{0.5mu}", "\\hspace{0.5mu}"),
    ("\\hspace{18mu}", "\\hspace{1.0em}"),
    ("\\hspace{9mu}", "\\hspace{0.5em}"),
    ("\\hspace{2.123456789012345678901234567mu}", ""),
    ("\\rule{1mu}{7mu}", "\\rule{0em}{7.0mu}"),
    ("\\hspace{-5mu}", "\\hspace{-5.0mu}"),
    // pt and bp are tenths of an em here.
    ("\\hspace{3pt}", "\\hspace{0.3em}"),
    ("\\hspace{0.1234567890123456789012345pt}", ""),
    // 5.6e-39 em: 39 digits, which the reader cannot hold, so the print keeps pt.
    (
        "\\hspace{.00000000000000000000000000000000000056pt}",
        "\\hspace{0.00000000000000000000000000000000000056pt}",
    ),
    ("\\rule{1bp}{1234567890123456789012345.6789bp}", ""),
];

#[test]
fn long_and_mu_dimensions_round_trip() {
    for &(src, want) in TABLE {
        if let Err(msg) = check(src) {
            panic!("{msg}");
        }
        if !want.is_empty() {
            let printed = match parse(src) {
                Ok(tree) => tree.to_latex(),
                Err(e) => panic!("{src:?} did not parse: {e}"),
            };
            // The printer writes a space after a command name (`\\hspace {`).
            assert_eq!(printed.replacen(' ', "", 1), want, "print of {src:?}");
        }
    }
}

/// A sweep with no randomness, over the lengths where the reader's `i128`
/// runs out: `1` and `9` runs of every length with the point at every place,
/// and `2^-k` (the decimals with the most digits per bit), in every unit.
#[test]
fn sweep_digit_runs_and_powers_of_one_half() {
    let mut texts = Vec::new();
    for len in 1..=45usize {
        for digit in ['1', '9'] {
            let run: String = std::iter::repeat_n(digit, len).collect();
            texts.push(run.clone());
            for at in 0..=len {
                texts.push(format!("{}.{}", &run[..at], &run[at..]));
            }
            texts.push(format!("1{}", "0".repeat(len)));
        }
    }
    let mut five = 1u128;
    for k in 1..=55usize {
        five *= 5;
        let digits = five.to_string();
        texts.push(format!(
            "0.{}{digits}",
            "0".repeat(k.saturating_sub(digits.len()))
        ));
    }
    for text in &texts {
        for unit in ["", "em", "mu", "pt", "bp"] {
            for src in [
                format!("\\hspace{{{text}{unit}}}"),
                format!("\\rule{{1}}{{-{text}{unit}}}"),
            ] {
                if let Err(msg) = check(&src) {
                    panic!("{msg}");
                }
            }
        }
    }
}

fn runner() -> TestRunner {
    let cases = std::env::var("LATEX_RUST_DIM_CASES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(4096);
    let mut config = Config::with_cases(cases);
    config.failure_persistence = Some(Box::new(FileFailurePersistence::Off));
    TestRunner::new_with_rng(config, TestRng::from_seed(RngAlgorithm::ChaCha, &[42; 32]))
}

/// A dimension: optional sign, `len` digits (zeros are common), a point
/// anywhere in them or none, a unit the parser accepts (or none), with or
/// without a space before the unit.
fn dim_text(len: std::ops::RangeInclusive<usize>) -> impl Strategy<Value = String> {
    let digit = prop_oneof![3 => Just('0'), 7 => proptest::char::range('0', '9')];
    (
        proptest::sample::select(&["", "+", "-"][..]),
        proptest::collection::vec(digit, len),
        any::<bool>(),
        0usize..=60,
        proptest::sample::select(&["", "em", "mu", "pt", "bp", " em", " mu"][..]),
    )
        .prop_map(|(sign, digits, point, at, unit)| {
            let all: String = digits.into_iter().collect();
            let at = at % (all.len() + 1);
            let number = if point {
                format!("{}.{}", &all[..at], &all[at..])
            } else {
                all
            };
            format!("{sign}{number}{unit}")
        })
}

/// `\hspace{d}` and `\rule{w}{d}`, the two places a parse produces a `Dim`.
fn dim_command(len: std::ops::RangeInclusive<usize>) -> impl Strategy<Value = String> {
    (dim_text(len.clone()), dim_text(len), any::<bool>()).prop_map(|(a, b, hspace)| {
        if hspace {
            format!("\\hspace{{{a}}}")
        } else {
            format!("\\rule{{{a}}}{{{b}}}")
        }
    })
}

#[test]
fn prop_dimension_text_round_trips_and_prints_once() {
    let mut runner = runner();
    if let Err(e) = runner.run(&dim_command(1..=60), |src| {
        check(&src).map_err(TestCaseError::fail)
    }) {
        panic!("{e}");
    }
}

/// The same property at the edge of what a `Dim` holds (`i128` is 39 digits,
/// a point-and-unit scale by 10 or 18 overflows one digit sooner): this is
/// where `X.0` stopped reading back, and where em stops being the unit.
#[test]
fn prop_dimension_text_near_the_i128_limit() {
    let mut runner = runner();
    if let Err(e) = runner.run(&dim_command(34..=41), |src| {
        check(&src).map_err(TestCaseError::fail)
    }) {
        panic!("{e}");
    }
}

/// The other place a decimal is read into a `Dim`: color components, which
/// are rounded to a byte (so the tree holds no `Dim`) and print as hex. A long
/// decimal must give the same color after the round trip, or a typed error.
#[test]
fn prop_color_components_with_long_decimals_round_trip() {
    let part = (
        proptest::collection::vec(proptest::char::range('0', '9'), 1..=45),
        0usize..=45,
    )
        .prop_map(|(digits, at)| {
            let all: String = digits.into_iter().collect();
            let at = at % (all.len() + 1);
            format!("{}.{}", &all[..at], &all[at..])
        });
    let strategy = (
        proptest::sample::select(&["\\color", "\\textcolor"][..]),
        proptest::sample::select(&["rgb", "cmyk", "gray"][..]),
        proptest::collection::vec(part, 4),
    )
        .prop_map(|(cmd, model, parts)| {
            let n = match model {
                "rgb" => 3,
                "cmyk" => 4,
                _ => 1,
            };
            format!("{cmd}[{model}]{{{}}}{{x}}", parts[..n].join(","))
        });
    let mut runner = runner();
    let parsed = Cell::new(0u32);
    let total = Cell::new(0u32);
    let outcome = runner.run(&strategy, |src| {
        total.set(total.get() + 1);
        parsed.set(parsed.get() + u32::from(parse(&src).is_ok()));
        check(&src).map_err(TestCaseError::fail)
    });
    if let Err(e) = outcome {
        panic!("{e}");
    }
    let (parsed, total) = (parsed.get(), total.get());
    assert!(parsed * 4 > total, "only {parsed} of {total} parsed");
}

/// The property must not pass because everything is rejected.
#[test]
fn most_generated_dimensions_parse() {
    let mut runner = runner();
    let parsed = Cell::new(0u32);
    let total = Cell::new(0u32);
    let outcome = runner.run(&dim_command(1..=60), |src| {
        total.set(total.get() + 1);
        parsed.set(parsed.get() + u32::from(parse(&src).is_ok()));
        Ok(())
    });
    assert!(outcome.is_ok());
    let (parsed, total) = (parsed.get(), total.get());
    assert!(parsed * 2 > total, "only {parsed} of {total} parsed");
}
