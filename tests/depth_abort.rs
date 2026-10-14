//! Subprocess harness for parser depth.
//!
//! Acceptance depends only on the input and `max_depth`. Each case runs on an
//! 8 MiB stack, a 1 MiB stack, and a 256 KiB stack. The same assertions hold
//! in debug and release: Pop runs this file both ways. Within the limit the
//! parse is `Ok`. Past the limit it is `Err(Malformed)` naming that limit.
//! An abort is a failure. `Ok` and `Malformed` are never interchangeable.

use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use latex_rust::{parse, parse_with_options, ParseError, ParseOptions, DEFAULT_MAX_NESTING_DEPTH};

const CHILD_OK: i32 = 17;
const STACKS: [usize; 3] = [8 << 20, 1 << 20, 256 << 10];

fn shape(name: &str, n: usize) -> String {
    match name {
        "braces" => "{".repeat(n) + "x" + &"}".repeat(n),
        "frac" => "\\frac{1}{".repeat(n) + "2" + &"}".repeat(n),
        "sqrt" => "\\sqrt{".repeat(n) + "x" + &"}".repeat(n),
        "sup" => "x^{".repeat(n) + "y" + &"}".repeat(n),
        "left" => "\\left(".repeat(n) + "x" + &"\\right)".repeat(n),
        "mathrm" => "\\mathrm{".repeat(n) + "x" + &"}".repeat(n),
        "styled_frac" => "\\mathrm{".to_string() + &shape("frac", n) + "}",
        other => panic!("unknown shape {other}"),
    }
}

fn limit_n(limit: &str) -> Result<usize, String> {
    if limit == "default" {
        Ok(DEFAULT_MAX_NESTING_DEPTH)
    } else {
        limit
            .parse()
            .map_err(|err: std::num::ParseIntError| err.to_string())
    }
}

fn outcome(name: &str, n: usize, limit: &str) -> Result<bool, String> {
    let input = shape(name, n);
    let depth = limit_n(limit)?;
    let parsed = if limit == "default" {
        parse(&input).map(|_| ())
    } else {
        let opts = ParseOptions::new().with_max_depth(depth);
        parse_with_options(&input, &opts).map(|_| ())
    };
    let needle = format!("input nests deeper than {depth} levels");
    match parsed {
        Ok(_) => Ok(true),
        Err(ParseError::Malformed(msg)) if msg == needle => Ok(false),
        Err(err) => Err(format!("unexpected {err}")),
    }
}

fn child_code() -> i32 {
    let name = match std::env::var("LATEX_RUST_DEPTH_SHAPE") {
        Ok(name) => name,
        Err(err) => {
            eprintln!("shape: {err}");
            return 3;
        }
    };
    let n = match std::env::var("LATEX_RUST_DEPTH_N")
        .ok()
        .and_then(|s| s.parse().ok())
    {
        Some(n) => n,
        None => {
            eprintln!("missing n");
            return 3;
        }
    };
    let limit = match std::env::var("LATEX_RUST_DEPTH_LIMIT") {
        Ok(limit) => limit,
        Err(err) => {
            eprintln!("limit: {err}");
            return 3;
        }
    };
    let expect_ok = match std::env::var("LATEX_RUST_DEPTH_EXPECT") {
        Ok(s) if s == "ok" => true,
        Ok(s) if s == "err" => false,
        Ok(s) => {
            eprintln!("expect {s}");
            return 3;
        }
        Err(err) => {
            eprintln!("expect: {err}");
            return 3;
        }
    };
    let stack = match std::env::var("LATEX_RUST_DEPTH_STACK") {
        Ok(s) => match s.parse() {
            Ok(stack) => stack,
            Err(err) => {
                eprintln!("stack: {err}");
                return 3;
            }
        },
        Err(_) => STACKS[0],
    };
    let handle = match std::thread::Builder::new()
        .name("audit-stack".into())
        .stack_size(stack)
        .spawn(move || outcome(&name, n, &limit))
    {
        Ok(handle) => handle,
        Err(err) => {
            eprintln!("spawn stack thread: {err}");
            return 3;
        }
    };
    match handle.join() {
        Ok(Ok(is_ok)) if is_ok == expect_ok => CHILD_OK,
        Ok(Ok(is_ok)) => {
            eprintln!("parse ok={is_ok}, expected ok={expect_ok}");
            2
        }
        Ok(Err(err)) => {
            eprintln!("{err}");
            3
        }
        Err(_) => {
            eprintln!("parse thread panicked");
            4
        }
    }
}

#[test]
fn depth_child() {
    if std::env::var_os("LATEX_RUST_DEPTH_CHILD").is_none() {
        return;
    }
    std::process::exit(child_code());
}

fn exe() -> std::path::PathBuf {
    match std::env::current_exe() {
        Ok(path) => path,
        Err(err) => panic!("current_exe: {err}"),
    }
}

fn run_case_on(shape_name: &str, n: usize, limit: &str, expect: &str, stack: usize) {
    let mut child = match Command::new(exe())
        .arg("depth_child")
        .arg("--exact")
        .env("LATEX_RUST_DEPTH_CHILD", "1")
        .env("LATEX_RUST_DEPTH_SHAPE", shape_name)
        .env("LATEX_RUST_DEPTH_N", n.to_string())
        .env("LATEX_RUST_DEPTH_LIMIT", limit)
        .env("LATEX_RUST_DEPTH_EXPECT", expect)
        .env("LATEX_RUST_DEPTH_STACK", stack.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(err) => panic!("{shape_name}@{n}: spawn: {err}"),
    };
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if start.elapsed() >= Duration::from_secs(30) => {
                if let Err(err) = child.kill() {
                    panic!("{shape_name}@{n}: kill after 30s: {err}");
                }
                let _ = child.wait();
                panic!("{shape_name}@{n} limit={limit}: still running after 30s");
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
            Err(err) => panic!("{shape_name}@{n}: wait: {err}"),
        }
    };
    let mut err_text = String::new();
    if let Some(mut stderr) = child.stderr.take() {
        let _ = std::io::Read::read_to_string(&mut stderr, &mut err_text);
    }
    assert!(
        status.code() == Some(CHILD_OK),
        "{shape_name}@{n} limit={limit} stack={stack} expect={expect}: {}\n{err_text}",
        status_word(&status)
    );
}

fn status_word(status: &ExitStatus) -> String {
    match status.code() {
        Some(code) => format!("exit {code}"),
        None => "killed by signal (abort or stack overflow)".to_string(),
    }
}

fn on_every_stack(shape_name: &str, n: usize, limit: &str, expect: &str) {
    for stack in STACKS {
        run_case_on(shape_name, n, limit, expect, stack);
    }
}

#[test]
fn default_limit_rejects_deep_input() {
    for name in ["braces", "frac", "sqrt", "sup", "left", "mathrm"] {
        on_every_stack(name, 100_000, "default", "err");
    }
}

/// 3,500 and 3,531 brace groups are under a limit of 20,000, so both are `Ok`.
#[test]
fn raised_limit_parses_deep_braces() {
    for n in [3_500_usize, 3_531] {
        on_every_stack("braces", n, "20000", "ok");
    }
}

#[test]
fn default_limit_rejects_3531_braces() {
    on_every_stack("braces", 3_531, "default", "err");
}

/// Inside `with_max_depth(20_000)` the parse is `Ok`. One level past it is
/// `Malformed` naming 20,000. A brace, `\left`, `\frac`, `\sqrt`, a
/// superscript and `\mathrm` each cost one level: a braced command argument
/// costs no more than a bare one (spec 5, Q2).
#[test]
fn raised_limit_must_not_abort() {
    for n in [10_000_usize, 19_999] {
        for name in ["braces", "left", "frac", "sqrt", "sup", "mathrm"] {
            on_every_stack(name, n, "20000", "ok");
        }
    }
    on_every_stack("left", 3_531, "20000", "ok");
    // `\mathrm{\frac…}` starts one level down, so 19,998 still fits and 19,999 does not.
    on_every_stack("styled_frac", 3_531, "20000", "ok");
    on_every_stack("styled_frac", 19_998, "20000", "ok");
    for name in ["braces", "left", "frac", "sqrt", "sup", "mathrm"] {
        on_every_stack(name, 20_000, "20000", "err");
    }
    on_every_stack("styled_frac", 19_999, "20000", "err");
}
