//! Printer brace economy: a reprint never nests deeper than the source.
//!
//! "Depth" is the smallest `ParseOptions::max_depth` that accepts the input.
//! For every accepted `s`, with `t = parse(s)`:
//! `depth(print(t)) <= depth(s)` and `parse(print(t)) == t`.
//!
//! The table holds the 15 fuzz crashes (minimized) that broke this: the
//! reprint was rejected as nesting deeper than 32 levels. Each is also a
//! seed in `fuzz/corpus/parse/roundtrip-depth-*`. The property runs on a fixed seed,
//! 4,096 cases by default; `LATEX_RUST_DEPTH_CASES` overrides the count.

use latex_rust::{parse, parse_with_options, ParseOptions};
use proptest::prelude::*;
use proptest::test_runner::{
    Config, FileFailurePersistence, RngAlgorithm, TestCaseError, TestRng, TestRunner,
};

const LIMIT: usize = 256;

/// Smallest `max_depth` that accepts `s`, or `None` if it never parses.
fn depth(s: &str) -> Option<usize> {
    let ok = |d: usize| parse_with_options(s, &ParseOptions::new().with_max_depth(d)).is_ok();
    if !ok(LIMIT) {
        return None;
    }
    let (mut lo, mut hi) = (0, LIMIT);
    while hi - lo > 1 {
        let mid = (lo + hi) / 2;
        if ok(mid) {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    Some(hi)
}

/// `Err` describes the first broken promise for an accepted `s`.
fn check(s: &str) -> Result<(), String> {
    let Some(src) = depth(s) else {
        return Ok(());
    };
    let tree = parse_with_options(s, &ParseOptions::new().with_max_depth(LIMIT))
        .map_err(|e| format!("{s:?}: {e}"))?
        .0;
    let printed = tree.to_latex();
    let Some(out) = depth(&printed) else {
        return Err(format!("{s:?} printed {printed:?}, which does not parse"));
    };
    if out > src {
        return Err(format!(
            "{s:?} (depth {src}) printed {printed:?} (depth {out})"
        ));
    }
    let again = parse_with_options(&printed, &ParseOptions::new().with_max_depth(LIMIT))
        .map_err(|e| format!("{printed:?}: {e}"))?
        .0;
    if again != tree {
        return Err(format!(
            "{s:?} printed {printed:?}, which parses to another tree"
        ));
    }
    Ok(())
}

/// The minimized fuzz crashes. The name is the seed file.
const CRASHES: &[(&str, &str)] = &[
    ("roundtrip-depth-03a818ac", "lo_{==_{{lo_{=_{==_{=_{={lo_{=_{=_{{lo_\u{0}}<}a}}}}<}}a}}<}}"),
    ("roundtrip-depth-1d3eba56", "h{\\choose\\choose\\choose\\choose\\choose\\choose\\choose\\choose\\choose a\\choose}"),
    ("roundtrip-depth-2831ee77", "\u{1e}{\\choose\\choose\\choose\\choose\\choose\\choose\\choose\\choose\\choose\"\\choose}"),
    ("roundtrip-depth-2ab0f7cc", "{\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over4}"),
    ("roundtrip-depth-32fcc59b", "\u{1e}{\\choose\\choose\\choose\\choose\\choose\\choose\\choose\\choose\\choose\\choose\"}"),
    ("roundtrip-depth-3896821d", "{\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over-\\over}"),
    ("roundtrip-depth-426209ab", "{\\it\\choose\\choose\\choose\\choose\\choose\\choose\\choose\\choose\\choose-o\\choose}"),
    ("roundtrip-depth-46a6374d", "{\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over[\\over}"),
    ("roundtrip-depth-46dc6a92", "{\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over4}"),
    ("roundtrip-depth-7225d264", "\u{1e}{\\choose\\choose\\choose\\choose\\choose\\choose\\choose\\choose\\choose\\choose d}"),
    ("roundtrip-depth-8a7759eb", "{\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over-}"),
    ("roundtrip-depth-a66a4405", "lo_{==_{{lo_{==_{==_{=_{==_{{lo_{==_{={{}<}}}}<}}a}}}}<}}"),
    ("roundtrip-depth-aa588734", "{\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over)\\over}"),
    ("roundtrip-depth-cbe83c33", "{\\it\\choose\\choose\\choose\\choose\\choose\\choose\\choose\\choose\\choose g\\choose}"),
    ("roundtrip-depth-fa46a27f", "{\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over\\over4\\over}"),
];

#[test]
fn crash_seeds_are_in_the_corpus() {
    for (name, input) in CRASHES {
        let path = format!("{}/fuzz/corpus/parse/{name}", env!("CARGO_MANIFEST_DIR"));
        let seed = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
        assert_eq!(&seed, input, "{name}");
    }
}

#[test]
fn crash_seeds_reprint_no_deeper() {
    let failures: Vec<String> = CRASHES
        .iter()
        .filter_map(|(name, input)| check(input).err().map(|e| format!("{name}: {e}")))
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn crash_seeds_reprint_under_default_limit() {
    for (name, input) in CRASHES {
        let Ok(tree) = parse(input) else { continue };
        let printed = tree.to_latex();
        assert!(parse(&printed).is_ok(), "{name}: {printed:?}");
    }
}

/// Small shapes behind each crash family, plus the three printer habits.
#[test]
fn brace_economy_cases() {
    let cases = [
        r"\frac{}{x}",
        r"{\over x}",
        r"{a\over b}",
        r"{ab\over c}",
        r"\frac ab",
        r"\frac{ab}{c}",
        r"{\over\over\over x}",
        r"{a\over b\over c}",
        r"{a\choose b}",
        r"\binom ab",
        r"{\choose\choose\choose d}",
        r"\left(\frac ab\right)",
        r"a_b",
        r"a_{bc}",
        r"a_{b\{}",
        r"a_{b<c}",
        r"lo_{==_{a}<}",
        r"\sqrt a",
        r"\sqrt{a+b}",
        r"x^\frac12",
        r"\frac a\frac bc",
        r"ab<=c",
        r"\sin x",
        r"a\left(b\right)c",
    ];
    let failures: Vec<String> = cases.iter().filter_map(|s| check(s).err()).collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

const LEAVES: &[&str] = &[
    "a",
    "b",
    "x",
    "1",
    "2",
    "+",
    "-",
    "<",
    "=",
    "!",
    "(",
    ")",
    "[",
    "]",
    "\\{",
    "\\}",
    "\\alpha",
    "\\sin",
    "\\infty",
    "'",
    "\\,",
    "\\text{t}",
    "′",
    "x′",
    ".",
];

fn math() -> impl Strategy<Value = String> {
    let leaf = proptest::sample::select(LEAVES).prop_map(str::to_string);
    leaf.prop_recursive(6, 48, 4, |inner| {
        let arg = prop_oneof![
            inner.clone().prop_map(|s| format!("{{{s}}}")),
            proptest::sample::select(&["a", "1", "\\alpha", "-"][..]).prop_map(|s| format!(" {s}")),
        ];
        prop_oneof![
            proptest::collection::vec(inner.clone(), 1..4).prop_map(|v| v.join("")),
            inner.clone().prop_map(|s| format!("{{{s}}}")),
            (arg.clone(), arg.clone()).prop_map(|(a, b)| format!("\\frac{a}{b}")),
            (arg.clone(), arg.clone()).prop_map(|(a, b)| format!("\\binom{a}{b}")),
            (inner.clone(), inner.clone()).prop_map(|(a, b)| format!("{{{a}\\over {b}}}")),
            (inner.clone(), inner.clone()).prop_map(|(a, b)| format!("{{{a}\\choose {b}}}")),
            (inner.clone(), arg.clone()).prop_map(|(a, b)| format!("{a}_{b}")),
            (inner.clone(), arg.clone()).prop_map(|(a, b)| format!("{a}^{b}")),
            arg.clone().prop_map(|a| format!("\\sqrt{a}")),
            arg.prop_map(|a| format!("\\hat{a}")),
            inner.prop_map(|s| format!("\\left({s}\\right)")),
        ]
    })
}

fn runner() -> TestRunner {
    let cases = std::env::var("LATEX_RUST_DEPTH_CASES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(4096);
    let mut config = Config::with_cases(cases);
    config.failure_persistence = Some(Box::new(FileFailurePersistence::Off));
    TestRunner::new_with_rng(config, TestRng::from_seed(RngAlgorithm::ChaCha, &[42; 32]))
}

#[test]
fn prop_reprint_is_no_deeper_and_round_trips() {
    let result = runner().run(&math(), |s| check(&s).map_err(TestCaseError::fail));
    if let Err(e) = result {
        panic!("{e}");
    }
}
