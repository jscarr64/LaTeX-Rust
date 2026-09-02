# Contributing

Repository: <https://github.com/jscarr64/LaTeX-Rust>

LaTeX-Rust ships when the gold suite is complete and green. A capability in
`documents/latex-rust-build-sheet.md` is not done without a passing gold.

Symbol and command coverage is locked against `data/symbols.tsv` (and the
inventory in `documents/` when that file is present). Do not invent a second
symbol list.

## Rules

- **100% pure Rust.** No C, no Python, no shell commands, no shelling out
  (`std::process`, `Command`, `system`), no Node/JavaScript, no external tools.
- **Dependencies must be Rust crates only.** Allowed render/font stack:
  `ttf-parser`, `tiny-skia` (feature `png`), `egui` (feature `egui`). Layout math
  is [zenith-float](https://crates.io/crates/zenith-float) 1.0 (`Dim` /
  `ExactNum`). Do not depend on an inner kernel crate.
- **If it cannot be done in Rust, return `Err`.** Do not call something else.
  Never invent a render.
- **Fix code to pass gold — never change gold to pass code.** Gold is the
  contract. Do not edit golds to hide a wrong render or a silent parse.
- **Every new capability requires a gold test** before marking it complete in
  the build sheet.
- **No hardware `f32` / `f64` in layout math.** Use `Dim`. Renderers may convert
  `Dim` to IEEE bits only at the pixel/shape emission boundary.
- **Run `cargo test` after every change** before committing. Also run the
  feature configurations you touched (`--features png`, `--features egui`, or
  both).
- **Dual-license all new files MIT OR Apache-2.0.**
- **Do not relicense the STIX font files.** STIX Two Math remains SIL OFL 1.1.

## Crate layout

Follow `documents/latex-rust-build-sheet.md` §3:

`src/parser`, `src/layout`, `src/font`, `src/render/{svg,png,egui}`, `golds/`,
`benches/`.

## Pull requests

Open issues and pull requests at <https://github.com/jscarr64/LaTeX-Rust>.
