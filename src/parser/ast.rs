//! Typed math AST produced by the parser.

use crate::color::Color;
use crate::dim::Dim;

/// TeX math atom class (Appendix G).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AtomKind {
    /// Ordinary (`x`, `1`, `\alpha` as a letter-like glyph).
    Ord,
    /// Large operator class.
    Op,
    /// Binary operator (`+`, `\times`).
    Bin,
    /// Relation (`=`, `\leq`).
    Rel,
    /// Opening delimiter.
    Open,
    /// Closing delimiter.
    Close,
    /// Punctuation (`,`).
    Punct,
    /// Inner (fraction-like).
    Inner,
}

impl AtomKind {
    fn gold(self) -> &'static str {
        match self {
            Self::Ord => "Ord",
            Self::Op => "Op",
            Self::Bin => "Bin",
            Self::Rel => "Rel",
            Self::Open => "Open",
            Self::Close => "Close",
            Self::Punct => "Punct",
            Self::Inner => "Inner",
        }
    }
}

/// Accent or decoration applied to a nucleus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccentKind {
    /// `\hat`
    Hat,
    /// `\check`
    Check,
    /// `\breve`
    Breve,
    /// `\acute`
    Acute,
    /// `\grave`
    Grave,
    /// `\tilde`
    Tilde,
    /// `\bar`
    Bar,
    /// `\vec`
    Vec,
    /// `\dot`
    Dot,
    /// `\ddot`
    Ddot,
    /// `\dddot`
    Dddot,
    /// `\ddddot`
    Ddddot,
    /// `\widehat`
    WideHat,
    /// `\widetilde`
    WideTilde,
    /// `\overline`
    Overline,
    /// `\underline`
    Underline,
    /// `\overbrace`
    Overbrace,
    /// `\underbrace`
    Underbrace,
    /// `\overleftarrow`
    Overleftarrow,
    /// `\overrightarrow`
    Overrightarrow,
    /// `\overleftrightarrow`
    Overleftrightarrow,
    /// `\underleftarrow`
    Underleftarrow,
    /// `\underrightarrow`
    Underrightarrow,
    /// `\underleftrightarrow`
    Underleftrightarrow,
    /// `\cancel`
    Cancel,
    /// `\bcancel`
    BCancel,
    /// `\xcancel`
    XCancel,
    /// `\boxed`
    Boxed,
    /// `\mathring`
    Ring,
    /// `\not`
    Not,
}

impl AccentKind {
    pub(crate) fn gold(self) -> &'static str {
        match self {
            Self::Hat => "hat",
            Self::Check => "check",
            Self::Breve => "breve",
            Self::Acute => "acute",
            Self::Grave => "grave",
            Self::Tilde => "tilde",
            Self::Bar => "bar",
            Self::Vec => "vec",
            Self::Dot => "dot",
            Self::Ddot => "ddot",
            Self::Dddot => "dddot",
            Self::Ddddot => "ddddot",
            Self::WideHat => "widehat",
            Self::WideTilde => "widetilde",
            Self::Overline => "overline",
            Self::Underline => "underline",
            Self::Overbrace => "overbrace",
            Self::Underbrace => "underbrace",
            Self::Overleftarrow => "overleftarrow",
            Self::Overrightarrow => "overrightarrow",
            Self::Overleftrightarrow => "overleftrightarrow",
            Self::Underleftarrow => "underleftarrow",
            Self::Underrightarrow => "underrightarrow",
            Self::Underleftrightarrow => "underleftrightarrow",
            Self::Cancel => "cancel",
            Self::BCancel => "bcancel",
            Self::XCancel => "xcancel",
            Self::Boxed => "boxed",
            Self::Ring => "mathring",
            Self::Not => "not",
        }
    }
}

/// Font / text style for a run of characters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextStyle {
    /// `\mathrm`
    Rm,
    /// `\mathbf`
    Bf,
    /// `\mathit`
    It,
    /// `\mathsf`
    Sf,
    /// `\mathtt`
    Tt,
    /// `\mathbb`
    Bb,
    /// `\mathcal`
    Cal,
    /// `\mathfrak`
    Frak,
    /// `\mathscr`
    Scr,
    /// `\boldsymbol`
    Boldsymbol,
    /// `\pmb`
    Pmb,
    /// `\text`
    Text,
}

impl TextStyle {
    fn gold(self) -> &'static str {
        match self {
            Self::Rm => "rm",
            Self::Bf => "bf",
            Self::It => "it",
            Self::Sf => "sf",
            Self::Tt => "tt",
            Self::Bb => "bb",
            Self::Cal => "cal",
            Self::Frak => "frak",
            Self::Scr => "scr",
            Self::Boldsymbol => "boldsymbol",
            Self::Pmb => "pmb",
            Self::Text => "text",
        }
    }
}

/// Horizontal skip.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpaceKind {
    /// `\,`
    Thin,
    /// `\:` or `\>`
    Medium,
    /// `\;`
    Thick,
    /// `\!`
    NegThin,
    /// `\quad`
    Quad,
    /// `\qquad`
    Qquad,
    /// `\ ` (control space)
    ControlSpace,
    /// `\hspace{...}` in em.
    Hspace(Dim),
}

/// Matrix / alignment environment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatrixStyle {
    /// `{matrix}`
    Matrix,
    /// `{pmatrix}`
    Pmatrix,
    /// `{bmatrix}`
    Bmatrix,
    /// `{vmatrix}`
    Vmatrix,
    /// `{Vmatrix}`
    VVmatrix,
    /// `{Bmatrix}`
    BBmatrix,
    /// `{cases}`
    Cases,
    /// `{array}`
    Array,
    /// `{aligned}`
    Aligned,
    /// `{align}`
    Align,
    /// `{gather}`
    Gather,
    /// `{multline}`
    Multline,
    /// `{equation}`
    Equation,
    /// `{split}`
    Split,
}

impl MatrixStyle {
    fn gold(self) -> &'static str {
        match self {
            Self::Matrix => "matrix",
            Self::Pmatrix => "pmatrix",
            Self::Bmatrix => "bmatrix",
            Self::Vmatrix => "vmatrix",
            Self::VVmatrix => "Vmatrix",
            Self::BBmatrix => "Bmatrix",
            Self::Cases => "cases",
            Self::Array => "array",
            Self::Aligned => "aligned",
            Self::Align => "align",
            Self::Gather => "gather",
            Self::Multline => "multline",
            Self::Equation => "equation",
            Self::Split => "split",
        }
    }

    /// `align` / `gather` / `multline` / `equation` take display style and numbers.
    #[must_use]
    pub fn is_display_env(self) -> bool {
        matches!(
            self,
            Self::Align | Self::Gather | Self::Multline | Self::Equation
        )
    }

    /// Rows are numbered unless `\nonumber` / `\notag` / `\tag` says otherwise.
    #[must_use]
    pub fn numbers_rows(self) -> bool {
        matches!(self, Self::Align | Self::Gather)
    }

    /// One equation number for the whole environment (last line for `multline`).
    #[must_use]
    pub fn numbers_once(self) -> bool {
        matches!(self, Self::Equation | Self::Multline)
    }
}

/// One column of an `{array}` preamble (`l`, `c`, `r`, `|`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColSpec {
    /// `l`
    Left,
    /// `c`
    Center,
    /// `r`
    Right,
    /// `|`
    VRule,
}

impl ColSpec {
    fn gold(self) -> char {
        match self {
            Self::Left => 'l',
            Self::Center => 'c',
            Self::Right => 'r',
            Self::VRule => '|',
        }
    }

    /// True for `|`.
    #[must_use]
    pub fn is_rule(self) -> bool {
        matches!(self, Self::VRule)
    }
}

/// Per-row equation number in numbered environments.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EqNumber {
    /// Auto number when the environment numbers rows; none otherwise.
    Default,
    /// `\nonumber` / `\notag`
    Suppress,
    /// `\tag{...}` or `\tag*{...}`
    Tag {
        /// `\tag*` — no parentheses around the tag.
        star: bool,
        /// Tag body (text style at layout).
        body: Box<MathNode>,
    },
}

/// One row of a matrix / alignment environment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EnvRow {
    /// Alignment cells (`&`-separated).
    Cells {
        /// Column entries.
        cells: Vec<MathNode>,
        /// Number / tag for this row.
        number: EqNumber,
        /// `\label{...}` keys bound to this row's number.
        labels: Vec<String>,
    },
    /// `\hline`
    Hline,
    /// `\intertext{...}`
    Intertext(Box<MathNode>),
}

impl EnvRow {
    /// A data row with default numbering and no labels.
    #[must_use]
    pub fn cells(cells: Vec<MathNode>) -> Self {
        Self::Cells {
            cells,
            number: EqNumber::Default,
            labels: Vec::new(),
        }
    }

    fn gold(&self) -> String {
        match self {
            Self::Hline => "(hline)".into(),
            Self::Intertext(n) => format!("(intertext {})", n.gold()),
            Self::Cells {
                cells,
                number,
                labels,
            } => {
                let mut s = String::from("(");
                for (i, c) in cells.iter().enumerate() {
                    if i > 0 {
                        s.push(' ');
                    }
                    s.push_str(&c.gold());
                }
                match number {
                    EqNumber::Default => {}
                    EqNumber::Suppress => s.push_str(" (nonumber)"),
                    EqNumber::Tag { star: false, body } => {
                        s.push_str(&format!(" (tag {})", body.gold()));
                    }
                    EqNumber::Tag { star: true, body } => {
                        s.push_str(&format!(" (tagstar {})", body.gold()));
                    }
                }
                for lab in labels {
                    s.push_str(&format!(" (label {lab})"));
                }
                s.push(')');
                s
            }
        }
    }
}

/// Which integral glyph.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntegralKind {
    /// `\int`
    Int,
    /// `\iint`
    Iint,
    /// `\iiint`
    Iiint,
    /// `\oint`
    Oint,
    /// `\oiint`
    Oiint,
}

impl IntegralKind {
    fn gold(self) -> &'static str {
        match self {
            Self::Int => "int",
            Self::Iint => "iint",
            Self::Iiint => "iiint",
            Self::Oint => "oint",
            Self::Oiint => "oiint",
        }
    }
}

/// `\phantom` / `\vphantom` / `\hphantom`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhantomKind {
    /// `\phantom`
    Full,
    /// `\vphantom`
    Vertical,
    /// `\hphantom`
    Horizontal,
}

impl PhantomKind {
    fn gold(self) -> &'static str {
        match self {
            Self::Full => "phantom",
            Self::Vertical => "vphantom",
            Self::Horizontal => "hphantom",
        }
    }
}

/// A `\left` / `\right` delimiter (or `.` for empty).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Delimiter {
    /// `\left.` / `\right.`
    Empty,
    /// Literal character (`(`, `[`, `|`).
    Char(char),
    /// Named delimiter (`langle`, `{` from `\{`).
    Named(String),
}

impl Delimiter {
    fn gold(&self) -> String {
        match self {
            Self::Empty => ".".into(),
            Self::Char(c) => c.to_string(),
            Self::Named(n) => {
                if n == "{" || n == "}" || n == "|" {
                    format!("\\{n}")
                } else {
                    n.clone()
                }
            }
        }
    }
}

/// `\big` / `\Big` / `\bigg` / `\Bigg` (and `l`/`r`/`m` siblings).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DelimSize {
    /// `\big` — about 1.2 em.
    Big,
    /// `\Big` — about 1.8 em.
    Big2,
    /// `\bigg` — about 2.4 em.
    Bigg,
    /// `\Bigg` — about 3.0 em.
    Bigg2,
}

impl DelimSize {
    fn gold(self) -> &'static str {
        match self {
            Self::Big => "big",
            Self::Big2 => "Big",
            Self::Bigg => "bigg",
            Self::Bigg2 => "Bigg",
        }
    }

    /// Map a control sequence to a size. `None` if it is not a `\big` family command.
    #[must_use]
    pub fn from_command(name: &str) -> Option<Self> {
        match name {
            "big" | "bigl" | "bigr" | "bigm" => Some(Self::Big),
            "Big" | "Bigl" | "Bigr" | "Bigm" => Some(Self::Big2),
            "bigg" | "biggl" | "biggr" | "biggm" => Some(Self::Bigg),
            "Bigg" | "Biggl" | "Biggr" | "Biggm" => Some(Self::Bigg2),
            _ => None,
        }
    }

    /// Open / Close / Rel from the `l` / `r` / `m` suffix; `None` for unsuffixed `\big`.
    #[must_use]
    pub fn class_from_command(name: &str) -> Option<AtomKind> {
        if name.ends_with('l') {
            Some(AtomKind::Open)
        } else if name.ends_with('r') {
            Some(AtomKind::Close)
        } else if name.ends_with('m') {
            Some(AtomKind::Rel)
        } else {
            None
        }
    }
}

/// Typed math-mode syntax tree.
///
/// Produced by [`crate::parse()`]. [`MathNode::gold`] is the gold-stable debug form.
/// The enum is non-exhaustive so a later release can add a variant. A
/// downstream match needs a wildcard.
///
/// # Examples
///
/// ```
/// use latex_rust::parse;
///
/// let n = parse("x^2").unwrap();
/// assert!(n.gold().contains("sup"));
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum MathNode {
    /// Single character with a TeX atom class.
    Atom(char, AtomKind),
    /// `\frac` / `\dfrac` / `\tfrac` / `\cfrac` / `{a \over b}`.
    Fraction(Box<MathNode>, Box<MathNode>),
    /// `\sqrt` / `\sqrt[n]`. Index `None` is a square root.
    Radical(Option<Box<MathNode>>, Box<MathNode>),
    /// `x^{}`
    Superscript(Box<MathNode>, Box<MathNode>),
    /// `x_{}`
    Subscript(Box<MathNode>, Box<MathNode>),
    /// `x_{}^{}`
    SubSup(Box<MathNode>, Box<MathNode>, Box<MathNode>),
    /// `\left ... \right`
    Delimited(Delimiter, Box<MathNode>, Delimiter),
    /// `\big` / `\Big` / `\bigg` / `\Bigg` (and `l`/`r`/`m` forms).
    SizedDelim(Delimiter, DelimSize, AtomKind),
    /// Horizontal list of nodes.
    Row(Vec<MathNode>),
    /// Matrix or multiline environment. `colspec` is the `{array}` preamble (empty otherwise).
    Matrix(MatrixStyle, Vec<ColSpec>, Vec<EnvRow>),
    /// `\substack{...}` — stacked script-style lines.
    Substack(Vec<MathNode>),
    /// `\ref{key}`
    Ref(String),
    /// `\tag{...}` / `\tag*{...}` (peeled into [`EnvRow`] when inside an environment).
    Tag {
        /// `\tag*`
        star: bool,
        /// Tag body.
        body: Box<MathNode>,
    },
    /// `\label{key}`
    Label(String),
    /// `\nonumber` / `\notag`
    NoNumber,
    /// `\hline` (peeled into [`EnvRow::Hline`] in environments).
    Hline,
    /// `\intertext{...}`
    Intertext(Box<MathNode>),
    /// `\sum` with optional lower / upper limits.
    Sum(Option<Box<MathNode>>, Option<Box<MathNode>>),
    /// `\int` family with optional limits.
    Integral(IntegralKind, Option<Box<MathNode>>, Option<Box<MathNode>>),
    /// `\prod` with optional limits.
    Product(Option<Box<MathNode>>, Option<Box<MathNode>>),
    /// `\lim` with optional subscript.
    Limit(Option<Box<MathNode>>),
    /// `\overset` / `\underset` / `\stackrel` (base, over, under).
    OverUnder(Box<MathNode>, Option<Box<MathNode>>, Option<Box<MathNode>>),
    /// Accent or decoration on a nucleus.
    Accent(Box<MathNode>, AccentKind),
    /// `\cancelto{value}{expr}`
    CancelTo(Box<MathNode>, Box<MathNode>),
    /// Styled character run (`\mathrm`, `\text`, …).
    Text(String, TextStyle),
    /// Explicit math skip.
    Space(SpaceKind),
    /// Named operator (`\sin`). The flag is `\limits` vs `\nolimits`.
    Operator(String, bool),
    /// Named glyph (`\alpha`, `\times`). The string is the control-sequence name.
    Symbol(String),
    /// `\color` applied to a following math list.
    Color(Color, Box<MathNode>),
    /// `\textcolor{...}{...}`
    TextColor(Color, Box<MathNode>),
    /// `\colorbox{...}{...}`
    ColorBox(Color, Box<MathNode>),
    /// `\fcolorbox{border}{fill}{body}`
    FColorBox(Color, Color, Box<MathNode>),
    /// Vertical strut (height, depth) in em.
    Strut(Dim, Dim),
    /// `\phantom` family.
    Phantom(PhantomKind, Box<MathNode>),
}

impl Drop for MathNode {
    fn drop(&mut self) {
        // Box drop walks children on the native stack. A 10,000-deep `\left`
        // or `\frac` is a valid tree under a raised `max_depth` and must drop
        // on a 256 KiB thread, so the children move to a heap stack first.
        let mut stack = Vec::new();
        self.drain_children(&mut stack);
        while let Some(mut child) = stack.pop() {
            child.drain_children(&mut stack);
        }
    }
}

fn push_box(out: &mut Vec<MathNode>, slot: &mut Box<MathNode>) {
    out.push(std::mem::replace(slot.as_mut(), MathNode::Hline));
}

fn push_opt(out: &mut Vec<MathNode>, slot: &mut Option<Box<MathNode>>) {
    if let Some(child) = slot.take() {
        out.push(*child);
    }
}

impl MathNode {
    fn drain_children(&mut self, out: &mut Vec<MathNode>) {
        match self {
            Self::Fraction(a, b) | Self::Superscript(a, b) | Self::Subscript(a, b) => {
                push_box(out, a);
                push_box(out, b);
            }
            Self::CancelTo(a, b) => {
                push_box(out, a);
                push_box(out, b);
            }
            Self::SubSup(a, b, c) => {
                push_box(out, a);
                push_box(out, b);
                push_box(out, c);
            }
            Self::Radical(index, body) => {
                push_opt(out, index);
                push_box(out, body);
            }
            Self::Delimited(_, body, _) | Self::Intertext(body) => push_box(out, body),
            Self::Row(items) | Self::Substack(items) => out.append(items),
            Self::Matrix(_, _, rows) => {
                for row in rows {
                    row.drain_nodes(out);
                }
            }
            Self::Tag { body, .. } => push_box(out, body),
            Self::Sum(lower, upper) | Self::Product(lower, upper) => {
                push_opt(out, lower);
                push_opt(out, upper);
            }
            Self::Integral(_, lower, upper) => {
                push_opt(out, lower);
                push_opt(out, upper);
            }
            Self::Limit(lower) => push_opt(out, lower),
            Self::OverUnder(base, over, under) => {
                push_box(out, base);
                push_opt(out, over);
                push_opt(out, under);
            }
            Self::Accent(body, _)
            | Self::Color(_, body)
            | Self::TextColor(_, body)
            | Self::ColorBox(_, body)
            | Self::Phantom(_, body) => push_box(out, body),
            Self::FColorBox(_, _, body) => push_box(out, body),
            Self::Atom(_, _)
            | Self::SizedDelim(_, _, _)
            | Self::Ref(_)
            | Self::Label(_)
            | Self::NoNumber
            | Self::Hline
            | Self::Text(_, _)
            | Self::Space(_)
            | Self::Operator(_, _)
            | Self::Symbol(_)
            | Self::Strut(_, _) => {}
        }
    }
}

impl EnvRow {
    fn drain_nodes(&mut self, out: &mut Vec<MathNode>) {
        match self {
            Self::Cells { cells, number, .. } => {
                out.append(cells);
                number.drain_node(out);
            }
            Self::Hline => {}
            Self::Intertext(body) => push_box(out, body),
        }
    }
}

impl EqNumber {
    fn drain_node(&mut self, out: &mut Vec<MathNode>) {
        if let Self::Tag { body, .. } = self {
            push_box(out, body);
        }
    }
}

impl MathNode {
    /// Gold-stable S-expression. One space between items; no trailing space.
    #[must_use]
    pub fn gold(&self) -> String {
        match self {
            Self::Atom(c, k) => format!("(atom {} {})", k.gold(), quote_atom(*c)),
            Self::Fraction(n, d) => format!("(frac {} {})", n.gold(), d.gold()),
            Self::Radical(None, r) => format!("(sqrt {})", r.gold()),
            Self::Radical(Some(i), r) => format!("(sqrtn {} {})", i.gold(), r.gold()),
            Self::Superscript(b, e) => format!("(sup {} {})", b.gold(), e.gold()),
            Self::Subscript(b, s) => format!("(sub {} {})", b.gold(), s.gold()),
            Self::SubSup(b, s, e) => format!("(subsup {} {} {})", b.gold(), s.gold(), e.gold()),
            Self::Delimited(l, b, r) => {
                format!("(delim {} {} {})", l.gold(), b.gold(), r.gold())
            }
            Self::SizedDelim(d, sz, k) => {
                format!("(big {} {} {})", sz.gold(), k.gold(), quote_delim(d))
            }
            Self::Row(items) => {
                if items.is_empty() {
                    "(row)".into()
                } else {
                    let mut s = String::from("(row");
                    for it in items {
                        s.push(' ');
                        s.push_str(&it.gold());
                    }
                    s.push(')');
                    s
                }
            }
            Self::Matrix(style, spec, rows) => {
                let mut s = format!("(matrix {}", style.gold());
                if !spec.is_empty() {
                    s.push(' ');
                    for c in spec {
                        s.push(c.gold());
                    }
                }
                for row in rows {
                    s.push(' ');
                    s.push_str(&row.gold());
                }
                s.push(')');
                s
            }
            Self::Substack(lines) => {
                let mut s = String::from("(substack");
                for ln in lines {
                    s.push(' ');
                    s.push_str(&ln.gold());
                }
                s.push(')');
                s
            }
            Self::Ref(k) => format!("(ref {k})"),
            Self::Tag { star: false, body } => format!("(tag {})", body.gold()),
            Self::Tag { star: true, body } => format!("(tagstar {})", body.gold()),
            Self::Label(k) => format!("(label {k})"),
            Self::NoNumber => "(nonumber)".into(),
            Self::Hline => "(hline)".into(),
            Self::Intertext(n) => format!("(intertext {})", n.gold()),
            Self::Sum(lo, hi) => format!("(sum {} {})", opt(lo), opt(hi)),
            Self::Integral(k, lo, hi) => {
                format!("({} {} {})", k.gold(), opt(lo), opt(hi))
            }
            Self::Product(lo, hi) => format!("(prod {} {})", opt(lo), opt(hi)),
            Self::Limit(lo) => format!("(lim {})", opt(lo)),
            Self::OverUnder(b, over, under) => {
                format!("(overunder {} {} {})", b.gold(), opt(over), opt(under))
            }
            Self::Accent(b, a) => format!("(accent {} {})", a.gold(), b.gold()),
            Self::CancelTo(v, e) => format!("(cancelto {} {})", v.gold(), e.gold()),
            Self::Text(t, st) => format!("(text {} {})", st.gold(), quote_text(t)),
            Self::Space(SpaceKind::Thin) => "(space thin)".into(),
            Self::Space(SpaceKind::Medium) => "(space medium)".into(),
            Self::Space(SpaceKind::Thick) => "(space thick)".into(),
            Self::Space(SpaceKind::NegThin) => "(space negthin)".into(),
            Self::Space(SpaceKind::Quad) => "(space quad)".into(),
            Self::Space(SpaceKind::Qquad) => "(space qquad)".into(),
            Self::Space(SpaceKind::ControlSpace) => "(space control)".into(),
            Self::Space(SpaceKind::Hspace(d)) => format!("(space hspace {})", dim_gold(d)),
            Self::Operator(name, false) => format!("(op {name})"),
            Self::Operator(name, true) => format!("(op {name} limits)"),
            Self::Symbol(name) => format!("(symbol {name})"),
            Self::Color(c, b) => format!("(color {} {})", c.css_hex(), b.gold()),
            Self::TextColor(c, b) => format!("(textcolor {} {})", c.css_hex(), b.gold()),
            Self::ColorBox(c, b) => format!("(colorbox {} {})", c.css_hex(), b.gold()),
            Self::FColorBox(border, fill, b) => {
                format!(
                    "(fcolorbox {} {} {})",
                    border.css_hex(),
                    fill.css_hex(),
                    b.gold()
                )
            }
            Self::Strut(h, d) => format!("(strut {} {})", dim_gold(h), dim_gold(d)),
            Self::Phantom(k, b) => format!("({} {})", k.gold(), b.gold()),
        }
    }

    /// LaTeX reprint of this tree.
    ///
    /// The method is total: every node prints, and it does not return
    /// [`Err`](crate::Error) or panic. Braces and spaces may differ from the
    /// source that built the tree, so compare trees, then compare a reprint
    /// to a reprint. It is not a byte-preserving formatter.
    ///
    /// Every script base and script argument, every accent body, every
    /// fraction argument, and every `\overset` / `\underset` argument is
    /// braced. An alphabetic control word is followed by a space, so
    /// `\sin x` does not become `\sinx`. `\overline` is that full command.
    ///
    /// Every [`MathNode::Fraction`] is `\frac`, including a fraction that
    /// came from `\dfrac`, `\tfrac`, `\cfrac`, or `\over`. Those commands do
    /// not store a display style, so the printer cannot put it back.
    ///
    /// A [`MathNode::Color`] is its own group. A color nested in a row
    /// therefore does not swallow the nodes that follow it.
    #[must_use]
    pub fn to_latex(&self) -> String {
        let mut out = String::new();
        write_node(&mut out, self);
        out
    }
}

fn opt(n: &Option<Box<MathNode>>) -> String {
    match n {
        None => "_".into(),
        Some(x) => x.gold(),
    }
}

fn quote_delim(d: &Delimiter) -> String {
    match d {
        Delimiter::Empty => ".".into(),
        Delimiter::Char(c) => quote_atom(*c),
        Delimiter::Named(n) => n.clone(),
    }
}

fn quote_atom(c: char) -> String {
    match c {
        '"' => "'\"'".into(),
        '\'' => "\"'\"".into(),
        _ => format!("\"{c}\""),
    }
}

fn quote_text(t: &str) -> String {
    format!("\"{}\"", t.replace('\\', "\\\\").replace('"', "\\\""))
}

fn dim_gold(d: &Dim) -> String {
    const RATIOS: [(i64, i64); 10] = [
        (0, 1),
        (1, 1),
        (2, 1),
        (1, 2),
        (1, 18),
        (2, 18),
        (3, 18),
        (7, 10),
        (3, 10),
        (1, 10),
    ];
    for (n, den) in RATIOS {
        if d.eq_dim(&Dim::ratio(n, den)) {
            if den == 1 {
                return n.to_string();
            }
            return format!("{n}/{den}");
        }
    }
    for i in -64i64..65 {
        if d.eq_dim(&Dim::from_i64(i)) {
            return i.to_string();
        }
    }
    d.to_dec_string()
}

fn write_node(out: &mut String, node: &MathNode) {
    match node {
        MathNode::Atom(c, _) => write_atom(out, *c),
        MathNode::Fraction(num, den) => {
            write_cmd(out, "frac");
            write_group(out, num);
            write_group(out, den);
        }
        MathNode::Radical(index, body) => {
            write_cmd(out, "sqrt");
            if let Some(index) = index {
                out.push('[');
                write_node(out, index);
                out.push(']');
            }
            write_group(out, body);
        }
        MathNode::Superscript(base, sup) => write_scripts(out, base, None, Some(sup)),
        MathNode::Subscript(base, sub) => write_scripts(out, base, Some(sub), None),
        MathNode::SubSup(base, sub, sup) => write_scripts(out, base, Some(sub), Some(sup)),
        MathNode::Delimited(open, body, close) => {
            write_cmd(out, "left");
            write_delim(out, open);
            write_node(out, body);
            write_cmd(out, "right");
            write_delim(out, close);
        }
        MathNode::SizedDelim(delim, size, kind) => {
            out.push('\\');
            out.push_str(size.gold());
            out.push_str(sized_suffix(*kind, delim));
            out.push(' ');
            write_delim(out, delim);
        }
        MathNode::Row(items) => write_row(out, items),
        MathNode::Matrix(style, spec, rows) => write_matrix(out, *style, spec, rows),
        MathNode::Substack(lines) => {
            write_cmd(out, "substack");
            out.push('{');
            for (i, line) in lines.iter().enumerate() {
                if i > 0 {
                    out.push_str("\\\\");
                }
                write_node(out, line);
            }
            out.push('}');
        }
        MathNode::Ref(key) => write_braced_text(out, "ref", key),
        MathNode::Tag { star, body } => {
            write_cmd(out, "tag");
            if *star {
                out.push('*');
            }
            write_group(out, body);
        }
        MathNode::Label(key) => write_braced_text(out, "label", key),
        MathNode::NoNumber => write_cmd(out, "nonumber"),
        MathNode::Hline => write_cmd(out, "hline"),
        MathNode::Intertext(body) => write_intertext(out, body),
        MathNode::Sum(lower, upper) => {
            write_op_scripts(out, "sum", lower.as_deref(), upper.as_deref())
        }
        MathNode::Product(lower, upper) => {
            write_op_scripts(out, "prod", lower.as_deref(), upper.as_deref())
        }
        MathNode::Integral(kind, lower, upper) => {
            write_op_scripts(out, kind.gold(), lower.as_deref(), upper.as_deref())
        }
        MathNode::Limit(lower) => write_op_scripts(out, "lim", lower.as_deref(), None),
        MathNode::OverUnder(base, over, under) => {
            write_over_under(out, base, over.as_deref(), under.as_deref())
        }
        MathNode::Accent(body, kind) => {
            write_cmd(out, kind.gold());
            write_group(out, body);
        }
        MathNode::CancelTo(value, expr) => {
            write_cmd(out, "cancelto");
            write_group(out, value);
            write_group(out, expr);
        }
        MathNode::Text(text, style) => {
            write_cmd(out, style_cmd(*style));
            out.push('{');
            push_escaped(out, text);
            out.push('}');
        }
        MathNode::Space(kind) => write_space(out, kind),
        MathNode::Operator(name, limits) => write_operator(out, name, *limits),
        MathNode::Symbol(name) => write_cmd(out, name),
        MathNode::Color(color, body) => {
            out.push('{');
            write_cmd(out, "color");
            push_html(out, *color);
            write_node(out, body);
            out.push('}');
        }
        MathNode::TextColor(color, body) => write_paint(out, "textcolor", *color, None, body),
        MathNode::ColorBox(color, body) => write_paint(out, "colorbox", *color, None, body),
        MathNode::FColorBox(border, fill, body) => {
            write_paint(out, "fcolorbox", *border, Some(*fill), body)
        }
        MathNode::Strut(height, depth) => write_strut(out, height, depth),
        MathNode::Phantom(kind, body) => {
            write_cmd(out, kind.gold());
            write_group(out, body);
        }
    }
}

fn write_scripts(
    out: &mut String,
    base: &MathNode,
    sub: Option<&MathNode>,
    sup: Option<&MathNode>,
) {
    write_group(out, base);
    if let Some(sub) = sub {
        out.push('_');
        write_group(out, sub);
    }
    if let Some(sup) = sup {
        out.push('^');
        write_group(out, sup);
    }
}

fn write_op_scripts(out: &mut String, name: &str, sub: Option<&MathNode>, sup: Option<&MathNode>) {
    write_cmd(out, name);
    if let Some(sub) = sub {
        out.push('_');
        write_group(out, sub);
    }
    if let Some(sup) = sup {
        out.push('^');
        write_group(out, sup);
    }
}

fn write_group(out: &mut String, node: &MathNode) {
    out.push('{');
    write_node(out, node);
    out.push('}');
}

/// Brace each child so a nested row stays nested, and so `<` `=` and a run of
/// letters are not glued back into a relation or a bare operator.
fn write_row(out: &mut String, items: &[MathNode]) {
    if items.is_empty() {
        out.push_str("{}");
        return;
    }
    for item in items {
        write_group(out, item);
    }
}

fn write_matrix(out: &mut String, style: MatrixStyle, spec: &[ColSpec], rows: &[EnvRow]) {
    write_cmd(out, "begin");
    out.push('{');
    out.push_str(style.gold());
    out.push('}');
    if matches!(style, MatrixStyle::Array) || !spec.is_empty() {
        out.push('{');
        for col in spec {
            out.push(col.gold());
        }
        out.push('}');
    }
    let mut prev_cells = false;
    for row in rows {
        let cells = matches!(row, EnvRow::Cells { .. });
        if prev_cells {
            out.push_str("\\\\");
        }
        prev_cells = cells;
        match row {
            EnvRow::Hline => write_cmd(out, "hline"),
            EnvRow::Intertext(body) => write_intertext(out, body),
            EnvRow::Cells {
                cells,
                number,
                labels,
            } => {
                for (i, cell) in cells.iter().enumerate() {
                    if i > 0 {
                        out.push('&');
                    }
                    write_node(out, cell);
                    if i + 1 == cells.len() {
                        write_eq_meta(out, number, labels);
                    }
                }
                if cells.is_empty() {
                    write_eq_meta(out, number, labels);
                }
            }
        }
    }
    write_cmd(out, "end");
    out.push('{');
    out.push_str(style.gold());
    out.push('}');
}

fn write_eq_meta(out: &mut String, number: &EqNumber, labels: &[String]) {
    for key in labels {
        write_braced_text(out, "label", key);
    }
    match number {
        EqNumber::Default => {}
        EqNumber::Suppress => write_cmd(out, "nonumber"),
        EqNumber::Tag { star, body } => {
            write_cmd(out, "tag");
            if *star {
                out.push('*');
            }
            write_group(out, body);
        }
    }
}

fn write_over_under(
    out: &mut String,
    base: &MathNode,
    over: Option<&MathNode>,
    under: Option<&MathNode>,
) {
    if let MathNode::Accent(body, kind @ (AccentKind::Overbrace | AccentKind::Underbrace)) = base {
        write_cmd(out, kind.gold());
        write_group(out, body);
        if let Some(under) = under {
            out.push('_');
            write_group(out, under);
        }
        if let Some(over) = over {
            out.push('^');
            write_group(out, over);
        }
        return;
    }
    if let MathNode::Symbol(name) = base {
        let cmd = match (name.as_str(), over) {
            ("longrightarrow", Some(_)) => Some("xrightarrow"),
            ("longleftarrow", Some(_)) => Some("xleftarrow"),
            _ => None,
        };
        if let Some(cmd) = cmd {
            write_cmd(out, cmd);
            if let Some(under) = under {
                out.push('[');
                write_node(out, under);
                out.push(']');
            }
            if let Some(over) = over {
                write_group(out, over);
            }
            return;
        }
    }
    match (over, under) {
        (Some(over), Some(under)) => {
            write_cmd(out, "overset");
            write_group(out, over);
            out.push('{');
            write_cmd(out, "underset");
            write_group(out, under);
            write_group(out, base);
            out.push('}');
        }
        (Some(over), None) => {
            write_cmd(out, "overset");
            write_group(out, over);
            write_group(out, base);
        }
        (None, Some(under)) => {
            write_cmd(out, "underset");
            write_group(out, under);
            write_group(out, base);
        }
        (None, None) => write_node(out, base),
    }
}

fn write_intertext(out: &mut String, body: &MathNode) {
    write_cmd(out, "intertext");
    out.push('{');
    if let MathNode::Text(text, _) = body {
        push_escaped(out, text);
    } else {
        write_node(out, body);
    }
    out.push('}');
}

fn write_paint(out: &mut String, cmd: &str, first: Color, second: Option<Color>, body: &MathNode) {
    write_cmd(out, cmd);
    push_html(out, first);
    if let Some(second) = second {
        push_html(out, second);
    }
    write_group(out, body);
}

fn write_operator(out: &mut String, name: &str, limits: bool) {
    match operator_limits(name) {
        Some(default) => {
            write_cmd(out, name);
            if limits != default {
                write_cmd(out, if limits { "limits" } else { "nolimits" });
            }
        }
        None => {
            write_cmd(out, "operatorname");
            out.push('{');
            push_escaped(out, name);
            out.push('}');
            if limits {
                write_cmd(out, "limits");
            }
        }
    }
}

fn operator_limits(name: &str) -> Option<bool> {
    Some(match name {
        "sin" | "cos" | "tan" | "cot" | "sec" | "csc" | "arcsin" | "arccos" | "arctan" | "sinh"
        | "cosh" | "tanh" | "coth" | "log" | "ln" | "lg" | "exp" | "limsup" | "liminf" | "sup"
        | "inf" | "max" | "min" | "det" | "dim" | "ker" | "deg" | "gcd" | "lcm" | "Pr" | "arg" => {
            false
        }
        "coprod" | "bigcup" | "bigcap" | "bigsqcup" | "bigvee" | "bigwedge" | "bigoplus"
        | "bigotimes" | "biguplus" => true,
        _ => return None,
    })
}

fn write_space(out: &mut String, kind: &SpaceKind) {
    match kind {
        SpaceKind::Thin => out.push_str("\\,"),
        SpaceKind::Medium => out.push_str("\\:"),
        SpaceKind::Thick => out.push_str("\\;"),
        SpaceKind::NegThin => out.push_str("\\!"),
        SpaceKind::Quad => write_cmd(out, "quad"),
        SpaceKind::Qquad => write_cmd(out, "qquad"),
        SpaceKind::ControlSpace => out.push_str("\\ "),
        SpaceKind::Hspace(dim) => {
            write_cmd(out, "hspace");
            out.push('{');
            out.push_str(&dim.to_dec_string());
            out.push_str("em}");
        }
    }
}

fn write_strut(out: &mut String, height: &Dim, depth: &Dim) {
    if height.eq_dim(&Dim::ratio(7, 10)) && depth.eq_dim(&Dim::ratio(3, 10)) {
        write_cmd(out, "strut");
        return;
    }
    write_cmd(out, "rule");
    out.push_str("{0em}{");
    out.push_str(&height.to_dec_string());
    out.push_str("em}");
}

fn write_atom(out: &mut String, c: char) {
    match c {
        '{' | '}' | '%' | '\\' => {
            out.push('\\');
            out.push(c);
        }
        '<' | '>' | '!' => {
            out.push(c);
            out.push(' ');
        }
        _ => out.push(c),
    }
}

fn write_delim(out: &mut String, delim: &Delimiter) {
    match delim {
        Delimiter::Empty => out.push('.'),
        Delimiter::Char(c) => write_atom(out, *c),
        Delimiter::Named(name) if name == "{" || name == "}" || name == "|" => {
            out.push('\\');
            out.push_str(name);
        }
        Delimiter::Named(name) => write_cmd(out, name),
    }
}

fn sized_suffix(kind: AtomKind, delim: &Delimiter) -> &'static str {
    if kind == delim_class(delim) {
        return "";
    }
    match kind {
        AtomKind::Open => "l",
        AtomKind::Close => "r",
        AtomKind::Rel => "m",
        AtomKind::Ord | AtomKind::Op | AtomKind::Bin | AtomKind::Punct | AtomKind::Inner => "",
    }
}

fn delim_class(delim: &Delimiter) -> AtomKind {
    match delim {
        Delimiter::Char(c) => atom_class(*c),
        Delimiter::Named(name) if name == "{" => AtomKind::Open,
        Delimiter::Named(name) if name == "}" => AtomKind::Close,
        Delimiter::Empty | Delimiter::Named(_) => AtomKind::Open,
    }
}

fn atom_class(c: char) -> AtomKind {
    match c {
        '+' | '-' | '*' | '±' | '∓' | '·' | '×' | '÷' => AtomKind::Bin,
        '=' | '<' | '>' | '≠' | '≤' | '≥' | '≈' | '≡' => AtomKind::Rel,
        '(' | '[' | '{' => AtomKind::Open,
        ')' | ']' | '}' => AtomKind::Close,
        ',' | ';' | '!' | '?' | ':' => AtomKind::Punct,
        _ => AtomKind::Ord,
    }
}

fn style_cmd(style: TextStyle) -> &'static str {
    match style {
        TextStyle::Rm => "mathrm",
        TextStyle::Bf => "mathbf",
        TextStyle::It => "mathit",
        TextStyle::Sf => "mathsf",
        TextStyle::Tt => "mathtt",
        TextStyle::Bb => "mathbb",
        TextStyle::Cal => "mathcal",
        TextStyle::Frak => "mathfrak",
        TextStyle::Scr => "mathscr",
        TextStyle::Boldsymbol => "boldsymbol",
        TextStyle::Pmb => "pmb",
        TextStyle::Text => "text",
    }
}

fn write_cmd(out: &mut String, name: &str) {
    out.push('\\');
    out.push_str(name);
    if !name.is_empty() && name.chars().all(|c| c.is_ascii_alphabetic()) {
        out.push(' ');
    }
}

fn write_braced_text(out: &mut String, cmd: &str, text: &str) {
    write_cmd(out, cmd);
    out.push('{');
    push_escaped(out, text);
    out.push('}');
}

fn push_escaped(out: &mut String, text: &str) {
    for c in text.chars() {
        match c {
            '\\' | '{' | '}' | '%' => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
}

fn push_html(out: &mut String, color: Color) {
    out.push_str("[HTML]{");
    let hex = color.css_hex();
    match hex.strip_prefix('#') {
        Some(body) => out.push_str(body),
        None => out.push_str(&hex),
    }
    out.push('}');
}
