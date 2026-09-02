# LaTeX-Rust Build Sheet — Color Support Addition

**Status:** Merged into [`latex-rust-build-sheet.md`](latex-rust-build-sheet.md)
on 2026-09-01 (§4.8, Milestone 8, §5, §6 golds, §8). Keep this file as the
original addendum.

Color model parsing, the 8 standard names, 68 dvipsnames, `\definecolor`
registration, and unsupported-model errors are implemented and golded as hex
(`Color::css_hex`). SVG `fill`/`stroke`, PNG, egui, and color AST nodes remain
Milestone 8 render/parser work.

---

---

## Addition to §4 Capability Inventory — New Section 4.8 Color

### 4.8 Color

| Capability | Status | Notes |
|---|---|---|
| `\color{name}` | ⬜ | Named color — affects all following content in scope |
| `\textcolor{name}{expr}` | ⬜ | Color applied to expression only |
| `\colorbox{name}{expr}` | ⬜ | Background color box around expression |
| `\fcolorbox{border}{bg}{expr}` | ⬜ | Framed color box |
| `\definecolor{name}{model}{spec}` | ⬜ | Custom color definition |
| Named colors (standard LaTeX set) | ⬜ | black white red green blue cyan magenta yellow + dvipsnames set |
| RGB color model `{rgb}{r,g,b}` | ⬜ | Values 0.0–1.0 |
| HTML hex color model `{HTML}{RRGGBB}` | ⬜ | 6-digit hex |
| CMYK color model `{cmyk}{c,m,y,k}` | ⬜ | Values 0.0–1.0 |
| Gray model `{gray}{g}` | ⬜ | Value 0.0–1.0 |
| Color inheritance / scope | ⬜ | Color scoped to current group `{}` |
| SVG fill / stroke emission | ⬜ | `fill` and `stroke` attributes on path elements |
| PNG color pass-through | ⬜ | tiny-skia color support |
| egui color pass-through | ⬜ | egui `Color32` emission |
| Unsupported color model | ⬜ | Returns `Err(Unsupported)` — never a fake color |

---

## Addition to §7 Build Milestones — Insert as New Milestone 8, Renumber Existing 8–10 to 9–11

### Milestone 8 — Color Support

- [ ] Color AST nodes — `ColorNode`, `TextColorNode`, `ColorBoxNode`, `FColorBoxNode`
- [ ] Color model parser — named, rgb, HTML hex, cmyk, gray
- [ ] Standard named color table — full dvipsnames set
- [ ] `\definecolor` custom color registration
- [ ] Color scope propagation through layout engine
- [ ] SVG renderer — `fill` and `stroke` attribute emission on colored glyphs
- [ ] PNG renderer — tiny-skia color pass-through
- [ ] egui renderer — `Color32` emission
- [ ] Gold tests for every color capability
- [ ] Full corpus pass before Milestone 9

---

## Addition to §5 What LaTeX-Rust Does Not Do

**Remove this line from §5:**
> Color (`\color{}`) in v1.0 — planned for v1.1

**It is now in scope for v1.0 — implemented in Milestone 8.**

---

## Addition to §8 Dependencies — No New Dependencies Required

Color support uses existing dependencies:
- SVG color via string attributes — no new dependency
- `tiny-skia` already handles color natively — no new dependency  
- `egui` `Color32` already available — no new dependency

---

## Gold Test Examples for Color

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
