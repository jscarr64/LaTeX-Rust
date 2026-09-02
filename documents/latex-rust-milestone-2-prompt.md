# LaTeX-Rust — Milestone 2 Prompt
**Parser Complete**  
**Repository:** https://github.com/jscarr64/LaTeX-Rust  
**Branch:** main  
**Prerequisite:** Milestone 1 green — catalog at `data/symbols.tsv`, 226 commands, `cargo test` green  

---

## Prompt for Opus

You are building Milestone 2 of LaTeX-Rust — a pure Rust LaTeX math renderer. Milestone 1 is complete and green. The symbol catalog is at `data/symbols.tsv` with 226 unique commands. The public API entry point is `latex_rust::lookup()`.

Milestone 2 goal: **Parser Complete** — LaTeX math string → `MathNode` AST for every capability in the build sheet.

---

### What to build

Build a complete LaTeX math parser that produces a typed `MathNode` AST. The parser must handle every node type listed below. Unknown or unsupported input returns `Err(ParseError::Unsupported)` — never a partial parse accepted silently, never a fake node invented.

**MathNode variants to implement:**

```rust
MathNode
├── Atom(char, AtomKind)
├── Fraction(Box<MathNode>, Box<MathNode>)
├── Radical(Option<Box<MathNode>>, Box<MathNode>)
├── Superscript(Box<MathNode>, Box<MathNode>)
├── Subscript(Box<MathNode>, Box<MathNode>)
├── SubSup(Box<MathNode>, Box<MathNode>, Box<MathNode>)
├── Delimited(Delimiter, Box<MathNode>, Delimiter)
├── Row(Vec<MathNode>)
├── Matrix(MatrixStyle, Vec<Vec<MathNode>>)
├── Sum(Option<Box<MathNode>>, Option<Box<MathNode>>)
├── Integral(Option<Box<MathNode>>, Option<Box<MathNode>>)
├── Product(Option<Box<MathNode>>, Option<Box<MathNode>>)
├── Limit(Option<Box<MathNode>>)
├── OverUnder(Box<MathNode>, Option<Box<MathNode>>, Option<Box<MathNode>>)
├── Accent(Box<MathNode>, AccentKind)
├── Text(String, TextStyle)
├── Space(SpaceKind)
├── Operator(String, bool)
├── Symbol(SymbolKind)
├── Color(ColorSpec, Box<MathNode>)
├── ColorBox(ColorSpec, Box<MathNode>)
├── Strut(Dim, Dim)
└── Phantom(Box<MathNode>)
```

---

### Parser requirements

- Pratt parser at the core — extend what exists in the Accumath engine, do not start from scratch
- Handles inline math `$...$`, display math `$$...$$` and `\[...\]`, and raw expression input
- Correct operator precedence throughout
- `ParseError` variants:
  - `Unsupported(String)` — command exists but not yet implemented
  - `Unknown(String)` — command not in catalog and not a known structure
  - `Malformed(String)` — syntactically invalid input
  - `UnmatchedDelimiter` — `\left` without `\right` or mismatched pair
- Error messages must name the specific command or position — never a generic "parse error"

---

### Commands to parse — complete list

**Structure:**
`\frac` `\dfrac` `\tfrac` `\cfrac` `\binom` `\dbinom` `\tbinom` `\genfrac`
`\sqrt` `\sqrt[n]` `^` `_` `\left` `\right` `\big` `\Big` `\bigg` `\Bigg`
`\begin{matrix}` `\begin{pmatrix}` `\begin{bmatrix}` `\begin{vmatrix}`
`\begin{Vmatrix}` `\begin{Bmatrix}` `\begin{cases}` `\begin{array}`
`\begin{aligned}` `\begin{align}` `\begin{gather}` `\begin{multline}`
`\begin{equation}` `\tag` `\label` `\nonumber` `\notag`

**Accents:**
`\hat` `\check` `\breve` `\acute` `\grave` `\tilde` `\bar` `\vec`
`\dot` `\ddot` `\dddot` `\widehat` `\widetilde` `\overline` `\underline`
`\overbrace` `\underbrace` `\overleftarrow` `\overrightarrow`
`\overset` `\underset` `\stackrel` `\cancel` `\bcancel` `\xcancel` `\boxed`

**Font styles:**
`\mathrm` `\mathbf` `\mathit` `\mathsf` `\mathtt` `\mathbb`
`\mathcal` `\mathfrak` `\mathscr` `\boldsymbol` `\pmb`
`\text` `\operatorname`

**Spacing:**
`\,` `\:` `\;` `\!` `\quad` `\qquad` `\hspace` `\phantom` `\vphantom` `\hphantom`

**Color:**
`\color` `\textcolor` `\colorbox` `\fcolorbox` `\definecolor`

**Named operators:**
`\sin` `\cos` `\tan` `\cot` `\sec` `\csc`
`\arcsin` `\arccos` `\arctan`
`\sinh` `\cosh` `\tanh` `\coth`
`\log` `\ln` `\lg` `\exp`
`\lim` `\limsup` `\liminf`
`\sup` `\inf` `\max` `\min`
`\det` `\dim` `\ker` `\deg`
`\gcd` `\lcm` `\Pr` `\arg`

**Large operators:**
`\sum` `\prod` `\int` `\iint` `\iiint` `\oint` `\oiint`
`\coprod` `\bigcup` `\bigcap` `\bigsqcup` `\bigvee` `\bigwedge`
`\bigoplus` `\bigotimes` `\biguplus`

**All 226 catalog symbols** from `data/symbols.tsv`

---

### Gold tests required

Every MathNode variant must have at least one gold test. Gold tests:
- Specify input LaTeX string
- Specify expected AST structure (serialized or matched)
- Specify expected `ParseError` variant for invalid inputs
- Must fail before the implementation and pass after
- Never changed to pass code — code is changed to pass gold

Minimum gold coverage:
- One gold per MathNode variant
- One gold per `ParseError` variant
- One gold per font style
- One gold per accent type
- One gold per named operator
- One gold per large operator
- One gold for each matrix environment
- One gold for each multiline environment
- One gold for color and colorbox
- One gold for `\definecolor` with each color model
- One gold for `\left \right` with each delimiter pair
- One gold confirming unknown command returns `Err(Unknown)`
- One gold confirming malformed input returns `Err(Malformed)`

---

### Rules

- Fix code to pass gold — never change gold to pass code
- Run `cargo test` after every node type implementation before moving to the next
- Every unsupported command returns `Err(Unsupported)` — never a fake node
- No hardware float (`f32` / `f64`) anywhere in the parser
- No panics on any input — malformed LaTeX returns `Err`, not a crash
- No partial parses silently accepted
- Push to main only when `cargo test` is fully green
- Do not start the next node type until the current one has a passing gold
