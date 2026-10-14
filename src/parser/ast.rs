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

/// One item of a text-mode run (`\text{…}` body).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextItem {
    /// A maximal run of plain characters (no whitespace).
    Str(String),
    /// Collapsed inter-word space.
    Space,
    /// A `\ ` control space.
    ControlSpace,
    /// A `~` no-break space.
    NoBreakSpace,
    /// A text symbol such as `\ldots`, `\quad`, `\qquad`.
    Symbol(&'static str),
    /// A nested text command with its own style.
    Nested(TextStyle, TextRun),
    /// Math embedded in text with `\(…\)` or `$…$`.
    Math(Box<MathNode>),
}

/// Body of a text-mode command. Adjacent `Str` items are merged, no `Str` is
/// empty, and no `Space` follows a `Space`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextRun(Vec<TextItem>);

impl TextRun {
    /// Creates a new `TextRun` from a vector of items, enforcing invariants.
    #[must_use]
    pub fn new(items: Vec<TextItem>) -> Self {
        let mut run = Self::default();
        for item in items {
            run.push(item);
        }
        run
    }

    /// Creates a `TextRun` from a plain string, converting spaces to `Space` items.
    #[must_use]
    pub fn plain(s: &str) -> Self {
        let mut run = Self::default();
        for c in s.chars() {
            if c == ' ' {
                run.push(TextItem::Space);
            } else {
                run.push_char(c);
            }
        }
        run
    }

    /// Appends an item to the run, maintaining invariants.
    pub fn push(&mut self, item: TextItem) {
        match (self.0.last_mut(), item) {
            (_, TextItem::Str(s)) if s.is_empty() => {}
            (Some(TextItem::Str(prev)), TextItem::Str(s)) => prev.push_str(&s),
            (Some(TextItem::Space), TextItem::Space) => {}
            (_, item) => self.0.push(item),
        }
    }

    /// Appends a character to the last `Str` item or creates a new one.
    pub fn push_char(&mut self, c: char) {
        if let Some(TextItem::Str(s)) = self.0.last_mut() {
            s.push(c);
        } else {
            self.0.push(TextItem::Str(c.to_string()));
        }
    }

    /// Appends all items from another `TextRun`.
    pub fn append(&mut self, other: TextRun) {
        for item in other.0 {
            self.push(item);
        }
    }

    /// Returns a slice of the items in this run.
    #[must_use]
    pub fn items(&self) -> &[TextItem] {
        &self.0
    }

    /// Checks if the run is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns `Some` only when the run contains exactly one `Str` item.
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self.0.as_slice() {
            [TextItem::Str(s)] => Some(s),
            _ => None,
        }
    }

    /// Converts the run to a plain string representation.
    #[must_use]
    pub fn to_plain_string(&self) -> String {
        let mut result = String::new();
        for item in &self.0 {
            match item {
                TextItem::Str(s) => result.push_str(s),
                TextItem::Space | TextItem::ControlSpace | TextItem::NoBreakSpace => {
                    result.push(' ');
                }
                TextItem::Symbol(name) => {
                    if *name == "ldots" {
                        result.push('\u{2026}');
                    } else {
                        result.push(' ');
                    }
                }
                TextItem::Nested(_, run) => result.push_str(&run.to_plain_string()),
                TextItem::Math(_) => {}
            }
        }
        result
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

/// Which command built a [`MathNode::Fraction`].
///
/// `\frac` and `{a \over b}` are [`FracStyle::Frac`]. `\dfrac`, `\tfrac`, and
/// `\cfrac` keep their own style, which layout uses and which [`MathNode::gold`]
/// and [`MathNode::to_latex`] print back.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum FracStyle {
    /// `\frac` and infix `\over`.
    Frac,
    /// `\dfrac`. Layout uses display style.
    Dfrac,
    /// `\tfrac`. Layout uses text style.
    Tfrac,
    /// `\cfrac`. Layout uses display style.
    Cfrac,
}

impl FracStyle {
    pub(crate) fn gold(self) -> &'static str {
        match self {
            Self::Frac => "frac",
            Self::Dfrac => "dfrac",
            Self::Tfrac => "tfrac",
            Self::Cfrac => "cfrac",
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
#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum MathNode {
    /// Single character with a TeX atom class.
    Atom(char, AtomKind),
    /// `\frac` / `\dfrac` / `\tfrac` / `\cfrac` / `{a \over b}`.
    ///
    /// The style is the command. `\over` is [`FracStyle::Frac`].
    Fraction(FracStyle, Box<MathNode>, Box<MathNode>),
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
    Text(TextRun, TextStyle),
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

impl Clone for MathNode {
    fn clone(&self) -> Self {
        clone_node(self)
    }
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
            Self::Fraction(_, a, b) | Self::Superscript(a, b) | Self::Subscript(a, b) => {
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

enum CloneTask<'a> {
    Node(&'a MathNode),
    Finish(Finish),
}

enum Finish {
    Pair(PairKind),
    Radical {
        has_index: bool,
    },
    SubSup,
    Delimited(Delimiter, Delimiter),
    List(ListKind, usize),
    Matrix {
        style: MatrixStyle,
        cols: Vec<ColSpec>,
        rows: Vec<RowPlan>,
    },
    Tag {
        star: bool,
    },
    Limits {
        kind: LimitKind,
        has_lo: bool,
        has_hi: bool,
    },
    Limit {
        has_lo: bool,
    },
    OverUnder {
        has_over: bool,
        has_under: bool,
    },
    Accent(AccentKind),
    Wrap(WrapKind),
    Intertext,
}

enum PairKind {
    Fraction(FracStyle),
    Superscript,
    Subscript,
    CancelTo,
}

enum ListKind {
    Row,
    Substack,
}

enum LimitKind {
    Sum,
    Product,
    Integral(IntegralKind),
}

enum WrapKind {
    Color(Color),
    TextColor(Color),
    ColorBox(Color),
    FColorBox(Color, Color),
    Phantom(PhantomKind),
}

enum RowPlan {
    Hline,
    Intertext,
    Cells {
        n: usize,
        number: NumPlan,
        labels: Vec<String>,
    },
}

enum NumPlan {
    Default,
    Suppress,
    Tag { star: bool },
}

fn clone_node(root: &MathNode) -> MathNode {
    let mut tasks = vec![CloneTask::Node(root)];
    let mut done = Vec::new();
    while let Some(task) = tasks.pop() {
        match task {
            CloneTask::Node(node) => schedule_clone(node, &mut tasks, &mut done),
            CloneTask::Finish(finish) => finish_clone(finish, &mut done),
        }
    }
    done.pop().unwrap_or(MathNode::Hline)
}

fn pop_node(done: &mut Vec<MathNode>) -> MathNode {
    done.pop().unwrap_or(MathNode::Hline)
}

fn schedule_clone<'a>(
    node: &'a MathNode,
    tasks: &mut Vec<CloneTask<'a>>,
    done: &mut Vec<MathNode>,
) {
    match node {
        MathNode::Atom(ch, kind) => done.push(MathNode::Atom(*ch, *kind)),
        MathNode::SizedDelim(delim, size, kind) => {
            done.push(MathNode::SizedDelim(delim.clone(), *size, *kind));
        }
        MathNode::Ref(key) => done.push(MathNode::Ref(key.clone())),
        MathNode::Label(key) => done.push(MathNode::Label(key.clone())),
        MathNode::NoNumber => done.push(MathNode::NoNumber),
        MathNode::Hline => done.push(MathNode::Hline),
        MathNode::Text(text, style) => done.push(MathNode::Text(text.clone(), *style)),
        MathNode::Space(kind) => done.push(MathNode::Space(kind.clone())),
        MathNode::Operator(name, limits) => {
            done.push(MathNode::Operator(name.clone(), *limits));
        }
        MathNode::Symbol(name) => done.push(MathNode::Symbol(name.clone())),
        MathNode::Strut(height, depth) => {
            done.push(MathNode::Strut(height.clone(), depth.clone()));
        }
        MathNode::Fraction(style, a, b) => {
            tasks.push(CloneTask::Finish(Finish::Pair(PairKind::Fraction(*style))));
            tasks.push(CloneTask::Node(b));
            tasks.push(CloneTask::Node(a));
        }
        MathNode::Superscript(a, b) => {
            tasks.push(CloneTask::Finish(Finish::Pair(PairKind::Superscript)));
            tasks.push(CloneTask::Node(b));
            tasks.push(CloneTask::Node(a));
        }
        MathNode::Subscript(a, b) => {
            tasks.push(CloneTask::Finish(Finish::Pair(PairKind::Subscript)));
            tasks.push(CloneTask::Node(b));
            tasks.push(CloneTask::Node(a));
        }
        MathNode::CancelTo(a, b) => {
            tasks.push(CloneTask::Finish(Finish::Pair(PairKind::CancelTo)));
            tasks.push(CloneTask::Node(b));
            tasks.push(CloneTask::Node(a));
        }
        MathNode::SubSup(a, b, c) => {
            tasks.push(CloneTask::Finish(Finish::SubSup));
            tasks.push(CloneTask::Node(c));
            tasks.push(CloneTask::Node(b));
            tasks.push(CloneTask::Node(a));
        }
        MathNode::Radical(index, body) => {
            tasks.push(CloneTask::Finish(Finish::Radical {
                has_index: index.is_some(),
            }));
            tasks.push(CloneTask::Node(body));
            if let Some(index) = index {
                tasks.push(CloneTask::Node(index));
            }
        }
        MathNode::Delimited(left, body, right) => {
            tasks.push(CloneTask::Finish(Finish::Delimited(
                left.clone(),
                right.clone(),
            )));
            tasks.push(CloneTask::Node(body));
        }
        MathNode::Row(items) => {
            tasks.push(CloneTask::Finish(Finish::List(ListKind::Row, items.len())));
            for item in items.iter().rev() {
                tasks.push(CloneTask::Node(item));
            }
        }
        MathNode::Substack(items) => {
            tasks.push(CloneTask::Finish(Finish::List(
                ListKind::Substack,
                items.len(),
            )));
            for item in items.iter().rev() {
                tasks.push(CloneTask::Node(item));
            }
        }
        MathNode::Matrix(style, cols, rows) => schedule_matrix(*style, cols, rows, tasks),
        MathNode::Tag { star, body } => {
            tasks.push(CloneTask::Finish(Finish::Tag { star: *star }));
            tasks.push(CloneTask::Node(body));
        }
        MathNode::Sum(lower, upper) => {
            schedule_limits(LimitKind::Sum, lower, upper, tasks);
        }
        MathNode::Product(lower, upper) => {
            schedule_limits(LimitKind::Product, lower, upper, tasks);
        }
        MathNode::Integral(kind, lower, upper) => {
            schedule_limits(LimitKind::Integral(*kind), lower, upper, tasks);
        }
        MathNode::Limit(lower) => {
            tasks.push(CloneTask::Finish(Finish::Limit {
                has_lo: lower.is_some(),
            }));
            if let Some(lower) = lower {
                tasks.push(CloneTask::Node(lower));
            }
        }
        MathNode::OverUnder(base, over, under) => {
            tasks.push(CloneTask::Finish(Finish::OverUnder {
                has_over: over.is_some(),
                has_under: under.is_some(),
            }));
            if let Some(under) = under {
                tasks.push(CloneTask::Node(under));
            }
            if let Some(over) = over {
                tasks.push(CloneTask::Node(over));
            }
            tasks.push(CloneTask::Node(base));
        }
        MathNode::Accent(body, kind) => {
            tasks.push(CloneTask::Finish(Finish::Accent(*kind)));
            tasks.push(CloneTask::Node(body));
        }
        MathNode::Color(color, body) => {
            tasks.push(CloneTask::Finish(Finish::Wrap(WrapKind::Color(*color))));
            tasks.push(CloneTask::Node(body));
        }
        MathNode::TextColor(color, body) => {
            tasks.push(CloneTask::Finish(Finish::Wrap(WrapKind::TextColor(*color))));
            tasks.push(CloneTask::Node(body));
        }
        MathNode::ColorBox(color, body) => {
            tasks.push(CloneTask::Finish(Finish::Wrap(WrapKind::ColorBox(*color))));
            tasks.push(CloneTask::Node(body));
        }
        MathNode::FColorBox(border, fill, body) => {
            tasks.push(CloneTask::Finish(Finish::Wrap(WrapKind::FColorBox(
                *border, *fill,
            ))));
            tasks.push(CloneTask::Node(body));
        }
        MathNode::Phantom(kind, body) => {
            tasks.push(CloneTask::Finish(Finish::Wrap(WrapKind::Phantom(*kind))));
            tasks.push(CloneTask::Node(body));
        }
        MathNode::Intertext(body) => {
            tasks.push(CloneTask::Finish(Finish::Intertext));
            tasks.push(CloneTask::Node(body));
        }
    }
}

fn schedule_limits<'a>(
    kind: LimitKind,
    lower: &'a Option<Box<MathNode>>,
    upper: &'a Option<Box<MathNode>>,
    tasks: &mut Vec<CloneTask<'a>>,
) {
    tasks.push(CloneTask::Finish(Finish::Limits {
        kind,
        has_lo: lower.is_some(),
        has_hi: upper.is_some(),
    }));
    if let Some(upper) = upper {
        tasks.push(CloneTask::Node(upper));
    }
    if let Some(lower) = lower {
        tasks.push(CloneTask::Node(lower));
    }
}

fn schedule_matrix<'a>(
    style: MatrixStyle,
    cols: &[ColSpec],
    rows: &'a [EnvRow],
    tasks: &mut Vec<CloneTask<'a>>,
) {
    let mut plan = Vec::with_capacity(rows.len());
    let mut kids = Vec::new();
    for row in rows {
        match row {
            EnvRow::Hline => plan.push(RowPlan::Hline),
            EnvRow::Intertext(body) => {
                plan.push(RowPlan::Intertext);
                kids.push(body.as_ref());
            }
            EnvRow::Cells {
                cells,
                number,
                labels,
            } => {
                let num = match number {
                    EqNumber::Default => NumPlan::Default,
                    EqNumber::Suppress => NumPlan::Suppress,
                    EqNumber::Tag { star, .. } => NumPlan::Tag { star: *star },
                };
                plan.push(RowPlan::Cells {
                    n: cells.len(),
                    number: num,
                    labels: labels.clone(),
                });
                for cell in cells {
                    kids.push(cell);
                }
                if let EqNumber::Tag { body, .. } = number {
                    kids.push(body);
                }
            }
        }
    }
    tasks.push(CloneTask::Finish(Finish::Matrix {
        style,
        cols: cols.to_vec(),
        rows: plan,
    }));
    for kid in kids.into_iter().rev() {
        tasks.push(CloneTask::Node(kid));
    }
}

fn finish_clone(finish: Finish, done: &mut Vec<MathNode>) {
    match finish {
        Finish::Pair(kind) => {
            let b = Box::new(pop_node(done));
            let a = Box::new(pop_node(done));
            done.push(match kind {
                PairKind::Fraction(style) => MathNode::Fraction(style, a, b),
                PairKind::Superscript => MathNode::Superscript(a, b),
                PairKind::Subscript => MathNode::Subscript(a, b),
                PairKind::CancelTo => MathNode::CancelTo(a, b),
            });
        }
        Finish::SubSup => {
            let c = Box::new(pop_node(done));
            let b = Box::new(pop_node(done));
            let a = Box::new(pop_node(done));
            done.push(MathNode::SubSup(a, b, c));
        }
        Finish::Radical { has_index } => {
            let body = Box::new(pop_node(done));
            let index = if has_index {
                Some(Box::new(pop_node(done)))
            } else {
                None
            };
            done.push(MathNode::Radical(index, body));
        }
        Finish::Delimited(left, right) => {
            let body = Box::new(pop_node(done));
            done.push(MathNode::Delimited(left, body, right));
        }
        Finish::List(kind, n) => {
            let start = done.len().saturating_sub(n);
            let items = done.split_off(start);
            done.push(match kind {
                ListKind::Row => MathNode::Row(items),
                ListKind::Substack => MathNode::Substack(items),
            });
        }
        Finish::Tag { star } => {
            let body = Box::new(pop_node(done));
            done.push(MathNode::Tag { star, body });
        }
        Finish::Limits {
            kind,
            has_lo,
            has_hi,
        } => {
            let upper = if has_hi {
                Some(Box::new(pop_node(done)))
            } else {
                None
            };
            let lower = if has_lo {
                Some(Box::new(pop_node(done)))
            } else {
                None
            };
            done.push(match kind {
                LimitKind::Sum => MathNode::Sum(lower, upper),
                LimitKind::Product => MathNode::Product(lower, upper),
                LimitKind::Integral(integral) => MathNode::Integral(integral, lower, upper),
            });
        }
        Finish::Limit { has_lo } => {
            let lower = if has_lo {
                Some(Box::new(pop_node(done)))
            } else {
                None
            };
            done.push(MathNode::Limit(lower));
        }
        Finish::OverUnder {
            has_over,
            has_under,
        } => {
            let under = if has_under {
                Some(Box::new(pop_node(done)))
            } else {
                None
            };
            let over = if has_over {
                Some(Box::new(pop_node(done)))
            } else {
                None
            };
            let base = Box::new(pop_node(done));
            done.push(MathNode::OverUnder(base, over, under));
        }
        Finish::Accent(kind) => {
            let body = Box::new(pop_node(done));
            done.push(MathNode::Accent(body, kind));
        }
        Finish::Wrap(kind) => {
            let body = Box::new(pop_node(done));
            done.push(match kind {
                WrapKind::Color(color) => MathNode::Color(color, body),
                WrapKind::TextColor(color) => MathNode::TextColor(color, body),
                WrapKind::ColorBox(color) => MathNode::ColorBox(color, body),
                WrapKind::FColorBox(border, fill) => MathNode::FColorBox(border, fill, body),
                WrapKind::Phantom(phantom) => MathNode::Phantom(phantom, body),
            });
        }
        Finish::Intertext => {
            let body = Box::new(pop_node(done));
            done.push(MathNode::Intertext(body));
        }
        Finish::Matrix { style, cols, rows } => {
            let n = matrix_child_count(&rows);
            let start = done.len().saturating_sub(n);
            let kids = done.split_off(start);
            let mut kids = kids.into_iter();
            let mut built = Vec::with_capacity(rows.len());
            for row in rows {
                built.push(match row {
                    RowPlan::Hline => EnvRow::Hline,
                    RowPlan::Intertext => {
                        EnvRow::Intertext(Box::new(kids.next().unwrap_or(MathNode::Hline)))
                    }
                    RowPlan::Cells { n, number, labels } => {
                        let mut cells = Vec::with_capacity(n);
                        for _ in 0..n {
                            cells.push(kids.next().unwrap_or(MathNode::Hline));
                        }
                        let number = match number {
                            NumPlan::Default => EqNumber::Default,
                            NumPlan::Suppress => EqNumber::Suppress,
                            NumPlan::Tag { star } => EqNumber::Tag {
                                star,
                                body: Box::new(kids.next().unwrap_or(MathNode::Hline)),
                            },
                        };
                        EnvRow::Cells {
                            cells,
                            number,
                            labels,
                        }
                    }
                });
            }
            done.push(MathNode::Matrix(style, cols, built));
        }
    }
}

fn matrix_child_count(rows: &[RowPlan]) -> usize {
    let mut n = 0usize;
    for row in rows {
        match row {
            RowPlan::Hline => {}
            RowPlan::Intertext => n = n.saturating_add(1),
            RowPlan::Cells {
                n: cells, number, ..
            } => {
                n = n.saturating_add(*cells);
                if matches!(number, NumPlan::Tag { .. }) {
                    n = n.saturating_add(1);
                }
            }
        }
    }
    n
}

impl MathNode {
    /// Gold-stable S-expression. One space between items; no trailing space.
    ///
    /// The walk is an explicit heap stack, so a tree `parse` accepted cannot
    /// overflow the caller's stack here.
    #[must_use]
    pub fn gold(&self) -> String {
        let mut stack = vec![GoldOp::Node(self)];
        let mut out = String::new();
        while let Some(op) = stack.pop() {
            match op {
                GoldOp::Text(text) => out.push_str(&text),
                GoldOp::Node(node) => {
                    let mut forward = Vec::new();
                    gold_expand(node, &mut forward);
                    for op in forward.into_iter().rev() {
                        stack.push(op);
                    }
                }
            }
        }
        out
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
    /// A [`MathNode::Fraction`] prints as the command in its [`FracStyle`].
    /// `\over` is stored as [`FracStyle::Frac`], so it prints as `\frac`.
    ///
    /// A [`MathNode::Color`] is its own group. A color nested in a row
    /// therefore does not swallow the nodes that follow it.
    #[must_use]
    pub fn to_latex(&self) -> String {
        let mut buf = LatexBuf::new();
        let mut stack = vec![LatexOp::Node(self)];
        while let Some(op) = stack.pop() {
            match op {
                LatexOp::Text(text) => buf.out.push_str(&text),
                LatexOp::Node(node) => {
                    let mut forward = LatexBuf::new();
                    write_node(&mut forward, node);
                    for op in forward.ops.into_iter().rev() {
                        stack.push(op);
                    }
                }
            }
        }
        buf.out
    }
}

enum LatexOp<'a> {
    Node(&'a MathNode),
    Text(String),
}

struct LatexBuf<'a> {
    ops: Vec<LatexOp<'a>>,
    out: String,
}

impl<'a> LatexBuf<'a> {
    fn new() -> Self {
        Self {
            ops: Vec::new(),
            out: String::new(),
        }
    }

    fn push_str(&mut self, text: &str) {
        if let Some(LatexOp::Text(prev)) = self.ops.last_mut() {
            prev.push_str(text);
            return;
        }
        self.ops.push(LatexOp::Text(text.to_string()));
    }

    fn push(&mut self, c: char) {
        if let Some(LatexOp::Text(prev)) = self.ops.last_mut() {
            prev.push(c);
            return;
        }
        self.ops.push(LatexOp::Text(c.to_string()));
    }

    fn node(&mut self, node: &'a MathNode) {
        self.ops.push(LatexOp::Node(node));
    }
}

enum GoldOp<'a> {
    Node(&'a MathNode),
    Text(String),
}

fn gold_text(buf: &mut Vec<GoldOp<'_>>, text: impl Into<String>) {
    buf.push(GoldOp::Text(text.into()));
}

fn gold_node<'a>(buf: &mut Vec<GoldOp<'a>>, node: &'a MathNode) {
    buf.push(GoldOp::Node(node));
}

fn gold_opt<'a>(buf: &mut Vec<GoldOp<'a>>, node: &'a Option<Box<MathNode>>) {
    match node {
        None => gold_text(buf, "_"),
        Some(node) => gold_node(buf, node),
    }
}

fn gold_expand<'a>(node: &'a MathNode, buf: &mut Vec<GoldOp<'a>>) {
    match node {
        MathNode::Atom(c, k) => gold_text(buf, format!("(atom {} {})", k.gold(), quote_atom(*c))),
        MathNode::Fraction(style, n, d) => {
            gold_text(buf, format!("({} ", style.gold()));
            gold_node(buf, n);
            gold_text(buf, " ");
            gold_node(buf, d);
            gold_text(buf, ")");
        }
        MathNode::Radical(None, r) => {
            gold_text(buf, "(sqrt ");
            gold_node(buf, r);
            gold_text(buf, ")");
        }
        MathNode::Radical(Some(i), r) => {
            gold_text(buf, "(sqrtn ");
            gold_node(buf, i);
            gold_text(buf, " ");
            gold_node(buf, r);
            gold_text(buf, ")");
        }
        MathNode::Superscript(b, e) => {
            gold_text(buf, "(sup ");
            gold_node(buf, b);
            gold_text(buf, " ");
            gold_node(buf, e);
            gold_text(buf, ")");
        }
        MathNode::Subscript(b, s) => {
            gold_text(buf, "(sub ");
            gold_node(buf, b);
            gold_text(buf, " ");
            gold_node(buf, s);
            gold_text(buf, ")");
        }
        MathNode::SubSup(b, s, e) => {
            gold_text(buf, "(subsup ");
            gold_node(buf, b);
            gold_text(buf, " ");
            gold_node(buf, s);
            gold_text(buf, " ");
            gold_node(buf, e);
            gold_text(buf, ")");
        }
        MathNode::Delimited(l, b, r) => {
            gold_text(buf, format!("(delim {} ", l.gold()));
            gold_node(buf, b);
            gold_text(buf, format!(" {})", r.gold()));
        }
        MathNode::SizedDelim(d, sz, k) => {
            gold_text(
                buf,
                format!("(big {} {} {})", sz.gold(), k.gold(), quote_delim(d)),
            );
        }
        MathNode::Row(items) => {
            if items.is_empty() {
                gold_text(buf, "(row)");
            } else {
                gold_text(buf, "(row");
                for it in items {
                    gold_text(buf, " ");
                    gold_node(buf, it);
                }
                gold_text(buf, ")");
            }
        }
        MathNode::Matrix(style, spec, rows) => {
            let mut head = format!("(matrix {}", style.gold());
            if !spec.is_empty() {
                head.push(' ');
                for c in spec {
                    head.push(c.gold());
                }
            }
            gold_text(buf, head);
            for row in rows {
                gold_text(buf, " ");
                gold_row(buf, row);
            }
            gold_text(buf, ")");
        }
        MathNode::Substack(lines) => {
            gold_text(buf, "(substack");
            for ln in lines {
                gold_text(buf, " ");
                gold_node(buf, ln);
            }
            gold_text(buf, ")");
        }
        MathNode::Ref(k) => gold_text(buf, format!("(ref {k})")),
        MathNode::Tag { star: false, body } => {
            gold_text(buf, "(tag ");
            gold_node(buf, body);
            gold_text(buf, ")");
        }
        MathNode::Tag { star: true, body } => {
            gold_text(buf, "(tagstar ");
            gold_node(buf, body);
            gold_text(buf, ")");
        }
        MathNode::Label(k) => gold_text(buf, format!("(label {k})")),
        MathNode::NoNumber => gold_text(buf, "(nonumber)"),
        MathNode::Hline => gold_text(buf, "(hline)"),
        MathNode::Intertext(n) => {
            gold_text(buf, "(intertext ");
            gold_node(buf, n);
            gold_text(buf, ")");
        }
        MathNode::Sum(lo, hi) => {
            gold_text(buf, "(sum ");
            gold_opt(buf, lo);
            gold_text(buf, " ");
            gold_opt(buf, hi);
            gold_text(buf, ")");
        }
        MathNode::Integral(k, lo, hi) => {
            gold_text(buf, format!("({} ", k.gold()));
            gold_opt(buf, lo);
            gold_text(buf, " ");
            gold_opt(buf, hi);
            gold_text(buf, ")");
        }
        MathNode::Product(lo, hi) => {
            gold_text(buf, "(prod ");
            gold_opt(buf, lo);
            gold_text(buf, " ");
            gold_opt(buf, hi);
            gold_text(buf, ")");
        }
        MathNode::Limit(lo) => {
            gold_text(buf, "(lim ");
            gold_opt(buf, lo);
            gold_text(buf, ")");
        }
        MathNode::OverUnder(b, over, under) => {
            gold_text(buf, "(overunder ");
            gold_node(buf, b);
            gold_text(buf, " ");
            gold_opt(buf, over);
            gold_text(buf, " ");
            gold_opt(buf, under);
            gold_text(buf, ")");
        }
        MathNode::Accent(b, a) => {
            gold_text(buf, format!("(accent {} ", a.gold()));
            gold_node(buf, b);
            gold_text(buf, ")");
        }
        MathNode::CancelTo(v, e) => {
            gold_text(buf, "(cancelto ");
            gold_node(buf, v);
            gold_text(buf, " ");
            gold_node(buf, e);
            gold_text(buf, ")");
        }
        MathNode::Text(t, st) => {
            let plain = t
                .items()
                .iter()
                .all(|item| matches!(item, TextItem::Str(_) | TextItem::Space));
            if plain {
                gold_text(
                    buf,
                    format!("(text {} {})", st.gold(), quote_text(&t.to_plain_string())),
                );
            } else {
                gold_text(buf, format!("(text {} ", st.gold()));
                gold_run(buf, t);
                gold_text(buf, ")");
            }
        }
        MathNode::Space(SpaceKind::Thin) => gold_text(buf, "(space thin)"),
        MathNode::Space(SpaceKind::Medium) => gold_text(buf, "(space medium)"),
        MathNode::Space(SpaceKind::Thick) => gold_text(buf, "(space thick)"),
        MathNode::Space(SpaceKind::NegThin) => gold_text(buf, "(space negthin)"),
        MathNode::Space(SpaceKind::Quad) => gold_text(buf, "(space quad)"),
        MathNode::Space(SpaceKind::Qquad) => gold_text(buf, "(space qquad)"),
        MathNode::Space(SpaceKind::ControlSpace) => gold_text(buf, "(space control)"),
        MathNode::Space(SpaceKind::Hspace(d)) => {
            gold_text(buf, format!("(space hspace {})", dim_gold(d)));
        }
        MathNode::Operator(name, false) => gold_text(buf, format!("(op {name})")),
        MathNode::Operator(name, true) => gold_text(buf, format!("(op {name} limits)")),
        MathNode::Symbol(name) => gold_text(buf, format!("(symbol {name})")),
        MathNode::Color(c, b) => {
            gold_text(buf, format!("(color {} ", c.css_hex()));
            gold_node(buf, b);
            gold_text(buf, ")");
        }
        MathNode::TextColor(c, b) => {
            gold_text(buf, format!("(textcolor {} ", c.css_hex()));
            gold_node(buf, b);
            gold_text(buf, ")");
        }
        MathNode::ColorBox(c, b) => {
            gold_text(buf, format!("(colorbox {} ", c.css_hex()));
            gold_node(buf, b);
            gold_text(buf, ")");
        }
        MathNode::FColorBox(border, fill, b) => {
            gold_text(
                buf,
                format!("(fcolorbox {} {} ", border.css_hex(), fill.css_hex()),
            );
            gold_node(buf, b);
            gold_text(buf, ")");
        }
        MathNode::Strut(h, d) => {
            gold_text(buf, format!("(strut {} {})", dim_gold(h), dim_gold(d)));
        }
        MathNode::Phantom(k, b) => {
            gold_text(buf, format!("({} ", k.gold()));
            gold_node(buf, b);
            gold_text(buf, ")");
        }
    }
}

fn gold_row<'a>(buf: &mut Vec<GoldOp<'a>>, row: &'a EnvRow) {
    match row {
        EnvRow::Hline => gold_text(buf, "(hline)"),
        EnvRow::Intertext(n) => {
            gold_text(buf, "(intertext ");
            gold_node(buf, n);
            gold_text(buf, ")");
        }
        EnvRow::Cells {
            cells,
            number,
            labels,
        } => {
            gold_text(buf, "(");
            for (i, cell) in cells.iter().enumerate() {
                if i > 0 {
                    gold_text(buf, " ");
                }
                gold_node(buf, cell);
            }
            match number {
                EqNumber::Default => {}
                EqNumber::Suppress => gold_text(buf, " (nonumber)"),
                EqNumber::Tag { star: false, body } => {
                    gold_text(buf, " (tag ");
                    gold_node(buf, body);
                    gold_text(buf, ")");
                }
                EqNumber::Tag { star: true, body } => {
                    gold_text(buf, " (tagstar ");
                    gold_node(buf, body);
                    gold_text(buf, ")");
                }
            }
            for lab in labels {
                gold_text(buf, format!(" (label {lab})"));
            }
            gold_text(buf, ")");
        }
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

/// `(run item…)` for a text run that is not plain characters and spaces.
fn gold_run<'a>(buf: &mut Vec<GoldOp<'a>>, run: &'a TextRun) {
    gold_text(buf, "(run");
    for item in run.items() {
        match item {
            TextItem::Str(s) => gold_text(buf, format!(" {}", quote_text(s))),
            TextItem::Space => gold_text(buf, " space"),
            TextItem::ControlSpace => gold_text(buf, " ctrlspace"),
            TextItem::NoBreakSpace => gold_text(buf, " nbsp"),
            TextItem::Symbol(n) => gold_text(buf, format!(" (symbol {n})")),
            TextItem::Nested(st, r) => {
                gold_text(buf, format!(" (nested {} ", st.gold()));
                gold_run(buf, r);
                gold_text(buf, ")");
            }
            TextItem::Math(m) => {
                gold_text(buf, " (math ");
                gold_node(buf, m);
                gold_text(buf, ")");
            }
        }
    }
    gold_text(buf, ")");
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

fn write_node<'a>(out: &mut LatexBuf<'a>, node: &'a MathNode) {
    match node {
        MathNode::Atom(c, _) => write_atom(out, *c),
        MathNode::Fraction(style, num, den) => {
            write_cmd(out, style.gold());
            write_group(out, num);
            write_group(out, den);
        }
        MathNode::Radical(index, body) => {
            write_cmd(out, "sqrt");
            if let Some(index) = index {
                out.push('[');
                out.node(index);
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
            out.node(body);
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
                out.node(line);
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
            if *style == TextStyle::Text
                || text
                    .as_str()
                    .is_some_and(|t| t.chars().all(|c| c.is_ascii_alphabetic()))
            {
                write_cmd(out, style_cmd(*style));
                out.push('{');
                push_escaped(out, &text.to_plain_string());
                out.push('}');
            } else {
                write_cmd(out, style_cmd(*style));
                out.push('{');
                write_cmd(out, "text");
                out.push('{');
                push_escaped(out, &text.to_plain_string());
                out.push('}');
                out.push('}');
            }
        }
        MathNode::Space(kind) => write_space(out, kind),
        MathNode::Operator(name, limits) => write_operator(out, name, *limits),
        MathNode::Symbol(name) => write_cmd(out, name),
        MathNode::Color(color, body) => {
            out.push('{');
            write_cmd(out, "color");
            push_html(out, *color);
            out.node(body);
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

/// The items of a row of two or more primes of one spelling (all U+2032
/// atoms, or all `\prime` symbols). Such a run prints unbraced inside one
/// `^{…}` group, so its reprint costs one brace level for any run length.
fn prime_run(node: &MathNode) -> Option<&[MathNode]> {
    match node {
        MathNode::Row(items)
            if items.len() >= 2
                && (items
                    .iter()
                    .all(|item| matches!(item, MathNode::Atom('′', AtomKind::Ord)))
                    || items
                        .iter()
                        .all(|item| matches!(item, MathNode::Symbol(name) if name == "prime"))) =>
        {
            Some(items)
        }
        _ => None,
    }
}

fn write_scripts<'a>(
    out: &mut LatexBuf<'a>,
    base: &'a MathNode,
    sub: Option<&'a MathNode>,
    sup: Option<&'a MathNode>,
) {
    write_group(out, base);
    if let Some(sub) = sub {
        out.push('_');
        write_group(out, sub);
    }
    if let Some(sup) = sup {
        out.push('^');
        match prime_run(sup) {
            Some(items) => {
                out.push('{');
                for item in items {
                    out.node(item);
                }
                out.push('}');
            }
            None => write_group(out, sup),
        }
    }
}

fn write_op_scripts<'a>(
    out: &mut LatexBuf<'a>,
    name: &str,
    sub: Option<&'a MathNode>,
    sup: Option<&'a MathNode>,
) {
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

fn write_group<'a>(out: &mut LatexBuf<'a>, node: &'a MathNode) {
    out.push('{');
    out.node(node);
    out.push('}');
}

/// Brace each child so a nested row stays nested, and so `<` `=` and a run of
/// letters are not glued back into a relation or a bare operator.
fn write_row<'a>(out: &mut LatexBuf<'a>, items: &'a [MathNode]) {
    if items.is_empty() {
        out.push_str("{}");
        return;
    }
    for item in items {
        write_group(out, item);
    }
}

fn write_matrix<'a>(
    out: &mut LatexBuf<'a>,
    style: MatrixStyle,
    spec: &[ColSpec],
    rows: &'a [EnvRow],
) {
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
                    out.node(cell);
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

fn write_eq_meta<'a>(out: &mut LatexBuf<'a>, number: &'a EqNumber, labels: &[String]) {
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

fn write_over_under<'a>(
    out: &mut LatexBuf<'a>,
    base: &'a MathNode,
    over: Option<&'a MathNode>,
    under: Option<&'a MathNode>,
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
                out.node(under);
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
        (None, None) => out.node(base),
    }
}

fn write_intertext<'a>(out: &mut LatexBuf<'a>, body: &'a MathNode) {
    write_cmd(out, "intertext");
    out.push('{');
    if let MathNode::Text(text, _) = body {
        push_escaped(out, &text.to_plain_string());
    } else {
        out.node(body);
    }
    out.push('}');
}

fn write_paint<'a>(
    out: &mut LatexBuf<'a>,
    cmd: &str,
    first: Color,
    second: Option<Color>,
    body: &'a MathNode,
) {
    write_cmd(out, cmd);
    push_html(out, first);
    if let Some(second) = second {
        push_html(out, second);
    }
    write_group(out, body);
}

fn write_operator(out: &mut LatexBuf<'_>, name: &str, limits: bool) {
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

fn write_space(out: &mut LatexBuf<'_>, kind: &SpaceKind) {
    match kind {
        SpaceKind::Thin => out.push_str("\\,"),
        SpaceKind::Medium => out.push_str("\\:"),
        SpaceKind::Thick => out.push_str("\\;"),
        SpaceKind::NegThin => out.push_str("\\!"),
        SpaceKind::Quad => write_cmd(out, "quad"),
        SpaceKind::Qquad => write_cmd(out, "qquad"),
        // The space is the control symbol, not disposable padding. `preprocess`
        // keeps one trailing ASCII space after an odd run of backslashes, so
        // this reprint parses again. Do not drop the space or replace it with
        // a token that trimming would delete.
        SpaceKind::ControlSpace => out.push_str("\\ "),
        SpaceKind::Hspace(dim) => {
            write_cmd(out, "hspace");
            out.push('{');
            out.push_str(&dim.to_dec_string());
            out.push_str("em}");
        }
    }
}

fn write_strut(out: &mut LatexBuf<'_>, height: &Dim, depth: &Dim) {
    if height.eq_dim(&Dim::ratio(7, 10)) && depth.eq_dim(&Dim::ratio(3, 10)) {
        write_cmd(out, "strut");
        return;
    }
    write_cmd(out, "rule");
    out.push_str("{0em}{");
    out.push_str(&height.to_dec_string());
    out.push_str("em}");
}

fn write_atom(out: &mut LatexBuf<'_>, c: char) {
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

fn write_delim(out: &mut LatexBuf<'_>, delim: &Delimiter) {
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

fn write_cmd(out: &mut LatexBuf<'_>, name: &str) {
    out.push('\\');
    out.push_str(name);
    if !name.is_empty() && name.chars().all(|c| c.is_ascii_alphabetic()) {
        out.push(' ');
    }
}

fn write_braced_text(out: &mut LatexBuf<'_>, cmd: &str, text: &str) {
    write_cmd(out, cmd);
    out.push('{');
    push_escaped(out, text);
    out.push('}');
}

/// Escapes special LaTeX characters in text.
///
/// The first character is escaped if it is a space, '[', or ']'.
/// All characters are escaped if they are '\', '{', '}', '%', '$', '&', '_', or '^'.
fn push_escaped(out: &mut LatexBuf<'_>, text: &str) {
    for (i, c) in text.chars().enumerate() {
        match c {
            ' ' | '[' | ']' if i == 0 => {
                out.push('\\');
                out.push(c);
            }
            '\\' | '{' | '}' | '%' | '$' | '&' | '_' | '^' => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
}

fn push_html(out: &mut LatexBuf<'_>, color: Color) {
    out.push_str("[HTML]{");
    let hex = color.css_hex();
    match hex.strip_prefix('#') {
        Some(body) => out.push_str(body),
        None => out.push_str(&hex),
    }
    out.push('}');
}
