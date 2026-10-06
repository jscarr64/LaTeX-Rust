// SPDX-License-Identifier: MIT OR Apache-2.0

use std::cmp::Ordering;

use latex_rust::{
    layout, parse, styled_char, BoxContent, Dim, MathBox, MathFont, MathParams, MathStyle,
    TextStyle,
};

fn single_glyph_x(bx: &MathBox) -> Option<(Dim, u16)> {
    match &bx.content {
        BoxContent::Glyph { glyph_id, .. } => Some((Dim::zero(), *glyph_id)),

        BoxContent::HList(children) => {
            let mut x = Dim::zero();
            let mut found = None;

            for child in children {
                if let Some((inner_x, glyph_id)) = single_glyph_x(child) {
                    if found.is_some() {
                        return None;
                    }

                    found = Some((&x + &inner_x, glyph_id));
                }

                x = &x + &child.width;
            }

            found
        }

        BoxContent::Color(_, inner)
        | BoxContent::BackColor(_, inner)
        | BoxContent::Frame { inner, .. } => single_glyph_x(inner),

        _ => None,
    }
}

fn accent_branches(source: &str, style: MathStyle, font: &MathFont) -> (MathBox, MathBox, MathBox) {
    let ast = parse(source).expect("parse accent case");

    let tree = layout(&ast, font, style).expect("layout accent case");

    let BoxContent::Overlap(children) = &tree.content else {
        panic!("accent did not produce an overlap");
    };

    let [base, accent] = children.as_slice() else {
        panic!("accent overlap did not contain exactly two branches");
    };

    (tree.clone(), base.clone(), accent.clone())
}

fn smallest_covering_variant(
    font: &MathFont,
    fixed_ch: char,
    seed_ch: char,
    target_width: &Dim,
) -> u16 {
    let fixed = font.glyph(fixed_ch).expect("fixed accent glyph");

    let seed = font.glyph(seed_ch).expect("combining accent seed");

    let mut selected_id = fixed.glyph_id;

    let mut selected_width = fixed.advance.clone();

    let mut covered = selected_width
        .cmp(target_width)
        .is_some_and(|ordering| ordering != Ordering::Less);

    for glyph_id in font.horizontal_variants(seed.glyph_id) {
        if glyph_id == seed.glyph_id {
            continue;
        }

        let candidate = font
            .glyph_id(seed_ch, glyph_id)
            .expect("horizontal accent variant");

        if candidate.advance.is_zero() {
            continue;
        }

        let covers = candidate
            .advance
            .cmp(target_width)
            .is_some_and(|ordering| ordering != Ordering::Less);

        let narrower = candidate
            .advance
            .cmp(&selected_width)
            .is_some_and(|ordering| ordering == Ordering::Less);

        let wider = candidate
            .advance
            .cmp(&selected_width)
            .is_some_and(|ordering| ordering == Ordering::Greater);

        if covers && (!covered || narrower) {
            selected_id = glyph_id;
            selected_width = candidate.advance;
            covered = true;
        } else if !covered && wider {
            selected_id = glyph_id;
            selected_width = candidate.advance;
        }
    }

    selected_id
}

#[test]
fn hat_tilde_accents_follow_math_attachment_and_accent_base_height() {
    let font = MathFont::stix_two_math().expect("embedded STIX Two Math");

    let params = MathParams::from_font(&font).expect("OpenType MATH constants");

    let italic_j = styled_char('J', TextStyle::It);

    let base_metrics = font.glyph(italic_j).expect("italic J glyph");

    let expected_raise = (&base_metrics.height - &params.accent_base_height).clamp_nonneg();

    for (source, fixed_ch) in [
        (r"\hat J", 'ˆ'),
        (r"\widehat J", 'ˆ'),
        (r"\tilde J", '˜'),
        (r"\widetilde J", '˜'),
    ] {
        let (_, base, accent) = accent_branches(source, MathStyle::Text, &font);

        let (base_x, base_glyph_id) = single_glyph_x(&base).expect("single base glyph");

        let (accent_x, accent_glyph_id) = single_glyph_x(&accent).expect("single accent glyph");

        assert_eq!(base_glyph_id, base_metrics.glyph_id, "{source}: base glyph");

        assert!(
            (&accent.shift - &base.shift).eq_dim(&expected_raise),
            "{source}: accent raise was {}, expected {}",
            (&accent.shift - &base.shift).to_dec_string(),
            expected_raise.to_dec_string()
        );

        let accent_metrics = font
            .glyph_id(fixed_ch, accent_glyph_id)
            .expect("selected accent metrics");

        let base_attachment = font
            .top_accent_attachment(base_metrics.glyph_id)
            .unwrap_or_else(|| &base_metrics.advance / &Dim::from_i64(2));

        let accent_attachment = font
            .top_accent_attachment(accent_glyph_id)
            .unwrap_or_else(|| &accent_metrics.advance / &Dim::from_i64(2));

        let expected_x = &base_attachment - &accent_attachment;

        let actual_x = &accent_x - &base_x;

        assert!(
            actual_x.eq_dim(&expected_x),
            "{source}: accent x offset was {}, expected {}",
            actual_x.to_dec_string(),
            expected_x.to_dec_string()
        );
    }
}

#[test]
fn script_style_hat_attachment_uses_script_scale() {
    let font = MathFont::stix_two_math().expect("embedded STIX Two Math");

    let params = MathParams::from_font(&font).expect("OpenType MATH constants");

    let scale = params.scale(MathStyle::Script);

    assert!(
        !scale.eq_dim(&Dim::one()),
        "fixture requires a non-unit script scale"
    );

    let (_, base, accent) = accent_branches(r"\hat J", MathStyle::Script, &font);

    let (base_x, base_id) = single_glyph_x(&base).expect("single script base glyph");

    let (accent_x, accent_id) = single_glyph_x(&accent).expect("single script accent glyph");

    let base_metrics = font.glyph_id('J', base_id).expect("script base metrics");

    let accent_metrics = font
        .glyph_id('ˆ', accent_id)
        .expect("script accent metrics");

    let base_attachment = font.top_accent_attachment(base_id).map_or_else(
        || &base_metrics.advance * &scale / &Dim::from_i64(2),
        |value| value * &scale,
    );

    let accent_attachment = font.top_accent_attachment(accent_id).map_or_else(
        || &accent_metrics.advance * &scale / &Dim::from_i64(2),
        |value| value * &scale,
    );

    let expected = &base_attachment - &accent_attachment;

    let actual = &accent_x - &base_x;

    assert!(
        actual.eq_dim(&expected),
        "script hat x offset was {}, expected {}",
        actual.to_dec_string(),
        expected.to_dec_string()
    );
}

#[test]
fn wide_accents_use_the_smallest_font_variant_that_covers() {
    let font = MathFont::stix_two_math().expect("embedded STIX Two Math");

    let base_ast = parse("XYZ").expect("parse base");

    let base = layout(&base_ast, &font, MathStyle::Text).expect("layout base");

    for (source, fixed_ch, seed_ch) in [
        (r"\widehat{XYZ}", 'ˆ', '\u{0302}'),
        (r"\widetilde{XYZ}", '˜', '\u{0303}'),
    ] {
        let expected = smallest_covering_variant(&font, fixed_ch, seed_ch, &base.width);

        let fixed = font.glyph(fixed_ch).expect("fixed accent glyph");

        let chosen = font
            .glyph_id(seed_ch, expected)
            .expect("covering accent glyph");

        assert_ne!(
            expected, fixed.glyph_id,
            "{source}: fixture needs a wide variant"
        );

        assert!(
            chosen
                .advance
                .cmp(&base.width)
                .is_some_and(|ordering| ordering != Ordering::Less,),
            "{source}: accent must cover the nucleus"
        );

        let (_, _, accent) = accent_branches(source, MathStyle::Text, &font);

        let (_, actual) = single_glyph_x(&accent).expect("single wide accent glyph");

        assert_eq!(actual, expected, "{source}: selected accent variant");
    }
}
