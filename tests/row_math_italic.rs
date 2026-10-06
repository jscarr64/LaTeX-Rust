// SPDX-License-Identifier: MIT OR Apache-2.0

use latex_rust::{layout, parse, styled_char, Dim, MathFont, MathStyle, TextStyle};

fn layout_width(source: &str, font: &MathFont) -> Dim {
    let ast = parse(source).expect("parse row-math-italic case");

    layout(&ast, font, MathStyle::Text)
        .expect("layout row-math-italic case")
        .width
}

fn variable_advance_with_italic(ch: char, font: &MathFont) -> Dim {
    let glyph = font
        .glyph(styled_char(ch, TextStyle::It))
        .expect("default math italic glyph");

    &glyph.advance + &font.italic_correction(glyph.glyph_id)
}

#[test]
fn rows_add_math_italic_correction_without_duplicating_scripted_nuclei() {
    let font = MathFont::stix_two_math().expect("embedded STIX Two Math");

    let expected_xyz = ['X', 'Y', 'Z']
        .into_iter()
        .map(|ch| variable_advance_with_italic(ch, &font))
        .fold(Dim::zero(), |sum, width| &sum + &width);

    assert_eq!(layout_width("XYZ", &font), expected_xyz,);

    let y = font
        .glyph(styled_char('y', TextStyle::It))
        .expect("default math italic y");

    let y_with_italic = &y.advance + &font.italic_correction(y.glyph_id);

    let expected_sup_row = &layout_width("x^2", &font) + &y_with_italic;

    assert_eq!(layout_width("x^2y", &font), expected_sup_row,);

    let expected_sub_row = &layout_width("x_2", &font) + &y_with_italic;

    assert_eq!(layout_width("x_2y", &font), expected_sub_row,);
}
