//! `max_depth` stays at the default.
//!
//! An abort kills the fuzzer, so this target never calls `with_max_depth`,
//! including the one input in 256 a coverage note might want to raise.
//! Values above the default, including past 3,531, belong to the subprocess
//! test. Do not "fix" coverage by raising the limit in this process.

#![no_main]

use std::cell::OnceCell;
use std::sync::atomic::{AtomicU64, Ordering};

use latex_rust::{latex_to_svg, parse, preprocess, MathFont, SvgOptions};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(input) = std::str::from_utf8(data) else {
        return;
    };
    let _preprocessed = preprocess(input);
    let Ok(node) = parse(input) else {
        maybe_svg(data, input);
        return;
    };
    let printed = node.to_latex();
    let Ok(again) = parse(&printed) else {
        panic!("reprint did not parse");
    };
    assert_eq!(again, node);
    assert_eq!(again.to_latex(), printed);
    maybe_svg(data, input);
});

fn maybe_svg(data: &[u8], input: &str) {
    static SEEN: AtomicU64 = AtomicU64::new(0);
    let n = SEEN.fetch_add(1, Ordering::Relaxed);
    if n % 64 != 0 || data.len() >= 4096 {
        return;
    }
    thread_local! {
        static FONT: OnceCell<Option<MathFont>> = const { OnceCell::new() };
    }
    FONT.with(|slot| {
        let font = slot.get_or_init(|| MathFont::stix_two_math().ok());
        if let Some(font) = font {
            let _ = latex_to_svg(input, font, &SvgOptions::new());
        }
    });
}
