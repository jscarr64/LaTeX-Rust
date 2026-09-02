# LaTeX-Rust — Milestone 5 Prompt
**Symbol Coverage**  
**Repository:** https://github.com/jscarr64/LaTeX-Rust  
**Branch:** main  
**Prerequisite:** Milestone 4 green — SVG renderer producing correct output, all render golds passing, `cargo test` green  

---

## Prompt for Opus

You are building Milestone 5 of LaTeX-Rust — a pure Rust LaTeX math renderer. Milestones 1 through 4 are complete and green. The parser, layout engine, and SVG renderer are all production ready and gold verified.

Milestone 5 goal: **Complete Symbol Coverage** — every symbol in the catalog renders correctly, every font style renders correctly, every named operator renders correctly.

---

### What to build

Complete symbol coverage means every LaTeX math symbol that a researcher, student, or engineer would reasonably expect to use renders correctly. No gaps. No fake glyphs. If a symbol is not in the font it returns `Err(GlyphNotFound)` — never a substitution silently made.

---

### Symbol categories to complete

#### Greek Letters — Full Set

**Lowercase:**
`\alpha` `\beta` `\gamma` `\delta` `\epsilon` `\varepsilon` `\zeta` `\eta`
`\theta` `\vartheta` `\iota` `\kappa` `\varkappa` `\lambda` `\mu` `\nu`
`\xi` `\pi` `\varpi` `\rho` `\varrho` `\sigma` `\varsigma` `\tau`
`\upsilon` `\phi` `\varphi` `\chi` `\psi` `\omega`

**Uppercase:**
`\Gamma` `\Delta` `\Theta` `\Lambda` `\Xi` `\Pi` `\Sigma` `\Upsilon`
`\Phi` `\Psi` `\Omega`

**Bold Greek (via \boldsymbol):**
All of the above in bold weight

#### Hebrew Letters
`\aleph` `\beth` `\gimel` `\daleth`

#### Binary Operators — Full Set
`+` `-` `\times` `\div` `\pm` `\mp` `\cdot` `\ast` `\star`
`\circ` `\bullet` `\cap` `\cup` `\sqcap` `\sqcup`
`\vee` `\wedge` `\oplus` `\ominus` `\otimes` `\oslash` `\odot`
`\bigcirc` `\dagger` `\ddagger` `\amalg` `\setminus` `\wr`
`\triangleleft` `\triangleright` `\lhd` `\rhd` `\unlhd` `\unrhd`

#### Relations — Full Set
`=` `<` `>` `\leq` `\geq` `\neq` `\ll` `\gg` `\doteq`
`\sim` `\simeq` `\approx` `\cong` `\equiv` `\prec` `\succ`
`\preceq` `\succeq` `\subset` `\supset` `\subseteq` `\supseteq`
`\sqsubset` `\sqsupset` `\sqsubseteq` `\sqsupseteq`
`\in` `\ni` `\notin` `\vdash` `\dashv` `\models` `\perp`
`\mid` `\parallel` `\bowtie` `\Join` `\smile` `\frown`
`\asymp` `\propto` `\between`

#### Negated Relations
`\not\leq` `\not\geq` `\not\sim` `\not\approx` `\not\equiv`
`\not\subset` `\not\supset` `\not\subseteq` `\not\supseteq`
`\nless` `\ngtr` `\nleq` `\ngeq` `\nsim` `\ncong`
`\nprec` `\nsucc` `\nvdash` `\nvDash` `\nVdash`

#### Arrows — Full Set
`\leftarrow` `\rightarrow` `\leftrightarrow`
`\Leftarrow` `\Rightarrow` `\Leftrightarrow`
`\longleftarrow` `\longrightarrow` `\longleftrightarrow`
`\Longleftarrow` `\Longrightarrow` `\Longleftrightarrow`
`\uparrow` `\downarrow` `\updownarrow`
`\Uparrow` `\Downarrow` `\Updownarrow`
`\nearrow` `\searrow` `\swarrow` `\nwarrow`
`\hookleftarrow` `\hookrightarrow`
`\leftharpoonup` `\leftharpoondown`
`\rightharpoonup` `\rightharpoondown`
`\rightleftharpoons` `\leftrightharpoons`
`\mapsto` `\longmapsto`
`\to` `\gets`
`\iff` `\implies` `\impliedby`

#### Dots
`\ldots` `\cdots` `\vdots` `\ddots` `\iddots`
`\dotsc` `\dotsb` `\dotsm` `\dotsi` `\dotso`

#### Set and Logic Symbols
`\emptyset` `\varnothing` `\infty` `\partial` `\nabla`
`\forall` `\exists` `\nexists` `\neg` `\lnot`
`\land` `\lor` `\top` `\bot`
`\angle` `\measuredangle` `\sphericalangle`
`\triangle` `\square` `\blacksquare` `\diamond` `\blackdiamond`
`\clubsuit` `\diamondsuit` `\heartsuit` `\spadesuit`

#### Miscellaneous Symbols
`\hbar` `\imath` `\jmath` `\ell`
`\Re` `\Im` `\wp` `\mho`
`\prime` `\backprime` `\sharp` `\flat` `\natural`
`\surd` `\checkmark` `\maltese`
`\S` `\P` `\dag` `\ddag`

#### Font Styles — Full Coverage

**Blackboard Bold `\mathbb`:**
`\mathbb{R}` `\mathbb{C}` `\mathbb{Z}` `\mathbb{Q}` `\mathbb{N}`
`\mathbb{P}` `\mathbb{F}` `\mathbb{H}` `\mathbb{A}` — full Latin uppercase set

**Calligraphic `\mathcal`:**
Full Latin uppercase set `\mathcal{A}` through `\mathcal{Z}`

**Fraktur `\mathfrak`:**
Full Latin uppercase and lowercase sets

**Script `\mathscr`:**
Full Latin uppercase set

**Roman `\mathrm`:**
Full Latin uppercase and lowercase, digits

**Bold `\mathbf`:**
Full Latin uppercase and lowercase, digits, Greek

**Italic `\mathit`:**
Full Latin uppercase and lowercase

**Sans-serif `\mathsf`:**
Full Latin uppercase and lowercase, digits

**Typewriter `\mathtt`:**
Full Latin uppercase and lowercase, digits

**Bold Symbol `\boldsymbol`:**
Greek letters and select symbols in bold weight

#### AMS Symbols — Full amssymb Set
All symbols from the `amssymb` package including:
- Additional binary operators
- Additional relations
- Additional arrows
- Additional miscellaneous symbols
- Blackboard bold digits `\mathbb{0}` through `\mathbb{9}`

---

### Gold tests required

Every symbol category must have gold tests. Minimum coverage:

**Greek:**
- All 24 lowercase Greek letters — correct glyph for each
- All 11 uppercase Greek letters — correct glyph for each
- Variant forms `\varepsilon` `\vartheta` `\varpi` `\varrho` `\varsigma` `\varphi` — correct variant glyph

**Hebrew:**
- All 4 Hebrew letters — correct glyph

**Binary operators:**
- One gold per operator — correct glyph
- Correct spacing classification (Binary atom type)

**Relations:**
- One gold per relation — correct glyph
- Correct spacing classification (Relation atom type)

**Negated relations:**
- `\not\leq` — composite glyph correct
- `\nless` `\ngtr` — dedicated glyph correct

**Arrows:**
- One gold per arrow — correct glyph
- Extensible arrows at multiple lengths

**Font styles:**
- `\mathbb{R}` — blackboard bold R correct glyph
- `\mathcal{L}` — calligraphic L correct glyph
- `\mathfrak{g}` — fraktur g correct glyph
- `\mathbf{x}` — bold x correct glyph
- `\mathrm{d}` — roman d correct glyph (upright)
- Full coverage: one gold per font style per letter class

**AMS symbols:**
- Representative gold for each AMS symbol subcategory

**Error cases:**
- Symbol not in font → `Err(GlyphNotFound)` — never a fake glyph
- Unknown font style → `Err(Unsupported)`

---

### Atom type classification

Every symbol must be classified with the correct TeX atom type for spacing:

| Type | Examples |
|---|---|
| Ordinary | Letters, digits, `\alpha`, `\infty` |
| Large Op | `\sum` `\int` `\prod` `\bigcup` |
| Binary | `+` `-` `\times` `\cup` `\cap` |
| Relation | `=` `<` `>` `\leq` `\subset` |
| Open | `(` `[` `\{` `\langle` |
| Close | `)` `]` `\}` `\rangle` |
| Punctuation | `,` `;` `:` `\ldots` |
| Inner | `\frac` fractions in text mode |

Atom type determines inter-atom spacing from the TeXbook Table 18 implemented in Milestone 3. A wrong atom type produces wrong spacing — this is a correctness failure not a style preference.

---

### Rules

- Fix code to pass gold — never change gold to pass code
- If a glyph is not in the font return `Err(GlyphNotFound)` — never substitute a different glyph silently
- Run `cargo test` after every symbol category before moving to the next
- Atom type must be correct for every symbol — wrong atom type is a bug
- No panics on any input
- Push to main only when `cargo test` is fully green
- Full corpus pass before declaring Milestone 5 complete
- Every symbol in `data/symbols.tsv` must have a passing gold before Milestone 5 closes
