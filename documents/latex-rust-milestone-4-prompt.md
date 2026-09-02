# LaTeX-Rust — Milestone 4 Prompt
**SVG Renderer**  
**Repository:** https://github.com/jscarr64/LaTeX-Rust  
**Branch:** main  
**Prerequisite:** Milestone 3 green — layout engine producing correct `MathBox` tree, all layout golds passing, `cargo test` green  

---

## Prompt for Opus

You are building Milestone 4 of LaTeX-Rust — a pure Rust LaTeX math renderer. Milestones 1, 2, and 3 are complete and green. The parser produces a correct `MathNode` AST. The layout engine produces a correct `MathBox` tree with all dimensions in `zenith-float` `Dim`.

Milestone 4 goal: **SVG Renderer** — `MathBox` tree → self-contained SVG output.

---

### What to build

A complete SVG renderer that converts a `MathBox` tree into a self-contained SVG document. Every glyph is emitted as a `<path>` element extracted from the TrueType font — no font embedding required, no external font dependencies, scales perfectly at any size.

---

### Output requirements

The SVG output must be:
- Self-contained — no external font references, no CDN links, no runtime dependencies
- Scalable — correct at any size, no pixel artifacts
- Valid SVG 1.1
- Renderable in any SVG-capable context — browser, desktop app, document
- Minimal — no unnecessary attributes, no bloat
- Color-correct — `\textcolor` and `\colorbox` produce correct `fill` and `stroke` attributes

---

### Architecture

```
MathBox tree
    ↓
SVG builder
    ├── GlyphExtractor     — TrueType → SVG path data
    ├── TransformStack     — position tracking during tree walk
    ├── ColorStack         — color scope propagation
    └── SvgWriter          — element emission
    ↓
SVG document string
```

---

### Glyph extraction

Every glyph is extracted from the TrueType font as an SVG path:

- Use `ttf-parser` to extract glyph outlines
- Convert TrueType contours to SVG `d` attribute path data
  - `M` moveto
  - `L` lineto
  - `C` cubic bezier
  - `Q` quadratic bezier
  - `Z` closepath
- Apply transform for position, scale, and baseline
- Cache extracted paths — do not re-extract the same glyph twice
- Font: STIX Two Math primary, Latin Modern Math fallback

---

### SVG structure

```svg
<svg xmlns="http://www.w3.org/2000/svg"
     width="Wpt" height="Hpt"
     viewBox="0 0 W H">
  <g>
    <!-- glyphs as <path> elements -->
    <!-- rules as <rect> elements -->
    <!-- color groups as <g fill="..."> -->
  </g>
</svg>
```

Rules (fraction bars, radical vinculum, overline, underline):
- Emit as `<rect>` elements with exact dimensions from `MathBox`

Kerns and glue:
- Applied as transform offsets — not emitted as SVG elements

Color:
- `\textcolor{color}{expr}` → `<g fill="color">...</g>` wrapping the expression glyphs
- `\colorbox{color}{expr}` → `<rect fill="color"/>` background + expression glyphs on top
- Named colors → hex values from the standard color table
- RGB → `rgb(r,g,b)` or hex
- HTML hex → `#RRGGBB` directly

---

### API

```rust
/// Render a MathBox tree to a self-contained SVG string
pub fn render_svg(
    box_tree: &MathBox,
    options: &SvgOptions,
) -> Result<String, RenderError>

pub struct SvgOptions {
    pub font_size_pt: f64,      // reference font size in points
    pub color: Option<ColorSpec>, // default text color
    pub display: bool,          // display math centering
}

pub enum RenderError {
    Unsupported(String),
    FontError(String),
    GlyphNotFound(char),
}
```

---

### Gold tests required

Every render path must have a gold test. Gold tests specify:
- Input LaTeX string (parsed and laid out by Milestones 2 and 3)
- Expected SVG output — either exact string match or structural match
- Expected dimensions: `width` and `height` attributes on the `<svg>` element
- Expected glyph count: number of `<path>` elements
- Expected color attributes where color is used

Minimum gold coverage:

**Basic glyphs:**
- Single character `x` — one path element, correct position
- Single digit `2` — one path element
- Greek letter `\alpha` — correct glyph extracted

**Structures:**
- `\frac{1}{2}` — two glyph paths, one rect for bar, correct vertical positions
- `\sqrt{x}` — glyph path for radical, rect for vinculum, correct dimensions
- `x^2` — two glyph paths, superscript at correct position
- `x_2` — two glyph paths, subscript at correct position
- `x_2^3` — three glyph paths, correct positions for both

**Delimiters:**
- `\left( x \right)` — extensible parenthesis correct height
- `\left\{ \frac{1}{2} \right\}` — extensible brace correct height

**Operators:**
- `\sum_{k=1}^{n}` display — operator glyph, limits above and below
- `\int_0^1` display — operator glyph, limits as sub/sup

**Matrix:**
- `\begin{pmatrix} a & b \\ c & d \end{pmatrix}` — all four glyph paths, delimiter sizing

**Accents:**
- `\hat{x}` — base glyph and accent glyph at correct position
- `\overline{x+y}` — glyph paths and rect for overline

**Color:**
- `\textcolor{red}{x}` — path element has `fill="#ff0000"`
- `{\color{blue} x + y} + z` — x and y in blue group, z outside
- `\colorbox{yellow}{\frac{1}{2}}` — yellow rect behind fraction glyphs
- `\definecolor{mygreen}{rgb}{0,0.6,0}` then `\textcolor{mygreen}{f}` — custom color correct

**Multiline:**
- `\begin{aligned} x &= 1 \\ y &= 2 \end{aligned}` — two rows, alignment correct

**Error cases:**
- Glyph not in font → `Err(GlyphNotFound)`
- Unsupported render path → `Err(Unsupported)`

---

### Performance requirement

- Single display equation render: < 1ms
- Glyph cache hit rate: > 95% on typical mathematical expressions
- No glyph extracted more than once per render call for the same font size

---

### Rules

- Fix code to pass gold — never change gold to pass code
- No `f32` or `f64` in dimension calculations — use `Dim` from the layout engine
- `f64` is permitted only for SVG coordinate emission (SVG spec uses decimal numbers)
- Run `cargo test` after every render path before moving to the next
- Every unsupported render path returns `Err(RenderError::Unsupported)` — never a fake render
- No panics on any input
- Glyph cache must be per render call — no global mutable state
- Push to main only when `cargo test` is fully green
- Full corpus pass before declaring Milestone 4 complete
