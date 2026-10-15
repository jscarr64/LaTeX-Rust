//! Preprocess must return, and the rewrite must stay linear in the input.

#![no_main]

use latex_rust::preprocess;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(input) = std::str::from_utf8(data) else {
        return;
    };
    let out = preprocess(input);
    assert!(out.len() < 8 * input.len() + 64);
});
