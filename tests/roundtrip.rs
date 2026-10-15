//! `parse(to_latex(t)) == t`, and printing that tree again is stable.
//!
//! Fast runs use 2,000 generated trees. Release runs use 50,000.
//! `LATEX_RUST_ROUNDTRIP_CASES` overrides the count.

use latex_rust::{parse_with_options, ParseOptions};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

struct Gold {
    name: String,
    kind: String,
    input: String,
    flags: String,
    pair: String,
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

fn load_golds(file: &str) -> Vec<Gold> {
    let path = format!("{}/golds/{file}", env!("CARGO_MANIFEST_DIR"));
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(err) => panic!("{file}: {err}"),
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
            "kind" => rec.kind = v,
            "input" => rec.input = v,
            "flags" => rec.flags = v,
            "pair" => rec.pair = v,
            _ => {}
        }
    }
    if seen {
        recs.push(rec);
    }
    recs
}

fn blank() -> Gold {
    Gold {
        name: String::new(),
        kind: String::new(),
        input: String::new(),
        flags: String::new(),
        pair: String::new(),
    }
}

fn options_for(name: &str, flags: &str) -> ParseOptions {
    let mut options = ParseOptions::new();
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

fn round_trip(name: &str, input: &str, options: &ParseOptions) {
    let node = match parse_with_options(input, options) {
        Ok((node, _)) => node,
        Err(err) => panic!("{name}: {input:?} did not parse: {err}"),
    };
    let printed = node.to_latex();
    let again = match parse_with_options(&printed, options) {
        Ok((node, _)) => node,
        Err(err) => panic!("{name}: reprint {printed:?} of {input:?}: {err}"),
    };
    assert_eq!(again, node, "{name}\ninput: {input}\nprinted: {printed}");
    let second = again.to_latex();
    assert_eq!(
        second, printed,
        "{name}: reprint was not idempotent\n{printed}\n{second}"
    );
}

fn round_trip_file(file: &str, minimum: usize) {
    let recs = load_golds(file);
    let mut seen = 0usize;
    for rec in &recs {
        if !rec.kind.is_empty() && rec.kind != "ast" {
            continue;
        }
        if rec.input.is_empty() {
            continue;
        }
        let options = options_for(&rec.name, &rec.flags);
        round_trip(&format!("{file}:{}", rec.name), &rec.input, &options);
        if !rec.pair.is_empty() {
            round_trip(&format!("{file}:{} pair", rec.name), &rec.pair, &options);
        }
        seen += 1;
    }
    assert!(
        seen >= minimum,
        "{file} rounded {seen} trees, want at least {minimum}"
    );
}

#[test]
fn gold_files_round_trip() {
    round_trip_file("parse.toml", 100);
    round_trip_file("accumath.toml", 20);
    round_trip_file("lexicon.toml", 28);
}

#[test]
fn nested_color_keeps_its_groups() {
    let input = r"{\color{red} x + {\color{blue} y} + z}";
    let options = ParseOptions::new();
    let node = match parse_with_options(input, &options) {
        Ok((node, _)) => node,
        Err(err) => panic!("{input}: {err}"),
    };
    let printed = node.to_latex();
    let again = match parse_with_options(&printed, &options) {
        Ok((node, _)) => node,
        Err(err) => panic!("nested color reprint {printed:?}: {err}"),
    };
    assert_eq!(
        again.gold(),
        r#"(color #ff0000 (row (atom Ord "x") (atom Bin "+") (color #0000ff (atom Ord "y")) (atom Bin "+") (atom Ord "z")))"#,
        "{printed}"
    );
    assert_eq!(again.to_latex(), printed);
}

fn roundtrip_cases() -> u32 {
    if let Ok(raw) = std::env::var("LATEX_RUST_ROUNDTRIP_CASES") {
        if let Ok(n) = raw.parse::<u32>() {
            if n > 0 {
                return n;
            }
        }
    }
    if cfg!(debug_assertions) {
        2_000
    } else {
        50_000
    }
}

fn source_grammar() -> impl Strategy<Value = String> {
    let leaf = prop_oneof![
        Just("x".to_string()),
        Just("y".to_string()),
        Just("2".to_string()),
        Just("+".to_string()),
        Just("=".to_string()),
        Just(r"\alpha".to_string()),
        Just(r"\sin".to_string()),
        Just(r"\leq".to_string()),
        Just(r"\infty".to_string()),
        Just(r"\sum".to_string()),
        Just("f'".to_string()),
        Just("f'''".to_string()),
        Just(r"\frac{\overline{x}}{2}".to_string()),
        Just("(-b)^{n}".to_string()),
        Just(r"\begin{pmatrix}a&b\\c&d\end{pmatrix}".to_string()),
        Just(r"\begin{cases}1&x<0\\0&\text{otherwise}\end{cases}".to_string()),
        Just(r"{\color{red} x + {\color{blue} y} + z}".to_string()),
        Just("α".to_string()),
        Just("≤".to_string()),
        Just("√x".to_string()),
        Just("x²".to_string()),
        Just("sin(x)".to_string()),
        Just("<=".to_string()),
        Just("−b".to_string()),
    ];
    leaf.prop_recursive(3, 24, 6, |inner| {
        prop_oneof![
            inner.clone().prop_map(|e| format!(r"\frac{{{e}}}{{2}}")),
            inner.clone().prop_map(|e| format!(r"\sqrt{{{e}}}")),
            inner.clone().prop_map(|e| format!(r"\overline{{{e}}}")),
            inner.clone().prop_map(|e| format!("{{{e}}}^{{n}}")),
            inner.clone().prop_map(|e| format!("{{{e}}}_{{i}}")),
            inner.clone().prop_map(|e| format!("({e})^{{2}}")),
            inner.clone().prop_map(|e| format!("{e}'")),
            (inner.clone(), inner.clone())
                .prop_map(|(a, b)| format!(r"\frac{{\overline{{{a}}}}}{{{b}}}")),
            (inner.clone(), inner.clone())
                .prop_map(|(a, b)| { format!(r"\begin{{matrix}}{a}&{b}\\{a}&{b}\end{{matrix}}") }),
            (inner.clone(), inner.clone())
                .prop_map(|(a, b)| { format!(r"\begin{{cases}}{a}&{b}\\{b}&{a}\end{{cases}}") }),
            (inner.clone(), inner.clone()).prop_map(|(a, b)| {
                format!(r"{{ \color{{red}} {a} + {{ \color{{blue}} {b} }} }}")
            }),
            (inner.clone(), inner.clone()).prop_map(|(a, b)| format!("{a}+{b}")),
        ]
    })
}

fn options_from_bits(bits: u8) -> ParseOptions {
    ParseOptions::new()
        .with_ascii_relations(bits & 1 != 0)
        .with_bare_operators(bits & 2 != 0)
        .with_unicode_symbols(bits & 4 != 0)
        .with_prime_symbol(bits & 8 != 0)
}

proptest! {
    #![proptest_config({
        let mut config = ProptestConfig::with_cases(roundtrip_cases());
        config.failure_persistence = Some(Box::new(FileFailurePersistence::Off));
        config
    })]
    #[test]
    fn generated_trees_round_trip(src in source_grammar(), bits in 0u8..16) {
        let options = options_from_bits(bits);
        let Ok((node, _)) = parse_with_options(&src, &options) else {
            return Ok(());
        };
        let printed = node.to_latex();
        let again = match parse_with_options(&printed, &options) {
            Ok((node, _)) => node,
            Err(err) => panic!("reprint of {src:?} as {printed:?}: {err}"),
        };
        prop_assert_eq!(&again, &node, "src {:?} printed {:?}", src, printed);
        prop_assert_eq!(again.to_latex(), printed);
    }
}

#[test]
fn case_count_follows_the_tier() {
    let cases = roundtrip_cases();
    let floor = if cfg!(debug_assertions) {
        2_000
    } else {
        50_000
    };
    if std::env::var("LATEX_RUST_ROUNDTRIP_CASES").is_err() {
        assert_eq!(cases, floor);
    }
}
