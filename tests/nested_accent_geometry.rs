// SPDX-License-Identifier: MIT OR Apache-2.0

use latex_rust::{layout, parse, BoxContent, MathBox, MathFont, MathParams, MathStyle};

fn semantic_child(bx: &MathBox) -> &MathBox {
    match &bx.content {
        BoxContent::HList(children) => {
            let mut meaningful = children
                .iter()
                .filter(|child| !matches!(&child.content, BoxContent::Empty | BoxContent::Kern(_)));

            let inner = meaningful
                .next()
                .expect("accent wrapper must contain one semantic child");

            assert!(
                meaningful.next().is_none(),
                "accent wrapper contained more than one semantic child"
            );

            semantic_child(inner)
        }

        _ => bx,
    }
}

fn assert_hat_tilde_layer(
    bx: &MathBox,
    params: &MathParams,
    style: MathStyle,
    remaining_layers: usize,
) {
    let BoxContent::Overlap(children) = &bx.content else {
        panic!("hat/tilde accent must be an overlap");
    };

    let [base, accent] = children.as_slice() else {
        panic!("hat/tilde overlap must contain base and accent branches");
    };

    let scale = params.scale(style);

    let accent_base_height = &params.accent_base_height * &scale;

    // For a nested accent chain:
    // LuaTeX/OpenType raise a top accent only by the amount by
    // which the completed base exceeds AccentBaseHeight. For a
    // nested chain, `base` is the already-built inner accent box.
    let expected_raise = (&base.height - &accent_base_height).clamp_nonneg();

    let actual_raise = &accent.shift - &base.shift;

    assert!(
        actual_raise.eq_dim(&expected_raise,),
        "accent raise {} != corrected raise {}",
        actual_raise.to_dec_string(),
        expected_raise.to_dec_string()
    );

    // The historical defect used max(base_height,
    // AccentBaseHeight) for non-zero-width accents.
    let old_buggy_raise = base.height.max(&accent_base_height);

    assert!(
        !actual_raise.eq_dim(&old_buggy_raise,),
        "fixture does not distinguish the old compounded accent raise"
    );

    let expected_height = base.height.max(&(&accent.height + &expected_raise));

    let expected_depth = base
        .depth
        .max(&(&accent.depth - &expected_raise).clamp_nonneg());

    assert!(
        bx.height.eq_dim(&expected_height,),
        "accent height {} != completed geometry {}",
        bx.height.to_dec_string(),
        expected_height.to_dec_string()
    );

    assert!(
        bx.depth.eq_dim(&expected_depth,),
        "accent depth {} != completed geometry {}",
        bx.depth.to_dec_string(),
        expected_depth.to_dec_string()
    );

    if remaining_layers > 1 {
        let inner = semantic_child(base);

        assert!(
            matches!(&inner.content, BoxContent::Overlap(_)),
            "nested accent base must retain the completed inner accent"
        );

        // Accent nuclei are recursively laid out in cramped style.
        assert_hat_tilde_layer(inner, params, style.cramp(), remaining_layers - 1);
    }
}

#[test]
fn nested_hat_tilde_chain_propagates_completed_inner_geometry() {
    let font = MathFont::stix_two_math().expect("embedded STIX Two Math");

    let params = MathParams::from_font(&font).expect("OpenType MATH constants");

    let ast = parse(r"\widehat{\widetilde{\widehat{ABC}}}").expect("nested hat/tilde expression");

    let tree = layout(&ast, &font, MathStyle::Display).expect("nested accent layout");

    assert_hat_tilde_layer(&tree, &params, MathStyle::Display, 3);
}

#[test]
fn single_hat_tilde_keeps_the_existing_accent_geometry() {
    let font = MathFont::stix_two_math().expect("embedded STIX Two Math");

    let params = MathParams::from_font(&font).expect("OpenType MATH constants");

    for source in [
        r"\hat{ABC}",
        r"\widehat{ABC}",
        r"\tilde{ABC}",
        r"\widetilde{ABC}",
    ] {
        let ast = parse(source).expect("single hat/tilde expression");

        let tree = layout(&ast, &font, MathStyle::Display).expect("single accent layout");

        assert_hat_tilde_layer(&tree, &params, MathStyle::Display, 1);
    }
}
