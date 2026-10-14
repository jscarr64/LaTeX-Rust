//! Prime runs are one flat row, print as `^{′′′}` (or `^{\prime \prime …}`
//! with `prime_symbol`), and survive a reprint at the default depth.

use latex_rust::{parse_with_options, MathNode, ParseOptions};

const ATOM: &str = r#"(atom Ord "′")"#;
const SYMBOL: &str = "(symbol prime)";

fn parse_ok(input: &str, options: &ParseOptions) -> MathNode {
    match parse_with_options(input, options) {
        Ok((node, _)) => node,
        Err(err) => panic!("{input:?} did not parse: {err}"),
    }
}

/// Parse, print, parse again, and require the same tree and a stable print.
fn round_trip(input: &str, options: &ParseOptions) -> String {
    let node = parse_ok(input, options);
    let printed = node.to_latex();
    let again = match parse_with_options(&printed, options) {
        Ok((node, _)) => node,
        Err(err) => panic!("reprint {printed:?} of {input:?}: {err}"),
    };
    assert_eq!(again, node, "input {input:?} printed {printed:?}");
    assert_eq!(again.to_latex(), printed, "reprint of {input:?} not stable");
    printed
}

fn primes(n: usize) -> String {
    "'".repeat(n)
}

fn flat_gold(base: &str, prime: &str, n: usize) -> String {
    if n == 1 {
        format!("(sup {base} {prime})")
    } else {
        format!("(sup {base} (row{}))", format!(" {prime}").repeat(n))
    }
}

#[test]
fn crash_input_round_trips() {
    let input = format!("{{[{}}}", primes(34));
    round_trip(&input, &ParseOptions::new());
}

#[test]
fn prime_runs_are_flat_and_print_flat() {
    let options = ParseOptions::new();
    let base = r#"(atom Ord "x")"#;
    for n in [1usize, 2, 33, 100, 1000] {
        let input = format!("x{}", primes(n));
        let node = parse_ok(&input, &options);
        assert_eq!(node.gold(), flat_gold(base, ATOM, n), "n={n}");
        let printed = round_trip(&input, &options);
        assert_eq!(printed, format!("{{x}}^{{{}}}", "′".repeat(n)), "n={n}");
    }
}

#[test]
fn primes_mixed_with_scripts_round_trip() {
    let options = ParseOptions::new();
    let inputs = [
        format!("x_{{1}}{}", primes(33)),
        format!("x{}_{{1}}", primes(40)),
        format!("x{}^{{2}}", primes(33)),
        format!("x_{{i}}{}^{{2}}", primes(100)),
        format!("{{x{}}}^{{2}}", primes(34)),
        format!("x^{{ab}}{}", primes(34)),
    ];
    for input in &inputs {
        round_trip(input, &options);
    }
    let node = parse_ok("x''^2", &options);
    assert_eq!(
        node.gold(),
        format!(r#"(sup (sup (atom Ord "x") (row {ATOM} {ATOM})) (atom Ord "2"))"#)
    );
}

#[test]
fn prime_symbol_runs_are_flat() {
    let options = ParseOptions::new().with_prime_symbol(true);
    let base = r#"(atom Ord "x")"#;
    for n in [1usize, 2, 33, 100] {
        let input = format!("x{}", primes(n));
        let node = parse_ok(&input, &options);
        assert_eq!(node.gold(), flat_gold(base, SYMBOL, n), "n={n}");
        let printed = round_trip(&input, &options);
        assert_eq!(
            printed,
            format!("{{x}}^{{{}}}", r"\prime ".repeat(n)),
            "n={n}"
        );
    }
    let explicit = format!("x^{{{}}}", r"\prime".repeat(40));
    let printed = round_trip(&explicit, &ParseOptions::new());
    assert_eq!(printed, format!("{{x}}^{{{}}}", r"\prime ".repeat(40)));
    let literal = format!("x^{{{}}}", "′".repeat(40));
    for options in [ParseOptions::new(), options] {
        assert_eq!(
            parse_ok(&literal, &options).gold(),
            flat_gold(base, ATOM, 40)
        );
        round_trip(&literal, &options);
    }
}
