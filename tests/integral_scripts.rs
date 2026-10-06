// SPDX-License-Identifier: MIT OR Apache-2.0

use core::cmp::Ordering;

use latex_rust::{layout, parse, BoxContent, Dim, MathBox, MathFont, MathParams, MathStyle};

fn layout_source(source: &str, style: MathStyle, font: &MathFont) -> MathBox {
    let ast = parse(source).expect("parse integral case");

    layout(&ast, font, style).expect("layout integral case")
}

struct ScriptConstants {
    subscript_top_max: Dim,
    subscript_baseline_drop_min: Dim,
    superscript_bottom_min: Dim,
    superscript_bottom_max_with_subscript: Dim,
    superscript_baseline_drop_max: Dim,
}

fn script_constants(font: &MathFont) -> ScriptConstants {
    let face = ttf_parser::Face::parse(font.bytes(), 0).expect("parse embedded STIX face");

    let constants = face
        .tables()
        .math
        .and_then(|math| math.constants)
        .expect("embedded STIX MATH constants");

    let units_per_em = font.units_per_em();

    let fu = |value: i16| Dim::from_font_units(i64::from(value), units_per_em);

    ScriptConstants {
        subscript_top_max: fu(constants.subscript_top_max().value),

        subscript_baseline_drop_min: fu(constants.subscript_baseline_drop_min().value),

        superscript_bottom_min: fu(constants.superscript_bottom_min().value),

        superscript_bottom_max_with_subscript: fu(constants
            .superscript_bottom_max_with_subscript()
            .value),

        superscript_baseline_drop_max: fu(constants.superscript_baseline_drop_max().value),
    }
}

fn integral_attachment_parts(tree: &MathBox, expected: char) -> (&MathBox, &MathBox, &MathBox) {
    let BoxContent::HList(children) = &tree.content else {
        panic!("scripted display integral must be an HList");
    };

    assert!(children.len() >= 3, "integral attachment is incomplete");

    let base = &children[0];

    assert!(matches!(
        &base.content,
        BoxContent::Glyph {
            ch,
            ..
        } if *ch == expected
    ));

    let italic_kern = &children[1];

    assert!(matches!(&italic_kern.content, BoxContent::Kern(_)));

    let slot = &children[2];

    assert!(matches!(&slot.content, BoxContent::Overlap(_)));

    (base, italic_kern, slot)
}

#[test]
fn display_contour_integral_uses_axis_baseline_drop_and_nolimits_width() {
    let font = MathFont::stix_two_math().expect("embedded STIX Two Math");

    let params = MathParams::from_font(&font).expect("OpenType MATH parameters");

    let constants = script_constants(&font);

    let style = MathStyle::Display;

    let scale = params.scale(style);

    let tree = layout_source(r"\oint_\Gamma", style, &font);

    let (base, italic_kern, slot) = integral_attachment_parts(&tree, '∮');

    assert!(
        !base.italic.is_zero(),
        "fixture needs nonzero integral italic correction"
    );

    let expected_kern = -base.italic.clone();

    assert!(
        italic_kern.width.eq_dim(&expected_kern,),
        "no-limits width kern {} != -italic {}",
        italic_kern.width.to_dec_string(),
        expected_kern.to_dec_string()
    );

    let raw_center = &(&base.height - &base.depth) / &Dim::from_i64(2);

    let axis = &params.axis_height * &scale;

    assert!((&raw_center + &base.shift).eq_dim(&axis));

    let BoxContent::Overlap(branches) = &slot.content else {
        unreachable!();
    };

    let [lower] = branches.as_slice() else {
        panic!("subscript-only contour integral must contain one lower branch");
    };

    assert!(matches!(
        lower.shift.cmp(&Dim::zero(),),
        Some(Ordering::Less)
    ));

    let base_depth = (&base.depth - &base.shift).clamp_nonneg();

    let standard = &params.subscript_shift_down * &scale;

    let from_base = &base_depth + &(&constants.subscript_baseline_drop_min * &scale);

    let from_top = (&lower.height - &(&constants.subscript_top_max * &scale)).clamp_nonneg();

    let expected_drop = standard.max(&from_base).max(&from_top);

    let actual_drop = -lower.shift.clone();

    assert!(
        actual_drop.eq_dim(&expected_drop,),
        "integral subscript drop {} != OpenType constrained {}",
        actual_drop.to_dec_string(),
        expected_drop.to_dec_string()
    );
}

#[test]
fn display_integral_offsets_only_superscript_by_math_italic_correction() {
    let font = MathFont::stix_two_math().expect("embedded STIX Two Math");

    let params = MathParams::from_font(&font).expect("OpenType MATH parameters");

    let constants = script_constants(&font);

    let style = MathStyle::Display;

    let scale = params.scale(style);

    let tree = layout_source(r"\int_0^1", style, &font);

    let (base, italic_kern, slot) = integral_attachment_parts(&tree, '∫');

    assert!(
        !base.italic.is_zero(),
        "fixture needs nonzero integral italic correction"
    );

    assert!(italic_kern.width.eq_dim(&(-base.italic.clone()),));

    let BoxContent::Overlap(branches) = &slot.content else {
        unreachable!();
    };

    let [upper, lower] = branches.as_slice() else {
        panic!("paired integral scripts must contain upper and lower branches");
    };

    let BoxContent::HList(upper_parts) = &upper.content else {
        panic!("upper integral script must carry the horizontal italic offset");
    };

    let [upper_kern, upper_body] = upper_parts.as_slice() else {
        panic!("upper integral script must contain italic kern and body");
    };

    assert!(upper_kern.width.eq_dim(&base.italic,));

    assert!(matches!(&upper_kern.content, BoxContent::Kern(_)));

    let lower_body = layout_source("0", MathStyle::Script, &font);

    assert!(
        lower.width.eq_dim(&lower_body.width,),
        "subscript width must not include the superscript italic offset"
    );

    let base_height = (&base.height + &base.shift).clamp_nonneg();

    let base_depth = (&base.depth - &base.shift).clamp_nonneg();

    let mut expected_upper = (&params.superscript_shift_up * &scale)
        .max(&(&base_height - &(&constants.superscript_baseline_drop_max * &scale)))
        .max(&(&upper_body.depth + &(&constants.superscript_bottom_min * &scale)));

    let mut expected_lower = (&params.subscript_shift_down * &scale)
        .max(&(&base_depth + &(&constants.subscript_baseline_drop_min * &scale)))
        .max(&(&lower.height - &(&constants.subscript_top_max * &scale)).clamp_nonneg());

    let gap = &expected_upper + &expected_lower - &upper_body.depth - &lower.height;

    let min_gap = &params.sub_superscript_gap_min * &scale;

    if matches!(gap.cmp(&min_gap), Some(Ordering::Less)) {
        expected_lower = &expected_lower + (&min_gap - &gap);
    }

    let current_bottom = &expected_upper - &upper_body.depth;

    let target_bottom = &constants.superscript_bottom_max_with_subscript * &scale;

    if matches!(current_bottom.cmp(&target_bottom,), Some(Ordering::Less)) {
        let raise = &target_bottom - &current_bottom;

        let lowered_subscript = &expected_lower - &raise;

        if !matches!(
            lowered_subscript.cmp(&Dim::zero(),),
            Some(Ordering::Less) | None
        ) {
            expected_upper = &expected_upper + &raise;

            expected_lower = lowered_subscript;
        }
    }

    assert!(upper.shift.eq_dim(&expected_upper,));

    assert!((-lower.shift.clone()).eq_dim(&expected_lower,));
}

#[test]
fn top_level_row_propagates_repaired_integral_geometry() {
    let font = MathFont::stix_two_math().expect("embedded STIX Two Math");

    let standalone = layout_source(r"\oint_\Gamma", MathStyle::Display, &font);

    let row = layout_source(r"\oint_\Gamma z", MathStyle::Display, &font);

    let BoxContent::HList(children) = &row.content else {
        panic!("top-level expression must be an HList");
    };

    let integral = children
        .iter()
        .find(|child| {
            let BoxContent::HList(parts) = &child.content else {
                return false;
            };

            parts.first().is_some_and(|base| {
                matches!(
                    &base.content,
                    BoxContent::Glyph {
                        ch,
                        ..
                    } if *ch == '∮'
                )
            })
        })
        .expect("row must retain the scripted contour integral child");

    assert!(integral.width.eq_dim(&standalone.width,));

    assert!(integral.height.eq_dim(&standalone.height,));

    assert!(integral.depth.eq_dim(&standalone.depth,));

    let expected_depth = children
        .iter()
        .fold(Dim::zero(), |depth, child| depth.max(&child.depth));

    assert!(row.depth.eq_dim(&expected_depth,));
}

#[test]
fn unscripted_double_integral_keeps_existing_variant_path() {
    let font = MathFont::stix_two_math().expect("embedded STIX Two Math");

    let tree = layout_source(r"\iint", MathStyle::Display, &font);

    assert!(matches!(
        &tree.content,
        BoxContent::Glyph {
            ch,
            ..
        } if *ch == '∬'
    ));

    assert!(tree.shift.is_zero());
}
