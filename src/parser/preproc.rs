//! String-level LaTeX math sanitizer run before tokenization.
//!
//! Rewrites that preserve math meaning for a renderer: `{a \over b}` →
//! `\frac`, `{n \choose k}` → `\binom`, `\tfrac`/`\dfrac` → `\frac`, plain-TeX
//! font switches, `\mbox` → `\text`. Extensible `\left`/`\right`, skips, and
//! accents are left intact.

/// Normalize raw math input for the tokenizer.
///
/// Rewrites that preserve math meaning: `{a \over b}` → `\frac`, `{n \choose k}`
/// → `\binom`, `\tfrac`/`\dfrac` → `\frac`, plain-TeX font switches, `\mbox` →
/// `\text`.
///
/// # Examples
///
/// ```
/// use latex_rust::preprocess;
///
/// assert!(preprocess(r"{a \over b}").contains(r"\frac"));
/// ```
#[must_use]
pub fn preprocess(raw_input: &str) -> String {
    let input = raw_input.trim().to_string();
    if input.is_empty() {
        return input;
    }

    let mut input = input;
    input = input.replace(r"{ (}", "(").replace(r"{ )}", ")");
    input = input.replace(r"{ ( }", "(").replace(r"{ ) }", ")");
    input = input.replace(r"{[}", "[").replace(r"{]}", "]");

    input = input
        .replace(r"\tfrac", r"\frac")
        .replace(r"\dfrac", r"\frac")
        .replace(r"\cfrac", r"\frac");

    input = convert_plain_tex_font_scopes(&input);
    input = convert_over_fractions(&input);
    input = convert_choose(&input);
    input = input.replace(r"\mbox{", r"\text{");
    input
}

fn convert_plain_tex_font_scopes(input: &str) -> String {
    let map = [
        (r"\rm", r"\mathrm"),
        (r"\bf", r"\mathbf"),
        (r"\cal", r"\mathcal"),
        (r"\it", r"\mathit"),
        (r"\sf", r"\mathsf"),
        (r"\tt", r"\mathtt"),
    ];
    let mut result = input.to_string();
    for (old, new) in map {
        let mut search = 0;
        while let Some(rel) = result[search..].find(old) {
            let abs = search + rel;
            let body = abs + old.len();
            if abs > 0 && result.as_bytes()[abs - 1] == b'{' {
                if let Some(close) = result[abs..].find('}') {
                    let end = abs + close;
                    let content = result[body..end].trim();
                    let rep = format!("{new}{{{content}}}");
                    result.replace_range((abs - 1)..=end, &rep);
                    search = abs - 1 + rep.len();
                    continue;
                }
            }
            search = abs + old.len();
        }
    }
    result
}

/// `{a \over b}` → `\frac{a}{b}`.
///
/// A hit is a whole control word: the character after `\over` is not a letter,
/// so `\overline` stays an accent. The input is scanned once from the left.
/// A second unbraced `\over` in that group is peeled in the same pass, which
/// is what the old leftmost search produced, without reading the input again.
/// `\choose` is a later pass.
fn convert_over_fractions(input: &str) -> String {
    convert_infix(input, InfixKind::Over)
}

/// `{n \choose k}` → `\binom{n}{k}`, same whole-command scan as `\over`.
fn convert_choose(input: &str) -> String {
    convert_infix(input, InfixKind::Choose)
}

fn convert_infix(input: &str, kind: InfixKind) -> String {
    let mut layers = Vec::new();
    layers.push(Layer::base());
    let mut i = 0;
    while i < input.len() {
        if input.as_bytes()[i] == b'\\' {
            if let Some(len) = match_infix(input, i, kind) {
                if start_denominator(&mut layers, kind) || peel(&mut layers, kind) {
                    i += len;
                    continue;
                }
            }
        }
        let Some(ch) = input[i..].chars().next() else {
            break;
        };
        i += ch.len_utf8();
        if ch == '}' && close_denominator(&mut layers) {
            continue;
        }
        push_char(&mut layers, ch);
    }
    collapse(layers)
}

#[derive(Clone, Copy)]
enum InfixKind {
    Over,
    Choose,
}

struct Pending {
    head: FracHead,
    /// Earlier unbraced commands in this same group. `{a \over b \over c}`
    /// emits `\frac{a}` before `\frac{b}{c}`, so the second fraction is the
    /// next nucleus rather than a braced denominator.
    earlier: Vec<FracHead>,
}

struct FracHead {
    kind: InfixKind,
    num: String,
}

impl FracHead {
    fn emit(&self, den: &str) -> String {
        let num = self.num.trim();
        match self.kind {
            InfixKind::Over => format!(r"\frac{{{}}}{{{}}}", num, den),
            InfixKind::Choose => format!(r"\binom{{{}}}{{{}}}", num, den),
        }
    }

    fn open_only(&self) -> String {
        let num = self.num.trim();
        match self.kind {
            InfixKind::Over => format!(r"\frac{{{}}}", num),
            InfixKind::Choose => format!(r"\binom{{{}}}", num),
        }
    }

    fn command(&self) -> &'static str {
        match self.kind {
            InfixKind::Over => "\\over",
            InfixKind::Choose => "\\choose",
        }
    }
}

struct Layer {
    out: String,
    braces: Vec<usize>,
    pending: Option<Pending>,
}

impl Layer {
    fn base() -> Self {
        Self {
            out: String::new(),
            braces: Vec::new(),
            pending: None,
        }
    }

    fn den(kind: InfixKind, num: String) -> Self {
        Self {
            out: String::new(),
            braces: Vec::new(),
            pending: Some(Pending {
                head: FracHead { kind, num },
                earlier: Vec::new(),
            }),
        }
    }
}

/// `kind` at `i`, only when the next character is not a letter.
fn match_infix(input: &str, i: usize, kind: InfixKind) -> Option<usize> {
    let word = match kind {
        InfixKind::Over => r"\over",
        InfixKind::Choose => r"\choose",
    };
    let rest = input[i..].strip_prefix(word)?;
    if continues_control_word(rest) {
        None
    } else {
        Some(word.len())
    }
}

fn continues_control_word(rest: &str) -> bool {
    matches!(rest.chars().next(), Some(c) if c.is_ascii_alphabetic())
}

/// The innermost `{` before a whole `\over` or `\choose` opens the fraction.
/// Its body, already copied, is the numerator. The following input is the
/// denominator and is scanned by a fresh layer so a nested command is rewritten
/// before this one closes.
fn start_denominator(layers: &mut Vec<Layer>, kind: InfixKind) -> bool {
    let Some(open) = layers.last().and_then(|layer| layer.braces.last().copied()) else {
        return false;
    };
    let num = {
        let Some(layer) = layers.last_mut() else {
            return false;
        };
        // Keep the raw numerator. `emit` trims a finished fraction. An unclosed
        // group is restored through `collapse`, and trimming here would drop
        // the space in `{a \over`.
        let num = layer.out[open + 1..].to_string();
        layer.out.truncate(open);
        layer.braces.pop();
        num
    };
    layers.push(Layer::den(kind, num));
    true
}

/// A second unbraced command in the denominator has no `{` of its own.
/// `{a \over b \over c}` becomes `\frac{a}\frac{b}{c}` in this same pass:
/// the text so far is the next numerator, and the previous numerator is
/// emitted with only its first argument.
fn peel(layers: &mut Vec<Layer>, kind: InfixKind) -> bool {
    let ready = matches!(
        layers.last(),
        Some(layer) if layer.pending.is_some() && layer.braces.is_empty()
    );
    if !ready || layers.len() < 2 {
        return false;
    }
    let Some(done) = layers.pop() else {
        return false;
    };
    let Some(pending) = done.pending else {
        layers.push(done);
        return false;
    };
    let Pending { head, mut earlier } = pending;
    earlier.push(head);
    layers.push(Layer {
        out: String::new(),
        braces: Vec::new(),
        pending: Some(Pending {
            head: FracHead {
                kind,
                num: done.out,
            },
            earlier,
        }),
    });
    true
}

/// A `}` that belongs to the fraction, not to a group inside the denominator.
fn close_denominator(layers: &mut Vec<Layer>) -> bool {
    let ready = matches!(
        layers.last(),
        Some(layer) if layer.pending.is_some() && layer.braces.is_empty()
    );
    if !ready || layers.len() < 2 {
        return false;
    }
    let Some(done) = layers.pop() else {
        return false;
    };
    let Some(pending) = done.pending else {
        layers.push(done);
        return false;
    };
    let mut rep = String::new();
    for open in &pending.earlier {
        rep.push_str(&open.open_only());
    }
    rep.push_str(&pending.head.emit(done.out.trim()));
    let Some(parent) = layers.last_mut() else {
        return false;
    };
    parent.out.push_str(&rep);
    true
}

fn push_char(layers: &mut [Layer], ch: char) {
    let Some(layer) = layers.last_mut() else {
        return;
    };
    match ch {
        '{' => layer.braces.push(layer.out.len()),
        '}' => {
            layer.braces.pop();
        }
        _ => {}
    }
    layer.out.push(ch);
}

/// An unclosed `{a \over` keeps the original command text.
fn collapse(mut layers: Vec<Layer>) -> String {
    while layers.len() > 1 {
        let Some(done) = layers.pop() else {
            break;
        };
        let Some(parent) = layers.last_mut() else {
            return done.out;
        };
        match done.pending {
            Some(pending) => {
                parent.out.push('{');
                for open in &pending.earlier {
                    parent.out.push_str(&open.num);
                    parent.out.push_str(open.command());
                }
                parent.out.push_str(&pending.head.num);
                parent.out.push_str(pending.head.command());
                parent.out.push_str(&done.out);
            }
            None => parent.out.push_str(&done.out),
        }
    }
    match layers.pop() {
        Some(layer) => layer.out,
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn left_right_kept() {
        let s = preprocess(r"\left(\frac{1}{2}\right)");
        assert!(s.contains(r"\left"));
        assert!(s.contains(r"\right"));
    }

    #[test]
    fn tfrac_becomes_frac() {
        assert_eq!(preprocess(r"\tfrac{a}{b}"), r"\frac{a}{b}");
    }

    #[test]
    fn over_becomes_frac() {
        assert_eq!(preprocess(r"{a \over b}"), r"\frac{a}{b}");
        assert_eq!(
            preprocess(r"{a \over {b \over c}}"),
            r"\frac{a}{\frac{b}{c}}"
        );
        assert_eq!(
            preprocess(r"{ {a \over b} \over c }"),
            r"\frac{\frac{a}{b}}{c}"
        );
        assert_eq!(
            preprocess(r"{a \over b}{c \over d}"),
            r"\frac{a}{b}\frac{c}{d}"
        );
        // A second unbraced command is peeled in the same pass, so the fractions
        // sit side by side. `\frac`'s second argument is still the next nucleus.
        assert_eq!(preprocess(r"{a \over b \over c}"), r"\frac{a}\frac{b}{c}");
        assert_eq!(
            preprocess(r"{a \over b \over c \over d}"),
            r"\frac{a}\frac{b}\frac{c}{d}"
        );
        assert_eq!(
            preprocess(r"{a \over b \over {c \over d}}"),
            r"\frac{a}\frac{b}{\frac{c}{d}}"
        );
        assert_eq!(
            preprocess(r"{a \over {b \over c}+d}"),
            r"\frac{a}{\frac{b}{c}+d}"
        );
        assert_eq!(
            preprocess(r"{a \over b+{c \over d}}"),
            r"\frac{a}{b+\frac{c}{d}}"
        );
        assert_eq!(
            preprocess(r"{a \over b \over c+d}"),
            r"\frac{a}\frac{b}{c+d}"
        );
        assert_eq!(preprocess(r"{a \over}"), r"\frac{a}{}");
        assert_eq!(preprocess(r"{a \over"), r"{a \over");
        assert_eq!(preprocess(r"{a \over b \over"), r"{a \over b \over");
        assert_eq!(preprocess(r"a \over b"), r"a \over b");
    }

    #[test]
    fn trailing_period_kept() {
        assert_eq!(preprocess("x^2."), "x^2.");
    }

    #[test]
    fn empty() {
        assert_eq!(preprocess("   "), "");
        assert!(matches!(
            crate::parse(""),
            Err(crate::ParseError::Malformed(_))
        ));
        assert!(matches!(
            crate::parse("   "),
            Err(crate::ParseError::Malformed(_))
        ));
    }

    #[test]
    fn overline_kept() {
        assert_eq!(preprocess(r"\overline{z}"), r"\overline{z}");
        assert_eq!(
            preprocess(r"\frac{\overline{x}}{2}"),
            r"\frac{\overline{x}}{2}"
        );
        assert_eq!(preprocess(r"{\overset{a}{b}}"), r"{\overset{a}{b}}");
    }

    #[test]
    fn choose_becomes_binom() {
        assert_eq!(preprocess(r"{n \choose k}"), r"\binom{n}{k}");
        assert_eq!(
            preprocess(r"{n \choose k \choose m}"),
            r"\binom{n}\binom{k}{m}"
        );
        // `\over` is rewritten before `\choose`, matching the previous two
        // passes. Mixed groups keep that order.
        assert_eq!(
            preprocess(r"{a \choose b \over c}"),
            r"\frac\binom{a}{b}{c}"
        );
        assert_eq!(
            preprocess(r"{a \over b \choose c}"),
            r"\frac{a}\binom{b}{c}"
        );
        assert_eq!(
            preprocess(r"{a \choose {b \over c}}"),
            r"\binom{a}{\frac{b}{c}}"
        );
        assert_eq!(
            preprocess(r"{a \over {b \choose c}}"),
            r"\frac{a}{\binom{b}{c}}"
        );
    }

    #[test]
    fn mbox_becomes_text() {
        assert_eq!(preprocess(r"\mbox{Diagonal}"), r"\text{Diagonal}");
    }

    #[test]
    fn spacing_kept() {
        let s = preprocess(r"a\,b");
        assert!(s.contains(r"\,"));
    }
}
