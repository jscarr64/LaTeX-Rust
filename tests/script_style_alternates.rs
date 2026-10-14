// SPDX-License-Identifier: MIT OR Apache-2.0

use latex_rust::{layout, parse, BoxContent, MathFont, MathStyle};

fn glyph_id(source: &str, style: MathStyle, font: &MathFont) -> u16 {
    let ast = parse(source).expect("parse glyph case");
    let bx = layout(&ast, font, style).expect("layout glyph case");
    let BoxContent::Glyph { glyph_id, .. } = bx.content else {
        panic!("single glyph case must stay a glyph box");
    };
    glyph_id
}

#[test]
fn script_styles_use_open_type_ssty_alternates() {
    let font = MathFont::stix_two_math().expect("STIX Two Math");
    // STIX Two Math digit 2: base, `ssty` alternate 0, `ssty` alternate 1.
    let base = font.glyph('2').expect("digit 2").glyph_id;
    assert_eq!(base, 1139);
    assert_eq!(glyph_id("2", MathStyle::Text, &font), 1139);
    assert_eq!(glyph_id("2", MathStyle::Script, &font), 4275);
    assert_eq!(glyph_id("2", MathStyle::ScriptScript, &font), 4524);
}
