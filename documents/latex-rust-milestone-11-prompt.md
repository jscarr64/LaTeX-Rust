# LaTeX-Rust — Milestone 11 Prompt
**Polish and Release**  
**Repository:** https://github.com/jscarr64/LaTeX-Rust  
**Branch:** main  
**Prerequisite:** Milestone 10 green — all renderers gold verified, all three feature configurations passing, `cargo test` green  

---

## Prompt for Opus

You are completing Milestone 11 of LaTeX-Rust — a pure Rust LaTeX math renderer. Milestones 1 through 10 are complete and green. Every capability is implemented, gold verified, and production ready.

Milestone 11 goal: **Polish and Release** — documentation complete, benchmarks run, crates.io publish ready. This is the milestone that turns a working codebase into a professional open source release.

---

### What to complete

Nothing new is implemented in this milestone. Every public API is already built and verified. This milestone documents, measures, and packages what exists.

---

### Documentation

#### rustdoc — Every public item

Every public item in the crate must have a rustdoc comment. No exceptions. `cargo doc --no-deps` must produce zero warnings.

**Required for every public function:**
```rust
/// One-line summary ending in a period.
///
/// Longer description if needed. Explain what the function does,
/// not how it does it.
///
/// # Arguments
///
/// * `box_tree` - The laid-out math expression to render.
/// * `options` - Rendering configuration.
///
/// # Returns
///
/// The rendered SVG as a self-contained string, or a `RenderError`
/// if the expression contains unsupported constructs.
///
/// # Errors
///
/// * `RenderError::Unsupported` — expression contains a construct
///   not supported by this renderer.
/// * `RenderError::GlyphNotFound` — a required glyph is absent
///   from the font.
///
/// # Examples
///
/// ```rust
/// use latex_rust::{layout, render_svg, SvgOptions};
///
/// let box_tree = layout(r"\frac{1}{2}").unwrap();
/// let svg = render_svg(&box_tree, &SvgOptions::default()).unwrap();
/// assert!(svg.contains("<svg"));
/// ```
pub fn render_svg(box_tree: &MathBox, options: &SvgOptions) -> Result<String, RenderError>
```

Every example in rustdoc must compile and pass as a doctest. `cargo test --doc` must be green.

**Required for every public struct:**
- Summary line
- Field documentation on every public field
- At least one `# Examples` block

**Required for every public enum:**
- Summary line
- Documentation on every variant
- Explanation of when each variant occurs

#### README.md

Complete README with:

```markdown
# LaTeX-Rust

Pure Rust LaTeX math renderer. No JavaScript. No webview. No runtime dependencies.

## Features

- Parses LaTeX math to a typed AST
- TeX-faithful layout engine (Appendix G exact)  
- SVG output — self-contained, scalable, no font embedding required
- PNG output (feature = "png") via tiny-skia
- egui integration (feature = "egui") for native desktop apps
- Full symbol coverage — Greek, AMS, arrows, operators, font styles
- Complete accent and decoration support
- Multiline environments — align, gather, multline, cases, array
- Color support — named, RGB, HTML, CMYK, gray, \definecolor
- 100% pure Rust — no C, no Python, no shell, no subprocesses
- MIT OR Apache-2.0

## Quick Start

[installation and basic example]

## Usage

[SVG, PNG, egui examples]

## Supported LaTeX

[link to capability inventory]

## Performance

[benchmark results table]

## License

MIT OR Apache-2.0
```

#### CHANGELOG.md

```markdown
# Changelog

## [1.0.0] — YYYY-MM-DD

### Added
- Complete LaTeX math parser
- TeX-faithful layout engine
- SVG renderer
- PNG renderer (feature = "png")
- egui renderer (feature = "egui")  
- 226-command symbol catalog
- Full Greek, AMS, arrow symbol coverage
- Complete accent and decoration support
- Multiline environments
- Color support
- zenith-float integration for arbitrary precision layout math
```

#### CONTRIBUTING.md — Verify and complete

Confirm these rules are in CONTRIBUTING.md and clearly stated:
- 100% pure Rust — no C, no Python, no shell, no subprocesses
- Dependencies must be Rust crates only
- Fix code to pass gold — never change gold to pass code
- Every new capability requires a gold test before marking complete
- No hardware `f32`/`f64` in layout math — use `Dim`
- Run `cargo test` after every change before committing
- Dual-license all new files MIT OR Apache-2.0
- Do not relicense the STIX font files

---

### Benchmarks

Run `cargo bench` and record results. Benchmarks must cover:

| Benchmark | Target | Must pass |
|---|---|---|
| Parse `\frac{1}{2}` | < 50µs | Yes |
| Parse full display equation | < 200µs | Yes |
| Layout `\frac{1}{2}` | < 100µs | Yes |
| Layout full display equation | < 500µs | Yes |
| SVG render `\frac{1}{2}` | < 500µs | Yes |
| SVG render full display equation | < 1ms | Yes |
| PNG render at 144 DPI | < 5ms | Yes |
| PNG render at 300 DPI | < 15ms | Yes |
| egui render inline (cache miss) | < 2ms | Yes |
| egui render inline (cache hit) | < 0.1ms | Yes |

If any benchmark does not meet its target do not proceed to publish. Profile, fix, re-run.

Compare against KaTeX reference renders where possible and document the comparison in README.md.

---

### Final checklist before publish

#### Code quality
- [ ] `cargo clippy -- -D warnings` — zero warnings
- [ ] `cargo fmt --check` — zero formatting issues
- [ ] `cargo test` — green
- [ ] `cargo test --features png` — green
- [ ] `cargo test --features egui` — green
- [ ] `cargo test --features png,egui` — green
- [ ] `cargo test --doc` — all doctests green
- [ ] `cargo bench` — all targets met

#### Documentation
- [ ] `cargo doc --no-deps` — zero warnings
- [ ] Every public function has rustdoc with examples
- [ ] Every public struct has rustdoc with field docs
- [ ] Every public enum has rustdoc with variant docs
- [ ] README.md complete with benchmark results
- [ ] CHANGELOG.md complete
- [ ] CONTRIBUTING.md verified

#### Cargo.toml
- [ ] Version = "1.0.0"
- [ ] Description accurate and complete
- [ ] Keywords — `latex`, `math`, `rendering`, `svg`, `rust`
- [ ] Categories — `rendering`, `science`, `mathematics`
- [ ] License = "MIT OR Apache-2.0"
- [ ] Repository link correct
- [ ] Documentation link correct
- [ ] `exclude` — golds/, benches/fixtures/, documents/ excluded from published crate
- [ ] All optional dependencies correctly feature-gated
- [ ] MSRV (minimum supported Rust version) specified

#### Verify pure Rust claim
- [ ] `grep -r "std::process::Command" src/` — zero results
- [ ] `grep -r "unsafe" src/` — zero results (or each reviewed and justified)
- [ ] Dependency tree audit — every dependency is pure Rust
- [ ] `cargo tree --edges features` — no C or system library dependencies

#### Repository hygiene
- [ ] `.gitignore` complete — target/, *.DS_Store, etc
- [ ] LICENSE-MIT present
- [ ] LICENSE-APACHE present
- [ ] No debug artifacts committed
- [ ] All milestone prompt documents in `documents/`
- [ ] Build sheet current and accurate

---

### Publish

```bash
# Dry run first — always
cargo publish --dry-run

# Review output carefully
# If dry run is clean

cargo publish
```

After publish:
- Verify crates.io listing is correct
- Verify documentation rendered correctly on docs.rs
- Verify README renders correctly on crates.io
- Post announcement to r/rust

---

### Announcement post — r/rust

Keep it technical, honest, and brief. No hype. Let the work speak.

```
Title: LaTeX-Rust 1.0 — pure Rust LaTeX math renderer, no JS dependencies

LaTeX-Rust is a pure Rust crate for rendering LaTeX math expressions.

What it does:
- Parses LaTeX math to a typed AST
- TeX-faithful layout (Appendix G exact, all dimensions in zenith-float)
- SVG output — self-contained, no font embedding
- PNG output (feature = "png")
- egui integration (feature = "egui")
- Full symbol coverage — Greek, AMS, arrows, operators, font styles
- Color support — named, RGB, HTML, CMYK
- Multiline environments — align, gather, cases, array

100% pure Rust. No JavaScript. No webview. No C. No subprocesses.
MIT OR Apache-2.0.

Repository: https://github.com/jscarr64/LaTeX-Rust
crates.io: https://crates.io/crates/latex-rust
```

---

### Rules

- Fix code to pass gold — never change gold to pass code
- Do not publish until every checklist item is complete
- Do not publish if any benchmark target is missed — profile and fix first
- Do not publish if `cargo doc` has warnings — fix them first
- Do not publish if any test configuration is not green
- 100% pure Rust — verify before publish with grep and cargo tree
- The publish is permanent — get it right before pushing the button
