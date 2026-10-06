// SPDX-License-Identifier: MIT OR Apache-2.0

use core::cmp::Ordering;

use latex_rust::{
    layout_with_em_size_pt, parse, BoxContent, Dim, MathBox, MathFont, MathParams, MathStyle,
};

fn hlist_children(bx: &MathBox) -> &[MathBox] {
    let BoxContent::HList(children) = &bx.content else {
        panic!("expected HList");
    };

    children
}

fn vlist_children(bx: &MathBox) -> &[MathBox] {
    let BoxContent::VList(children) = &bx.content else {
        panic!("expected VList");
    };

    children
}

fn environment_stack(tree: &MathBox) -> &MathBox {
    if matches!(&tree.content, BoxContent::VList(_)) {
        return tree;
    }

    hlist_children(tree)
        .iter()
        .find(|child| matches!(&child.content, BoxContent::VList(_)))
        .expect("environment stack")
}

fn assert_axis_centered(stack: &MathBox, axis: &Dim) {
    let height = (&stack.height + &stack.shift).clamp_nonneg();

    let depth = (&stack.depth - &stack.shift).clamp_nonneg();

    let center = &(&height - &depth) / &Dim::from_i64(2);

    assert!(
        center.eq_dim(axis),
        "stack center {} != axis {}",
        center.to_dec_string(),
        axis.to_dec_string()
    );
}

fn kern_width(bx: &MathBox) -> &Dim {
    let BoxContent::Kern(width) = &bx.content else {
        panic!("expected kern");
    };

    width
}

#[test]
fn matrix_uses_textstyle_physical_array_spacing_and_axis_center() {
    let font = MathFont::stix_two_math().expect("STIX Two Math");

    let params = MathParams::from_font(&font).expect("MATH constants");

    let style = MathStyle::Display;

    let em_size_pt = Dim::from_i64(20);

    let axis = &params.axis_height * &params.scale(style);

    let ast = parse(r"\begin{bmatrix}\frac{1}{2}&x\\y&z\end{bmatrix}").expect("bmatrix");

    let tree = layout_with_em_size_pt(&ast, &font, style, &em_size_pt).expect("bmatrix layout");

    let outer = hlist_children(&tree);

    assert_eq!(
        outer.len(),
        3,
        "expected left delimiter, stack and right delimiter"
    );

    let stack = &outer[1];

    assert_axis_centered(stack, &axis);

    for delimiter in [&outer[0], &outer[2]] {
        let center = &(&delimiter.height - &delimiter.depth) / &Dim::from_i64(2);

        assert!((&center + &delimiter.shift).eq_dim(&axis));
    }

    let children = vlist_children(stack);

    assert_eq!(
        children.len(),
        3,
        "row, lineskip, row: the fraction is taller than \\baselineskip"
    );

    let rows = [&children[0], &children[2]];

    let baselineskip = &Dim::from_i64(12) / &em_size_pt;

    let line_skip = &Dim::from_i64(1) / &em_size_pt;

    let candidate = &(&baselineskip - &rows[0].depth) - &rows[1].height;

    assert!(
        matches!(candidate.cmp(&Dim::zero()), Some(Ordering::Less)),
        "fixture must cross into \\lineskip"
    );

    assert!(children[1].depth.eq_dim(&line_skip));

    let parts = hlist_children(rows[0]);

    assert_eq!(
        parts.len(),
        4,
        "strut, first cell, intercolumn kern, second cell"
    );

    let fraction_ast = parse(r"\frac{1}{2}").expect("fraction");

    let text_fraction = layout_with_em_size_pt(&fraction_ast, &font, MathStyle::Text, &em_size_pt)
        .expect("text fraction");

    let display_fraction =
        layout_with_em_size_pt(&fraction_ast, &font, MathStyle::Display, &em_size_pt)
            .expect("display fraction");

    assert!(parts[1].height.eq_dim(&text_fraction.height));

    assert!(parts[1].depth.eq_dim(&text_fraction.depth));

    assert_ne!(parts[1].height, display_fraction.height);

    let expected_gap = &Dim::from_i64(10) / &em_size_pt;

    assert!(kern_width(&parts[2]).eq_dim(&expected_gap));

    assert_ne!(
        expected_gap,
        Dim::one(),
        "20pt fixture must expose physical spacing"
    );
}

#[test]
fn cases_use_arraystretch_quad_gap_and_axis_center() {
    let font = MathFont::stix_two_math().expect("STIX Two Math");

    let params = MathParams::from_font(&font).expect("MATH constants");

    let style = MathStyle::Display;

    let em_size_pt = Dim::from_i64(10);

    let axis = &params.axis_height * &params.scale(style);

    let ast = parse(r"\begin{cases}x,&x<0\\y,&x\ge0\end{cases}").expect("cases");

    let tree = layout_with_em_size_pt(&ast, &font, style, &em_size_pt).expect("cases layout");

    let outer = hlist_children(&tree);

    assert_eq!(outer.len(), 2, "expected left brace and cases stack");

    let stack = &outer[1];

    assert_axis_centered(stack, &axis);

    let children = vlist_children(stack);

    assert_eq!(
        children.len(),
        3,
        "row, lineskip, row: arraystretch makes the strut taller than \\baselineskip"
    );

    let baselineskip = &Dim::from_i64(12) / &em_size_pt;

    let line_skip = &Dim::from_i64(1) / &em_size_pt;

    let array_stretch = Dim::ratio(6, 5);

    let strut_box = &baselineskip * &array_stretch;

    let expected_strut_height = &Dim::ratio(7, 10) * &strut_box;

    let expected_strut_depth = &Dim::ratio(3, 10) * &strut_box;

    let candidate = &(&baselineskip - &children[0].depth) - &children[2].height;

    assert!(
        matches!(candidate.cmp(&Dim::zero()), Some(Ordering::Less)),
        "fixture must cross into \\lineskip"
    );

    assert!(children[1].depth.eq_dim(&line_skip));

    for row in [&children[0], &children[2]] {
        let parts = hlist_children(row);

        assert_eq!(parts.len(), 4, "strut, first cell, quad, second cell");

        let BoxContent::Rule = &parts[0].content else {
            panic!("expected cases strut");
        };

        assert!(parts[0].height.eq_dim(&expected_strut_height));

        assert!(parts[0].depth.eq_dim(&expected_strut_depth));

        assert!(kern_width(&parts[2]).eq_dim(&Dim::one()));
    }
}

#[test]
fn aligned_applies_empty_ord_right_field_preamble() {
    let font = MathFont::stix_two_math().expect("STIX Two Math");

    let em_size_pt = Dim::from_i64(10);

    let ast = parse(r"\begin{aligned}a&=b\end{aligned}").expect("aligned");

    let tree =
        layout_with_em_size_pt(&ast, &font, MathStyle::Text, &em_size_pt).expect("aligned layout");

    let stack = environment_stack(&tree);

    let rows = vlist_children(stack);

    assert_eq!(rows.len(), 1,);

    let left = parse("a").expect("left field");

    let right = parse("=b").expect("right field");

    let prefixed = parse("{}=b").expect("empty-Ord right field");

    let left = layout_with_em_size_pt(&left, &font, MathStyle::Display, &em_size_pt)
        .expect("left display layout");

    let right = layout_with_em_size_pt(&right, &font, MathStyle::Display, &em_size_pt)
        .expect("right display layout");

    let prefixed = layout_with_em_size_pt(&prefixed, &font, MathStyle::Display, &em_size_pt)
        .expect("prefixed display layout");

    let leading = (&prefixed.width - &right.width).clamp_nonneg();

    assert!(!leading.is_zero(), "fixture must expose Ord-to-Rel spacing");

    let expected = &left.width + &prefixed.width;

    assert!(rows[0].width.eq_dim(&expected));
}

#[test]
fn aligned_uses_displaystyle_cells_and_physical_minalignsep() {
    let font = MathFont::stix_two_math().expect("STIX Two Math");

    let em_size_pt = Dim::from_i64(20);

    let ast = parse(r"\begin{aligned}\frac{1}{2}&=x&y&=z\end{aligned}").expect("aligned");

    let tree =
        layout_with_em_size_pt(&ast, &font, MathStyle::Text, &em_size_pt).expect("aligned layout");

    let stack = environment_stack(&tree);

    let rows = vlist_children(stack);

    assert_eq!(rows.len(), 1,);

    let parts = hlist_children(&rows[0]);

    assert_eq!(parts.len(), 6, "strut, four fields and one pair separator");

    let fraction_ast = parse(r"\frac{1}{2}").expect("fraction");

    let display_fraction =
        layout_with_em_size_pt(&fraction_ast, &font, MathStyle::Display, &em_size_pt)
            .expect("display fraction");

    assert!(parts[1].height.eq_dim(&display_fraction.height));

    assert!(parts[1].depth.eq_dim(&display_fraction.depth));

    let expected_pair_gap = &Dim::from_i64(10) / &em_size_pt;

    assert!(kern_width(&parts[3]).eq_dim(&expected_pair_gap));

    assert_ne!(
        expected_pair_gap,
        Dim::one(),
        "20pt fixture must expose physical spacing"
    );
}

#[test]
fn aligned_uses_jot_lineskip_and_centers_complete_stack() {
    let font = MathFont::stix_two_math().expect("STIX Two Math");

    let params = MathParams::from_font(&font).expect("MATH constants");

    let style = MathStyle::Display;

    let em_size_pt = Dim::from_i64(20);

    let axis = &params.axis_height * &params.scale(style);

    let ast = parse(r"\begin{aligned}a&=b\\c&=d\end{aligned}").expect("aligned");

    let tree = layout_with_em_size_pt(&ast, &font, style, &em_size_pt).expect("aligned layout");

    let stack = environment_stack(&tree);

    assert_axis_centered(stack, &axis);

    let children = vlist_children(stack);

    assert_eq!(children.len(), 3, "row, inter-row glue, row");

    let first = &children[0];

    let gap = &children[1];

    let second = &children[2];

    let BoxContent::Empty = &gap.content else {
        panic!("expected aligned inter-row glue");
    };

    let jot = &Dim::from_i64(3) / &em_size_pt;

    let baseline_skip = &(&Dim::from_i64(12) / &em_size_pt) + &jot;

    let line_skip = &(&Dim::from_i64(1) / &em_size_pt) + &jot;

    let candidate = &(&baseline_skip - &first.depth) - &second.height;

    let expected_gap = if matches!(candidate.cmp(&jot), Some(Ordering::Less)) {
        line_skip
    } else {
        candidate
    };

    assert!(gap.depth.eq_dim(&expected_gap,));
}
