# Changelog

## [2.1.1] — 2026-10-06

The dvips color names now carry their license, and `TealBlue` matches
`drivers.dtx`. No API changes.

### Fixed

- `TealBlue` is the upstream dvipsnames CMYK `0.86, 0, 0.34, 0.02`. The
  table shipped `0.86, 0.09, 0.54, 0.10`, which is not in `drivers.dtx`
  and has no other documented source. A formula that names `TealBlue`
  therefore changes color. The other 67 tuples were already the upstream
  values.

### Added

- `data/dvipsnames.tsv` identifies itself as a modified form of
  `dvipsnam.def` from the `%<*dvipsnames>` section of `drivers.dtx`
  (Copyright 1994 David Carlisle and Sebastian Rahtz; 1995-1999 David
  Carlisle; 2000-2026 The LaTeX Project; LPPL-1.3c or later). The
  modifications are the tab-separated layout and the alphabetical
  re-sort. `NOTICE` records the same terms, including where to obtain
  an unmodified copy of the graphics bundle.
- `tests/dvipsnames_upstream.rs` checks all 68 CMYK tuples against the
  values transcribed from `drivers.dtx`.

### Known issues

README and CHANGELOG had no Known issues section. GitHub issues #1
through #7 and #11 are already fixed in 2.0.0, 2.0.1, and 2.1.0.

`ttf-parser` 0.25 is flagged unmaintained (RUSTSEC-2026-0192, informational,
no patched release). It stays in 2.1.1. `MathFont::face` returns
`ttf_parser::Face`, and the crate re-exports `ttf_parser` so embedders
share that version (issue #7). Layout reads the OpenType MATH table,
glyph assemblies, and GSUB `ssty` through that face, and the SVG, PNG,
and egui backends consume its outlines. RustSec points at `skrifa`.
`read-fonts` merged MATH table support on 2026-09-08 (fontations pull
request 2105), but those types are not a stand-in for `ttf_parser::Face`,
and no maintained API-compatible fork is published. Replacing the parser
would be a breaking change to the public face API and a rewrite of the
metric and outline paths. That does not belong in a patch. The allowed
font stack in CONTRIBUTING.md is still `ttf-parser`.

## [2.1.0] — 2026-10-02

Layout follows more of TeX and the OpenType MATH table. Dimensions of fractions,
delimiters, radicals, scripts, accents, large operators, integrals, and AMSMath
grids change. `layout_with_em_size_pt` and
`layout_with_numbering_and_em_size_pt` take the physical root em so absolute
TeX lengths stay stable when the render size changes. SVG, PNG, and egui pass
their existing font size (12 pt, 12 pt, and 14 pt). `layout` still assumes 10 pt.

### Fixed

- Delimiter-less fractions are as wide as the numerator, the denominator, and
  the rule. Automatic delimiters use TeX's delimiter factor and a physical 5 pt
  `\delimitershortfall`, and sit on the math axis.
- Radical variant slack goes into the rule gap and the descent.
  `radicalDegreeBottomRaisePercent` raises the bottom of the degree.
- Display large operators sit on the math axis. Upper and lower limits each
  obey their own baseline and gap minima.
- Script style uses OpenType `ssty` alternates. `SpaceAfterScript` is taken at
  the parent style. A row adds math italic correction to a bare variable, and
  does not add it again when that variable's scripts already include it.
- Wide hats and tildes use the smallest MATH horizontal variant that covers the
  nucleus, placed with `TopAccentAttachment`.
- Matrices, cases, and aligned environments use their TeX styles and physical
  column spacing. Row rhythm uses LaTeX's 12 pt `\baselineskip`. `aligned` adds
  `\jot` the way amsmath's `\openup` does. `cases` uses `\arraystretch` 1.2 and
  a `\quad` between columns.
- A display integral with side scripts is centered on the math axis. Its italic
  correction shifts the superscript and not the subscript.
- Exact-rational decimal formatting no longer overflows while printing a
  remainder. egui tessellation rejects vertex indexes that do not fit in a
  `u32`.

### Thanks

baselogic (pull request 13) wrote the layout corrections and the regression
tests for delimiter sizing, fraction width, radical variant slack, large-operator
limits, script placement, row italic correction, AMSMath column spacing,
display-integral side scripts, and both overflow checks.

## [2.0.1] — 2026-09-27

A fix for the egui backend. No API changes; upgrade with
`cargo update -p latex-rust`.

### Fixed

- The egui backend renders every glyph. Before, formulas using `H`, math italic
  `t` (so `\partial_t u`), `π`, `σ`, `#`, `$`, `€`, `\dashv`, `\natural`, bold
  and blackboard-bold capitals and about 260 other glyphs failed with
  `Error::Unsupported { what: "glyph tessellation" }`. `B`, `k`, `τ`, `↦` and
  about 140 other glyphs were drawn with a stray triangle outside the glyph.
  The ear clipper that triangulates glyph outlines accepted ears whose closing
  diagonal ran through another vertex. Its result is now checked exactly, and
  outlines it cannot triangulate correctly are triangulated by exact scanline
  trapezoids under the non-zero rule, as the SVG and PNG backends fill them.
  Glyphs that were already drawn correctly get the same meshes as before.

## [2.0.0] — 2026-09-27

This release fixes the eight defects Tom Clark reported while integrating
latex-rust into IronLAB (issues #1–#7 and #11). Several of them change what the
renderer draws, so many golds were re-recorded, each in its own commit next to
the fix that moved it.

### Breaking changes

- `BoxContent::Glyph` has a new `scale: Dim` field, the factor the layout engine
  applied to the glyph's metrics. The variant is now `#[non_exhaustive]`, so
  match it with `BoxContent::Glyph { ch, glyph_id, .. }` and build it with the
  new `BoxContent::glyph(ch, glyph_id, scale)`. (#1)
- Parsing and layout now return `Err` for input nested deeper than
  `DEFAULT_MAX_NESTING_DEPTH` (32). Before, such input overflowed the stack and
  aborted the process. (#6)
- Output changes: variable letters are drawn in math italic, math-mode `-` is
  drawn as U+2212, script glyphs are drawn smaller, and diacritic accents sit
  lower. Code that pinned the old dimensions or images will see new values.

### Fixed

- Superscripts, subscripts and limits are drawn at script size in the SVG, PNG
  and egui renderers. Before, they were laid out at script size but drawn at
  full size. (#1)
- Unstyled Latin letters and lowercase Greek letters are set in math italic, as
  TeX does. Digits, uppercase Greek, `\mathrm`, `\text` and operator names stay
  upright. (#2)
- A math-mode `-` is drawn as U+2212 MINUS SIGN instead of the hyphen. It keeps
  the `Bin` class, and `-` inside `\text{...}` is still a hyphen. (#3)
- Font switches such as `\mathrm` reach nested subformulae (script bases,
  fractions, radicals). (#4)
- Diacritic accents (`\hat`, `\dot`, `\bar` and the rest) are placed by the TeX
  rule, lowering them by the font's accent base height. (#5)
- Deeply nested input returns `Err` instead of aborting the process. (#6)
- `STIX_TWO_MATH_OTF` is a `static`, so the font is linked into a binary once
  rather than once per use. (#11)

### Added

- `BoxContent::glyph(ch, glyph_id, scale)`. (#1)
- `Dim::as_ratio()` returns a `Dim`'s exact value as a numerator and
  denominator. (#7)
- `MathFont::face()` exposes the parsed OpenType face, and the crate re-exports
  `ttf_parser` so consumers can name its types. (#7)
- `ParseOptions`, `parse_with_options`, `layout_with_max_depth` and
  `DEFAULT_MAX_NESTING_DEPTH` let embedders choose the nesting limit. (#6)
- `tests/script_scale.rs` checks script glyph size in every backend, and new
  golds pin `a-b` to U+2212. (#1, #3)

### Thanks

Tom Clark (IronLAB) reported all eight issues with reproductions and proposed
fixes, and contributed the pull requests for #4, #7 and #11 and the first depth
limit for #6.

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
