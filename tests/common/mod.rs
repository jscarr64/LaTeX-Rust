//! Shared helpers for the lexicon tests. Not a test binary.

use latex_rust::{parse, parse_with_options, ParseError, ParseOptions};

pub fn gold(input: &str) -> Result<String, ParseError> {
    Ok(parse(input)?.gold())
}

pub fn gold_with(input: &str, options: &ParseOptions) -> Result<String, ParseError> {
    Ok(parse_with_options(input, options)?.0.gold())
}

pub fn flag(name: &str) -> ParseOptions {
    let options = ParseOptions::new();
    match name {
        "ascii" => options.with_ascii_relations(true),
        "bare" => options.with_bare_operators(true),
        "unicode" => options.with_unicode_symbols(true),
        "prime" => options.with_prime_symbol(true),
        other => panic!("unknown lexicon flag {other}"),
    }
}
