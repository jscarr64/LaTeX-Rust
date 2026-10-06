// SPDX-License-Identifier: MIT OR Apache-2.0

use latex_rust::{
    latex_to_svg, layout, layout_with_em_size_pt, layout_with_max_depth, parse, render_svg, Dim,
    MathFont, MathStyle, SvgOptions,
};

#[test]
fn fraction_width_is_derived_from_its_content() {
    let font = MathFont::stix_two_math().expect("STIX Two Math");
    let style = MathStyle::Text;

    let numerator_ast = parse("1").expect("numerator");
    let denominator_ast = parse("2").expect("denominator");
    let numerator = layout(&numerator_ast, &font, style.numerator()).expect("numerator layout");
    let denominator =
        layout(&denominator_ast, &font, style.denominator()).expect("denominator layout");
    let expected_width = numerator.width.max(&denominator.width);

    let ast = parse(r"\frac{1}{2}").expect("fraction");
    let fraction = layout(&ast, &font, style).expect("fraction layout");

    assert!(
        fraction.width.eq_dim(&expected_width),
        "fraction width was {}, expected {}",
        fraction.width.to_dec_string(),
        expected_width.to_dec_string()
    );

    let ten_pt =
        layout_with_em_size_pt(&ast, &font, style, &Dim::from_i64(10)).expect("10 pt fraction");

    let twenty_pt =
        layout_with_em_size_pt(&ast, &font, style, &Dim::from_i64(20)).expect("20 pt fraction");

    assert!(ten_pt.width.eq_dim(&twenty_pt.width));
    assert!(ten_pt.height.eq_dim(&twenty_pt.height));
    assert!(ten_pt.depth.eq_dim(&twenty_pt.depth));
}

#[test]
fn svg_frontend_uses_the_renderer_em_size_for_delimiters() {
    let font = MathFont::stix_two_math().expect("STIX Two Math");
    let source = r"\left(\rule{0pt}{1.96em}\right)";
    let ast = parse(source).expect("delimited rule");

    let ten_pt = layout_with_em_size_pt(&ast, &font, MathStyle::Display, &Dim::from_i64(10))
        .expect("10 pt layout");

    let twenty_pt = layout_with_em_size_pt(&ast, &font, MathStyle::Display, &Dim::from_i64(20))
        .expect("20 pt layout");

    let mut options = SvgOptions::new();
    options.font_size_pt = Dim::from_i64(20);
    options.display = true;

    let expected = render_svg(&twenty_pt, &font, &options).expect("20 pt render");
    let ten_pt_render =
        render_svg(&ten_pt, &font, &options).expect("10 pt layout rendered at 20 pt");

    assert_ne!(
        expected, ten_pt_render,
        "fixture must distinguish the two physical em sizes"
    );

    let actual = latex_to_svg(source, &font, &options).expect("SVG frontend");

    assert_eq!(actual, expected);
}

#[test]
fn explicit_em_size_validation_and_max_depth_keep_compatible_defaults() {
    let font = MathFont::stix_two_math().expect("STIX Two Math");
    let ast = parse(r"\frac{1}{2}").expect("fraction");

    let default = layout(&ast, &font, MathStyle::Text).expect("default layout");
    let bounded = layout_with_max_depth(&ast, &font, MathStyle::Text, 64).expect("bounded layout");

    assert!(default.width.eq_dim(&bounded.width));
    assert!(default.height.eq_dim(&bounded.height));
    assert!(default.depth.eq_dim(&bounded.depth));

    assert!(layout_with_em_size_pt(&ast, &font, MathStyle::Text, &Dim::zero(),).is_err());

    assert!(layout_with_em_size_pt(&ast, &font, MathStyle::Text, &Dim::ratio(1, 0),).is_err());
}
