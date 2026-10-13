//! Scripts bind to a parenthesized row, and `f'^2` nests.
//!
//! `{(-b)}^{n}` and `\left(-b\right)^{n}` already group. `-b^{n}` and
//! `a/b^{n}` keep the power on `b`. Big-operator limits stay on the operator.

use latex_rust::parse;

fn expect_gold(problems: &mut Vec<String>, input: &str, expect: &str) {
    match parse(input) {
        Ok(node) => {
            let got = node.gold();
            if got != expect {
                problems.push(format!("{input}\n  got:    {got}\n  expect: {expect}"));
            }
        }
        Err(err) => problems.push(format!("{input}: {err}")),
    }
}

fn finish(problems: Vec<String>) {
    assert!(problems.is_empty(), "binding:\n{}", problems.join("\n"));
}

#[test]
fn parenthesized_base_takes_the_script() {
    let mut problems = Vec::new();
    let row_neg = r#"(row (atom Open "(") (atom Bin "-") (atom Ord "b") (atom Close ")"))"#;
    expect_gold(
        &mut problems,
        r"(-b)^{n}",
        &format!(r#"(sup {row_neg} (atom Ord "n"))"#),
    );
    expect_gold(
        &mut problems,
        r"(-b)^n",
        &format!(r#"(sup {row_neg} (atom Ord "n"))"#),
    );
    expect_gold(
        &mut problems,
        r"(-b)_{n}",
        &format!(r#"(sub {row_neg} (atom Ord "n"))"#),
    );
    expect_gold(
        &mut problems,
        r"(-b)_i^n",
        &format!(r#"(subsup {row_neg} (atom Ord "i") (atom Ord "n"))"#),
    );
    expect_gold(
        &mut problems,
        r"(a/b)^{n}",
        r#"(sup (row (atom Open "(") (atom Ord "a") (atom Ord "/") (atom Ord "b") (atom Close ")")) (atom Ord "n"))"#,
    );
    expect_gold(
        &mut problems,
        r"(a/b)_i",
        r#"(sub (row (atom Open "(") (atom Ord "a") (atom Ord "/") (atom Ord "b") (atom Close ")")) (atom Ord "i"))"#,
    );
    expect_gold(
        &mut problems,
        r"(\frac{a}{b})^{n}",
        r#"(sup (row (atom Open "(") (frac (atom Ord "a") (atom Ord "b")) (atom Close ")")) (atom Ord "n"))"#,
    );
    expect_gold(
        &mut problems,
        r"(2x)^{2}",
        r#"(sup (row (atom Open "(") (atom Ord "2") (atom Ord "x") (atom Close ")")) (atom Ord "2"))"#,
    );
    expect_gold(
        &mut problems,
        r"x^(-b)",
        &format!(r#"(sup (atom Ord "x") {row_neg})"#),
    );
    expect_gold(
        &mut problems,
        r"(f(x))'",
        r#"(sup (row (atom Open "(") (atom Ord "f") (atom Open "(") (atom Ord "x") (atom Close ")") (atom Close ")")) (atom Ord "′"))"#,
    );
    expect_gold(
        &mut problems,
        r"((a))^{n}",
        r#"(sup (row (atom Open "(") (atom Open "(") (atom Ord "a") (atom Close ")") (atom Close ")")) (atom Ord "n"))"#,
    );
    expect_gold(
        &mut problems,
        r"(((-b)))^{n}",
        r#"(sup (row (atom Open "(") (atom Open "(") (atom Open "(") (atom Bin "-") (atom Ord "b") (atom Close ")") (atom Close ")") (atom Close ")")) (atom Ord "n"))"#,
    );
    expect_gold(
        &mut problems,
        r"(a(b))^{n}",
        r#"(sup (row (atom Open "(") (atom Ord "a") (atom Open "(") (atom Ord "b") (atom Close ")") (atom Close ")")) (atom Ord "n"))"#,
    );
    expect_gold(
        &mut problems,
        r"x^((a))",
        r#"(sup (atom Ord "x") (row (atom Open "(") (atom Open "(") (atom Ord "a") (atom Close ")") (atom Close ")")))"#,
    );
    finish(problems);
}

#[test]
fn prime_then_superscript_nests() {
    let nested = r#"(sup (sup (atom Ord "f") (atom Ord "′")) (atom Ord "2"))"#;
    let mut problems = Vec::new();
    expect_gold(&mut problems, r"f'^2", nested);
    expect_gold(&mut problems, r"{f'}^2", nested);
    expect_gold(
        &mut problems,
        r"f'",
        r#"(sup (atom Ord "f") (atom Ord "′"))"#,
    );
    expect_gold(
        &mut problems,
        r"f''",
        r#"(sup (atom Ord "f") (row (atom Ord "′") (atom Ord "′")))"#,
    );
    expect_gold(
        &mut problems,
        r"f'''",
        r#"(sup (atom Ord "f") (row (row (atom Ord "′") (atom Ord "′")) (atom Ord "′")))"#,
    );
    finish(problems);
}

#[test]
fn single_nucleus_scripts_stay() {
    let mut problems = Vec::new();
    let cases = [
        (r"x^n", r#"(sup (atom Ord "x") (atom Ord "n"))"#),
        (r"x_i", r#"(sub (atom Ord "x") (atom Ord "i"))"#),
        (
            r"x_i^n",
            r#"(subsup (atom Ord "x") (atom Ord "i") (atom Ord "n"))"#,
        ),
        (r"x'", r#"(sup (atom Ord "x") (atom Ord "′"))"#),
        (
            r"-b^n",
            r#"(row (atom Bin "-") (sup (atom Ord "b") (atom Ord "n")))"#,
        ),
        (
            r"-b_i",
            r#"(row (atom Bin "-") (sub (atom Ord "b") (atom Ord "i")))"#,
        ),
        (
            r"-b_i^n",
            r#"(row (atom Bin "-") (subsup (atom Ord "b") (atom Ord "i") (atom Ord "n")))"#,
        ),
        (
            r"-b'",
            r#"(row (atom Bin "-") (sup (atom Ord "b") (atom Ord "′")))"#,
        ),
        (
            r"a/b^n",
            r#"(row (atom Ord "a") (atom Ord "/") (sup (atom Ord "b") (atom Ord "n")))"#,
        ),
        (
            r"a/b_i",
            r#"(row (atom Ord "a") (atom Ord "/") (sub (atom Ord "b") (atom Ord "i")))"#,
        ),
        (
            r"a/b_i^n",
            r#"(row (atom Ord "a") (atom Ord "/") (subsup (atom Ord "b") (atom Ord "i") (atom Ord "n")))"#,
        ),
        (
            r"a/b'",
            r#"(row (atom Ord "a") (atom Ord "/") (sup (atom Ord "b") (atom Ord "′")))"#,
        ),
        (
            r"\frac{a}{b}^n",
            r#"(sup (frac (atom Ord "a") (atom Ord "b")) (atom Ord "n"))"#,
        ),
        (
            r"\frac{a}{b}_i",
            r#"(sub (frac (atom Ord "a") (atom Ord "b")) (atom Ord "i"))"#,
        ),
        (
            r"\frac{a}{b}_i^n",
            r#"(subsup (frac (atom Ord "a") (atom Ord "b")) (atom Ord "i") (atom Ord "n"))"#,
        ),
        (
            r"\frac{a}{b}'",
            r#"(sup (frac (atom Ord "a") (atom Ord "b")) (atom Ord "′"))"#,
        ),
        (
            r"{(-b)}^{n}",
            r#"(sup (row (atom Open "(") (atom Bin "-") (atom Ord "b") (atom Close ")")) (atom Ord "n"))"#,
        ),
        (
            r"\left(-b\right)^{n}",
            r#"(sup (delim ( (row (atom Bin "-") (atom Ord "b")) )) (atom Ord "n"))"#,
        ),
        (
            r"x^{-b}",
            r#"(sup (atom Ord "x") (row (atom Bin "-") (atom Ord "b")))"#,
        ),
        (
            r"\sum_{i=0}^{n}",
            r#"(sum (row (atom Ord "i") (atom Rel "=") (atom Ord "0")) (atom Ord "n"))"#,
        ),
        (
            r"\prod_{i=1}^{n}",
            r#"(prod (row (atom Ord "i") (atom Rel "=") (atom Ord "1")) (atom Ord "n"))"#,
        ),
        (r"\int_{0}^{1}", r#"(int (atom Ord "0") (atom Ord "1"))"#),
        (
            r"\lim_{x \to 0}",
            r#"(lim (row (atom Ord "x") (symbol to) (atom Ord "0")))"#,
        ),
    ];
    for (input, expect) in cases {
        expect_gold(&mut problems, input, expect);
    }
    finish(problems);
}
