//! Nesting-depth limit (1.0.5 backport of the 2.0.0 fix for #6).
//!
//! Before 1.0.5, deeply nested input overflowed the stack and aborted the
//! process. These tests check that such input returns `Err` instead, both on an
//! 8 MiB stack (the main-thread default) and on a small worker stack, and that
//! every input the limit admits still renders.

use latex_rust::{
    latex_to_svg, layout, parse, render_svg, AtomKind, Error, MathFont, MathNode, MathStyle,
    ParseError, SvgOptions,
};

/// The limit, as it appears in error messages. Matches 2.0.0.
const LIMIT: usize = 32;

/// The main-thread default on Linux.
const BIG_STACK: usize = 8 << 20;

/// A small worker stack. In an optimised build the parser refuses
/// pathological input within about 100 KiB, so 256 KiB is used there. An
/// unoptimised build uses several times more stack per level, so it gets the
/// 2 MiB `std::thread` default instead.
const SMALL_STACK: usize = if cfg!(debug_assertions) {
    2 << 20
} else {
    256 << 10
};

/// Stack on which the deepest admitted input must parse, lay out, render and
/// drop. Measured worst cases are about 512 KiB optimised and about 4 MiB
/// unoptimised (nested `matrix`), so 1 MiB and 8 MiB leave headroom.
const RENDER_STACK: usize = if cfg!(debug_assertions) {
    8 << 20
} else {
    1 << 20
};

fn on_stack(size: usize, f: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .stack_size(size)
        .spawn(f)
        .expect("spawn")
        .join()
        .expect("worker panicked");
}

/// A named nesting shape that builds input `n` levels deep.
type Shape = (&'static str, fn(usize) -> String);

fn frac(n: usize) -> String {
    "\\frac{1}{".repeat(n) + "2" + &"}".repeat(n)
}

fn braces(n: usize) -> String {
    "{".repeat(n) + "x" + &"}".repeat(n)
}

fn sqrt(n: usize) -> String {
    "\\sqrt{".repeat(n) + "x" + &"}".repeat(n)
}

fn shapes() -> Vec<Shape> {
    vec![
        ("braces", braces),
        ("frac", frac),
        ("sqrt", sqrt),
        ("sqrt_index", |n| {
            "\\sqrt[3]{".repeat(n) + "x" + &"}".repeat(n)
        }),
        ("sup", |n| "x^{".repeat(n) + "y" + &"}".repeat(n)),
        ("sub", |n| "x_{".repeat(n) + "y" + &"}".repeat(n)),
        ("left", |n| {
            "\\left(".repeat(n) + "x" + &"\\right)".repeat(n)
        }),
        ("mathrm", |n| "\\mathrm{".repeat(n) + "x" + &"}".repeat(n)),
        ("hat", |n| "\\hat{".repeat(n) + "x" + &"}".repeat(n)),
        ("color", |n| {
            "\\color{red}{".repeat(n) + "x" + &"}".repeat(n)
        }),
        ("boxed", |n| "\\boxed{".repeat(n) + "x" + &"}".repeat(n)),
        ("binom", |n| "\\binom{1}{".repeat(n) + "x" + &"}".repeat(n)),
        ("matrix", |n| {
            "\\begin{matrix}".repeat(n) + "x" + &"\\end{matrix}".repeat(n)
        }),
    ]
}

fn expect_too_deep(name: &str, n: usize, src: &str) {
    match parse(src) {
        Err(ParseError::Malformed(msg)) => assert_eq!(
            msg,
            format!("input nests deeper than {LIMIT} levels"),
            "{name}@{n}"
        ),
        other => panic!("{name}@{n}: expected Malformed, got {other:?}"),
    }
    let font = MathFont::stix_two_math().expect("font");
    assert!(
        latex_to_svg(src, &font, &SvgOptions::new()).is_err(),
        "{name}@{n}: latex_to_svg should refuse"
    );
}

fn required_cases_err(stack: usize) {
    on_stack(stack, || {
        for n in [700, 5_000] {
            expect_too_deep("frac", n, &frac(n));
        }
        expect_too_deep("braces", 10_000, &braces(10_000));
        for n in [LIMIT, 700, 5_000, 100_000] {
            expect_too_deep("sqrt", n, &sqrt(n));
        }
    });
}

#[test]
fn frac_700_5000_braces_10000_and_sqrt_err_on_8_mib_stack() {
    required_cases_err(BIG_STACK);
}

#[test]
fn frac_700_5000_braces_10000_and_sqrt_err_on_small_stack() {
    required_cases_err(SMALL_STACK);
}

#[test]
fn every_shape_errs_at_100_000_on_small_stack() {
    on_stack(SMALL_STACK, || {
        for (name, make) in shapes() {
            for n in [LIMIT, 1_000, 100_000] {
                expect_too_deep(name, n, &make(n));
            }
        }
    });
}

/// Deepest `n` for which `make(n)` parses.
fn deepest(make: fn(usize) -> String) -> usize {
    let mut n = 0;
    while parse(&make(n + 1)).is_ok() {
        n += 1;
        assert!(n < 1_000, "limit never reached");
    }
    n
}

#[test]
fn limit_admits_15_nested_fractions_and_31_nested_groups() {
    assert!(parse(&frac(15)).is_ok());
    assert!(parse(&frac(16)).is_err());
    assert!(parse(&braces(31)).is_ok());
    assert!(parse(&braces(32)).is_err());
}

#[test]
fn deepest_admitted_input_lays_out_renders_and_drops() {
    on_stack(RENDER_STACK, || {
        let font = MathFont::stix_two_math().expect("font");
        for (name, make) in shapes() {
            let n = deepest(make);
            assert!(n >= 15, "{name}: limit admits only {n} levels");
            let ast = parse(&make(n)).unwrap_or_else(|e| panic!("{name}@{n}: {e}"));
            let bx = layout(&ast, &font, MathStyle::Display)
                .unwrap_or_else(|e| panic!("{name}@{n}: parse accepted but layout refused: {e}"));
            render_svg(&bx, &font, &SvgOptions::new())
                .unwrap_or_else(|e| panic!("{name}@{n}: svg: {e}"));
            #[cfg(feature = "png")]
            latex_rust::render_png(&bx, &font, &latex_rust::PngOptions::new())
                .unwrap_or_else(|e| panic!("{name}@{n}: png: {e}"));
            #[cfg(feature = "egui")]
            latex_rust::shapes(
                &bx,
                &font,
                &latex_rust::EguiOptions::new(),
                egui::Pos2::ZERO,
                1.0,
            )
            .unwrap_or_else(|e| panic!("{name}@{n}: egui: {e}"));
        }
    });
}

#[test]
fn layout_refuses_a_hand_built_tree_that_is_too_deep() {
    on_stack(BIG_STACK, || {
        let font = MathFont::stix_two_math().expect("font");
        let mut node = MathNode::Atom('x', AtomKind::Ord);
        for _ in 0..1_000 {
            node = MathNode::Radical(None, Box::new(node));
        }
        match layout(&node, &font, MathStyle::Text) {
            Err(Error::Unsupported { what }) => {
                assert_eq!(what, format!("tree nests deeper than {LIMIT} levels"));
            }
            other => panic!("expected Unsupported, got {:?}", other.map(|_| ())),
        }
    });
}
