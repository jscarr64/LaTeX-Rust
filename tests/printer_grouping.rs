//! Printer grouping (spec section 5) and the paren lookahead rules (Q1, Q2).
//!
//! Q1: the paren lookahead skips the delimiter token after `\left`, `\right`,
//! `\middle` and the `\big` family, and the `[…]` option of a command that
//! takes one. Q2: command arguments always print braced, and a braced argument
//! costs no more depth than a bare one. G1-G3: section 5.2.

use latex_rust::{parse, parse_with_options, MathNode, ParseOptions};

fn depth(s: &str) -> Option<usize> {
    let ok = |d: usize| parse_with_options(s, &ParseOptions::new().with_max_depth(d)).is_ok();
    if !ok(256) {
        return None;
    }
    let (mut lo, mut hi) = (0, 256);
    while lo < hi {
        let mid = (lo + hi) / 2;
        if ok(mid) {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    Some(lo)
}

fn first_is_open_paren(src: &str) -> bool {
    match parse(src) {
        Ok(MathNode::Row(ref items)) => matches!(items.first(), Some(MathNode::Atom('(', _))),
        _ => false,
    }
}

fn round_trip(src: &str) -> String {
    let tree = parse(src).unwrap_or_else(|e| panic!("{src}: {e}"));
    let printed = tree.to_latex();
    let again = parse(&printed).unwrap_or_else(|e| panic!("{src} printed {printed}: {e}"));
    assert_eq!(again, tree, "{src} printed {printed}");
    assert_eq!(again.to_latex(), printed, "{src}");
    printed
}

// Q1 ---------------------------------------------------------------------

#[test]
fn lookahead_skips_the_delimiter_after_big_and_left() {
    // The `)` after `\big` is the delimiter, not a match for the first `(`.
    assert!(first_is_open_paren(r"(\big)^2"), r"(\big)^2");
    assert!(first_is_open_paren(r"(\Bigl)^2"), r"(\Bigl)^2");
    assert!(first_is_open_paren(r"(\left. x\right)^2"), r"\right)");
    round_trip(r"(\big)^2");
}

#[test]
fn lookahead_skips_optional_arguments() {
    // The `)` inside `[…]` belongs to the option.
    assert!(first_is_open_paren(r"(\sqrt[)^2]{x}"), r"\sqrt[…]");
    assert!(
        first_is_open_paren(r"(\xrightarrow[)^2]{x}"),
        r"\xrightarrow[…]"
    );
    round_trip(r"(\sqrt[)^2]{x}");
}

#[test]
fn lookahead_from_inside_an_option_stops_at_its_close() {
    // The `(` is inside the option: its match cannot be after the `]`.
    for src in [
        r"\xleftarrow[(]{)})^2",
        r"\sqrt[(]{)}^2",
        r"\xleftarrow[(]))^2",
    ] {
        round_trip(src);
    }
    // Outside any option a `]` does not stop the scan.
    match parse(r"(a]b)^2") {
        Ok(MathNode::Superscript(ref base, _)) => {
            assert!(matches!(base.as_ref(), MathNode::Row(_)));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_real_paren_script_still_reads_as_one() {
    for src in [
        r"(a)^2",
        r"(\left(a\right))^2",
        r"(\sqrt[3]{x})^2",
        r"(\big( a)^2",
    ] {
        match parse(src) {
            Ok(MathNode::Superscript(ref base, _)) => {
                assert!(matches!(base.as_ref(), MathNode::Row(_)), "{src}");
            }
            other => panic!("{src}: {other:?}"),
        }
    }
}

// Q2 ---------------------------------------------------------------------

#[test]
fn command_arguments_print_braced() {
    for (src, want) in [
        (r"\frac12", r"\frac {1}{2}"),
        (r"\frac 1\frac 12", r"\frac {1}{\frac {1}{2}}"),
        (r"\sqrt x", r"\sqrt {x}"),
        (r"\hat a", r"\hat {a}"),
        (r"\binom ab", r"\binom {a}{b}"),
        (r"\mathbf x", r"\mathbf {x}"),
    ] {
        assert_eq!(round_trip(src), want, "{src}");
    }
}

#[test]
fn a_braced_argument_costs_no_more_depth_than_a_bare_one() {
    for (bare, braced) in [
        (r"\frac ab", r"\frac{a}{b}"),
        (
            r"\frac 1\frac 1\frac 12",
            r"\frac{1}{\frac{1}{\frac{1}{2}}}",
        ),
        (r"x^a", r"x^{a}"),
        (r"\sqrt x", r"\sqrt {x}"),
    ] {
        assert_eq!(depth(bare), depth(braced), "{bare} vs {braced}");
    }
}

// G1-G3 ------------------------------------------------------------------

#[test]
fn scripts_stay_bare_first() {
    for (src, want) in [
        (r"x^a", r"x^a"),
        (r"x^{ab}", r"x^{a b}"),
        (r"x^\alpha", r"x^\alpha "),
        (r"x^{\frac12}", r"x^\frac {1}{2}"),
    ] {
        assert_eq!(round_trip(src), want, "{src}");
    }
}

#[test]
fn unbalanced_parens_in_a_script_are_grouped() {
    assert_eq!(round_trip(r"x^{)}"), r"x^{)}");
    assert_eq!(round_trip(r"x_{(}y"), r"x_{(}y");
}

#[test]
fn fake_groups_keep_two_groups() {
    assert_eq!(round_trip(r"\sqrt{{[}}"), r"\sqrt {{[}}");
    assert_eq!(
        round_trip(r"\xrightarrow[{{]}}]{x}"),
        r"\xrightarrow [{{]}}]{x}"
    );
}

#[test]
fn row_paren_pass_groups_the_item_holding_the_match() {
    for src in [
        r"(\not()'^x",
        r"(^))'^)",
        r"('^))^\ ",
        r"(^)\xleftarrow[)^x]]",
        r"(x{)}^2",
        r"((x{)}^2",
    ] {
        round_trip(src);
    }
}

// Cost ---------------------------------------------------------------------

/// A print must take time linear in the tree. The row paren pass once printed
/// each nested row twice per level, so `{(` nested k deep cost 2^k (fuzz r11,
/// slow unit `roundtrip-perf-paren-row`: 0.8 s for 2 KB).
#[test]
fn nested_paren_rows_print_in_linear_time() {
    let start = std::time::Instant::now();
    for k in [8usize, 16, 22, 64] {
        let src = "{(".repeat(k) + "x" + &")}".repeat(k);
        let (tree, _) = parse_with_options(&src, &ParseOptions::new().with_max_depth(512))
            .unwrap_or_else(|e| panic!("k={k}: {e}"));
        let printed = tree.to_latex();
        assert!(printed.len() >= 2 * k, "k={k}");
        assert!(
            start.elapsed() < std::time::Duration::from_secs(2),
            "k={k}: {:?}",
            start.elapsed()
        );
    }
}

#[test]
fn the_slow_seed_prints_fast() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/fuzz/corpus/parse/roundtrip-perf-paren-row"
    );
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let src = String::from_utf8_lossy(&bytes).into_owned();
    let tree = parse(&src).unwrap_or_else(|e| panic!("{e}"));
    let start = std::time::Instant::now();
    let printed = tree.to_latex();
    assert!(
        start.elapsed() < std::time::Duration::from_millis(100),
        "{:?}",
        start.elapsed()
    );
    assert_eq!(parse(&printed).ok().as_ref(), Some(&tree));
}
