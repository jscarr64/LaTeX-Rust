//! A row break outside a matrix, array, or substack names itself.

use latex_rust::parse;

#[test]
fn stray_row_break_names_itself() {
    for src in [r"a \\ b", r"\\", r"a \\", r"\cr"] {
        let err = parse(src).expect_err(src);
        assert_eq!(
            err.to_string(),
            r"row break \\ outside an environment",
            "{src}: {err:?}"
        );
    }
}

#[test]
fn row_break_inside_an_environment_stays_a_break() {
    let matrix = parse(r"\begin{matrix} a \\ b \end{matrix}").unwrap_or_else(|err| panic!("{err}"));
    assert!(matrix.gold().contains("matrix"), "{}", matrix.gold());
    let sub = parse(r"\substack{a \\ b}").unwrap_or_else(|err| panic!("{err}"));
    assert!(sub.gold().contains("substack"), "{}", sub.gold());
}
