// SPDX-License-Identifier: MIT OR Apache-2.0

use std::cmp::Ordering;

use latex_rust::{layout, parse, BoxContent, Dim, MathFont, MathParams, MathStyle};

fn selected_radical_glyph(font: &MathFont, target: &Dim, scale: &Dim) -> u16 {
    let base = font.glyph('√').expect("radical glyph");
    let mut best_fitting: Option<(u16, Dim)> = None;
    let mut tallest_short: Option<(u16, Dim)> = None;

    for glyph_id in font.vertical_variants(base.glyph_id) {
        let metrics = font
            .glyph_id('√', glyph_id)
            .expect("radical variant metrics");
        let span = &(&metrics.height + &metrics.depth) * scale;
        match span.cmp(target) {
            Some(Ordering::Equal | Ordering::Greater) => {
                let tighter = best_fitting.as_ref().map_or(true, |(_, best_span)| {
                    span.cmp(best_span) == Some(Ordering::Less)
                });
                if tighter {
                    best_fitting = Some((glyph_id, span));
                }
            }
            Some(Ordering::Less) => {
                let taller = tallest_short.as_ref().map_or(true, |(_, best_span)| {
                    span.cmp(best_span) == Some(Ordering::Greater)
                });
                if taller {
                    tallest_short = Some((glyph_id, span));
                }
            }
            None => panic!("non-finite radical dimensions"),
        }
    }

    best_fitting
        .or(tallest_short)
        .expect("at least the base radical glyph")
        .0
}

fn radical_glyph_id(tree: &latex_rust::MathBox) -> u16 {
    let BoxContent::HList(children) = &tree.content else {
        panic!("expected radical HList");
    };
    children
        .iter()
        .find_map(|child| match &child.content {
            BoxContent::Glyph {
                ch: '√', glyph_id,
            ..
            } => Some(*glyph_id),
            _ => None,
        })
        .expect("radical glyph")
}

#[test]
fn radical_variant_selection_excludes_extra_ascender_from_minimum_span() {
    let font = MathFont::stix_two_math().expect("STIX Two Math");
    let params = MathParams::from_font(&font).expect("MATH constants");
    let style = MathStyle::Display;
    let scale = params.scale(style);

    let radicand = parse(r"x^2+y^2").expect("radicand");
    let radicand_box = layout(&radicand, &font, style.cramp()).expect("radicand layout");
    let gap = &params.radical_display_style_vertical_gap * &scale;
    let thickness = &params.radical_rule_thickness * &scale;
    let extra = &params.radical_extra_ascender * &scale;

    let needed = &(&(&radicand_box.height + &radicand_box.depth) + &gap) + &thickness;
    let inflated_needed = &needed + &extra;
    let expected = selected_radical_glyph(&font, &needed, &scale);
    let defective = selected_radical_glyph(&font, &inflated_needed, &scale);
    assert_ne!(
        expected, defective,
        "fixture must cross a radical variant boundary only because of RadicalExtraAscender"
    );

    let ast = parse(r"\sqrt{x^2+y^2}").expect("radical");
    let tree = layout(&ast, &font, style).expect("radical layout");
    assert_eq!(radical_glyph_id(&tree), expected);
}
