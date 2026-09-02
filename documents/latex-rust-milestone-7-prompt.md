# LaTeX-Rust — Milestone 7 Prompt
**Advanced Structures**  
**Repository:** https://github.com/jscarr64/LaTeX-Rust  
**Branch:** main  
**Prerequisite:** Milestone 6 green — all accents and decorations gold verified, `cargo test` green  

---

## Prompt for Opus

You are building Milestone 7 of LaTeX-Rust — a pure Rust LaTeX math renderer. Milestones 1 through 6 are complete and green. Parser, layout engine, SVG renderer, symbol coverage, and accents are all production ready and gold verified.

Milestone 7 goal: **Advanced Structures** — multiline environments, equation numbering, alignment, and all remaining complex mathematical structures.

---

### What to build

Complete support for multiline mathematical environments and advanced structures. These are the environments researchers and students use for presenting multi-step derivations, systems of equations, and structured mathematical arguments.

---

### Environments to implement

#### Aligned Equations — `\begin{aligned}`

Single equation block with alignment points. Used inside `equation` or `$$`.

```latex
\begin{aligned}
  f(x) &= x^2 + 2x + 1 \\
       &= (x+1)^2
\end{aligned}
```

**Layout rules:**
- `&` marks alignment column — right side of `&` aligns vertically
- `\\` ends a row
- Rows separated by `\baselineskip` × `\arraystretch`
- No equation numbers on individual rows
- Width = sum of widest left column + widest right column per alignment pair

#### Align Environment — `\begin{align}`

Multiline equations each with optional equation number.

```latex
\begin{align}
  f(x) &= x^2 + 1 \\
  g(x) &= 2x - 3
\end{align}
```

**Layout rules:**
- Same alignment as `aligned`
- Each row gets an equation number by default (right-aligned)
- `\nonumber` or `\notag` suppresses number on that row
- `\tag{label}` replaces auto number with custom label
- Numbers increment per document — counter maintained across render calls if provided

#### Gather Environment — `\begin{gather}`

Multiple equations centered individually, each numbered.

```latex
\begin{gather}
  x + y = 1 \\
  2x - y = 3
\end{gather}
```

**Layout rules:**
- Each row centered independently
- Each row numbered by default
- `\nonumber` suppresses number
- Row spacing same as align

#### Multline Environment — `\begin{multline}`

Single long equation broken across multiple lines.

```latex
\begin{multline}
  f(x) = a_0 + a_1 x + a_2 x^2 \\
       + a_3 x^3 + a_4 x^4
\end{multline}
```

**Layout rules:**
- First line left-aligned
- Last line right-aligned
- Middle lines centered
- Single equation number on last line

#### Equation Environment — `\begin{equation}`

Single numbered equation in display style.

```latex
\begin{equation}
  E = mc^2
\end{equation}
```

**Layout rules:**
- Display math centered
- Equation number right-aligned on same baseline
- `\nonumber` or `\notag` suppresses number
- `\label{key}` registers for cross-reference

#### Cases Environment — `\begin{cases}`

Piecewise function definition.

```latex
f(x) = \begin{cases}
  x^2  & \text{if } x > 0 \\
  0    & \text{if } x = 0 \\
  -x^2 & \text{if } x < 0
\end{cases}
```

**Layout rules:**
- Large left brace sized to total height
- Left column: math mode
- Right column: math or text mode
- Columns separated by quad space
- No right delimiter

#### Array Environment — `\begin{array}`

General tabular structure in math mode.

```latex
\begin{array}{lcr}
  a & b & c \\
  d & e & f
\end{array}
```

**Layout rules:**
- Column spec: `l` left, `c` center, `r` right, `|` vertical rule
- `\\` ends row
- `\hline` horizontal rule between rows
- Column widths: maximum of all entries in that column
- Row heights: maximum height + depth of all entries in that row

#### Split Environment — `\begin{split}`

Single equation split across lines, used inside `equation`.

```latex
\begin{equation}
\begin{split}
  f(x) &= (x+1)^2 \\
       &= x^2 + 2x + 1
\end{split}
\end{equation}
```

**Layout rules:**
- Alignment same as `aligned`
- Single equation number from enclosing `equation`
- Width fills available display width

---

### Equation Numbering

Equation numbering must be consistent and configurable.

```rust
pub struct NumberingConfig {
    pub style: NumberStyle,
    pub start: usize,
    pub format: NumberFormat,
}

pub enum NumberStyle {
    Arabic,      // (1), (2), (3)
    Roman,       // (i), (ii), (iii)
    Alphabetic,  // (a), (b), (c)
}

pub enum NumberFormat {
    Parenthesized,  // (1)
    Bracketed,      // [1]
    Plain,          // 1
}
```

**`\tag{}`:**
- Replaces auto number with custom content
- `\tag*{}` — no parentheses around tag
- Tag content rendered in text style

**`\label{key}`:**
- Registers equation for cross-reference
- `\ref{key}` → equation number as string
- Forward references resolved in two-pass render

---

### Additional Structures

#### Substack — `\substack`

Multiple lines of limits for large operators.

```latex
\sum_{\substack{i=1 \\ i \neq j}}^{n}
```

**Layout:**
- Lines centered and stacked
- Used as sub or superscript of large operator
- Each line in script style

#### Underbrace / Overbrace with Stacked Labels

```latex
\underbrace{a + b + c}_{= S} \quad \overbrace{x + y}^{n \text{ terms}}
```

Already implemented in Milestone 6 — verify integration with multiline environments.

#### `\intertext` — Text Between Equation Rows

```latex
\begin{align}
  x &= 1 \\
  \intertext{Substituting into the second equation:}
  y &= 2
\end{align}
```

**Layout:**
- Text rendered in normal text mode
- Full width, no equation number
- Spacing above and below as paragraph

---

### Gold tests required

Every environment must have gold tests. Minimum coverage:

**aligned:**
- Two-row aligned — alignment column correct vertical position
- Three-row aligned — all rows aligned correctly
- Nested `\frac` in aligned — height/depth correct

**align:**
- Two equations — both numbered
- `\nonumber` on second row — only first numbered
- `\tag{*}` custom tag — tag rendered correctly

**gather:**
- Two centered equations — each centered independently
- Mixed `\nonumber` — only tagged rows numbered

**multline:**
- Two-line equation — first left, last right
- Three-line equation — first left, middle centered, last right

**equation:**
- Single equation with number
- `\nonumber` — no number rendered
- `\label` registered — `\ref` resolves correctly

**cases:**
- Three-case piecewise — brace sized correctly
- Text in right column — `\text{}` renders correctly
- Nested fraction in left column — height correct

**array:**
- 2×3 array with `lcr` spec — alignment correct per column
- `\hline` — horizontal rule rendered
- `|` in column spec — vertical rule rendered

**split:**
- Two-line split inside equation — single number, alignment correct

**substack:**
- Two-line substack as subscript — centered, script style

**intertext:**
- Text between align rows — correct spacing, no number

**Error cases:**
- Mismatched `\begin{align}` / `\end{gather}` → `Err(Malformed)`
- Unknown environment → `Err(Unsupported)`
- Missing `\\` at end of row — handled gracefully not panic

---

### Rules

- Fix code to pass gold — never change gold to pass code
- All layout arithmetic in `Dim` — no `f32` or `f64`
- Environment layout matches TeX output exactly
- Run `cargo test` after every environment before moving to the next
- No panics on any input — malformed environments return `Err`
- 100% pure Rust — no exceptions
- Push to main only when `cargo test` is fully green
- Full corpus pass before declaring Milestone 7 complete
