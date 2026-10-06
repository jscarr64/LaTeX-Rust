//! Every dvipsnames CMYK tuple against `drivers.dtx`.
//!
//! The tuples below are transcribed from the `%<*dvipsnames>` section of
//! `drivers.dtx` in the LaTeX graphics bundle (the section that generates
//! `dvipsnam.def`). Upstream order is the dvips color-wheel order.
//! `data/dvipsnames.tsv` stores the same tuples sorted by name.
//!
//! Source checked 2026-10-06:
//! <https://github.com/latex3/latex2e/blob/develop/required/graphics/drivers.dtx>
//!
//! Copyright 1994 David Carlisle and Sebastian Rahtz; 1995-1999 David
//! Carlisle; 2000-2026 The LaTeX Project. LPPL-1.3c or later.

use latex_rust::{named_color, parse_color_spec, Dim};

const TABLE: &str = include_str!("../data/dvipsnames.tsv");

/// `(name, c, m, y, k)` as written in `drivers.dtx`.
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

#[test]
fn dvipsnames_match_drivers_dtx() {
    assert_eq!(UPSTREAM.len(), 68, "drivers.dtx dvipsnames set");

    let table = parse_table(TABLE);
    assert_eq!(table.len(), UPSTREAM.len());

    let names: Vec<&str> = table.iter().map(|(name, _)| name.as_str()).collect();
    let mut alphabetical = names.clone();
    alphabetical.sort_unstable();
    assert_eq!(
        names, alphabetical,
        "dvipsnames.tsv rows stay alphabetically sorted"
    );

    for (name, c, m, y, k) in UPSTREAM {
        let row = table
            .iter()
            .find(|(n, _)| n == name)
            .unwrap_or_else(|| panic!("missing {name}"));
        let upstream = [Dim::parse(c), Dim::parse(m), Dim::parse(y), Dim::parse(k)];
        for channel in &upstream {
            assert!(!channel.is_nan(), "{name}");
        }
        assert_eq!(row.1, upstream, "{name} CMYK");

        let spec = format!("{c},{m},{y},{k}");
        let from_upstream = parse_color_spec("cmyk", &spec, None).unwrap_or_else(|err| {
            panic!("{name}: {err}");
        });
        let loaded = named_color(name).unwrap_or_else(|err| panic!("{name}: {err}"));
        assert_eq!(loaded, from_upstream, "{name} sRGB");
    }
}

fn parse_table(src: &str) -> Vec<(String, [Dim; 4])> {
    let mut lines = src.lines().filter(|line| {
        let trimmed = line.trim();
        !trimmed.is_empty() && !trimmed.starts_with('#')
    });
    let header = lines.next().expect("dvipsnames header");
    assert_eq!(header, "name\tc\tm\ty\tk");

    let mut out = Vec::new();
    for line in lines {
        let mut cols = line.split('\t');
        let name = cols.next().expect("name").to_string();
        let channels = [
            dim_channel(&name, cols.next()),
            dim_channel(&name, cols.next()),
            dim_channel(&name, cols.next()),
            dim_channel(&name, cols.next()),
        ];
        assert!(cols.next().is_none(), "{name} has extra columns");
        out.push((name, channels));
    }
    out
}

fn dim_channel(name: &str, text: Option<&str>) -> Dim {
    let text = text.unwrap_or_else(|| panic!("{name} missing channel"));
    let dim = Dim::parse(text);
    assert!(!dim.is_nan(), "{name} channel {text}");
    dim
}
