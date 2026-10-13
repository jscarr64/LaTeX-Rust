//! `{a \over b}` and a braced `\overline` must be rewritten in linear time.
//!
//! Release 2.1.1 rescans from the start after every hit. Four times the
//! input then costs about sixteen times the work. The bound is eight, which
//! a linear pass meets and that rescan misses. One release measurement of
//! 4,000 fractions must also finish in under 200 ms.

use std::hint::black_box;
use std::time::{Duration, Instant};

use latex_rust::preprocess;

const SMALL: usize = 1_000;
const BIG: usize = 4_000;
const ROUNDS: u32 = 4;

fn time_repeat(src: &str) -> Duration {
    let _ = black_box(preprocess(src).len());
    let start = Instant::now();
    for _ in 0..ROUNDS {
        let _ = black_box(preprocess(src).len());
    }
    start.elapsed()
}

fn check(problems: &mut Vec<String>, pattern: &str) {
    let small_src = pattern.repeat(SMALL);
    let big_src = pattern.repeat(BIG);
    let small = time_repeat(&small_src);
    let big = time_repeat(&big_src);
    let small_ns = small.as_nanos();
    if small_ns == 0 {
        problems.push(format!("{pattern}: timer returned 0 for n={SMALL}"));
        return;
    }
    let limit = small_ns.saturating_mul(8);
    if big.as_nanos() >= limit {
        problems.push(format!(
            "{pattern}: t({BIG})={big:?} is not under 8 * t({SMALL})={small:?}"
        ));
    }
}

#[test]
fn infix_rewrite_scales_linearly() {
    let mut problems = Vec::new();
    check(&mut problems, r"{a \over b}");
    check(&mut problems, r"\frac{\overline{x}}{2}");
    if !cfg!(debug_assertions) {
        let src = r"{a \over b}".repeat(BIG);
        let _ = black_box(preprocess(&src).len());
        let start = Instant::now();
        let _ = black_box(preprocess(&src).len());
        let once = start.elapsed();
        if once >= Duration::from_millis(200) {
            problems.push(format!(
                "t({BIG}) of {{a \\over b}} was {once:?}, bound is 200 ms"
            ));
        }
    }
    assert!(
        problems.is_empty(),
        "preprocess is not linear:\n{}",
        problems.join("\n")
    );
}
