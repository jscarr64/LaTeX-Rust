# Changelog

## [3.0.0] — unreleased

Issues found in one program led to a forensic audit of our other crates.
This release lists every finding and its fix. Nothing in this section is published.

### Fixed

- **Infix `\over` and `\choose` matched a prefix.** `convert_over_fractions`
  searched for the bytes `\over`, and `convert_choose` searched for `\choose`,
  with no control-word boundary. A `{` in front of `\overline`, `\overset`,
  `\overrightarrow`, `\overleftarrow`, `\overleftrightarrow`, or `\overbrace`
  became a fraction whose denominator was the rest of the command name.
  `\frac{\overline{x}}{2}` parsed as a fraction of an empty numerator over the
  letters `l i n e` and `x`. The same shape of bug was waiting in `\choose`.
  Both rewrites now accept a command only when the next character is not a
  letter. `\over` still runs before `\choose`, so a group that contains both
  keeps the previous order.

  Migration: `{a \over b}` is still `\frac{a}{b}`, and `{n \choose k}` is still
  `\binom{n}{k}`. Bare `\overline{x}` is still an accent. A string that used to
  parse as that broken fraction now parses as the accent or the `\overset`
  node. Stored `gold()` values for an `\over…` command that sat inside braces
  change. Callers that matched `Fraction` there should match `Accent` or
  `OverUnder`.

- **Repeated `{a \over b}` was quadratic.** Each replacement started the search
  again at the beginning of the string. 100,000 copies took 14.2 s in a
  release build. Each command is now rewritten in one left-to-right pass.
  The trees of real infix fractions are unchanged. No migration.

- **A script after `(...)` bound to one parenthesis.** `(-b)^{n}`, `(a/b)^{n}`,
  `(\frac{a}{b})^{n}`, `(2x)^{2}`, and `(f(x))'` put `^`, `_`, or `'` on the
  closing parenthesis. `x^(-b)` put the opening parenthesis in the exponent.
  A parenthesized row is now one nucleus, and the parentheses stay atoms in
  that row.

  Migration: a superscript, subscript, or prime after `(...)` applies to the
  parenthesized row. The old tree applied it to the nearest parenthesis.
  `{(-b)}^{n}` and `\left(-b\right)^{n}` stay as they were. `-b^{n}` and
  `a/b^{n}` still put the power on `b`.

- **`f'^2` flattened the prime and the exponent.** The superscript was a row
  of prime-then-2. It is now the square of primed `f`, the same tree as
  `{f'}^2`. `f'`, `f''`, and `f'''` are unchanged. `f^{\prime}` stays a
  `prime` symbol.

  Migration: `f'^2` is the square of primed `f`. `f'` and `f^{\prime}` stay
  different nodes.

- **A raised depth limit aborted the process, and acceptance depended on the build.**
  `with_max_depth(20_000)` parsed 3,500 brace groups and aborted at 3,531.
  The same limit aborted on deep `\left` and `\frac`. A stack budget on top
  of that returned `Malformed` for input that was still under the caller's
  limit, and a release build's smaller frames made 3,531 braces `Ok` while
  a debug build returned `Err`. Nesting now lives on a heap stack.
  `max_depth` is the only limit: the same input gets the same result in
  debug and release, on an 8 MiB stack and on a 256 KiB stack.

  Migration: the default depth stays 32. 3,500 and 3,531 brace groups at
  limit 20,000 return `Ok`, and so does the same nest of `\left` or a
  3,531-deep `\frac`. A brace group or a `\left` body is one level, so
  20,000 of them at that limit return `Err(Malformed)`. A braced argument
  such as `\frac` or a superscript costs two levels, so 10,000 of those
  return `Err(Malformed)`. Callers on the default limit of 32 see no change.
  `gold`, `to_latex`, layout, and SVG/PNG/egui render walk a heap stack, and
  so do `MathNode` / `MathBox` clone and drop. A tree the parser accepts can
  take those paths on a 256 KiB thread.

- **Empty input parsed as an empty row.** `parse("")` and whitespace returned
  `(row)`. `\frac{}{}` was a fraction of two empty rows. A bare prime was a
  superscript on an empty row. `^2` and `^{2}` were a superscript on that
  empty row, and `_i` was a subscript on it. `'^2` is a prime with no base.
  A bare `^` was already `Malformed`. `x^2` and `x_i` are unchanged.

  Migration: callers that treated the empty row as “no formula” should match
  on `Err`. `\frac{}{}`, a bare prime, `^2`, `^{2}`, `_i`, and `'^2` return
  `Err`. A script still needs a real base: `x^2` is the same tree.

- **`√`, a superscript run, and U+2212 were ordinary atoms.** `√x` and
  `√{x}` are a square root, the same tree as `\sqrt{x}`. A run of
  superscript digits, superscript letters, and a superscript minus
  (`²`, `³`, `ⁿ`, `⁻¹`, `²³ⁿ⁻¹`) is a superscript, the same tree as
  `x^{…}`. A superscript with no base is `Malformed`, the same as `^2`.
  U+2212 MINUS is a binary minus, the same atom as `-`.

  Migration: stored `gold()` values for those characters change. `√x^2`
  is the square of the radical, and `√{x^2}` is the radical of the square.
  `f'²` is the square of primed `f`, the same tree as `f'^2`. ASCII `-`
  is unchanged.

- **ASCII `<=`, `>=`, and `!=` were two atoms.** They stay two atoms
  unless requested. `ParseOptions::with_ascii_relations` maps adjacent
  `<=`, `>=`, and `!=` to the same symbols as `\le`, `\ge`, and `\ne`.
  A space still splits them, so `x < = 0` is two relation atoms.

  Migration: the flag defaults to off, so existing `gold()` values stay
  byte-identical. Accumath turns it on at `parse_with_options`.

- **Bare words were products of letters.** `sin(x)`, `zeta(s,a)`, and
  `otherwise` inside `cases` stay letters unless requested.
  `with_bare_operators` reads a maximal run of ASCII letters as one
  identifier. The whole run has to be in the operator table, or be
  `\zeta`, so `sinh` is `\sinh`, `xsin` and `sinx` stay letters, and
  `infty` is not `\inf`. A space ends the run, so `x sin` is `x` times
  `\sin`. `ω` is not an ASCII letter, so `sinω` is `\sin` followed by
  `ω`. Inside `cases`, the identifier `otherwise` is the text node of
  `\text{otherwise}`. Outside `cases` that word stays letters.

  Migration: the flag defaults to off. Existing `gold()` values stay
  byte-identical. Accumath turns it on at `parse_with_options`.

- **The same glyph could be a different node.** `α`, `Γ`, `π`, `≤`,
  `≥`, `≠`, `∞`, `×`, `·` (U+00B7), and `⋅` (U+22C5 DOT OPERATOR) stay
  ordinary atoms unless requested. `⋅` was Ord, not Bin and not
  `(symbol cdot)`. `with_unicode_symbols` maps those characters to the
  symbol nodes of `\alpha`, `\Gamma`, `\pi`, `\leq`, `\geq`, `\neq`,
  `\infty`, `\times`, and `\cdot`. `f'` stays a superscript of U+2032.
  `with_prime_symbol` spells `'` as `(symbol prime)`. `f''` is a row of
  two primes. `f'''` keeps its nested rows; it is not rewritten into a
  flat row of three primes.

  Migration: both flags default to off, so `f'` and `f^{\prime}` stay
  different nodes unless `prime_symbol` is on. `ω` stays an ordinary
  atom. Accumath turns the flags on at `parse_with_options`.

- **A nested color reprinted without its group.**
  `{\color{red} x + {\color{blue} y} + z}` already parsed as red over
  the row and blue only on `y`. `MathNode::to_latex` prints each
  `Color` in its own group, so the inner color does not swallow `+ z`
  on the next parse.

  Migration: the parse tree is unchanged. Callers that printed the tree
  themselves need the same grouping. `to_latex` is the reprint that
  round-trips.

- **`MathNode` and `ParseError` are `non_exhaustive`.** A later release
  can add a variant. `ParseOptions` already was.

  Migration: a downstream `match` that lists every variant of `MathNode`
  or `ParseError` needs a wildcard. `if let`, `matches!`, and
  constructing a variant are unchanged. `ParseOptions::new` still builds
  the defaults, including the four flags above set to off.

### Added

- `MathNode::to_latex` reprints a tree as LaTeX. It is total. Every
  script, accent, fraction, and `\overset` / `\underset` argument is
  braced, an alphabetic control word is followed by a space, and
  `\overline` is the full command. A `Fraction` prints as the command in
  its `FracStyle`. `\over` is stored as `FracStyle::Frac`, so it prints
  `\frac`. Braces and spaces may differ from the source; compare trees,
  then compare a reprint to a reprint.

  Migration: new method. `\frac` gold strings are unchanged. A styled
  fraction's gold spelling is in the `FracStyle` note below.

- `tests/roundtrip.rs` checks `parse(to_latex(t)) == t` and an idempotent
  reprint on the `parse.toml`, `accumath.toml`, and `lexicon.toml` golds,
  including trees built with the flags on. A grammar generator covers
  2,000 trees in debug and 50,000 in release. `LATEX_RUST_ROUNDTRIP_CASES`
  overrides the count. `proptest` is a dev-dependency.
- `fuzz/fuzz_targets/parse.rs` and `preprocess.rs`. The parse target
  keeps `max_depth` at the default, because an abort kills the fuzzer.
- `.github/workflows/ci.yml` runs fmt, clippy (`--all-targets
  --all-features -D warnings`), `cargo test`, `cargo test --release`,
  and a 60-second parse fuzz. A separate job resolves the lockfile with
  rustc 1.99 and runs `cargo audit --deny warnings` with cargo-audit
  0.22.2 on that same toolchain. `.github/workflows/fuzz-nightly.yml` runs the release
  tests, then parse for 16 minutes and preprocess for 2.

### Fixed

- **`\dfrac`, `\tfrac`, and `\cfrac` kept their style.** The tree is
  `Fraction(FracStyle, numerator, denominator)`. `gold()` spells them
  `(dfrac …)`, `(tfrac …)`, and `(cfrac …)`. `\frac` and `{a \over b}`
  stay `(frac …)`, byte-identical to the previous gold. `to_latex`
  prints `\dfrac `, `\tfrac `, `\cfrac `, or `\frac `. Layout forces
  display style for `\dfrac` and `\cfrac`, and text style for `\tfrac`.
  `\dbinom` and `\tbinom` stay `\binom`.

  Migration: a match on `Fraction(num, den)` becomes
  `Fraction(style, num, den)`. `\over` still prints `\frac`. Stored
  `gold()` for `\frac` is unchanged. Stored `gold()` for a styled
  fraction changes from `(frac …)` to the styled spelling.

- **A stray `\\` names itself.** Outside a row environment,
  `ParseError::RowBreakOutside` displays
  `row break \\ outside an environment`. `\cr` uses the same error.
  Inside `matrix`, `aligned`, and `\substack`, `\\` and `\cr` are still
  row breaks.

  Migration: callers that matched `ParseError::Unknown` for `\\` should
  match `RowBreakOutside`. The message is no longer
  `unknown command: \\`.

- **RUSTSEC-2026-0192, direct dependency.** `ttf-parser` has no patched
  release. This crate no longer depends on it. Outlines and `hmtx` /
  `cmap` / `GSUB` come from `skrifa` 0.39. The MATH table is read here,
  because `read-fonts` 0.36 does not expose MATH. `skrifa` stays at
  0.39. `MathFont::face` and the `ttf_parser` re-export are gone.
  `MathParams` carries the
  script-placement and stack constants layout already used. PNG and
  egui golds are unchanged.

  Migration: delete `ttf_parser` from downstream `use` lists. Replace
  `font.face()` with `glyph`, `glyph_id`, `glyph_count`,
  `italic_correction`, `top_accent_attachment`, `horizontal_variants`,
  `vertical_variants`, and `horizontal_assembly_parts`. A match that
  builds `MathParams` by field name needs the new fields.

- **A tree the parser accepts does not overflow the caller stack.**
  `gold`, `to_latex`, layout, SVG, PNG, and egui emission, and clone
  and drop of `MathNode` and `MathBox`, use an explicit heap stack.

  Migration: `layout_with_max_depth` is a nesting limit, not a stack
  budget. 3,531 brace groups and 9,999 `\frac` at limit 20,000 can be
  printed, laid out, and rendered to SVG on a 256 KiB thread, in debug
  and release.

- **A trailing control space was a stray backslash.** `preprocess` used
  `str::trim`. That deleted the space in `\ ` and in the reprint of
  `\ %comment`, and the tokenizer then reported `trailing backslash`.
  The space after an odd run of backslashes is the control space, so
  trimming keeps it. `to_latex` still prints `\ `, and that reprint
  parses again. An even run (`\\ `) is a control symbol plus an ordinary
  space, and that space is still trimmed. The fuzz seed is
  `fuzz/corpus/parse/control-space-comment` (`\ %@QJXi`).

  Migration: `\ ` at the end of input, and a control space before a
  comment, parse as `(space control)`. The end-of-input form used to be
  `trailing backslash`. The comment form parsed once and failed when the
  reprint was parsed again.

- **rustc 1.99 clippy.** The radical and delimiter variant tests use
  `is_none_or` instead of `map_or(true, …)`. SHA-256 walks blocks with
  `as_chunks` instead of `chunks_exact`. No parse or layout change.

### Changed

- **MSRV is 1.99.** `rust-version` is `"1.99"`. `rust-toolchain.toml`
  pins channel `1.99` with `clippy` and `rustfmt`. The CI test job, the
  lockfile step, the audit job, and the release-test step of the nightly
  fuzz workflow use rustc 1.99. The fuzz compile stays on nightly.

  Migration: a toolchain older than 1.99 cannot build this crate. This
  is a breaking change.

- **Optional `egui` is 0.36.2, with `epaint` 0.36.2.** The feature stays
  off by default, so a default build does not link either crate.
  `epaint` 0.34 switched its font backend from `ab_glyph` / `ttf-parser`
  to `skrifa`. 0.36.2 keeps that, so RUSTSEC-2026-0192 (`ttf-parser`
  0.25.1, unmaintained, no patched release) is gone from the
  `--all-features` tree. `cargo audit --deny warnings` has no ignores.
  MSRV 1.99 is above `egui` 0.36's rustc 1.95.

  Glyph meshes are still tessellated in this crate from `skrifa`
  outlines. `epaint` 0.35 shapes its own text with `harfrust` and types
  text cursors as `CharIndex` and `ByteIndex`. This renderer does not
  call that shaper, so those types are not on the emission path.
  `clip_rect_margin` and the renamed `Panel` methods are not used.
  Call sites pass `f32`, `Color32`, `Stroke`, and `CornerRadius`
  directly, which is what remains after `impl Into<f32>` arguments were
  removed. `Rounding` is `CornerRadius`. A frame stroke uses
  `StrokeKind::Middle` because the rectangle is already inset by half
  the stroke width.

  Migration: enable `features = ["egui"]` and depend on `egui` 0.36.2.
  `MathFont::face` is already gone. Downstream code that named `egui`
  0.28 types (`Rounding`, `ab_glyph`) has to move to 0.36.2.

### Known issues

3.0.0 is not published.

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
