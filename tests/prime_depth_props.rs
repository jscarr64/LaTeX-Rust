//! Trees with prime runs near the depth limit: parse → print → parse gives
//! the same tree, and the print needs at most two levels more than the source
//! (the `^` argument and its `{…}` group around a prime run), whatever the run
//! length. Before the fix the cost grew with the run.

use latex_rust::{parse_with_options, ParseOptions, DEFAULT_MAX_NESTING_DEPTH};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

fn options(depth: usize, prime_symbol: bool) -> ParseOptions {
    ParseOptions::new()
        .with_max_depth(depth)
        .with_prime_symbol(prime_symbol)
}

/// Smallest `max_depth` at which `src` parses, if any up to the default.
fn min_depth(src: &str, prime_symbol: bool) -> Option<usize> {
    (1..=DEFAULT_MAX_NESTING_DEPTH)
        .find(|&depth| parse_with_options(src, &options(depth, prime_symbol)).is_ok())
}

fn source(k: usize, n: usize, shape: u8) -> String {
    let run = "'".repeat(n);
    match shape {
        0 => format!("{}y{run}{}", "x^{".repeat(k), "}".repeat(k)),
        1 => format!("{}[{run}{}", "{".repeat(k + 1), "}".repeat(k + 1)),
        2 => format!("{}y_{{1}}{run}^{{2}}{}", "x^{".repeat(k), "}".repeat(k)),
        _ => format!("{}y{run}_{{1}}{}", "\\sqrt{".repeat(k), "}".repeat(k)),
    }
}

proptest! {
    #![proptest_config({
        let mut config = ProptestConfig::with_cases(512);
        config.failure_persistence = Some(Box::new(FileFailurePersistence::Off));
        config
    })]
    #[test]
    fn prime_runs_reprint_within_source_depth(
        k in 0usize..=16,
        n in 1usize..=200,
        shape in 0u8..4,
        prime_symbol in any::<bool>(),
    ) {
        let src = source(k, n, shape);
        let Some(depth) = min_depth(&src, prime_symbol) else {
            return Ok(());
        };
        let opts = options(depth, prime_symbol);
        let node = match parse_with_options(&src, &opts) {
            Ok((node, _)) => node,
            Err(err) => return Err(TestCaseError::fail(format!("{src:?}: {err}"))),
        };
        let printed = node.to_latex();
        let reprint = options(depth + 2, prime_symbol);
        let again = match parse_with_options(&printed, &reprint) {
            Ok((node, _)) => node,
            Err(err) => {
                return Err(TestCaseError::fail(format!(
                    "{src:?} printed {printed:?} at depth {depth} + 2: {err}"
                )))
            }
        };
        prop_assert_eq!(&again, &node, "src {:?} printed {:?}", src, printed);
        prop_assert_eq!(again.to_latex(), printed);
    }
}
