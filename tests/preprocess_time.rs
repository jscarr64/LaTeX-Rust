//! `{a \over b}` and a braced `\overline` must be handled in linear time.
//!
//! Release 2.1.1 rescanned from the start after every hit, so four times the
//! input cost about sixteen times the work. A linear pass costs about four
//! times; the bound is eight, which a linear pass meets and a rescan misses.
//!
//! The infix rewrite is done by the parser now, and `preprocess` is only a
//! trim (a copy of the input, a microsecond for this input, and not linear
//! on a time scale once it is large enough to measure, because the cost of
//! fresh memory jumps with the size). So `parse`, which calls `preprocess`,
//! is what is measured, at 1,000 and 4,000 copies (a few milliseconds a
//! run). Each size is the minimum of nine single runs: one slow run
//! (a scheduler pause, a cold cache, a busy CI runner) cannot fail the test,
//! while a quadratic pass is slow in every run. A release run of 4,000
//! fractions must also finish in under 200 ms.

use std::hint::black_box;
use std::time::{Duration, Instant};

use latex_rust::parse;

const RUNS: u32 = 9;
const SMALL: usize = 1_000;
const BIG: usize = 4_000;

fn best_of(src: &str, run: fn(&str) -> usize) -> Duration {
    // Warm-up call
    let _ = run(src);

    let mut min_duration = Duration::MAX;
    for _ in 0..RUNS {
        let start = Instant::now();
        let result = run(src);
        let duration = start.elapsed();
        min_duration = min_duration.min(duration);
        // Use black_box on the result to prevent optimization
        let _ = black_box(result);
    }
    min_duration
}

fn check(
    problems: &mut Vec<String>,
    what: &str,
    pattern: &str,
    small_n: usize,
    big_n: usize,
    run: fn(&str) -> usize,
) {
    let small_src = pattern.repeat(small_n);
    let big_src = pattern.repeat(big_n);

    let small = best_of(&small_src, run);
    let big = best_of(&big_src, run);

    let small_ns = small.as_nanos();
    if small_ns == 0 {
        problems.push(format!(
            "{what} {pattern}: timer returned 0 for n={small_n}"
        ));
        return;
    }

    let limit = small_ns.saturating_mul(8);
    if big.as_nanos() >= limit {
        problems.push(format!(
            "{what} {pattern}: t({big_n})={big:?} is not under 8 * t({small_n})={small:?}"
        ));
    }
}

fn run_parse(s: &str) -> usize {
    usize::from(black_box(parse(s).is_ok()))
}

#[test]
fn infix_rewrite_scales_linearly() {
    let mut problems = Vec::new();

    check(
        &mut problems,
        "parse",
        r"{a \over b}",
        SMALL,
        BIG,
        run_parse,
    );
    check(
        &mut problems,
        "parse",
        r"\frac{\overline{x}}{2}",
        SMALL,
        BIG,
        run_parse,
    );
    if !cfg!(debug_assertions) {
        let src = r"{a \over b}".repeat(BIG);
        let best = best_of(&src, run_parse);
        if best >= Duration::from_millis(200) {
            problems.push(format!(
                "parse {{a \\over b}}: t({BIG}) was {best:?}, bound is 200 ms"
            ));
        }
    }

    assert!(
        problems.is_empty(),
        "parse is not linear:\n{}",
        problems.join("\n")
    );
}
