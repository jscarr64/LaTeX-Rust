//! A control space is `\` plus one ASCII space. That space is the token,
//! not padding `trim` may delete.

use latex_rust::{parse, MathNode, SpaceKind};

fn control() -> MathNode {
    MathNode::Space(SpaceKind::ControlSpace)
}

#[test]
fn control_space_at_end_of_input() {
    let ast = parse(r"\ ").unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(ast, control());
}

#[test]
fn control_space_before_a_comment() {
    let src = "\\ %@QJXi";
    assert_eq!(src.as_bytes(), b"\\ %@QJXi");
    let ast = parse(src).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(ast, control());
}

#[test]
fn control_space_round_trips() {
    let ast = parse(r"\ ").unwrap_or_else(|err| panic!("{err}"));
    let printed = ast.to_latex();
    assert_eq!(printed, r"\ ");
    let again = parse(&printed).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(again, ast);
    assert_eq!(again.to_latex(), printed);

    let commented = parse("\\ %@QJXi").unwrap_or_else(|err| panic!("{err}"));
    let reprint = commented.to_latex();
    assert_eq!(reprint, r"\ ");
    assert_eq!(
        parse(&reprint).unwrap_or_else(|err| panic!("{err}")),
        commented
    );
}

#[test]
fn control_space_with_other_whitespace_around_it() {
    let ast = parse(" \t\\ \n x \t\\  \n").unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(
        ast.gold(),
        r#"(row (space control) (atom Ord "x") (space control))"#
    );
    let printed = ast.to_latex();
    assert_eq!(printed, r"\ x\ ");
    let again = parse(&printed).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(again, ast);
    assert_eq!(again.to_latex(), printed);
}
