// SPDX-License-Identifier: MIT OR Apache-2.0

use core::cmp::Ordering;

use latex_rust::{
    layout, parse, BoxContent, Dim, MathBox, MathFont, MathNode, MathParams, MathStyle,
};

fn substack_rows(ast: &MathNode) -> &[MathNode] {
    let MathNode::Substack(lines) = ast else {
        panic!("expected substack AST");
    };

    lines
}

fn stack_constants(font: &MathFont, params: &MathParams) -> (Dim, Dim) {
    let math = font.face().tables().math.expect("MATH table");

    let constants = math.constants.expect("MATH constants");

    let units_per_em = font.units_per_em();

    let fu = |value: i16| Dim::from_font_units(i64::from(value), units_per_em);

    let scale = params.scale(MathStyle::Script);

    let baseline_skip = &(fu(constants.stack_top_shift_up().value)
        + fu(constants.stack_bottom_shift_down().value))
        * &scale;

    let line_skip = &fu(constants.stack_gap_min().value) * &scale;

    (baseline_skip, line_skip)
}

fn expected_gaps(rows: &[MathBox], baseline_skip: &Dim, line_skip: &Dim) -> Vec<Dim> {
    rows.windows(2)
        .map(|pair| {
            let candidate = &(baseline_skip - &pair[0].depth) - &pair[1].height;

            if candidate
                .cmp(line_skip)
                .is_some_and(|ordering| ordering != Ordering::Less)
            {
                candidate
            } else {
                line_skip.clone()
            }
        })
        .collect()
}

#[test]
fn substack_uses_scriptstyle_rows_math_stack_spacing_and_vcenter() {
    let font = MathFont::stix_two_math().expect("STIX Two Math");

    let params = MathParams::from_font(&font).expect("MATH constants");

    let ast = parse(r"\substack{1\le i\le n\\i\ne j}").expect("substack");

    let lines = substack_rows(&ast);

    let script_rows: Vec<_> = lines
        .iter()
        .map(|line| layout(line, &font, MathStyle::Script).expect("script row"))
        .collect();

    let scriptscript_rows: Vec<_> = lines
        .iter()
        .map(|line| layout(line, &font, MathStyle::ScriptScript).expect("scriptscript row"))
        .collect();

    let script_width = script_rows
        .iter()
        .fold(Dim::zero(), |width, row| width.max(&row.width));

    let scriptscript_width = scriptscript_rows
        .iter()
        .fold(Dim::zero(), |width, row| width.max(&row.width));

    assert_ne!(
        script_width, scriptscript_width,
        "fixture must distinguish Script from ScriptScript rows"
    );

    let tree = layout(&ast, &font, MathStyle::Script).expect("substack layout");

    assert!(tree.width.eq_dim(&script_width,));

    let (baseline_skip, line_skip) = stack_constants(&font, &params);

    let gaps = expected_gaps(&script_rows, &baseline_skip, &line_skip);

    let rows_span = script_rows
        .iter()
        .fold(Dim::zero(), |span, row| &(&span + &row.height) + &row.depth);

    let expected_span = gaps.iter().fold(rows_span, |span, gap| &span + gap);

    let actual_span = &tree.height + &tree.depth;

    assert!(actual_span.eq_dim(&expected_span,));

    let axis = &params.axis_height * &params.scale(MathStyle::Script);

    let center = &(&tree.height - &tree.depth) / &Dim::from_i64(2);

    assert!(
        center.eq_dim(&axis),
        "substack center {} != Script axis {}",
        center.to_dec_string(),
        axis.to_dec_string()
    );
}

#[test]
fn display_sum_preserves_substack_vcenter_inside_lower_limit() {
    let font = MathFont::stix_two_math().expect("STIX Two Math");

    let params = MathParams::from_font(&font).expect("MATH constants");

    let lower_ast =
        parse(r"\substack{1\le i\le n\\1\le j\le m\\i\ne j}").expect("substack lower limit");

    let lower = layout(&lower_ast, &font, MathStyle::Script).expect("standalone substack");

    let sum_ast =
        parse(r"\sum_{\substack{1\le i\le n\\1\le j\le m\\i\ne j}}").expect("display sum");

    let sum = layout(&sum_ast, &font, MathStyle::Display).expect("display sum layout");

    assert!(sum.width.eq_dim(&lower.width,));

    let BoxContent::Overlap(branches) = &sum.content else {
        panic!("display sum with a lower limit must be an overlap");
    };

    let [_, lower_branch] = branches.as_slice() else {
        panic!("expected operator and lower-limit branches");
    };

    assert!(lower_branch
        .shift
        .cmp(&Dim::zero())
        .is_some_and(|ordering| { ordering == Ordering::Less },));

    let axis = &params.axis_height * &params.scale(MathStyle::Script);

    let intrinsic_center = &(&lower_branch.height - &lower_branch.depth) / &Dim::from_i64(2);

    assert!(
        intrinsic_center.eq_dim(&axis),
        "external limit shift must not overwrite substack vcenter"
    );
}
