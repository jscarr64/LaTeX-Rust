// SPDX-License-Identifier: MIT OR Apache-2.0

use latex_rust::{layout, parse, styled_char, MathFont, MathStyle, TextStyle};

#[test]
fn hat_tilde_accents_keep_the_direct_nucleus_italic_advance() {
    let font = MathFont::stix_two_math().expect("embedded STIX Two Math");

    let italic_j = font
        .glyph(styled_char('J', TextStyle::It))
        .expect("default math italic J");

    let italic = font.italic_correction(italic_j.glyph_id);

    assert!(
        !italic.is_zero(),
        "STIX fixture must have nonzero italic correction for math italic J"
    );

    let expected = &italic_j.advance + &italic;

    for source in [r"\hat{J}", r"\tilde{J}", r"\widehat{J}", r"\widetilde{J}"] {
        let ast = parse(source).expect("parse accent nucleus width case");

        let laid = layout(&ast, &font, MathStyle::Text).expect("layout accent nucleus width case");

        assert!(
            laid.width.eq_dim(&expected),
            "{source}: width {} != direct-nucleus width {}",
            laid.width.to_dec_string(),
            expected.to_dec_string()
        );
    }
}
