# Contributing

Repository: <https://github.com/jscarr64/LaTeX-Rust>

LaTeX-Rust ships when the gold suite is complete and green. A capability in
`documents/latex-rust-build-sheet.md` is not done without a passing gold.

Symbol and command coverage is locked against a table in `documents/` (CSV/TSV)
when that file is present. Do not invent a second symbol list.

- Gold is the contract. Change code to pass gold. Do not edit golds to hide a
  wrong render or a silent parse.
- This crate is 100% Rust. No C/C++, JavaScript, Python, or FFI numerics.
- Unsupported input must return `Err`. Never invent output.
- No hardware `f32` / `f64` in layout math. Use `Dim`.
- Run `cargo test` after each addition before starting the next.
- Dual-license new files MIT OR Apache-2.0. Do not relicense the STIX font.
- Open issues and pull requests at <https://github.com/jscarr64/LaTeX-Rust>.
