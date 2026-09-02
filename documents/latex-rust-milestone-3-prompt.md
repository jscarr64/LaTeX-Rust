# LaTeX-Rust — Milestone 3 Prompt
**Layout Engine**  
**Repository:** https://github.com/jscarr64/LaTeX-Rust  
**Branch:** main  
**Prerequisite:** Milestone 2 green — full parser producing correct `MathNode` AST, `cargo test` green  

---

## Prompt for Opus

You are building Milestone 3 of LaTeX-Rust — a pure Rust LaTeX math renderer. Milestones 1 and 2 are complete and green. The parser produces a complete `MathNode` AST. The symbol catalog has 226 entries.

Milestone 3 goal: **Layout Engine** — `MathNode` AST → `MathBox` tree using a TeX-faithful box model.

---

### What to build

A complete layout engine that converts a `MathNode` AST into a `MathBox` tree. Every dimension is computed using `zenith-float` `Dim` — no `f32` or `f64` anywhere in the layout path.

The layout engine must faithfully implement Appendix G of The TeXbook. No approximations. No shortcuts. If TeX produces a specific layout zenith-latex produces the same layout.

---

### Core types

```rust
pub struct Dim(SoftFloat);  // zenith-float arbitrary precision dimension

pub struct MathBox {
    pub width:   Dim,        // horizontal extent
    pub height:  Dim,        // above baseline
    pub depth:   Dim,        // below baseline  
    pub italic:  Dim,        // italic correction
    pub content: BoxContent,
}

pub enum BoxContent {
    Glyph(GlyphId, FontId),
    HBox(Vec<MathBox>),
    VBox(Vec<MathBox>),
    Rule(Dim, Dim, Dim),     // width, height, depth
    Kern(Dim),               // horizontal kern
    Glue(GlyphId),           // stretchable space
    Color(ColorSpec, Box<MathBox>),
}

pub enum MathStyle {
    Display,
    DisplayCramped,
    Text,
    TextCramped,
    Script,
    ScriptCramped,
    ScriptScript,
    ScriptScriptCramped,
}
```

---

### Layout rules to implement — TeX Appendix G exact

**Style machine:**
- Initial style from context (display math → Display, inline → Text)
- Style propagation rules through every node type
- Cramped style rules — when to cramp and when to restore
- Script and scriptscript sizing — σ8 and σ9 from font metrics

**Atom spacing (TeXbook Table 18):**
- All 8×8 inter-atom spacing combinations
- Ordinary, Large Op, Binary, Relation, Open, Close, Punctuation, Inner
- Thin space (3mu), medium space (4mu), thick space (5mu)
- Context-dependent — some spaces disappear in script style

**Fraction layout:**
- Bar thickness from font metric σ8
- Numerator shift-up: σ8 (display) or σ9 (text)
- Denominator shift-down: σ11 (display) or σ12 (text)
- Clearance rules — TeXbook §15 exactly
- Bar extends full width of widest of numerator and denominator

**Radical layout:**
- Vinculum height from font metric
- Gap between radicand top and vinculum
- Degree positioning for `\sqrt[n]`
- Vinculum extends to radicand width plus right padding

**Sub/superscript layout:**
- Shift-up for superscript: σ13 (display) or σ14 (text) or σ15 (cramped)
- Shift-down for subscript: σ16 or σ17
- Clearance between sub and sup when both present
- Italic correction applied to base before superscript

**Delimiter sizing:**
- Five fixed sizes: `\big \Big \bigg \Bigg`
- `\left \right` — minimum size to cover content height + depth
- Extensible delimiters — built from top, middle, bottom, and repeated extension glyphs
- All delimiter pairs: `( ) [ ] \{ \} | \| / \backslash \lfloor \rfloor \lceil \rceil \langle \rangle`

**Large operator layout:**
- Display style: limits above and below, centered on operator
- Text style: limits as sub/superscript
- Operator size: larger glyph variant in display style

**Accent placement:**
- Single-char accents: centered over base, shifted by italic correction
- Extensible accents: stretched to base width
- Accent height from font metric

**Overbrace / underbrace:**
- Extensible brace built from glyph parts
- Annotation positioned above/below with spacing

**Matrix layout:**
- Column alignment: left, center, right
- Row spacing: `\arraystretch` × baselineskip
- Column separator spacing
- Outer delimiter sizing to cover full matrix height

**Color propagation:**
- Color scoped to current group
- Propagates through all child boxes
- SVG fill/stroke attribute emission

**Spacing commands:**
- `\,` = 3mu thin space
- `\:` = 4mu medium space  
- `\;` = 5mu thick space
- `\!` = -3mu negative thin space
- `\quad` = 1em
- `\qquad` = 2em
- `\hspace{len}` = specified length

---

### Font metrics required

All layout computations use font metrics from STIX Two Math. Required metric parameters (σ1–σ22 in TeXbook notation):

| Parameter | Metric name | Used for |
|---|---|---|
| σ1 | x_height | Accent placement |
| σ2 | quad | Em width |
| σ5 | x_height | Math axis |
| σ6 | math_axis | Fraction bar position |
| σ8 | default_rule_thickness | Fraction bar thickness |
| σ9 | num1 | Numerator shift display |
| σ10 | num2 | Numerator shift text |
| σ11 | denom1 | Denominator shift display |
| σ12 | denom2 | Denominator shift text |
| σ13 | sup1 | Superscript shift display |
| σ14 | sup2 | Superscript shift text |
| σ15 | sup3 | Superscript shift cramped |
| σ16 | sub1 | Subscript shift |
| σ17 | sub2 | Subscript shift with sup |
| σ18 | sup_drop | Superscript drop |
| σ19 | sub_drop | Subscript drop |
| σ20 | delim1 | Delimiter size display |
| σ21 | delim2 | Delimiter size text |
| σ22 | axis_height | Math axis height |

All metric values stored as `Dim` (zenith-float) — no `f32` or `f64`.

---

### Gold tests required

Every layout rule must have at least one gold test. Gold tests specify:
- Input `MathNode` (or LaTeX string parsed by Milestone 2)
- Expected `MathBox` dimensions: `width`, `height`, `depth` in mu units
- Tolerances: exact match on rational dimensions, zenith-float precision on irrational

Minimum gold coverage:

**Style propagation:**
- Display style fraction → numerator in Text style, denominator in Text style
- Script style fraction → numerator in ScriptScript, denominator in ScriptScript
- Cramped style — superscript in cramped context

**Atom spacing:**
- Ordinary + Ordinary → no space
- Ordinary + Binary → thin space
- Ordinary + Relation → thick space  
- Binary suppressed after Open delimiter
- All 8×8 combinations that produce non-zero space

**Fraction:**
- `\frac{1}{2}` — bar thickness, num/denom positions, total height/depth
- `\frac{a+b}{c+d}` — wider numerator sets bar width
- Display vs text style shift differences

**Radical:**
- `\sqrt{x}` — vinculum height, clearance, total dimensions
- `\sqrt[3]{x}` — degree positioning

**Sub/superscript:**
- `x^2` — shift-up in display, text, cramped styles
- `x_2` — shift-down
- `x_2^3` — both, clearance between them
- Italic correction on `f^2`

**Delimiters:**
- `\left( \frac{1}{2} \right)` — delimiter sized to fraction height
- `\left\{ x \right\}` — extensible brace
- All four `\big \Big \bigg \Bigg` sizes

**Large operators:**
- `\sum_{k=1}^{n}` display — limits above/below, centered
- `\sum_{k=1}^{n}` text — limits as sub/sup
- `\int_0^1` — no limits above/below in display

**Matrices:**
- `\begin{pmatrix} a & b \\ c & d \end{pmatrix}` — column widths, row spacing, delimiter sizing

**Color:**
- `\textcolor{red}{x}` — ColorSpec propagated to glyph box
- Nested color scoping — inner color overrides outer

---

### Rules

- Fix code to pass gold — never change gold to pass code
- All dimensions in `Dim` (zenith-float) — no `f32` or `f64` in layout math
- Run `cargo test` after every layout rule before moving to the next
- Implement TeXbook Appendix G exactly — no approximations
- Every unsupported layout returns `Err(Unsupported)` — never a fake box
- No panics on any input
- Push to main only when `cargo test` is fully green
- Full corpus pass before declaring Milestone 3 complete
- Reference: The TeXbook Appendix G, STIX Two Math font specification
