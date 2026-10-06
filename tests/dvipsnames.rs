//! CMYK tuples in `data/dvipsnames.tsv` must match `dvipsnam.def`.
//!
//! Expected values are transcribed from the `%<*dvipsnames>` section of
//! `drivers.dtx` 2026-06-26 v3.0m (LaTeX graphics bundle):
//! <https://github.com/latex3/latex2e/blob/develop/required/graphics/drivers.dtx>
//! Copyright 1994 David Carlisle and Sebastian Rahtz; 1995-1999 David Carlisle;
//! 2000-2026 The LaTeX Project. LPPL-1.3c or later.

use std::collections::BTreeMap;

use latex_rust::{named_color, parse_color_spec};

/// Upstream order, as written in `drivers.dtx`. Each tuple is the `{c,m,y,k}` text.
const UPSTREAM: &[(&str, &str, &str, &str, &str)] = &[
    ("GreenYellow", "0.15", "0", "0.69", "0"),
    ("Yellow", "0", "0", "1", "0"),
    ("Goldenrod", "0", "0.10", "0.84", "0"),
    ("Dandelion", "0", "0.29", "0.84", "0"),
    ("Apricot", "0", "0.32", "0.52", "0"),
    ("Peach", "0", "0.50", "0.70", "0"),
    ("Melon", "0", "0.46", "0.50", "0"),
    ("YellowOrange", "0", "0.42", "1", "0"),
    ("Orange", "0", "0.61", "0.87", "0"),
    ("BurntOrange", "0", "0.51", "1", "0"),
    ("Bittersweet", "0", "0.75", "1", "0.24"),
    ("RedOrange", "0", "0.77", "0.87", "0"),
    ("Mahogany", "0", "0.85", "0.87", "0.35"),
    ("Maroon", "0", "0.87", "0.68", "0.32"),
    ("BrickRed", "0", "0.89", "0.94", "0.28"),
    ("Red", "0", "1", "1", "0"),
    ("OrangeRed", "0", "1", "0.50", "0"),
    ("RubineRed", "0", "1", "0.13", "0"),
    ("WildStrawberry", "0", "0.96", "0.39", "0"),
    ("Salmon", "0", "0.53", "0.38", "0"),
    ("CarnationPink", "0", "0.63", "0", "0"),
    ("Magenta", "0", "1", "0", "0"),
    ("VioletRed", "0", "0.81", "0", "0"),
    ("Rhodamine", "0", "0.82", "0", "0"),
    ("Mulberry", "0.34", "0.90", "0", "0.02"),
    ("RedViolet", "0.07", "0.90", "0", "0.34"),
    ("Fuchsia", "0.47", "0.91", "0", "0.08"),
    ("Lavender", "0", "0.48", "0", "0"),
    ("Thistle", "0.12", "0.59", "0", "0"),
    ("Orchid", "0.32", "0.64", "0", "0"),
    ("DarkOrchid", "0.40", "0.80", "0.20", "0"),
    ("Purple", "0.45", "0.86", "0", "0"),
    ("Plum", "0.50", "1", "0", "0"),
    ("Violet", "0.79", "0.88", "0", "0"),
    ("RoyalPurple", "0.75", "0.90", "0", "0"),
    ("BlueViolet", "0.86", "0.91", "0", "0.04"),
    ("Periwinkle", "0.57", "0.55", "0", "0"),
    ("CadetBlue", "0.62", "0.57", "0.23", "0"),
    ("CornflowerBlue", "0.65", "0.13", "0", "0"),
    ("MidnightBlue", "0.98", "0.13", "0", "0.43"),
    ("NavyBlue", "0.94", "0.54", "0", "0"),
    ("RoyalBlue", "1", "0.50", "0", "0"),
    ("Blue", "1", "1", "0", "0"),
    ("Cerulean", "0.94", "0.11", "0", "0"),
    ("Cyan", "1", "0", "0", "0"),
    ("ProcessBlue", "0.96", "0", "0", "0"),
    ("SkyBlue", "0.62", "0", "0.12", "0"),
    ("Turquoise", "0.85", "0", "0.20", "0"),
    ("TealBlue", "0.86", "0", "0.34", "0.02"),
    ("Aquamarine", "0.82", "0", "0.30", "0"),
    ("BlueGreen", "0.85", "0", "0.33", "0"),
    ("Emerald", "1", "0", "0.50", "0"),
    ("JungleGreen", "0.99", "0", "0.52", "0"),
    ("SeaGreen", "0.69", "0", "0.50", "0"),
    ("Green", "1", "0", "1", "0"),
    ("ForestGreen", "0.91", "0", "0.88", "0.12"),
    ("PineGreen", "0.92", "0", "0.59", "0.25"),
    ("LimeGreen", "0.50", "0", "1", "0"),
    ("YellowGreen", "0.44", "0", "0.74", "0"),
    ("SpringGreen", "0.26", "0", "0.76", "0"),
    ("OliveGreen", "0.64", "0", "0.95", "0.40"),
    ("RawSienna", "0", "0.72", "1", "0.45"),
    ("Sepia", "0", "0.83", "1", "0.70"),
    ("Brown", "0", "0.81", "1", "0.60"),
    ("Tan", "0.14", "0.42", "0.56", "0"),
    ("Gray", "0", "0", "0", "0.50"),
    ("Black", "0", "0", "0", "1"),
    ("White", "0", "0", "0", "0"),
];

fn table_text() -> String {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/data/dvipsnames.tsv");
    std::fs::read_to_string(path).expect("read dvipsnames.tsv")
}

fn tsv_rows(text: &str) -> Vec<(&str, &str, &str, &str, &str)> {
    let mut rows = Vec::new();
    let mut saw_header = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if !saw_header {
            assert_eq!(line, "name\tc\tm\ty\tk", "column header");
            saw_header = true;
            continue;
        }
        let mut cols = line.split('\t');
        let name = cols.next().expect("name");
        let c = cols.next().expect("c");
        let m = cols.next().expect("m");
        let y = cols.next().expect("y");
        let k = cols.next().expect("k");
        assert!(cols.next().is_none(), "extra column in {line}");
        rows.push((name, c, m, y, k));
    }
    assert!(saw_header, "missing column header");
    rows
}

#[test]
fn every_tuple_matches_drivers_dtx() {
    let text = table_text();
    let rows = tsv_rows(&text);
    assert_eq!(rows.len(), UPSTREAM.len());
    assert_eq!(rows.len(), 68);

    let mut shipped: BTreeMap<&str, (&str, &str, &str, &str)> = BTreeMap::new();
    for (name, c, m, y, k) in rows.iter().copied() {
        let previous = shipped.insert(name, (c, m, y, k));
        assert!(previous.is_none(), "duplicate name {name}");
    }

    for &(name, c, m, y, k) in UPSTREAM {
        let got = shipped
            .remove(name)
            .unwrap_or_else(|| panic!("missing {name}"));
        assert_eq!(got, (c, m, y, k), "{name}");
        let spec = format!("{c},{m},{y},{k}");
        let from_name = named_color(name).unwrap_or_else(|err| panic!("{name}: {err}"));
        let from_cmyk =
            parse_color_spec("cmyk", &spec, None).unwrap_or_else(|err| panic!("{name}: {err}"));
        assert_eq!(from_name, from_cmyk, "{name}");
    }
    assert!(
        shipped.is_empty(),
        "names absent from drivers.dtx: {:?}",
        shipped.keys().collect::<Vec<_>>()
    );
}

#[test]
fn rows_are_re_sorted_alphabetically() {
    let text = table_text();
    let rows = tsv_rows(&text);
    let names: Vec<&str> = rows.iter().map(|row| row.0).collect();
    let mut sorted = names.clone();
    sorted.sort_unstable();
    assert_eq!(names, sorted);
    assert_ne!(names, UPSTREAM.iter().map(|row| row.0).collect::<Vec<_>>());
}

#[test]
fn header_identifies_the_modified_file() {
    let text = table_text();
    let header = text
        .lines()
        .take_while(|line| line.starts_with('#') || line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    for needle in [
        "dvipsnam.def",
        "drivers.dtx",
        "David Carlisle",
        "Sebastian Rahtz",
        "The LaTeX Project",
        "LaTeX Project Public License",
        "1.3c",
        "modified version",
        "alphabetically",
        "does not provide support",
    ] {
        assert!(
            header.contains(needle),
            "dvipsnames.tsv header missing {needle}"
        );
    }
}

#[test]
fn notice_identifies_the_modified_file() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/NOTICE");
    let text = std::fs::read_to_string(path).expect("read NOTICE");
    for needle in [
        "data/dvipsnames.tsv",
        "dvipsnam.def",
        "drivers.dtx",
        "David Carlisle",
        "Sebastian Rahtz",
        "The LaTeX Project",
        "1.3c",
        "alphabetically",
        "does not provide support",
        "https://www.latex-project.org/lppl.txt",
    ] {
        assert!(text.contains(needle), "NOTICE missing {needle}");
    }
}
