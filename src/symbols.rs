//! Math-mode symbol catalog from `data/symbols.tsv`.
//!
//! Source tables live in `documents/`. Flutter UI columns are not loaded.
//! Duplicate `\sqrt{}` rows were collapsed to one entry.

use std::sync::OnceLock;

const TSV: &str = include_str!("../data/symbols.tsv");

/// How a catalog entry is used on a math keyboard / in the parser.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolKind {
    /// Single glyph (`\alpha`, `\times`).
    Symbol,
    /// Operator (`\sum`, `\int`, `+`).
    Operator,
    /// Structure that takes a body (`\frac`, `\sqrt`, matrices).
    Container,
    /// Accent or modifier (`\hat`, `\vec`).
    Modifier,
}

impl SymbolKind {
    fn parse(s: &str) -> Option<Self> {
        match s {
            "Symbol" => Some(Self::Symbol),
            "Operator" => Some(Self::Operator),
            "Container" => Some(Self::Container),
            "Modifier" => Some(Self::Modifier),
            _ => None,
        }
    }

    /// Catalog spelling.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Symbol => "Symbol",
            Self::Operator => "Operator",
            Self::Container => "Container",
            Self::Modifier => "Modifier",
        }
    }
}

/// One row of the shipped symbol table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SymbolEntry {
    /// Typical rendered character (may be a placeholder for containers).
    pub glyph: &'static str,
    /// High-level group (`Greek`, `Calculus`, …).
    pub category: &'static str,
    /// Keyboard / parser kind.
    pub kind: SymbolKind,
    /// Raw LaTeX from the table (`\alpha`, `\frac{}{}`, `+`).
    pub latex: &'static str,
    /// Human description from the table.
    pub description: &'static str,
}

impl SymbolEntry {
    /// Control-sequence name without `\`, or the raw character for `+` / `=`.
    #[must_use]
    pub fn command_name(&self) -> &str {
        command_name(self.latex)
    }
}

fn command_name(latex: &str) -> &str {
    let t = latex.trim();
    if let Some(rest) = t.strip_prefix('\\') {
        let n = rest.chars().take_while(|c| c.is_ascii_alphabetic()).count();
        if n > 0 {
            return &rest[..n];
        }
        if let Some(c) = rest.chars().next() {
            let end = c.len_utf8();
            return &rest[..end];
        }
    }
    t
}

fn parse_tsv(text: &'static str) -> Vec<SymbolEntry> {
    let mut rows = Vec::new();
    let mut lines = text.lines();
    let header = lines.next().expect("symbols.tsv header");
    assert_eq!(
        header, "glyph\tcategory\tkind\tlatex\tdescription",
        "symbols.tsv schema"
    );
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let mut cols = line.split('\t');
        let glyph = cols.next().expect("glyph");
        let category = cols.next().expect("category");
        let kind_s = cols.next().expect("kind");
        let latex = cols.next().expect("latex");
        let description = cols.next().unwrap_or("");
        let kind = SymbolKind::parse(kind_s).expect("symbol kind");
        rows.push(SymbolEntry {
            glyph,
            category,
            kind,
            latex,
            description,
        });
    }
    rows
}

fn catalog() -> &'static [SymbolEntry] {
    static CAT: OnceLock<Vec<SymbolEntry>> = OnceLock::new();
    CAT.get_or_init(|| parse_tsv(TSV)).as_slice()
}

/// All shipped symbols, in table order.
#[must_use]
pub fn symbols() -> &'static [SymbolEntry] {
    catalog()
}

/// Look up by raw table LaTeX (`\alpha`, `\frac{}{}`) or command name (`alpha`).
#[must_use]
pub fn lookup(query: &str) -> Option<&'static SymbolEntry> {
    let q = query.trim();
    let q_name = command_name(q);
    catalog()
        .iter()
        .find(|e| e.latex == q || e.command_name() == q || e.command_name() == q_name)
}

/// Number of entries in a category.
#[must_use]
pub fn category_count(category: &str) -> usize {
    catalog().iter().filter(|e| e.category == category).count()
}
