# Changelog

## [1.0.5] — 2026-09-27

Security fix, plus the 2.0.0 rendering fixes that fit the 1.x API. Deeply
nested input no longer aborts the process (#6), and superscripts, math
italic, the minus sign, `\mathbf` on fractions and accent heights now render
as in 2.0.0 (#1–#5). The egui backend no longer fails on glyphs it could not
triangulate (the same fix as 2.0.1). No public item is added, removed or
changed, so this is a drop-in replacement for 1.0.4. Upgrade with
`cargo update -p latex-rust`.

Rendered output changes: most formulas now draw differently from 1.0.4 (82 of
87 inputs in the maintainers' test corpus). On that corpus 1.0.5 draws the
same SVG, PNG and egui output as 2.0.1, with the same box dimensions. Code
that pinned 1.0.4 images, dimensions or glyph ids will see new values.

### Security

- Versions 1.0.0 through 1.0.4 parsed and laid out nested input recursively
  with no depth limit. Deeply nested input overflowed the stack, and a stack
  overflow in Rust aborts the process; `catch_unwind` cannot catch it. In an
  optimised build on an 8 MiB stack, 700 nested `\frac{1}{…}` (about 7 KB of
  input) or 10,000 nested braces were enough, and much less on smaller thread
  stacks. Any program that renders LaTeX from untrusted input could be
  terminated this way. (#6)
- `parse`, `parse_with_colors`, and the `latex_to_*` functions now return
  `ParseError::Malformed("input nests deeper than 32 levels")` for input that
  nests more than 32 levels. `layout` returns
  `Error::Unsupported { what: "tree nests deeper than 32 levels" }` for a tree
  built by hand that nests that deeply. Both are existing error variants, so
  the public API is unchanged.
- The count is of parser recursion levels, and a braced argument costs two, so
  the limit admits 15 nested `\frac`, `\sqrt`, or `x^{…}`, and 31 nested
  groups, `\left…\right` pairs, or environments. This is the same limit, with
  the same messages, as 2.0.0. The embedder-adjustable limit
  (`ParseOptions`, `layout_with_max_depth`) is new API and stays in 2.0.0.

### Fixed

- Superscripts, subscripts, limits and fraction parts in text style are drawn
  at script size in the SVG, PNG and egui renderers. Before, they were laid
  out at script size but drawn at full size. 2.0.0 added a `scale` field to
  `BoxContent::Glyph` for this; 1.0.5 does not change that type. The renderers
  instead work out the scale from the glyph box's dimensions, which the layout
  engine always sets to the font metrics times the style scale. A hand-built
  glyph box whose dimensions match no style is drawn at text size, as before.
  Box dimensions are unchanged. (#1)
- Unstyled Latin letters and lowercase Greek letters are set in math italic
  (for example `x` is drawn as U+1D465, and `h` as U+210E), as TeX does. Digits, uppercase
  Greek, `\mathrm`, `\text` and operator names stay upright. (#2)
- A math-mode `-` is drawn as U+2212 MINUS SIGN instead of the hyphen. It keeps
  the `Bin` class, and `-` inside `\text{...}` is still a hyphen. (#3)
- Font switches such as `\mathbf` and `\mathrm` reach nested subformulae
  (fractions, script bases, radicals), so `\mathbf{\frac{a}{b}}` is bold. (#4)
- Diacritic accents (`\hat`, `\dot`, `\bar` and the rest) are placed by the TeX
  rule, lowering them by the font's accent base height. (#5)
- The egui backend renders every glyph. Before, formulas using `H`, math italic
  `t`, `π`, `σ`, `#`, `$` and a few hundred other glyphs failed with
  `Error::Unsupported { what: "glyph tessellation" }`, and `B`, `k`, `τ`, `↦`
  and about 140 others were drawn with a stray triangle outside the glyph.
  Glyphs that were already drawn correctly get the same meshes as before.

### Not in 1.0.5

These 2.0.0 changes need new or changed public API, so they stay in 2.0.0:
`Dim::as_ratio()` and `MathFont::face()` with the `ttf_parser` re-export (#7),
`STIX_TWO_MATH_OTF` as a `static` (#11), and `ParseOptions`,
`parse_with_options`, `layout_with_max_depth` and `DEFAULT_MAX_NESTING_DEPTH`.

### Thanks

Tom Clark (IronLAB) reported all of these issues with reproductions and
proposed fixes, and wrote the fix for #4 (#8) and the depth limit (#10). Both
are backported here under his name.

## [1.0.4] — 2026-09-20

Clippy debt clear (`-D warnings`): `RowKind::Intertext` boxed to shrink enum size.

## [1.0.3] — 2026-09-19

Coordinated patch with zenith-float, hdf5-rust, and redb-view (pure-Rust FOSS family adjacent to Accumath; Accumath itself stays proprietary).

## [1.0.2] — 2026-09-04

Standalone layout math. This crate does not depend on a numeric library.

- `Dim` is an exact rational (`num/den`). No hardware `f32`/`f64` in layout.
- SHA-256 of the embedded face is in-tree.
- Removed the numeric-library dependency from `Cargo.toml`.

## [1.0.0] — 2026-09-02

### Added

- Complete LaTeX math parser (`parse`, gold-stable `MathNode`)
- TeX-faithful layout engine (Appendix G style, Table 18 spacing, `Dim`)
- SVG renderer (`render_svg` / `latex_to_svg`) — self-contained SVG 1.1, no font embedding
- PNG renderer (feature `png`) via `tiny-skia` 0.11, DPI-aware, transparent background by default
- egui renderer (feature `egui`) — TrueType tessellation to `egui::Shape` meshes, no SVG intermediate
- Math-mode symbol catalog (Greek, AMS, arrows, operators, font styles) locked to `data/symbols.tsv`
- Full accent and decoration support (TeX placement, extensible hats/arrows/braces, cancel, boxed)
- Multiline environments — `align`, `aligned`, `split`, `gather`, `multline`, `equation`, `{array}`, `{cases}`
- Color support — named, rgb, RGB, HTML, cmyk, gray, `\definecolor`, group scope, `\fcolorbox` borders
- Exact-rational layout math (no hardware `f32`/`f64` in layout)

### Milestone notes

Milestones 1–10 landed as 0.1.0 development commits. This release packages that
surface as 1.0.0: rustdoc, README benchmarks, clippy/fmt, and crates.io metadata.
The crate is publish-ready; crates.io upload is a separate step.

### [1.0.1] - 2026-09-02

### Version to 1.0.1
### Fixed

Resolved an issue where the docs.rs documentation build was failing due to deprecated Rust nightly features (doc_auto_cfg).
Removed:  two temporary build documents Prompt and color addition
Corrected email address of author from jscarr@gmail.com to jscarr1964@gmail.com

### Architecture

Crate modules: `parser/`, `layout/`, `font/`, `render/svg`, `render/png`,
`render/egui`, plus `golds/` and `benches/` in the repository.
