# Symbol table

Canonical crate catalog: [`data/symbols.tsv`](../data/symbols.tsv) (226 unique
commands). Loaded by `latex_rust::symbols`.

Upstream tables in this folder:

| File | Use |
|---|---|
| `latex_symbols_keyboard_ready.tsv` | Source of truth for glyph, description, LaTeX, **category**, key type. Flutter column is UI-only and is not compiled in. |
| `latex_symbols - Master LaTeX Symbols.tsv` | Same glyphs/LaTeX without category. Kept for reference. |

Dedup: one duplicate `\sqrt{}` row was dropped. `\sqrt[{}]{}` is kept as the
nth-root form.

Parser Milestone 5 golds will require every catalog command (except
out-of-scope document/TikZ constructs) to parse or return a named `Unsupported`.
