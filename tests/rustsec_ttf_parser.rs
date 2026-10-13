//! RUSTSEC-2026-0192 has no patched `ttf-parser` release. The manifest
//! must not depend on that crate.

#[test]
fn ttf_parser_is_not_a_dependency() {
    let manifest = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"));
    let listed = manifest.lines().any(|line| {
        let line = line.trim();
        line.starts_with("ttf-parser")
    });
    assert!(
        !listed,
        "RUSTSEC-2026-0192: ttf-parser is unmaintained and has no patched release"
    );
}
