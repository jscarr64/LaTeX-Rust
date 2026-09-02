//! Layout and SVG render timings for the build-sheet `benches/` slot.

use std::time::Instant;

use latex_rust::{latex_to_svg, layout, parse, MathFont, MathStyle, SvgOptions};

fn main() {
    let font = MathFont::stix_two_math().expect("STIX Two Math");
    let opt = SvgOptions::new();
    let samples = [
        r"\frac{a+b}{c+d}",
        r"\sum_{k=1}^{n} k",
        r"\begin{pmatrix}a&b\\c&d\end{pmatrix}",
    ];
    for tex in samples {
        let t0 = Instant::now();
        let ast = parse(tex).expect("parse");
        let _bx = layout(&ast, &font, MathStyle::Display).expect("layout");
        let _svg = latex_to_svg(tex, &font, &opt).expect("svg");
        println!("{tex} {:?}", t0.elapsed());
    }
}
