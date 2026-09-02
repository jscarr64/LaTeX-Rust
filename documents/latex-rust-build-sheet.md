#latex-rust — Build Sheet
**Version:** 0.1.0-plan  
**Date:** 2026-09-01  
**License:** MIT OR Apache-2.0  
**Status:** Pre-implementation build plan  

> Pure Rust LaTeX math renderer. No JavaScript. No webview. No runtime dependencies.  
> Surpasses MathJax and KaTeX in correctness, performance, and platform coverage.

---

## 1. Mission Statement

MathJax and KaTeX are JavaScript libraries. Every non-web application that needs LaTeX rendering must either bundle a JS runtime, shell out to a subprocess, or accept degraded output. latex-rust eliminates that dependency entirely.

latex-rust is a pure Rust crate that:
- Parses LaTeX math expressions into a typed AST
- Lays out that AST using a complete TeX-faithful box model
- Renders to SVG, PNG, or directly to egui primitives
- Runs identically on Windows, macOS, Linux, Android, and iOS
- Requires zero runtime dependencies beyond the Rust standard library

---

## 2. Where latex-rust Beats MathJax and KaTeX

| Capability | MathJax | KaTeX | latex-rust |
|---|---|---|---|
| Language | JavaScript | JavaScript | Pure Rust |
| Runtime dependency | Node / browser | Node / browser | None |
| Air-gap compatible | No | No | Yes |
| Arbitrary precision layout metrics | No | No | Yes (zenith-float) |
| Native desktop rendering | No | No | Yes |
| egui integration | No | No | Yes (native) |
| Android / iOS native | No | No | Yes |
| Startup time | Slow (JS parse) | Faster but JS | Instant |
| Memory footprint | Heavy | Moderate | Minimal |
| Honest error on unsupported input | Partial | Partial | Always |
| Hardware float in layout math | Yes | Yes | No (zenith-float) |
| Font metric precision | Limited | Limited | Full TrueType |

---

## 3. Architecture

```
latex-rust
├── parser/          LaTeX → AST
├── layout/          AST → Box model (TeX-faithful)
├── font/            Font metric tables, TrueType loader
├── render/
│   ├── svg/         Box model → SVG
│   ├── png/         Box model → PNG (via tiny-skia)
│   └── egui/        Box model → egui primitives (feature-gated)
├── golds/           Gold test vectors
└── benches/         Layout and render benchmarks
```

### 3.1 Parser

Input: LaTeX math string  
Output: `MathNode` AST  
Errors: `ParseError` — never a partial parse silently accepted

**Supported input forms:**
- Inline math: `$...$`
- Display math: `$$...$$` or `\[...\]`
- Raw expression (no delimiters, for programmatic use)

### 3.2 AST Nodes

Every node is a variant of `MathNode`. No implicit nodes. No surprises.

```
MathNode
├── Atom(char, AtomKind)           — single character/symbol
├── Fraction(num, den)             — \frac{}{} 
├── Radical(degree, radicand)      — \sqrt[n]{}
├── Superscript(base, exp)         — x^{}
├── Subscript(base, sub)           — x_{}
├── SubSup(base, sub, exp)         — x_{}^{}
├── Delimited(open, body, close)   — \left( \right)
├── Row(Vec<MathNode>)             — horizontal sequence
├── Matrix(rows, cols, Vec<MathNode>, style) — pmatrix, bmatrix, etc.
├── Sum(limits)                    — \sum
├── Integral(limits)               — \int
├── Product(limits)                — \prod
├── Limit(sub)                     — \lim
├── OverUnder(base, over, under)   — \overset \underset
├── Accent(base, accent)           — \hat \bar \vec \dot \ddot
├── Color(spec, body)              — \color after this point in a group
├── TextColor(spec, body)          — \textcolor{}{}
├── ColorBox(fill, body)           — \colorbox{}{}
├── FColorBox(border, fill, body)  — \fcolorbox{}{}{}
├── Text(string, style)            — \text{} \mathrm{} \mathbf{}
├── Space(SpaceKind)               — \, \; \quad \qquad
├── Operator(name, limits)         — \sin \cos \log named ops
├── Symbol(SymbolKind)             — Greek letters, arrows, relations
├── Strut(height, depth)           — vertical spacing
└── Phantom(MathNode)             — \phantom{}
```

### 3.3 Layout Engine (Box Model)

TeX-faithful box model. Every node produces a `Box`:

```rust
pub struct MathBox {
    pub width:   Dim,   // zenith-float arbitrary precision
    pub height:  Dim,   // above baseline
    pub depth:   Dim,   // below baseline
    pub italic:  Dim,   // italic correction
    pub content: BoxContent,
}
```

Layout rules follow Appendix G of The TeXbook exactly:
- Style propagation (display, text, script, scriptscript)
- Cramped style rules
- Mu (math unit) spacing table
- Delimiter sizing algorithm
- Radical vinculum height
- Fraction bar thickness
- Accent placement
- Operator limits placement

No approximations. No shortcuts. If TeX would render it one way latex-rust renders it that way.

### 3.4 Font System

TeX math requires specific font metrics. latex-rust ships with:
- **STIX Two Math** — full Unicode math coverage, OFL license, embeddable
- **Latin Modern Math** — TeX-native, OFL license, embeddable
- Font metric tables compiled to Rust at build time
- TrueType glyph outlines extracted at render time

Font metrics are stored as zenith-float `Dim` values — no hardware float in the layout path.

### 3.5 Renderers

**SVG renderer:**
- Self-contained SVG output
- Glyphs as `<path>` elements — no font embedding required
- Scales perfectly at any size
- Primary output format

**PNG renderer (feature = "png"):**
- Uses `tiny-skia` for rasterization
- DPI-aware
- Optional feature — not compiled by default

**egui renderer (feature = "egui"):**
- Emits egui `Shape` primitives directly
- Zero SVG intermediate
- Designed for Accumath UI integration

---

## 4. Capability Inventory

### 4.1 Core Math Structures

| Capability | Status | Notes |
|---|---|---|
| Fractions `\frac` | ⬜ | Display and text style |
| Binomial `\binom` | ⬜ | Via \genfrac |
| Radicals `\sqrt` `\sqrt[n]` | ⬜ | Vinculum extends to content width |
| Superscript / subscript | ⬜ | Stacking, cramped style |
| Sub+superscript simultaneous | ⬜ | x_a^b |
| Limits on operators | ⬜ | Display vs inline position |
| Delimiters `\left \right` | ⬜ | All sizes, extensible |
| Fixed-size delimiters | ⬜ | `\big \Big \bigg \Bigg` |
| Matrices | ⬜ | pmatrix bmatrix vmatrix Vmatrix |
| Cases `\begin{cases}` | ⬜ | |
| Arrays `\begin{array}` | ⬜ | Column alignment l c r |
| Aligned equations | ⬜ | `\begin{aligned}` |

### 4.2 Symbols

| Capability | Status | Notes |
|---|---|---|
| Greek lowercase α β γ … | ⬜ | Full set |
| Greek uppercase Α Β Γ … | ⬜ | Full set |
| Hebrew ℵ ℶ ℷ ℸ | ⬜ | |
| Binary operators + − × ÷ ± ∓ … | ⬜ | Full TeX set |
| Relations = < > ≤ ≥ ≠ ≈ ∼ … | ⬜ | Full TeX set |
| Arrows → ← ↔ ⇒ ⇔ … | ⬜ | Full TeX set |
| Large operators ∑ ∏ ∫ ∮ ∯ … | ⬜ | Display/text sizing |
| Dots ⋯ ⋮ ⋱ … | ⬜ | \cdots \vdots \ddots |
| Set symbols ∈ ∉ ⊂ ⊃ ∪ ∩ … | ⬜ | |
| Logic symbols ∀ ∃ ∧ ∨ ¬ … | ⬜ | |
| Blackboard bold ℝ ℂ ℤ ℚ ℕ … | ⬜ | \mathbb |
| Calligraphic 𝒜 ℬ 𝒞 … | ⬜ | \mathcal |
| Fraktur 𝔄 𝔅 ℭ … | ⬜ | \mathfrak |
| AMS symbols | ⬜ | Full amssymb set |

### 4.3 Accents and Decorations

| Capability | Status | Notes |
|---|---|---|
| `\hat \check \breve \acute \grave` | ⬜ | Single char |
| `\tilde \bar \vec \dot \ddot` | ⬜ | Single char |
| `\widehat \widetilde` | ⬜ | Extensible |
| `\overline \underline` | ⬜ | Extensible |
| `\overbrace \underbrace` | ⬜ | With annotation |
| `\overleftarrow \overrightarrow` | ⬜ | Extensible |
| `\overset \underset` | ⬜ | |
| `\stackrel` | ⬜ | |
| `\cancel \bcancel \xcancel` | ⬜ | |
| `\boxed` | ⬜ | |

### 4.4 Text and Font Styles

| Capability | Status | Notes |
|---|---|---|
| `\mathrm \mathbf \mathit` | ⬜ | |
| `\mathsf \mathtt \mathbb` | ⬜ | |
| `\mathcal \mathfrak \mathscr` | ⬜ | |
| `\text{}` | ⬜ | Text mode inside math |
| `\boldsymbol \pmb` | ⬜ | Bold math |
| `\operatorname{}` | ⬜ | Named operators |

### 4.5 Spacing

| Capability | Status | Notes |
|---|---|---|
| Automatic inter-atom spacing | ⬜ | TeX Appendix G table |
| `\,` thin space | ⬜ | |
| `\:` medium space | ⬜ | |
| `\;` thick space | ⬜ | |
| `\!` negative thin space | ⬜ | |
| `\quad \qquad` | ⬜ | |
| `\hspace{}` | ⬜ | |
| `\phantom \vphantom \hphantom` | ⬜ | |

### 4.6 Named Operators

| Capability | Status | Notes |
|---|---|---|
| `\sin \cos \tan \cot \sec \csc` | ⬜ | |
| `\arcsin \arccos \arctan` | ⬜ | |
| `\sinh \cosh \tanh \coth` | ⬜ | |
| `\log \ln \lg \exp` | ⬜ | |
| `\lim \limsup \liminf` | ⬜ | Limits above/below in display |
| `\sup \inf \max \min` | ⬜ | |
| `\det \dim \ker \deg` | ⬜ | |
| `\gcd \lcm` | ⬜ | |
| `\Pr \arg` | ⬜ | |

### 4.7 Multiline and Display

| Capability | Status | Notes |
|---|---|---|
| Display math centering | ⬜ | |
| Equation numbering | ⬜ | `\tag{}` |
| `\begin{equation}` | ⬜ | |
| `\begin{align}` | ⬜ | |
| `\begin{gather}` | ⬜ | |
| `\begin{multline}` | ⬜ | |
| `\nonumber \notag` | ⬜ | |
| `\label \ref` | ⬜ | Within document |

### 4.8 Color

Color is in scope for v1.0 (Milestone 8). Channel math uses `Dim` — never hardware `f32`/`f64`. An unsupported model or unknown name is `Err(Unsupported)`, never a silent default color.

| Capability | Status | Notes |
|---|---|---|
| `\color{name}` | ⬜ | Named color — affects all following content in scope |
| `\textcolor{name}{expr}` | ⬜ | Color applied to expression only |
| `\colorbox{name}{expr}` | ⬜ | Background color box around expression |
| `\fcolorbox{border}{bg}{expr}` | ⬜ | Framed color box |
| `\definecolor{name}{model}{spec}` | ⬜ | Custom color definition |
| Named colors (standard LaTeX set) | ⬜ | black white red green blue cyan magenta yellow + dvipsnames set |
| RGB color model `{rgb}{r,g,b}` | ⬜ | Values 0.0–1.0 via `Dim` |
| HTML hex color model `{HTML}{RRGGBB}` | ⬜ | 6-digit hex |
| CMYK color model `{cmyk}{c,m,y,k}` | ⬜ | Values 0.0–1.0 via `Dim`; naive `(1-c)(1-k)` to sRGB |
| Gray model `{gray}{g}` | ⬜ | Value 0.0–1.0 via `Dim` |
| Color inheritance / scope | ⬜ | Color scoped to current group `{}` |
| SVG fill / stroke emission | ⬜ | `fill` and `stroke` attributes on path elements |
| PNG color pass-through | ⬜ | tiny-skia color support |
| egui color pass-through | ⬜ | egui `Color32` emission |
| Unsupported color model | ⬜ | Returns `Err(Unsupported)` — never a fake color |

---

## 5. What latex-rust Does Not Do

These are explicitly out of scope. They return `Err(Unsupported)` — never a fake render.

- Text mode LaTeX (document structure, `\section`, `\begin{document}`)
- TikZ / PGF graphics
- Chemistry notation (`\ce{}`) — separate crate
- BibTeX / bibliography
- PDF output (separate crate)
- Font substitution / fallback synthesis
- Right-to-left math

Unsupported color *models* (for example `spot`) and unknown color names return `Err(Unsupported)`. They are not out of scope as a feature — they are honest failures inside the color system (see §4.8).

---

## 6. Gold Test Standard

Every capability entry in §4 that is marked ✅ must have a corresponding gold test.

A gold test for latex-rust specifies:
- Input LaTeX string
- Expected SVG output hash OR pixel-level comparison
- Expected box dimensions (width, height, depth) at reference size
- Expected parse success or named error

Gold format:
```toml
[[gold]]
name = "frac_simple"
input = "\\frac{1}{2}"
kind = "svg"
box_width_mu  = "10.5"
box_height_mu = "12.3"
box_depth_mu  = "4.1"
svg_hash = "sha256:abc123..."
```

Color golds (hex is the contract until SVG lands; SVG golds then lock `fill`/`stroke`):

```toml
[[gold]]
name = "textcolor_red_x"
input = "\\textcolor{red}{x}"
kind = "svg"
notes = "x glyph path has fill:#ff0000"

[[gold]]
name = "color_scope"
input = "{\\color{blue} x + y} + z"
kind = "svg"
notes = "x and y blue, z default color"

[[gold]]
name = "definecolor_custom"
input = "\\definecolor{mygreen}{rgb}{0.0,0.6,0.0}\\textcolor{mygreen}{f(x)}"
kind = "svg"
notes = "custom rgb color applied correctly"

[[gold]]
name = "colorbox_expression"
input = "\\colorbox{yellow}{\\frac{1}{2}}"
kind = "svg"
notes = "yellow background box around fraction"

[[gold]]
name = "unsupported_color_model"
input = "\\definecolor{x}{spot}{0.5}"
kind = "error"
expect = "Unsupported"
notes = "spot color model not supported — never a fake color"
```

Rules:
- Gold is the contract. Code must pass gold. Gold is never changed to pass code.
- A render that differs by one pixel from gold is a failure.
- A parse that returns a different error than gold is a failure.
- No capability is marked ✅ without a passing gold.

---

## 7. Build Milestones

Each milestone ends with a full gold suite pass before the next begins.

### Milestone 1 — Core Infrastructure
- [x] Crate scaffolding (`latex-rust`, dual license, GitHub)
- [x] `Dim` type wrapping zenith-float for layout math
- [x] Font metric loader (STIX Two Math)
- [x] `MathBox` type and basic composition
- [x] Parser skeleton — tokenizer only
- [x] Gold runner infrastructure
- [x] Math symbol catalog (`data/symbols.tsv`)
- [x] Color model parser + named/dvipsnames table (spec in §4.8; SVG in Milestone 8)

### Milestone 2 — Parser Complete
- [x] Full tokenizer
- [x] `MathNode` AST complete
- [x] Parser produces correct AST for all §4 node types
- [x] `ParseError` for all unsupported input
- [x] Gold tests for every node type

### Milestone 3 — Layout Engine
- [ ] Style machine (display / text / script / scriptscript / cramped)
- [ ] Atom spacing table (TeX Appendix G)
- [ ] Fraction layout
- [ ] Radical layout (vinculum)
- [ ] Sub/superscript layout
- [ ] Delimiter sizing
- [ ] Operator limits placement
- [ ] Matrix layout
- [ ] Gold tests for layout dimensions on every node type

### Milestone 4 — SVG Renderer
- [ ] Glyph path extraction from TrueType
- [ ] Box → SVG transform
- [ ] Self-contained SVG output
- [ ] Extensible delimiter rendering
- [ ] Gold tests for SVG output on representative expressions

### Milestone 5 — Symbol Coverage
- [ ] Full Greek alphabet
- [ ] Full AMS symbol set
- [ ] Blackboard bold, calligraphic, fraktur
- [ ] All named operators
- [ ] Gold tests for every symbol category

### Milestone 6 — Accents and Decorations
- [ ] All single-char accents
- [ ] Extensible accents
- [ ] Overbrace / underbrace
- [ ] Overline / underline
- [ ] Gold tests for all accent types

### Milestone 7 — Advanced Structures
- [ ] Multiline environments (align, gather, multline)
- [ ] Equation numbering
- [ ] Cases environment
- [ ] Gold tests for all environments

### Milestone 8 — Color Support
- [x] Color model parser — named, rgb, HTML hex, cmyk, gray (`parse_color_spec`, golds)
- [x] Standard named color table — 8 LaTeX names + 68 dvipsnames (`data/dvipsnames.tsv`)
- [x] `\definecolor` custom color registration (`ColorTable::define`)
- [x] Color AST nodes — `Color`, `TextColor`, `ColorBox`, `FColorBox` (`golds/parse.toml`)
- [ ] Color scope propagation through layout engine
- [ ] SVG renderer — `fill` and `stroke` attribute emission on colored glyphs
- [ ] PNG renderer — tiny-skia color pass-through
- [ ] egui renderer — `Color32` emission
- [~] Gold tests for every color capability (hex / tokenize / parse AST locked; SVG `fill` golds wait on Milestone 4+8 render)
- [ ] Full corpus pass before Milestone 9

### Milestone 9 — PNG Renderer (feature = "png")
- [ ] tiny-skia integration
- [ ] DPI-aware rasterization
- [ ] Gold tests for pixel output

### Milestone 10 — egui Renderer (feature = "egui")
- [ ] egui Shape emission
- [ ] Zero SVG intermediate
- [ ] Integration test with egui test harness
- [ ] Gold tests for shape output

### Milestone 11 — Polish and Release
- [ ] Full documentation (rustdoc on every public item)
- [ ] README with examples
- [ ] CHANGELOG
- [ ] Benchmarks vs KaTeX reference renders
- [ ] crates.io publish

---

## 8. Dependencies

| Crate | Purpose | Optional |
|---|---|---|
| `zenith-float` | Arbitrary precision layout math | No |
| `ttf-parser` | TrueType glyph extraction | No |
| `tiny-skia` | PNG rasterization | Yes (feature = "png") |
| `egui` | egui primitive emission | Yes (feature = "egui") |

Color uses these crates only: CSS `fill`/`stroke` strings in SVG, `tiny-skia` color when the `png` feature is on, `egui::Color32` when the `egui` feature is on. No extra dependency.

No other dependencies. No JavaScript. No webview. No runtime.

---

## 9. Performance Targets

| Operation | Target |
|---|---|
| Parse `\frac{a+b}{c+d}` | < 50µs |
| Layout full display equation | < 500µs |
| SVG render full display equation | < 1ms |
| PNG render at 144dpi | < 5ms |
| Cold start (first render) | < 10ms |

KaTeX cold start in a browser is typically 200-400ms. latex-rust target is < 10ms including font loading.

---

## 10. Correctness Standard

- No hardware float (`f32` / `f64`) in any layout calculation
- All dimension arithmetic uses `zenith-float` `Dim` type
- Font metrics stored and computed in `Dim`
- TeX Appendix G spacing table implemented exactly
- Unsupported input always returns `Err(Unsupported)` — never a fake render
- Color channels are `Dim` (or integer hex). Unsupported color models never synthesize a color
- A wrong render is worse than `Err(Unsupported)`

---

## 11. Licensing and Relationship to Accumath

latex-rust is MIT OR Apache-2.0. It is a standalone crate.

It does not mention Accumath. It does not link to Accumath. It does not require Accumath.

Accumath uses latex-rust as a dependency for UI rendering. That relationship is one-directional and not documented in this crate.

The font data (STIX Two Math, Latin Modern Math) is OFL licensed and may be embedded in the compiled binary.

---

## 12. Contact and Development

latex-rust is developed as a standalone open source contribution to the Rust ecosystem.

Issues, PRs, and capability requests tracked via the repository issue tracker.

No alpha or beta period. Ship when the gold suite is complete and green.

---

*This document is the engineering source of truth. A capability not listed here is not in the crate. A capability listed here without a gold is not complete.*
