//! Each pair is one property: the two inputs build the same `gold()`.

mod common;

use common::{flag, gold, gold_with};
use latex_rust::{ParseError, ParseOptions};

fn ascii() -> ParseOptions {
    flag("ascii")
}

fn bare() -> ParseOptions {
    flag("bare")
}

fn unicode() -> ParseOptions {
    flag("unicode")
}

fn prime() -> ParseOptions {
    flag("prime")
}

struct Pair {
    left: &'static str,
    right: &'static str,
    options: fn() -> ParseOptions,
}

#[test]
fn command_relation_aliases() -> Result<(), ParseError> {
    let pairs = [(r"\leq", r"\le"), (r"\geq", r"\ge"), (r"\neq", r"\ne")];
    for (left, right) in pairs {
        assert_eq!(gold(left)?, gold(right)?, "{left}");
    }
    Ok(())
}

#[test]
fn unicode_operations_match_commands() -> Result<(), ParseError> {
    let pairs = [
        ("√x", r"\sqrt{x}"),
        ("√{x}", r"\sqrt{x}"),
        ("x²", "x^{2}"),
        ("x³", "x^{3}"),
        ("xⁿ", "x^{n}"),
        ("x⁻¹", "x^{-1}"),
        ("\u{2212}", "-"),
    ];
    for (left, right) in pairs {
        assert_eq!(gold(left)?, gold(right)?, "{left}");
    }
    Ok(())
}

#[test]
fn optional_pairs_match_when_the_flag_is_on() -> Result<(), ParseError> {
    let pairs = [
        Pair {
            left: "<=",
            right: r"\leq",
            options: ascii,
        },
        Pair {
            left: ">=",
            right: r"\geq",
            options: ascii,
        },
        Pair {
            left: "!=",
            right: r"\neq",
            options: ascii,
        },
        Pair {
            left: "sin(x)",
            right: r"\sin(x)",
            options: bare,
        },
        Pair {
            left: "zeta(s,a)",
            right: r"\zeta(s,a)",
            options: bare,
        },
        Pair {
            left: r"\begin{cases} 0 & otherwise \end{cases}",
            right: r"\begin{cases} 0 & \text{otherwise} \end{cases}",
            options: bare,
        },
        Pair {
            left: "α",
            right: r"\alpha",
            options: unicode,
        },
        Pair {
            left: "Γ",
            right: r"\Gamma",
            options: unicode,
        },
        Pair {
            left: "π",
            right: r"\pi",
            options: unicode,
        },
        Pair {
            left: "≤",
            right: r"\leq",
            options: unicode,
        },
        Pair {
            left: "≥",
            right: r"\geq",
            options: unicode,
        },
        Pair {
            left: "≠",
            right: r"\neq",
            options: unicode,
        },
        Pair {
            left: "∞",
            right: r"\infty",
            options: unicode,
        },
        Pair {
            left: "×",
            right: r"\times",
            options: unicode,
        },
        Pair {
            left: "·",
            right: r"\cdot",
            options: unicode,
        },
        Pair {
            left: "⋅",
            right: r"\cdot",
            options: unicode,
        },
        Pair {
            left: "f'",
            right: r"f^{\prime}",
            options: prime,
        },
        Pair {
            left: "f''",
            right: r"f^{\prime\prime}",
            options: prime,
        },
    ];
    for pair in pairs {
        let options = (pair.options)();
        assert_eq!(
            gold_with(pair.left, &options)?,
            gold_with(pair.right, &options)?,
            "{}",
            pair.left
        );
    }
    let nested = gold_with("f'''", &prime())?;
    let flat = r#"(sup (atom Ord "f") (row (symbol prime) (symbol prime) (symbol prime)))"#;
    assert_ne!(nested, flat);
    Ok(())
}
