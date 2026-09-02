# Changelog

## 0.1.0 (unreleased)

### Milestone 3 — Layout engine

- `layout(&MathNode, &MathFont, MathStyle) -> MathBox` with TeX Appendix G style, Table 18 spacing, fractions, radicals, scripts, `\left\right` / `\big` sizes, operator limits, matrices, and color wrappers.
- Every dimension is [zenith-float](https://crates.io/crates/zenith-float) 1.0 `Dim`. This crate depends on that published crate, not on its inner kernel package.
- Golds in `golds/layout.toml`.

### Milestone 2 — Parser complete

- `src/parser/`: preprocess, tokenize, Pratt-style scripts, TeX math-list parse to `MathNode`.
- `ParseError::{Unsupported, Unknown, Malformed, UnmatchedDelimiter}` name the command or construct.
- Golds in `golds/parse.toml` for every node type, font style, accent, named/large operator, matrix and multiline environment, color models, and `\left`/`\right` pairs.

### Milestone 1 — Core infrastructure

- `Dim` wrapping zenith-float `ExactNum` for all layout arithmetic.
- `MathBox` with hpack / vpack composition.
- STIX Two Math 2.13 metric loader (`units_per_em`, advances, extents).
- LaTeX math tokenizer (no AST yet).
- Color model parser: named + dvipsnames, rgb / HTML / cmyk / gray via `Dim`; unknown models `Err(Unsupported)`.
- Gold runner and golds for dim, box, font, tokenize, symbols, and color models.
