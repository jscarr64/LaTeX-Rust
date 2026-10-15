//! Runner for `golds/lexicon.toml`. Each row is one equivalence pair.

use latex_rust::{parse_with_options, ParseOptions};

struct Rec {
    name: String,
    flags: String,
    input: String,
    pair: String,
    expect: String,
    forbid: Option<String>,
}

fn unescape(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('\\') => out.push('\\'),
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('"') => out.push('"'),
                Some(other) => out.push(other),
                None => break,
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn parse_value(raw: &str) -> String {
    let t = raw.trim();
    if let Some(s) = t.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
        unescape(s)
    } else {
        t.to_string()
    }
}

fn load_golds() -> Vec<Rec> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/golds/lexicon.toml");
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) => panic!("lexicon.toml: {err}"),
    };
    let mut recs = Vec::new();
    let mut rec = blank();
    let mut seen = false;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[[gold]]" {
            if seen {
                recs.push(rec);
            }
            rec = blank();
            seen = true;
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let v = parse_value(v);
        match k.trim() {
            "name" => rec.name = v,
            "flags" => rec.flags = v,
            "input" => rec.input = v,
            "pair" => rec.pair = v,
            "expect" => rec.expect = v,
            "forbid" => rec.forbid = Some(v),
            other => panic!("{}: unknown gold field {other}", rec.name),
        }
    }
    if seen {
        recs.push(rec);
    }
    recs
}

fn blank() -> Rec {
    Rec {
        name: String::new(),
        flags: String::new(),
        input: String::new(),
        pair: String::new(),
        expect: String::new(),
        forbid: None,
    }
}

fn options_for(name: &str, flags: &str) -> ParseOptions {
    let mut options = ParseOptions::new();
    if flags.is_empty() {
        return options;
    }
    for flag in flags.split_whitespace() {
        options = match flag {
            "ascii_relations" => options.with_ascii_relations(true),
            "bare_operators" => options.with_bare_operators(true),
            "unicode_symbols" => options.with_unicode_symbols(true),
            "prime_symbol" => options.with_prime_symbol(true),
            other => panic!("{name}: unknown flag {other}"),
        };
    }
    options
}

fn gold_of(name: &str, input: &str, options: &ParseOptions) -> String {
    match parse_with_options(input, options) {
        Ok((node, _)) => node.gold(),
        Err(err) => panic!("{name}: parse {input:?}: {err}"),
    }
}

#[test]
fn lexicon_pairs() {
    let recs = load_golds();
    assert!(
        recs.len() >= 28,
        "equivalence table has {} rows",
        recs.len()
    );
    for rec in &recs {
        assert!(!rec.name.is_empty(), "gold row missing name");
        assert!(!rec.input.is_empty(), "{}: missing input", rec.name);
        assert!(!rec.expect.is_empty(), "{}: missing expect", rec.name);
        let options = options_for(&rec.name, &rec.flags);
        let got = gold_of(&rec.name, &rec.input, &options);
        assert_eq!(got, rec.expect, "{}", rec.name);
        if !rec.pair.is_empty() {
            let pair = gold_of(&rec.name, &rec.pair, &options);
            assert_eq!(pair, rec.expect, "{} pair", rec.name);
        }
        if let Some(forbid) = &rec.forbid {
            assert_ne!(got, *forbid, "{} forbid", rec.name);
        }
    }
}
