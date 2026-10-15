//! Canonical print over the whole math grammar. For every generated `x`
//! that parses, with `t = parse(x)`:
//!
//! - `parse(print(t)) == t`;
//! - `print(parse(print(t))) == print(t)` (the print is a fixed point).
//!
//! `print` is `to_latex_with(options)` for the options the case parses with.
//!
//! The grammar reaches every `MathNode` variant: atoms, symbols, every
//! fraction form, radicals with an index, scripts and primes, `\left…\right`,
//! sized delimiters, matrices and environments with rows, tags and labels,
//! `\substack`, accents and over/under forms, `\cancelto`, text and font
//! styles, spaces and struts, operators and big operators with limits,
//! colors and boxes, phantoms, `\ref`, and `\intertext`. The four parser
//! options vary per case. The RNG seed is fixed; 4,096 cases by default,
//! `LATEX_RUST_CANON_CASES` overrides the count.

use latex_rust::{parse_with_options, ParseOptions};
use proptest::prelude::*;
use proptest::test_runner::{
    Config, FileFailurePersistence, RngAlgorithm, TestCaseError, TestRng, TestRunner,
};

const LEAVES: &[&str] = &[
    "x",
    "y",
    "2",
    "0",
    "+",
    "-",
    "=",
    "<",
    ">",
    "!",
    ",",
    ".",
    ";",
    ":",
    "|",
    "/",
    "*",
    "(",
    ")",
    "[",
    "]",
    "\\{",
    "\\}",
    "\\%",
    "'",
    "f'",
    "f''",
    "α",
    "≤",
    "→",
    "′",
    "x²",
    "√x",
    "−",
    "\\alpha",
    "\\Gamma",
    "\\infty",
    "\\leq",
    "\\to",
    "\\cdot",
    "\\ldots",
    "\\prime",
    "\\sin",
    "\\lim",
    "\\max",
    "\\sum",
    "\\int",
    "\\iint",
    "\\oint",
    "\\prod",
    "\\coprod",
    "\\bigcup",
    "\\operatorname{foo}",
    "\\,",
    "\\:",
    "\\;",
    "\\!",
    "\\quad",
    "\\qquad",
    "\\ ",
    "\\hspace{1em}",
    "\\strut",
    "\\rule{0em}{1em}",
    "\\big(",
    "\\Bigl[",
    "\\biggr)",
    "\\Bigm|",
    "\\text{a b}",
    "\\text{x $y$}",
    "\\mbox{m}",
    "\\mathrm{d}",
    "\\mathbf{v}",
    "\\mathbb{R}",
    "\\mathcal{L}",
    "\\mathfrak{g}",
    "\\boldsymbol{\\beta}",
    "\\mathit{ab}",
    "{\\rm r}",
    "{\\bf b}",
    "\\ref{e1}",
    "\\binom{n}{k}",
    "{\\(}",
    "\\)",
    "{}",
];

fn math() -> impl Strategy<Value = String> {
    let leaf = proptest::sample::select(LEAVES).prop_map(str::to_string);
    leaf.prop_recursive(5, 64, 4, |inner| {
        let arg = prop_oneof![
            3 => inner.clone().prop_map(|s| format!("{{{s}}}")),
            1 => proptest::sample::select(&["a", "1", "\\alpha", "-", "′"][..])
                .prop_map(|s| format!(" {s}")),
        ];
        let pair = (inner.clone(), inner.clone());
        prop_oneof![
            4 => proptest::collection::vec(inner.clone(), 1..4).prop_map(|v| v.join("")),
            2 => inner.clone().prop_map(|s| format!("{{{s}}}")),
            2 => (proptest::sample::select(&["frac", "dfrac", "tfrac", "cfrac", "binom"][..]), arg.clone(), arg.clone())
                .prop_map(|(c, a, b)| format!("\\{c}{a}{b}")),
            1 => pair.clone().prop_map(|(a, b)| format!("{{{a}\\over {b}}}")),
            1 => pair.clone().prop_map(|(a, b)| format!("{{{a}\\choose {b}}}")),
            1 => arg.clone().prop_map(|a| format!("\\sqrt{a}")),
            1 => (inner.clone(), arg.clone()).prop_map(|(i, a)| format!("\\sqrt[{i}]{a}")),
            2 => (inner.clone(), arg.clone()).prop_map(|(b, s)| format!("{{{b}}}_{s}")),
            2 => (inner.clone(), arg.clone()).prop_map(|(b, s)| format!("{{{b}}}^{s}")),
            1 => (arg.clone(), arg.clone()).prop_map(|(s, t)| format!("x_{s}^{t}")),
            1 => inner.clone().prop_map(|s| format!("{{{s}}}''")),
            1 => (proptest::sample::select(&["sum", "prod", "int", "lim", "bigcup", "sin\\limits", "max"][..]), arg.clone(), arg.clone())
                .prop_map(|(op, a, b)| format!("\\{op}_{a}^{b}")),
            1 => (proptest::sample::select(&["(", "[", "\\{", "|", "\\langle", "."][..]), inner.clone(), proptest::sample::select(&[")", "]", "\\}", "|", "\\rangle", "."][..]))
                .prop_map(|(l, s, r)| format!("\\left{l}{s}\\right{r}")),
            1 => pair.clone().prop_map(|(a, b)| format!("\\left({a}\\middle|{b}\\right)")),
            1 => (proptest::sample::select(&["matrix", "pmatrix", "bmatrix", "vmatrix", "cases", "aligned", "array}{cl"][..]), pair.clone())
                .prop_map(|(env, (a, b))| {
                    let name = env.split('}').next().unwrap_or(env);
                    format!("\\begin{{{env}}}{a}&{b}\\\\{b}&{a}\\end{{{name}}}")
                }),
            1 => pair.clone().prop_map(|(a, b)| format!("\\begin{{align}}{a}&={b}\\label{{k}}\\\\{b}&={a}\\tag{{2}}\\\\\\intertext{{so}}{a}\\nonumber\\end{{align}}")),
            1 => pair.clone().prop_map(|(a, b)| format!("\\sum_{{\\substack{{{a}\\\\{b}}}}}")),
            2 => (proptest::sample::select(&["hat", "bar", "vec", "tilde", "dot", "ddot", "overline", "underline", "widehat", "overrightarrow", "cancel"][..]), arg.clone())
                .prop_map(|(c, a)| format!("\\{c}{a}")),
            1 => (proptest::sample::select(&["overbrace", "underbrace"][..]), inner.clone(), arg.clone())
                .prop_map(|(c, a, s)| format!("\\{c}{{{a}}}^{s}_{s}")),
            1 => (proptest::sample::select(&["overset", "underset", "stackrel"][..]), arg.clone(), arg.clone())
                .prop_map(|(c, a, b)| format!("\\{c}{a}{b}")),
            1 => (proptest::sample::select(&["xrightarrow", "xleftarrow"][..]), inner.clone(), arg.clone())
                .prop_map(|(c, u, o)| format!("\\{c}[{u}]{o}")),
            1 => pair.clone().prop_map(|(a, b)| format!("\\cancelto{{{a}}}{{{b}}}")),
            1 => (proptest::sample::select(&["mathrm", "mathbf", "mathit", "mathsf", "mathtt", "mathbb", "mathcal", "boldsymbol"][..]), arg.clone())
                .prop_map(|(c, a)| format!("\\{c}{a}")),
            1 => inner.clone().prop_map(|s| format!("\\text{{t $ {s} $ u}}")),
            1 => inner.clone().prop_map(|s| format!("{{\\color{{red}} {s}}}")),
            1 => (proptest::sample::select(&["textcolor{blue}", "colorbox{yellow}", "fcolorbox{red}{white}"][..]), arg.clone())
                .prop_map(|(c, a)| format!("\\{c}{a}")),
            1 => (proptest::sample::select(&["phantom", "hphantom", "vphantom"][..]), arg)
                .prop_map(|(c, a)| format!("\\{c}{a}")),
            1 => inner.prop_map(|s| format!("({s})^2")),
        ]
    })
}

fn options_from_bits(bits: u8) -> ParseOptions {
    ParseOptions::new()
        .with_ascii_relations(bits & 1 != 0)
        .with_bare_operators(bits & 2 != 0)
        .with_unicode_symbols(bits & 4 != 0)
        .with_prime_symbol(bits & 8 != 0)
}

fn runner() -> TestRunner {
    let cases = std::env::var("LATEX_RUST_CANON_CASES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(4096);
    let mut config = Config::with_cases(cases);
    config.failure_persistence = Some(Box::new(FileFailurePersistence::Off));
    TestRunner::new_with_rng(config, TestRng::from_seed(RngAlgorithm::ChaCha, &[42; 32]))
}

/// `Err` names the first broken promise for an accepted `src`.
fn check(src: &str, options: &ParseOptions) -> Result<(), String> {
    let Ok((tree, _)) = parse_with_options(src, options) else {
        return Ok(());
    };
    let printed = tree.to_latex_with(options);
    let again = parse_with_options(&printed, options)
        .map_err(|e| format!("{src:?} printed {printed:?}: {e}"))?
        .0;
    if again != tree {
        return Err(format!(
            "{src:?} printed {printed:?}, which parses to another tree"
        ));
    }
    let second = again.to_latex_with(options);
    if second != printed {
        return Err(format!("{src:?} printed {printed:?}, then {second:?}"));
    }
    Ok(())
}

#[test]
fn prop_whole_grammar_print_is_canonical() {
    let mut runner = runner();
    let accepted = std::cell::Cell::new(0usize);
    let result = runner.run(&(math(), 0u8..16), |(src, bits)| {
        let options = options_from_bits(bits);
        if parse_with_options(&src, &options).is_ok() {
            accepted.set(accepted.get() + 1);
        }
        check(&src, &options).map_err(TestCaseError::fail)
    });
    if let Err(e) = result {
        panic!("{e}");
    }
    let accepted = accepted.get();
    eprintln!("accepted {accepted} inputs");
    assert!(accepted > 0, "the grammar produced no accepted input");
}

/// Findings of the property, each a seed in `fuzz/corpus/parse`, with the
/// option bits it failed under.
const FINDINGS: &[(&str, &str, u8)] = &[
    // `]` inside an optional argument ended it early (`\xrightarrow []]`).
    (
        "roundtrip-canon-option-bracket",
        r"\xrightarrow[\mathrm{]}]{x}",
        0,
    ),
    // `{[}` is read as a bare `[`, so `\sqrt{[}` opened an index.
    ("roundtrip-canon-sqrt-bracket", r"\sqrt{{[}}", 0),
    // A primed base before more primes: `x_x''` joined the base's prime.
    ("roundtrip-canon-primed-subsup", "{x_{x}^ ′}^ ′", 0),
    // With `prime_symbol`, a `'` mark reads back as `\prime`.
    (
        "roundtrip-canon-prime-symbol",
        "\\textcolor{blue}{{x}^ ′}",
        8,
    ),
    // A print starting with `\(` was read as an unclosed math fence (fuzz).
    ("roundtrip-canon-fence-open", r"{\( }x", 0),
    // A `\)` symbol inside text math closed the `\(` early.
    ("roundtrip-canon-text-math-close", r"\text{t $ \) $ u}", 0),
    // A `\big(` delimiter paren was counted by the paren lookahead (fuzz r9,
    // before Q1).
    ("roundtrip-canon-big-paren", r"(^x(\big())x^))^x", 0),
    // A `(` inside an option paired with a `)` after the option's `]` once
    // the body printed braced (fuzz r10).
    (
        "roundtrip-canon-option-inner-paren",
        "\\xleftarrow[(]))^\u{8}",
        0,
    ),
    // A figure space (U+2007) before a comment is an atom, but at the end of
    // the print `preprocess` trims it (fuzz r14).
    ("roundtrip-canon-edge-space", "\u{8}\u{2007}%", 0),
    // The `\)` check saw the fence-wrapped body `{\(\)}`, but the printer
    // wrote it unwrapped inside `\(…\)` (fuzz).
    ("roundtrip-canon-text-math-fence", r"\text{$\(\)$}", 0),
    // A primed `)` base closed an earlier `(`, so `(...)'` read back as a
    // scripted paren group (fuzz).
    ("roundtrip-canon-primed-close", "(^))'^)", 0),
    ("roundtrip-canon-primed-close-accent", r"(\not()'^x", 0),
    // A primed `(` base was not counted as open, so a later `)^` closed it.
    ("roundtrip-canon-primed-open", r"('^))^\ ", 0),
    // A `(` or `)` in an optional argument is not hidden from the paren
    // lookahead, so it paired with a `(` before the command (fuzz).
    ("roundtrip-canon-option-paren", r"(^)\xleftarrow[)^x]]", 0),
    (
        "roundtrip-canon-option-paren-open",
        "(^)\\xleftarrow[(]x))^\u{8}",
        0,
    ),
];

#[test]
fn findings_are_seeds_and_print_canonically() {
    for (name, src, bits) in FINDINGS {
        let path = format!("{}/fuzz/corpus/parse/{name}", env!("CARGO_MANIFEST_DIR"));
        let seed = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
        assert_eq!(&seed, src, "{name}");
        assert!(
            parse_with_options(src, &options_from_bits(*bits)).is_ok(),
            "{name}"
        );
        if let Err(e) = check(src, &options_from_bits(*bits)) {
            panic!("{name}: {e}");
        }
    }
}

/// Dimensions with no digit (`.`) or two points parsed to NaN, which printed
/// as `NaNem` and did not read back (fuzz `roundtrip-dim-dot`). They are now
/// a typed error.
#[test]
fn dimensions_without_a_number_are_rejected() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/fuzz/corpus/parse/roundtrip-dim-dot"
    );
    let seed = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    assert_eq!(seed, r"\rule{2}{.}");
    for src in [
        seed.as_str(),
        r"\rule{.}{1}",
        r"\hspace{.em}",
        r"\hspace{1.2.3em}",
        r"\rule{-.}{1}",
    ] {
        assert!(
            parse_with_options(src, &options_from_bits(0)).is_err(),
            "{src} should be rejected"
        );
    }
    // A number too large for the exact rational is NaN too (fuzz r12).
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/fuzz/corpus/parse/roundtrip-dim-overflow"
    );
    let seed = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    assert_eq!(seed, r"{\rule{2}{888888888888888888888888888888888888888}}");
    for src in [
        seed.as_str(),
        r"\hspace{99999999999999999999999999999999999999999999em}",
        r"\hspace{170141183460469231731687303715884105728mu}",
    ] {
        assert!(
            parse_with_options(src, &options_from_bits(0)).is_err(),
            "{src} should be rejected"
        );
    }
    for src in [r"\rule{.5}{1.}", r"\hspace{-.25em}"] {
        assert!(check(src, &options_from_bits(0)).is_ok(), "{src}");
    }
}

/// `preprocess` trims every Unicode whitespace character at the ends of the
/// input, but the tokenizer reads such a character as an atom, so a tree whose
/// print starts or ends with one (a comment hides it from the trim: `a\u{a0}%`)
/// lost it on the reparse. The print is now grouped when the trim would touch it.
#[test]
fn edge_whitespace_atoms_survive_the_trim() {
    for src in [
        "\u{8}\u{2007}%",
        "a\u{a0}%",
        "%\n\u{a0}a",
        "\u{2003}%",
        "%\n\u{a0}\u{a0}",
        "a\u{a0}%\n",
        "x^{\u{a0}}%",
        "a\u{b}%",
        "\u{c}\u{85}%",
        "%\n\u{b}a",
        "\\text{a\u{a0}}%",
    ] {
        if let Err(e) = check(src, &options_from_bits(0)) {
            panic!("{src:?}: {e}");
        }
    }
}
