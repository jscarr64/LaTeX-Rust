//! Subprocess harness for parser depth.
//!
//! An abort never returns to `thread::join`, so each case is a child
//! process. The child parses on an 8 MiB stack. That is the audit host's
//! main-thread stack (`ulimit -s` = 8192). In a release build, 3,500 brace
//! groups with `with_max_depth(20_000)` return `Ok`. 3,531 and deeper return
//! `Err(Malformed)`. A debug build uses more stack per frame, so the debug
//! run checks 1,500 instead of 3,500. Libtest's worker thread is 2 MiB,
//! so the child moves the parse onto an 8 MiB thread before it calls
//! `parse_with_options`.
//!
//! `raised_limit_must_not_abort` is part of the default run. 3,500 braces
//! at limit 20,000 stay `Ok` in a release build. 3,531 and deeper return
//! `Err(Malformed)` for braces, `\left`, and `\frac`. An abort is a failure.

use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use latex_rust::{parse, parse_with_options, ParseError, ParseOptions};

const CHILD_OK: i32 = 17;
const AUDIT_STACK: usize = 8 << 20;

fn shape(name: &str, n: usize) -> String {
    match name {
        "braces" => "{".repeat(n) + "x" + &"}".repeat(n),
        "frac" => "\\frac{1}{".repeat(n) + "2" + &"}".repeat(n),
        "sqrt" => "\\sqrt{".repeat(n) + "x" + &"}".repeat(n),
        "sup" => "x^{".repeat(n) + "y" + &"}".repeat(n),
        "left" => "\\left(".repeat(n) + "x" + &"\\right)".repeat(n),
        "mathrm" => "\\mathrm{".repeat(n) + "x" + &"}".repeat(n),
        other => panic!("unknown shape {other}"),
    }
}

fn outcome(name: &str, n: usize, limit: &str) -> Result<bool, String> {
    let input = shape(name, n);
    let parsed = if limit == "default" {
        parse(&input).map(|_| ())
    } else {
        let depth: usize = match limit.parse() {
            Ok(depth) => depth,
            Err(err) => return Err(err.to_string()),
        };
        let opts = ParseOptions::new().with_max_depth(depth);
        parse_with_options(&input, &opts).map(|_| ())
    };
    match parsed {
        Ok(()) => Ok(true),
        Err(ParseError::Malformed(_)) => Ok(false),
        Err(err) => Err(format!("expected Malformed, got {err}")),
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
    let handle = match std::thread::Builder::new()
        .name("audit-stack".into())
        .stack_size(AUDIT_STACK)
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

fn run_case(shape_name: &str, n: usize, limit: &str, expect: &str) {
    let mut child = match Command::new(exe())
        .arg("depth_child")
        .arg("--exact")
        .env("LATEX_RUST_DEPTH_CHILD", "1")
        .env("LATEX_RUST_DEPTH_SHAPE", shape_name)
        .env("LATEX_RUST_DEPTH_N", n.to_string())
        .env("LATEX_RUST_DEPTH_LIMIT", limit)
        .env("LATEX_RUST_DEPTH_EXPECT", expect)
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
            Ok(None) if start.elapsed() >= Duration::from_secs(10) => {
                if let Err(err) = child.kill() {
                    panic!("{shape_name}@{n}: kill after 10s: {err}");
                }
                let _ = child.wait();
                panic!("{shape_name}@{n} limit={limit}: still running after 10s");
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
        "{shape_name}@{n} limit={limit} expect={expect}: {}\n{err_text}",
        status_word(&status)
    );
}

fn status_word(status: &ExitStatus) -> String {
    match status.code() {
        Some(code) => format!("exit {code}"),
        None => "killed by signal (abort or stack overflow)".to_string(),
    }
}

#[test]
fn default_limit_rejects_deep_input() {
    for name in ["braces", "frac", "sqrt", "sup", "left", "mathrm"] {
        run_case(name, 100_000, "default", "err");
    }
}

#[test]
fn raised_limit_parses_deep_braces() {
    // Under the debug abort (about 1,850) and under the release abort (3,531).
    run_case("braces", 1_500, "20000", "ok");
    if !cfg!(debug_assertions) {
        run_case("braces", 3_500, "20000", "ok");
    }
}

#[test]
fn default_limit_rejects_3531_braces() {
    run_case("braces", 3_531, "default", "err");
}

/// 3,531 and deeper nests at `with_max_depth(20_000)` must be
/// `Err(Malformed)`, not a stack abort. 3,500 braces staying `Ok` is
/// `raised_limit_parses_deep_braces`.
#[test]
fn raised_limit_must_not_abort() {
    for n in [3_531_usize, 10_000] {
        run_case("braces", n, "20000", "err");
        run_case("left", n, "20000", "err");
        run_case("frac", n, "20000", "err");
    }
}
