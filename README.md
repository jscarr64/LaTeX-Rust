# LaTeX-Rust

> **Please upgrade to latex-rust 2.1.1.** Versions 1.0.0 through 1.0.4 abort
> the process with a stack overflow when given deeply nested input, which
> matters if you render LaTeX from untrusted sources. 1.0.5 fixes that for 1.x
> with no API changes (`cargo update -p latex-rust`). 2.0.0 has the same fixes
> plus breaking API changes; see the
> [2.0.0 release notes](https://github.com/jscarr64/LaTeX-Rust/releases/tag/v2.0.0).
> 1.0.5 and 2.0.1 also fix the egui backend, which failed with "glyph
> tessellation" on some glyphs (for example `\partial_t u`). 2.1.0 keeps those
> fixes and corrects TeX and OpenType MATH layout, so some formulas draw
> differently from 2.0.1. 2.1.1 keeps that layout and corrects the dvips name
> TealBlue to the CMYK tuple in `drivers.dtx`.

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE-APACHE)

Repository: <https://github.com/jscarr64/LaTeX-Rust>

Pure Rust LaTeX math renderer. No JavaScript. No webview. No runtime dependencies.

It parses LaTeX math into a typed AST, lays the AST out with a TeX-faithful box
model whose dimensions are exact rationals (`Dim`; no hardware `f32`/`f64` in the
layout path), and renders to SVG, PNG (`features = ["png"]`), or egui shapes
(`features = ["egui"]`).

## Features

- Parses LaTeX math to a typed AST
- TeX-faithful layout engine (Appendix G exact)
- SVG output — self-contained, scalable, no font embedding required
- PNG output (`features = ["png"]`) via tiny-skia
- egui integration (`features = ["egui"]`) for native desktop apps
- Full symbol coverage — Greek, AMS, arrows, operators, font styles
- Complete accent and decoration support
- Multiline environments — align, gather, multline, cases, array
- Color support — named, RGB, HTML, CMYK, gray, `\definecolor`
- 100% pure Rust — no C, no Python, no shell, no subprocesses
- MIT OR Apache-2.0

```
src/parser/     LaTeX → AST
src/layout/     AST → box model
src/font/       STIX Two Math metrics / TrueType loader
src/render/svg  box model → SVG
src/render/png  box model → PNG (`tiny-skia`, feature `png`)
src/render/egui box model → egui Shapes (feature `egui`)
golds/          gold tests (repository only)
benches/        parse / layout / render timings
```

This crate does not typeset documents. Text-mode LaTeX, TikZ, chemistry, and PDF
are out of scope and return `Err` — never a fake render.

Inventory: [capabilities](documents/CAPABILITIES.md). Patches: [CONTRIBUTING.md](CONTRIBUTING.md).

## Quick start

```toml
[dependencies]
latex-rust = "2.1"
```

```rust
use latex_rust::{parse, layout, latex_to_svg, MathFont, MathStyle, SvgOptions};

let ast = parse(r"\frac{1}{2}").expect("parse");
let font = MathFont::stix_two_math().expect("STIX Two Math");
let boxed = layout(&ast, &font, MathStyle::Text).expect("layout");
assert!(!boxed.width.is_zero());

let svg = latex_to_svg(r"\frac{1}{2}", &font, &SvgOptions::new()).expect("svg");
assert!(svg.contains("<svg"));
```

Layout math is exact rationals in this crate. It does not depend on a numeric
library.

MSRV is **1.76** (matches optional `egui` 0.28).

## Usage

### SVG

```rust
use latex_rust::{latex_to_svg, MathFont, SvgOptions};

let font = MathFont::stix_two_math().unwrap();
let svg = latex_to_svg(r"e^{i\pi}+1=0", &font, &SvgOptions::new()).unwrap();
```

### PNG

Enable with `features = ["png"]`. `tiny-skia` rasterizes the same `MathBox` tree
as the SVG backend. SIMD is selected at runtime:

- On x86_64 with AVX2 — tiny-skia runs at peak performance automatically
- On Apple Silicon — tiny-skia uses the ARM NEON path, also fast
- No user configuration needed
- No compile flags required
- Works correctly on every supported architecture

Without the feature, `latex_to_png` / `render_png` return `Err(Unsupported)`.

```rust
# #[cfg(feature = "png")]
# {
use latex_rust::{latex_to_png, MathFont, PngOptions};

let font = MathFont::stix_two_math().unwrap();
let png = latex_to_png(r"\frac{1}{2}", &font, &PngOptions::new()).unwrap();
assert!(png.starts_with(b"\x89PNG"));
# }
```

### egui

Enable with `features = ["egui"]`. Glyphs become `egui::Mesh`; rules and color
boxes become `Shape::Rect`. No SVG intermediate. Callers should keep the
returned `Vec<Shape>` if the same expression is painted every frame (cache hit).

```rust
# #[cfg(feature = "egui")]
# {
use latex_rust::{latex_to_shapes, EguiOptions, MathFont};

let font = MathFont::stix_two_math().unwrap();
let (shapes, rect) =
    latex_to_shapes(r"x^2", &font, &EguiOptions::new(), egui::Pos2::ZERO, 1.0).unwrap();
assert!(!shapes.is_empty());
let _ = rect;
# }
```

## Supported LaTeX

Math mode only. Command and environment coverage is [`data/symbols.tsv`](data/symbols.tsv).
Look up a command with `latex_rust::lookup`. Missing glyphs and unknown commands
are `Err`.

Color: `named_color` / `parse_color_spec`. Channel values use `Dim`. `spot` and
unknown names return `Err`. SVG, PNG, and egui all take `Color::to_rgba8()`.

## Performance

Times below are median-of-batch averages from `cargo bench --features png,egui`
on this machine (release, in-crate `Dim` layout). Re-run the bench on your
hardware.

| Operation | Target | Measured |
|---|---|---|
| Parse `\frac{1}{2}` | < 50µs | 1.8µs |
| Parse full display equation | < 200µs | 3.3µs |
| Layout `\frac{1}{2}` | < 100µs | 13µs |
| Layout full display equation | < 500µs | 47µs |
| SVG `\frac{1}{2}` | < 500µs | 232µs |
| SVG full display equation | < 1ms | 951µs |
| PNG 144 DPI | < 5ms | 142µs |
| PNG 300 DPI | < 15ms | 350µs |
| egui inline (cache miss) | < 2ms | 216µs |
| egui inline (cache hit) | < 0.1ms | 29µs |

KaTeX cold start in a browser is typically **200–400ms** (JS parse + layout +
font). This crate’s first `MathFont::stix_two_math()` plus an SVG of a short
expression is on the order of the layout+SVG rows above after the face is loaded
(embedded STIX Two Math, no network). There is no JS runtime to warm up.

## Fonts

The crate embeds **STIX Two Math** 2.13 (SIL OFL 1.1) and loads metrics through
`ttf-parser` using integer font units converted to `Dim`. Glyph outlines become
SVG `<path>` elements (and PNG/egui meshes when those features are on).

## Notes

2.1.1 is the current release. baselogic contributed the TeX and OpenType MATH
layout work in pull request 13. His commits are in the history under his name. They cover delimiter sizing with
a physical `\delimitershortfall`, fraction width without null-delimiter padding,
radical variant slack, large-operator limits, `ssty` script alternates,
`SpaceAfterScript`, row math italic correction, AMSMath column spacing,
display-integral side scripts, and the exact-rational and egui index overflow
checks.

Three rules were corrected on top of that work. A wide hat or tilde uses the
smallest MATH variant that covers the nucleus. AMSMath rows use LaTeX's 12 pt
`\baselineskip`, plus `\jot` in `aligned`. A radical degree is placed by the
bottom of its ink.

## Credits

`data/dvipsnames.tsv` is a modified extract of `dvipsnam.def` from `drivers.dtx`
in the LaTeX graphics bundle, Copyright 1994 David Carlisle and Sebastian Rahtz;
1995-1999 David Carlisle; 2000-2026 The LaTeX Project. The table is licensed
under the LaTeX Project Public License, version 1.3c or later. Rows in this
crate are sorted alphabetically. See `NOTICE`.

STIX Two Math 2.13 is embedded under the SIL Open Font License 1.1. See
`fonts/stix-two-math/OFL.txt` and `NOTICE`.

Tom Clark reported the 2.0 defects and contributed several of the fixes.
baselogic wrote the 2.1.0 TeX and OpenType MATH layout corrections. The
changelog records both.

## Acknowledgments

This crate was developed with help from AI assistants by SpaceXAI, Anthropic, OpenAI, Google, and Meta.

## Known issues

GitHub issues for this crate are closed. `ttf-parser` 0.25.1, which reads the
embedded STIX Two Math face, is marked unmaintained by RUSTSEC-2026-0192
(informational; no patched release). The maintained reader named there is
`skrifa`. `MathFont::face` returns `ttf_parser::Face`, and the crate re-exports
`ttf_parser`. Moving to `skrifa` changes that public type and the outline,
MATH, and GSUB code that uses it, so it waits for a breaking release.

## License

MIT OR Apache-2.0. STIX Two Math remains under the SIL Open Font License 1.1.
The dvips named-color table in `data/dvipsnames.tsv` remains under the LaTeX
Project Public License, version 1.3c or later. See `NOTICE`.
