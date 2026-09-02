# LaTeX-Rust

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE-APACHE)

Repository: <https://github.com/jscarr64/LaTeX-Rust>

Pure-Rust LaTeX **math** renderer. No JavaScript. No webview. No runtime beyond
the Rust standard library plus the crates listed below.

It parses LaTeX math into a typed AST, lays the AST out with a TeX-faithful box
model whose dimensions are [zenith-float](https://crates.io/crates/zenith-float)
`Dim` values (no hardware `f32`/`f64` in the layout path), and renders to SVG.
PNG and egui live under `src/render/` and return `Err` until those milestones.

```
src/parser/     LaTeX → AST
src/layout/     AST → box model
src/font/       STIX Two Math metrics / TrueType loader
src/render/svg  box model → SVG
src/render/png  box model → PNG (Unsupported until Milestone 9)
src/render/egui box model → egui (Unsupported until Milestone 10)
golds/          gold tests
benches/        layout / SVG timings
```

This crate does not typeset documents. Text-mode LaTeX, TikZ, chemistry, and PDF
are out of scope and return `Err` — never a fake render.

## Status

**Milestone 6** (accents and decorations): TeX accent placement in `Dim`,
extensible hats/arrows/braces from MATH variants and glyph assembly, cancel
diagonals as SVG `<line>`, `\boxed` as a stroked frame. Golds in
`golds/accents.toml`. Empty bases and unknown `wide*` commands are `Err`.

**Milestone 5** (symbol coverage): every catalog glyph, TeX atom class, and math
alphabet (`\mathbb`, `\mathcal`, `\mathfrak`, …) renders through STIX Two Math.
Golds in `golds/symbols.toml`. Missing glyphs are `Err`, never a substitute.

**Milestone 4** (SVG renderer): `MathBox` → self-contained SVG (`<path>` from STIX Two Math,
`<rect>` for rules). Golds in `golds/svg.toml`. Layout remains zenith-float `Dim` only.

## Install

```toml
[dependencies]
latex-rust = "0.1"
```

Until the crate is on crates.io:

```toml
[dependencies]
latex-rust = { git = "https://github.com/jscarr64/LaTeX-Rust" }
```

Layout math uses the published [zenith-float](https://crates.io/crates/zenith-float)
**1.0** crate (`ExactNum` software floats, no hardware `f32`/`f64`). This crate
depends on `zenith-float` only — not on its inner kernel package.

## Quick start

```rust
use latex_rust::{parse, layout, latex_to_svg, tokenize, MathFont, MathStyle, Dim, MathBox, SvgOptions};

let ast = parse(r"\frac{1}{2}").expect("parse");
assert_eq!(ast.gold(), r#"(frac (atom Ord "1") (atom Ord "2"))"#);

let tokens = tokenize(r"\frac{1}{2}").expect("tokens");
let font = MathFont::stix_two_math().expect("STIX Two Math");
assert_eq!(font.units_per_em(), 1000);

let boxed = layout(&ast, &font, MathStyle::Text).expect("layout");
assert!(!boxed.width.is_zero());

let svg = latex_to_svg(r"\frac{1}{2}", &font, &SvgOptions::new()).expect("svg");
assert!(svg.contains("<path"));
assert!(svg.contains("<rect"));

let em = Dim::one();
let half = Dim::ratio(1, 2);
let packed = MathBox::hpack(vec![
    MathBox::rule(em.clone(), Dim::zero(), Dim::zero()),
    MathBox::rule(half, Dim::zero(), Dim::zero()),
]);
assert_eq!(packed.width, Dim::ratio(3, 2));
```

## Fonts

The crate embeds **STIX Two Math** 2.13 (SIL OFL 1.1) and loads metrics through
`ttf-parser` using integer font units converted to `Dim` via zenith-float
rationals. Glyph outlines become SVG `<path>` elements.

Math-mode commands ship as `data/symbols.tsv`. Look up with `latex_rust::lookup`.
TeX atom class is `symbol_atom_kind`. Styled letters go through `styled_char`.

Color is in v1.0 (see `documents/latex-rust-color-addition.md`, merged into the
build sheet). Resolve with `named_color` / `parse_color_spec`. Channel values
use `Dim`. `spot` and unknown names return `Err` — never a fake color. SVG
`fill` lands with the renderer.

## License

MIT OR Apache-2.0. STIX Two Math remains under the SIL Open Font License 1.1.
