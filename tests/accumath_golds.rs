//! Runner for `golds/accumath.toml`.
//!
//! `expect` is the tree the release must build. `forbid`, when set, is a
//! tree the runner must not accept. That stops a gold edit that copies the
//! old output into `expect`.

use latex_rust::parse;

struct Rec {
    name: String,
    kind: String,
    input: String,
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
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/golds/accumath.toml");
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) => panic!("accumath.toml: {err}"),
    };
    let mut recs = Vec::new();
    let mut rec = Rec {
        name: String::new(),
        kind: String::new(),
        input: String::new(),
        expect: String::new(),
        forbid: None,
    };
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
            rec = Rec {
                name: String::new(),
                kind: String::new(),
                input: String::new(),
                expect: String::new(),
                forbid: None,
            };
            seen = true;
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let k = k.trim();
        let v = parse_value(v);
        match k {
            "name" => rec.name = v,
            "kind" => rec.kind = v,
            "input" => rec.input = v,
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

#[test]
fn accumath_golds() {
    let recs = load_golds();
    assert!(!recs.is_empty(), "no accumath golds loaded");
    let mut failed = Vec::new();
    for rec in recs {
        if rec.kind != "ast" {
            failed.push(format!("{}: kind {} is not ast", rec.name, rec.kind));
            continue;
        }
        let got = match parse(&rec.input) {
            Ok(node) => node.gold(),
            Err(err) => {
                failed.push(format!("{}: {err}", rec.name));
                continue;
            }
        };
        if rec.forbid.as_ref().is_some_and(|bad| bad == &got) {
            failed.push(format!(
                "{}: still the 2.1.1 tree\n  input: {}\n  got: {got}",
                rec.name, rec.input
            ));
        }
        if got != rec.expect {
            failed.push(format!(
                "{}:\n  input: {}\n  expect: {}\n  got:    {got}",
                rec.name, rec.input, rec.expect
            ));
        }
    }
    assert!(failed.is_empty(), "\n{}", failed.join("\n"));
}
