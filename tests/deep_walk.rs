//! Gold, reprint, layout, and SVG of a tree the parser accepts.
//!
//! Each case parses at `max_depth` 20_000, then runs `gold`, `to_latex`,
//! layout, and SVG on an 8 MiB stack, a 1 MiB stack, and a 256 KiB stack.
//! The same strings and the same success hold in debug and release. An
//! abort is a failure.

use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use latex_rust::{
    layout_with_max_depth, parse_with_options, render_svg, MathFont, MathStyle, ParseOptions,
    SvgOptions,
};

const CHILD_OK: i32 = 17;
const STACKS: [usize; 3] = [8 << 20, 1 << 20, 256 << 10];
const LIMIT: usize = 20_000;

fn shape(name: &str, n: usize) -> String {
    match name {
        "braces" => "{".repeat(n) + "x" + &"}".repeat(n),
        "frac" => "\\frac{1}{".repeat(n) + "2" + &"}".repeat(n),
        other => panic!("unknown shape {other}"),
    }
}

fn expect_frac_gold(n: usize) -> String {
    let mut s = String::new();
    for _ in 0..n {
        s.push_str("(frac (atom Ord \"1\") ");
    }
    s.push_str("(atom Ord \"2\")");
    for _ in 0..n {
        s.push(')');
    }
    s
}

fn expect_frac_latex(n: usize) -> String {
    let mut s = String::new();
    for _ in 0..n {
        s.push_str("\\frac {1}{");
    }
    s.push('2');
    for _ in 0..n {
        s.push('}');
    }
    s
}

fn walk(name: &str, n: usize) -> Result<(), String> {
    let input = shape(name, n);
    let opts = ParseOptions::new().with_max_depth(LIMIT);
    let (ast, _) = parse_with_options(&input, &opts).map_err(|err| format!("parse {err}"))?;
    let gold = ast.gold();
    let printed = ast.to_latex();
    match name {
        "braces" => {
            if gold != "(atom Ord \"x\")" {
                return Err(format!("braces gold {gold}"));
            }
            if printed != "x" {
                return Err(format!("braces latex {printed}"));
            }
        }
        "frac" => {
            let want_gold = expect_frac_gold(n);
            if gold != want_gold {
                return Err(format!(
                    "frac gold len {} want {}",
                    gold.len(),
                    want_gold.len()
                ));
            }
            let want_latex = expect_frac_latex(n);
            if printed != want_latex {
                return Err(format!(
                    "frac latex len {} want {}",
                    printed.len(),
                    want_latex.len()
                ));
            }
        }
        other => return Err(format!("unknown shape {other}")),
    }
    let font = MathFont::stix_two_math().map_err(|err| format!("font {err}"))?;
    let boxed = layout_with_max_depth(&ast, &font, MathStyle::Text, LIMIT)
        .map_err(|err| format!("layout {err}"))?;
    if boxed.dim_gold().is_empty() {
        return Err("empty dim gold".into());
    }
    let svg = render_svg(&boxed, &font, &SvgOptions::new()).map_err(|err| format!("svg {err}"))?;
    if !svg.contains("<svg") || !svg.contains("</svg>") {
        return Err("svg missing document".into());
    }
    Ok(())
}

fn child_code() -> i32 {
    let name = match std::env::var("LATEX_RUST_DEEP_SHAPE") {
        Ok(name) => name,
        Err(err) => {
            eprintln!("shape: {err}");
            return 3;
        }
    };
    let n = match std::env::var("LATEX_RUST_DEEP_N")
        .ok()
        .and_then(|s| s.parse().ok())
    {
        Some(n) => n,
        None => {
            eprintln!("missing n");
            return 3;
        }
    };
    let stack = match std::env::var("LATEX_RUST_DEEP_STACK") {
        Ok(s) => match s.parse::<usize>() {
            Ok(stack) => stack,
            Err(err) => {
                eprintln!("stack: {err}");
                return 3;
            }
        },
        Err(err) => {
            eprintln!("stack: {err}");
            return 3;
        }
    };
    let handle = match std::thread::Builder::new()
        .name("deep-walk".into())
        .stack_size(stack)
        .spawn(move || walk(&name, n))
    {
        Ok(handle) => handle,
        Err(err) => {
            eprintln!("spawn: {err}");
            return 3;
        }
    };
    match handle.join() {
        Ok(Ok(())) => CHILD_OK,
        Ok(Err(err)) => {
            eprintln!("{err}");
            2
        }
        Err(_) => {
            eprintln!("walk thread panicked");
            4
        }
    }
}

#[test]
fn deep_walk_child() {
    if std::env::var_os("LATEX_RUST_DEEP_CHILD").is_none() {
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

fn status_word(status: &ExitStatus) -> String {
    match status.code() {
        Some(code) => format!("exit {code}"),
        None => "killed by signal (abort or stack overflow)".to_string(),
    }
}

fn run_case_on(shape_name: &str, n: usize, stack: usize) {
    let mut child = match Command::new(exe())
        .arg("deep_walk_child")
        .arg("--exact")
        .env("LATEX_RUST_DEEP_CHILD", "1")
        .env("LATEX_RUST_DEEP_SHAPE", shape_name)
        .env("LATEX_RUST_DEEP_N", n.to_string())
        .env("LATEX_RUST_DEEP_STACK", stack.to_string())
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
            Ok(None) if start.elapsed() >= Duration::from_secs(180) => {
                if let Err(err) = child.kill() {
                    panic!("{shape_name}@{n}: kill after 180s: {err}");
                }
                let _ = child.wait();
                panic!("{shape_name}@{n} stack={stack}: still running after 180s");
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
        "{shape_name}@{n} stack={stack}: {}\n{err_text}",
        status_word(&status)
    );
}

#[test]
fn accepted_trees_walk_on_every_stack() {
    for stack in STACKS {
        run_case_on("braces", 3_531, stack);
        run_case_on("frac", 9_999, stack);
    }
}
