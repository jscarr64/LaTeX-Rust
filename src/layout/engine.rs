//! `MathNode` → `MathBox`. All sizes are [`Dim`](crate::Dim) (zenith-float 1.0).

use core::cmp::Ordering;

use crate::color::Color;
use crate::dim::Dim;
use crate::error::Error;
use crate::font::MathFont;
use crate::layout::metrics::MathParams;
use crate::layout::space::{atom_space_mu, convert_bin, space_width};
use crate::layout::style::MathStyle;
use crate::layout::{BoxContent, MathBox};
use crate::parser::{
    AccentKind, AtomKind, DelimSize, Delimiter, IntegralKind, MathNode, MatrixStyle, PhantomKind,
    SpaceKind,
};
use crate::symbols::lookup;

/// Layout `node` in `style` using STIX Two Math metrics.
pub fn layout(node: &MathNode, font: &MathFont, style: MathStyle) -> Result<MathBox, Error> {
    let params = MathParams::from_font(font)?;
    Engine { font, params }.layout(node, style)
}

struct Engine<'a> {
    font: &'a MathFont,
    params: MathParams,
}

struct Item {
    bx: MathBox,
    class: Option<AtomKind>,
}

impl Engine<'_> {
    fn layout(&self, node: &MathNode, style: MathStyle) -> Result<MathBox, Error> {
        Ok(self.item(node, style)?.bx)
    }

    fn item(&self, node: &MathNode, style: MathStyle) -> Result<Item, Error> {
        match node {
            MathNode::Atom(c, k) => {
                let bx = self.glyph(*c, style)?;
                Ok(Item {
                    bx,
                    class: Some(*k),
                })
            }
            MathNode::Symbol(name) => {
                let ch = symbol_char(name)?;
                let bx = self.glyph(ch, style)?;
                Ok(Item {
                    bx,
                    class: Some(symbol_class(name)),
                })
            }
            MathNode::Row(items) => self.row(items, style),
            MathNode::Fraction(num, den) => self.fraction(num, den, style),
            MathNode::Radical(deg, rad) => self.radical(deg.as_deref(), rad, style),
            MathNode::Superscript(base, exp) => self.scripts(base, None, Some(exp), style),
            MathNode::Subscript(base, sub) => self.scripts(base, Some(sub), None, style),
            MathNode::SubSup(base, sub, exp) => self.scripts(base, Some(sub), Some(exp), style),
            MathNode::Delimited(open, body, close) => self.delimited(open, body, close, style),
            MathNode::SizedDelim(d, size, k) => {
                let needed = self.explicit_delim_span(*size, style);
                Ok(Item {
                    class: Some(*k),
                    bx: self.delim_box(d, &needed, style)?,
                })
            }
            MathNode::Space(kind) => Ok(Item {
                bx: MathBox::kern(self.space_dim(kind, style)),
                class: None,
            }),
            MathNode::Strut(h, d) => {
                let s = self.params.scale(style);
                Ok(Item {
                    bx: MathBox {
                        width: Dim::zero(),
                        height: h * &s,
                        depth: d * &s,
                        italic: Dim::zero(),
                        content: BoxContent::Empty,
                    },
                    class: Some(AtomKind::Ord),
                })
            }
            MathNode::Phantom(kind, inner) => {
                let mut bx = self.layout(inner, style)?;
                match kind {
                    PhantomKind::Full => bx.content = BoxContent::Empty,
                    PhantomKind::Vertical => {
                        bx.width = Dim::zero();
                        bx.content = BoxContent::Empty;
                    }
                    PhantomKind::Horizontal => {
                        bx.height = Dim::zero();
                        bx.depth = Dim::zero();
                        bx.content = BoxContent::Empty;
                    }
                }
                Ok(Item {
                    class: class_of(inner),
                    bx,
                })
            }
            MathNode::Text(s, _) => self.text_run(s, style),
            MathNode::Operator(name, limits) => self.operator(name, *limits, style),
            MathNode::Sum(lo, hi) => self.large_op('∑', lo.as_deref(), hi.as_deref(), style, true),
            MathNode::Product(lo, hi) => {
                self.large_op('∏', lo.as_deref(), hi.as_deref(), style, true)
            }
            MathNode::Integral(k, lo, hi) => {
                let ch = match k {
                    IntegralKind::Int => '∫',
                    IntegralKind::Iint => '∬',
                    IntegralKind::Iiint => '∭',
                    IntegralKind::Oint => '∮',
                    IntegralKind::Oiint => '∯',
                };
                self.large_op(ch, lo.as_deref(), hi.as_deref(), style, false)
            }
            MathNode::Limit(sub) => {
                let op = self.text_run("lim", style)?;
                if let Some(s) = sub {
                    self.attach_limits(op.bx, None, Some(s), style, true)
                } else {
                    Ok(Item {
                        bx: op.bx,
                        class: Some(AtomKind::Op),
                    })
                }
            }
            MathNode::OverUnder(base, over, under) => {
                let b = self.layout(base, style)?;
                self.attach_limits(b, over.as_deref(), under.as_deref(), style, true)
            }
            MathNode::Accent(base, kind) => self.accent(base, *kind, style),
            MathNode::Matrix(ms, rows) => self.matrix(*ms, rows, style),
            MathNode::Color(c, body) | MathNode::TextColor(c, body) => {
                let inner = self.layout(body, style)?;
                Ok(Item {
                    class: class_of(body),
                    bx: color_wrap(*c, inner),
                })
            }
            MathNode::ColorBox(c, body) => {
                let inner = self.layout(body, style)?;
                let pad = self.params.mu(style) * Dim::from_i64(3);
                Ok(Item {
                    class: Some(AtomKind::Inner),
                    bx: color_wrap(*c, pad_box(inner, &pad)),
                })
            }
            MathNode::FColorBox(_border, fill, body) => {
                let inner = self.layout(body, style)?;
                let pad = self.params.mu(style) * Dim::from_i64(3);
                Ok(Item {
                    class: Some(AtomKind::Inner),
                    bx: color_wrap(*fill, pad_box(inner, &pad)),
                })
            }
        }
    }

    fn glyph(&self, ch: char, style: MathStyle) -> Result<MathBox, Error> {
        let g = self.font.glyph(ch)?;
        let s = self.params.scale(style);
        let italic = self.font.italic_correction(g.glyph_id);
        Ok(MathBox {
            width: &g.advance * &s,
            height: &g.height * &s,
            depth: &g.depth * &s,
            italic: &italic * &s,
            content: BoxContent::Glyph {
                ch,
                glyph_id: g.glyph_id,
            },
        })
    }

    fn glyph_id(&self, ch: char, gid: u16, style: MathStyle) -> Result<MathBox, Error> {
        let g = self.font.glyph_id(ch, gid)?;
        let s = self.params.scale(style);
        let italic = self.font.italic_correction(gid);
        Ok(MathBox {
            width: &g.advance * &s,
            height: &g.height * &s,
            depth: &g.depth * &s,
            italic: &italic * &s,
            content: BoxContent::Glyph { ch, glyph_id: gid },
        })
    }

    fn space_dim(&self, kind: &SpaceKind, style: MathStyle) -> Dim {
        let mu = self.params.mu(style);
        match kind {
            SpaceKind::Thin => mu * Dim::from_i64(3),
            SpaceKind::Medium => mu * Dim::from_i64(4),
            SpaceKind::Thick => mu * Dim::from_i64(5),
            SpaceKind::NegThin => -(mu * Dim::from_i64(3)),
            SpaceKind::Quad => self.params.em(style),
            SpaceKind::Qquad => self.params.em(style) * Dim::from_i64(2),
            SpaceKind::ControlSpace => self.params.em(style) / Dim::from_i64(3),
            SpaceKind::Hspace(d) => d * &self.params.scale(style),
        }
    }

    fn text_run(&self, s: &str, style: MathStyle) -> Result<Item, Error> {
        let mut kids = Vec::new();
        for c in s.chars() {
            if c == ' ' {
                kids.push(MathBox::kern(self.params.mu(style) * Dim::from_i64(4)));
            } else {
                kids.push(self.glyph(c, style)?);
            }
        }
        Ok(Item {
            bx: MathBox::hpack(kids),
            class: Some(AtomKind::Ord),
        })
    }

    fn operator(&self, name: &str, limits: bool, style: MathStyle) -> Result<Item, Error> {
        if let Some(ch) = single_glyph(name) {
            if !ch.is_ascii_alphabetic() {
                return self.large_op(ch, None, None, style, limits);
            }
        }
        self.text_run(name, style).map(|mut it| {
            it.class = Some(AtomKind::Op);
            it
        })
    }

    fn row(&self, items: &[MathNode], style: MathStyle) -> Result<Item, Error> {
        if items.is_empty() {
            return Ok(Item {
                bx: MathBox::empty(),
                class: Some(AtomKind::Ord),
            });
        }
        let mut laid = Vec::new();
        for n in items {
            laid.push(self.item(n, style)?);
        }
        let n = laid.len();
        let mut classes: Vec<Option<AtomKind>> = Vec::with_capacity(n);
        for i in 0..n {
            let prev = if i == 0 { None } else { laid[i - 1].class };
            let next = if i + 1 < n { laid[i + 1].class } else { None };
            classes.push(laid[i].class.map(|c| convert_bin(prev, c, next)));
        }
        let mut out = Vec::new();
        for i in 0..n {
            if i > 0 {
                if let (Some(l), Some(r)) = (classes[i - 1], classes[i]) {
                    let mu = atom_space_mu(l, r, style);
                    let w = space_width(mu, &self.params, style);
                    if !w.is_zero() {
                        out.push(MathBox::kern(w));
                    }
                }
            }
            out.push(laid[i].bx.clone());
        }
        let class = if n == 1 {
            classes[0]
        } else {
            Some(AtomKind::Ord)
        };
        Ok(Item {
            bx: MathBox::hpack(out),
            class,
        })
    }

    fn fraction(&self, num: &MathNode, den: &MathNode, style: MathStyle) -> Result<Item, Error> {
        let num_b = self.layout(num, style.numerator())?;
        let den_b = self.layout(den, style.denominator())?;
        let s = self.params.scale(style);
        let axis = &self.params.axis_height * &s;
        let thick = &self.params.fraction_rule_thickness * &s;
        let half = &thick / &Dim::from_i64(2);
        let (shift_up0, shift_dn0, gap_num, gap_den) = if style.is_display() {
            (
                &self.params.fraction_numerator_display_style_shift_up * &s,
                &self.params.fraction_denominator_display_style_shift_down * &s,
                &self.params.fraction_num_display_style_gap_min * &s,
                &self.params.fraction_denom_display_style_gap_min * &s,
            )
        } else {
            (
                &self.params.fraction_numerator_shift_up * &s,
                &self.params.fraction_denominator_shift_down * &s,
                &self.params.fraction_numerator_gap_min * &s,
                &self.params.fraction_denominator_gap_min * &s,
            )
        };
        let num_shift = shift_up0.max(&(&axis + &half + &gap_num + &num_b.depth));
        let den_shift = shift_dn0.max(&(&den_b.height + &gap_den + &half - &axis).clamp_nonneg());
        let width = num_b.width.max(&den_b.width);
        let num_c = center_in(num_b, &width);
        let den_c = center_in(den_b, &width);
        let rule = MathBox::rule(width.clone(), &axis + &half, Dim::zero());
        Ok(Item {
            class: Some(AtomKind::Inner),
            bx: MathBox {
                width,
                height: &num_shift + &num_c.height,
                depth: &den_shift + &den_c.depth,
                italic: Dim::zero(),
                content: BoxContent::VList(vec![num_c, rule, den_c]),
            },
        })
    }

    fn radical(
        &self,
        deg: Option<&MathNode>,
        rad: &MathNode,
        style: MathStyle,
    ) -> Result<Item, Error> {
        let rad_b = self.layout(rad, style.cramp())?;
        let s = self.params.scale(style);
        let thick = &self.params.radical_rule_thickness * &s;
        let extra = &self.params.radical_extra_ascender * &s;
        let gap = if style.is_display() {
            &self.params.radical_display_style_vertical_gap * &s
        } else {
            &self.params.radical_vertical_gap * &s
        };
        let needed = &rad_b.height + &rad_b.depth + &gap + &thick + &extra;
        let surd = self.sized_glyph('√', &needed, style)?;
        let bar = MathBox::rule(
            rad_b.width.clone(),
            &rad_b.height + &gap + &thick + &extra,
            Dim::zero(),
        );
        let rad_col = MathBox {
            width: rad_b.width.clone(),
            height: &rad_b.height + &gap + &thick + &extra,
            depth: rad_b.depth.clone(),
            italic: Dim::zero(),
            content: BoxContent::VList(vec![bar, rad_b.clone()]),
        };
        let mut kids = vec![surd, rad_col];
        let mut width = MathBox::hpack(vec![kids[0].clone(), kids[1].clone()]).width;
        let height = (&rad_b.height + &gap + &thick + &extra).max(&kids[0].height);
        let depth = rad_b.depth.max(&kids[0].depth);
        if let Some(d) = deg {
            let db = self.layout(d, MathStyle::ScriptScript)?;
            let before = &self.params.radical_kern_before_degree * &s;
            let after = &self.params.radical_kern_after_degree * &s;
            let pct = Dim::from_i64(i64::from(self.params.radical_degree_bottom_raise_percent))
                / Dim::from_i64(100);
            let raise = &height * &pct;
            let deg_box = MathBox {
                width: db.width.clone(),
                height: &db.height + &raise,
                depth: (&db.depth - &raise).clamp_nonneg(),
                italic: Dim::zero(),
                content: db.content,
            };
            kids = vec![
                MathBox::kern(before),
                deg_box,
                MathBox::kern(after),
                kids[0].clone(),
                kids[1].clone(),
            ];
            width = MathBox::hpack(kids.clone()).width;
        }
        Ok(Item {
            class: Some(AtomKind::Inner),
            bx: MathBox {
                width,
                height,
                depth,
                italic: Dim::zero(),
                content: BoxContent::HList(kids),
            },
        })
    }

    fn scripts(
        &self,
        base: &MathNode,
        sub: Option<&MathNode>,
        sup: Option<&MathNode>,
        style: MathStyle,
    ) -> Result<Item, Error> {
        let base_it = self.item(base, style)?;
        self.attach_scripts_to_box(base_it.bx, base_it.class, sub, sup, style)
    }

    fn attach_scripts_to_box(
        &self,
        base: MathBox,
        class: Option<AtomKind>,
        sub: Option<&MathNode>,
        sup: Option<&MathNode>,
        style: MathStyle,
    ) -> Result<Item, Error> {
        if sub.is_none() && sup.is_none() {
            return Ok(Item { bx: base, class });
        }
        let s = self.params.scale(style);
        let ss = style.into_script();
        let after = &self.params.space_after_script * &self.params.scale(ss);
        let mut sup_shift = Dim::zero();
        let mut sub_shift = Dim::zero();
        let sup_laid = if let Some(e) = sup {
            sup_shift = if style.is_cramped() {
                &self.params.superscript_shift_up_cramped * &s
            } else {
                &self.params.superscript_shift_up * &s
            };
            Some(self.layout(e, ss.cramp())?)
        } else {
            None
        };
        let sub_laid = if let Some(u) = sub {
            sub_shift = &self.params.subscript_shift_down * &s;
            Some(self.layout(u, ss)?)
        } else {
            None
        };
        if let (Some(sp), Some(sb)) = (&sup_laid, &sub_laid) {
            let gap = &sup_shift + &sub_shift - &sp.depth - &sb.height;
            let min_gap = &self.params.sub_superscript_gap_min * &s;
            if matches!(gap.cmp(&min_gap), Some(Ordering::Less)) {
                sub_shift = &sub_shift + (&min_gap - &gap);
            }
        }
        let mut kids = vec![base.clone()];
        let mut width = base.width.clone();
        if sup_laid.is_some() && !base.italic.is_zero() {
            kids.push(MathBox::kern(base.italic.clone()));
            width = &width + &base.italic;
        }
        let mut slot_w = Dim::zero();
        let mut slot_h = Dim::zero();
        let mut slot_d = Dim::zero();
        let mut slot_kids = Vec::new();
        if let Some(sp) = sup_laid {
            let raised = MathBox {
                width: sp.width.clone(),
                height: &sp.height + &sup_shift,
                depth: (&sp.depth - &sup_shift).clamp_nonneg(),
                italic: Dim::zero(),
                content: sp.content,
            };
            slot_w = slot_w.max(&raised.width);
            slot_h = slot_h.max(&raised.height);
            slot_d = slot_d.max(&raised.depth);
            slot_kids.push(raised);
        }
        if let Some(sb) = sub_laid {
            let lowered = MathBox {
                width: sb.width.clone(),
                height: (&sb.height - &sub_shift).clamp_nonneg(),
                depth: &sb.depth + &sub_shift,
                italic: Dim::zero(),
                content: sb.content,
            };
            slot_w = slot_w.max(&lowered.width);
            slot_h = slot_h.max(&lowered.height);
            slot_d = slot_d.max(&lowered.depth);
            slot_kids.push(lowered);
        }
        kids.push(MathBox {
            width: slot_w.clone(),
            height: slot_h.clone(),
            depth: slot_d.clone(),
            italic: Dim::zero(),
            content: BoxContent::VList(slot_kids),
        });
        if !after.is_zero() {
            kids.push(MathBox::kern(after.clone()));
        }
        width = &width + &slot_w + &after;
        Ok(Item {
            class,
            bx: MathBox {
                width,
                height: base.height.max(&slot_h),
                depth: base.depth.max(&slot_d),
                italic: Dim::zero(),
                content: BoxContent::HList(kids),
            },
        })
    }

    fn delimited(
        &self,
        open: &Delimiter,
        body: &MathNode,
        close: &Delimiter,
        style: MathStyle,
    ) -> Result<Item, Error> {
        let body_b = self.layout(body, style)?;
        let s = self.params.scale(style);
        let axis = &self.params.axis_height * &s;
        let above = (&body_b.height - &axis).clamp_nonneg();
        let below = &body_b.depth + &axis;
        let needed = above.max(&below) * Dim::from_i64(2);
        let left = self.delim_box(open, &needed, style)?;
        let right = self.delim_box(close, &needed, style)?;
        Ok(Item {
            class: Some(AtomKind::Inner),
            bx: MathBox::hpack(vec![left, body_b, right]),
        })
    }

    fn explicit_delim_span(&self, size: DelimSize, style: MathStyle) -> Dim {
        let em = self.params.em(style);
        let n = match size {
            DelimSize::Big => 12,
            DelimSize::Big2 => 18,
            DelimSize::Bigg => 24,
            DelimSize::Bigg2 => 30,
        };
        em * Dim::from_i64(n) / Dim::from_i64(10)
    }

    fn delim_box(&self, d: &Delimiter, needed: &Dim, style: MathStyle) -> Result<MathBox, Error> {
        match d {
            Delimiter::Empty => Ok(MathBox::empty()),
            Delimiter::Char(c) => self.sized_glyph(*c, needed, style),
            Delimiter::Named(n) => {
                let ch = named_delim(n)?;
                self.sized_glyph(ch, needed, style)
            }
        }
    }

    fn sized_glyph(&self, ch: char, needed: &Dim, style: MathStyle) -> Result<MathBox, Error> {
        let base = self.font.glyph(ch)?;
        let mut best = self.glyph(ch, style)?;
        let mut best_span = &best.height + &best.depth;
        for gid in self.font.vertical_variants(base.glyph_id) {
            let cand = self.glyph_id(ch, gid, style)?;
            let span = &cand.height + &cand.depth;
            if span.cmp(needed).is_some_and(|o| o != Ordering::Less)
                && (best_span.cmp(needed).is_some_and(|o| o == Ordering::Less)
                    || span.cmp(&best_span).is_some_and(|o| o == Ordering::Less))
            {
                best_span = span;
                best = cand;
            } else if best_span.cmp(needed).is_some_and(|o| o == Ordering::Less)
                && span.cmp(&best_span).is_some_and(|o| o == Ordering::Greater)
            {
                best_span = span;
                best = cand;
            }
        }
        Ok(best)
    }

    fn large_op(
        &self,
        ch: char,
        lo: Option<&MathNode>,
        hi: Option<&MathNode>,
        style: MathStyle,
        limits_in_display: bool,
    ) -> Result<Item, Error> {
        let min_h = if style.is_display() {
            self.params.display_operator_min_height.clone() * self.params.scale(style)
        } else {
            Dim::zero()
        };
        let op = self.sized_glyph(ch, &min_h, style)?;
        let use_limits = limits_in_display && style.is_display();
        self.attach_limits(op, hi, lo, style, use_limits)
    }

    fn attach_limits(
        &self,
        op: MathBox,
        over: Option<&MathNode>,
        under: Option<&MathNode>,
        style: MathStyle,
        as_limits: bool,
    ) -> Result<Item, Error> {
        if !as_limits {
            return self.attach_scripts_to_box(op, Some(AtomKind::Op), under, over, style);
        }
        let s = self.params.scale(style);
        let mut width = op.width.clone();
        let mut height = op.height.clone();
        let mut depth = op.depth.clone();
        let mut kids = vec![op.clone()];
        if let Some(o) = over {
            let ob = self.layout(o, style.into_script())?;
            let gap = self.params.upper_limit_gap_min.clone() * &s;
            let rise = self.params.upper_limit_baseline_rise_min.clone() * &s;
            let extra = gap.max(&rise);
            width = width.max(&ob.width);
            height = &height + &ob.height + &ob.depth + &extra;
            kids.insert(0, center_in(ob, &width));
        }
        if let Some(u) = under {
            let ub = self.layout(u, style.into_script())?;
            let gap = self.params.lower_limit_gap_min.clone() * &s;
            let drop = self.params.lower_limit_baseline_drop_min.clone() * &s;
            let extra = gap.max(&drop);
            width = width.max(&ub.width);
            depth = &depth + &ub.height + &ub.depth + &extra;
            kids.push(center_in(ub, &width));
        }
        Ok(Item {
            class: Some(AtomKind::Op),
            bx: MathBox {
                width,
                height,
                depth,
                italic: Dim::zero(),
                content: BoxContent::VList(kids),
            },
        })
    }

    fn accent(&self, base: &MathNode, kind: AccentKind, style: MathStyle) -> Result<Item, Error> {
        let b = self.layout(base, style)?;
        if matches!(
            kind,
            AccentKind::Overline
                | AccentKind::Underline
                | AccentKind::Overbrace
                | AccentKind::Underbrace
                | AccentKind::Boxed
                | AccentKind::Cancel
                | AccentKind::BCancel
                | AccentKind::XCancel
        ) {
            let s = self.params.scale(style);
            let gap = self.params.overbar_vertical_gap.clone() * &s;
            let thick = self.params.overbar_rule_thickness.clone() * &s;
            let extra = self.params.overbar_extra_ascender.clone() * &s;
            let mut height = b.height.clone();
            let mut depth = b.depth.clone();
            if matches!(kind, AccentKind::Underline | AccentKind::Underbrace) {
                depth = &depth + &gap + &thick + &self.params.underbar_extra_descender * &s;
            } else {
                height = &height + &gap + &thick + &extra;
            }
            if matches!(kind, AccentKind::Boxed) {
                let pad = self.params.mu(style) * Dim::from_i64(3);
                return Ok(Item {
                    class: Some(AtomKind::Ord),
                    bx: pad_box(b, &pad),
                });
            }
            return Ok(Item {
                class: Some(AtomKind::Ord),
                bx: MathBox {
                    width: b.width.clone(),
                    height,
                    depth,
                    italic: Dim::zero(),
                    content: BoxContent::HList(vec![b]),
                },
            });
        }
        let ch = accent_char(kind);
        let acc = self.glyph(ch, style)?;
        let s = self.params.scale(style);
        let raise = b.height.max(&(&self.params.accent_base_height * &s));
        let width = b.width.max(&acc.width);
        let acc_r = MathBox {
            width: acc.width.clone(),
            height: &acc.height + &raise,
            depth: Dim::zero(),
            italic: Dim::zero(),
            content: acc.content,
        };
        Ok(Item {
            class: Some(AtomKind::Ord),
            bx: MathBox {
                width: width.clone(),
                height: acc_r.height.max(&b.height),
                depth: b.depth.clone(),
                italic: Dim::zero(),
                content: BoxContent::HList(vec![center_in(b, &width), center_in(acc_r, &width)]),
            },
        })
    }

    fn matrix(
        &self,
        style_m: MatrixStyle,
        rows: &[Vec<MathNode>],
        style: MathStyle,
    ) -> Result<Item, Error> {
        if rows.is_empty() {
            return Ok(Item {
                bx: MathBox::empty(),
                class: Some(AtomKind::Inner),
            });
        }
        let ncols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
        let mut cells: Vec<Vec<MathBox>> = Vec::new();
        for row in rows {
            let mut rboxes = Vec::new();
            for c in 0..ncols {
                if c < row.len() {
                    rboxes.push(self.layout(&row[c], style)?);
                } else {
                    rboxes.push(MathBox::empty());
                }
            }
            cells.push(rboxes);
        }
        let mut col_w: Vec<Dim> = vec![Dim::zero(); ncols];
        for row in &cells {
            for (j, cell) in row.iter().enumerate() {
                col_w[j] = col_w[j].max(&cell.width);
            }
        }
        let col_sep = self.params.mu(style) * Dim::from_i64(10);
        let row_sep = self.params.em(style) / Dim::from_i64(5);
        let mut row_boxes = Vec::new();
        for (ri, row) in cells.into_iter().enumerate() {
            let mut parts = Vec::new();
            for (j, cell) in row.into_iter().enumerate() {
                if j > 0 {
                    parts.push(MathBox::kern(col_sep.clone()));
                }
                parts.push(center_in(cell, &col_w[j]));
            }
            if ri > 0 {
                row_boxes.push(MathBox {
                    width: Dim::zero(),
                    height: Dim::zero(),
                    depth: row_sep.clone(),
                    italic: Dim::zero(),
                    content: BoxContent::Empty,
                });
            }
            row_boxes.push(MathBox::hpack(parts));
        }
        let mut inner = MathBox::vpack(row_boxes);
        let needed = &inner.height + &inner.depth;
        let (ld, rd) = matrix_delims(style_m);
        if let Some(l) = ld {
            let left = self.sized_glyph(l, &needed, style)?;
            let right = match rd {
                Some(r) => self.sized_glyph(r, &needed, style)?,
                None => MathBox::empty(),
            };
            inner = MathBox::hpack(vec![left, inner, right]);
        }
        Ok(Item {
            class: Some(AtomKind::Inner),
            bx: inner,
        })
    }
}

fn color_wrap(c: Color, inner: MathBox) -> MathBox {
    MathBox {
        width: inner.width.clone(),
        height: inner.height.clone(),
        depth: inner.depth.clone(),
        italic: inner.italic.clone(),
        content: BoxContent::Color(c, Box::new(inner)),
    }
}

fn pad_box(inner: MathBox, pad: &Dim) -> MathBox {
    let w = &inner.width + pad + pad;
    let h = &inner.height + pad;
    let d = &inner.depth + pad;
    MathBox {
        width: w,
        height: h,
        depth: d,
        italic: Dim::zero(),
        content: BoxContent::HList(vec![
            MathBox::kern(pad.clone()),
            inner,
            MathBox::kern(pad.clone()),
        ]),
    }
}

fn center_in(inner: MathBox, width: &Dim) -> MathBox {
    if inner.width.eq_dim(width) {
        return inner;
    }
    let extra = width - &inner.width;
    let half = &extra / &Dim::from_i64(2);
    let h = inner.height.clone();
    let d = inner.depth.clone();
    let packed = MathBox::hpack(vec![
        MathBox::kern(half.clone()),
        inner,
        MathBox::kern(&extra - &half),
    ]);
    MathBox {
        width: packed.width,
        height: h,
        depth: d,
        italic: Dim::zero(),
        content: packed.content,
    }
}

fn class_of(n: &MathNode) -> Option<AtomKind> {
    match n {
        MathNode::Atom(_, k) => Some(*k),
        MathNode::Symbol(s) => Some(symbol_class(s)),
        MathNode::Operator(_, _)
        | MathNode::Sum(_, _)
        | MathNode::Product(_, _)
        | MathNode::Integral(_, _, _)
        | MathNode::Limit(_) => Some(AtomKind::Op),
        MathNode::Fraction(_, _)
        | MathNode::Radical(_, _)
        | MathNode::Matrix(_, _)
        | MathNode::Delimited(_, _, _) => Some(AtomKind::Inner),
        MathNode::SizedDelim(_, _, k) => Some(*k),
        MathNode::Superscript(b, _)
        | MathNode::Subscript(b, _)
        | MathNode::SubSup(b, _, _)
        | MathNode::Accent(b, _)
        | MathNode::OverUnder(b, _, _) => class_of(b),
        MathNode::Text(_, _) => Some(AtomKind::Ord),
        MathNode::Color(_, b)
        | MathNode::TextColor(_, b)
        | MathNode::ColorBox(_, b)
        | MathNode::FColorBox(_, _, b)
        | MathNode::Phantom(_, b) => class_of(b),
        MathNode::Row(v) if v.len() == 1 => class_of(&v[0]),
        MathNode::Row(_) => Some(AtomKind::Ord),
        MathNode::Space(_) | MathNode::Strut(_, _) => None,
    }
}

fn single_glyph(name: &str) -> Option<char> {
    let e = lookup(name)?;
    let mut chars = e.glyph.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) => Some(c),
        _ => None,
    }
}

fn symbol_char(name: &str) -> Result<char, Error> {
    single_glyph(name).ok_or_else(|| Error::Unsupported {
        what: format!("symbol \\{name}"),
    })
}

fn symbol_class(name: &str) -> AtomKind {
    match name {
        "times" | "div" | "cdot" | "pm" | "mp" | "ast" | "star" | "circ" | "bullet" | "oplus"
        | "ominus" | "otimes" | "oslash" | "odot" | "wedge" | "vee" | "cap" | "cup" | "sqcap"
        | "sqcup" | "uplus" | "amalg" | "dagger" | "ddagger" | "wr" | "bigcirc" | "unlhd"
        | "unrhd" | "triangleleft" | "triangleright" => AtomKind::Bin,
        "leq" | "geq" | "neq" | "equiv" | "approx" | "sim" | "simeq" | "cong" | "propto" | "in"
        | "notin" | "subset" | "supset" | "subseteq" | "supseteq" | "subsetneq" | "ll" | "gg"
        | "prec" | "succ" | "preceq" | "succeq" | "perp" | "parallel" | "mid" | "to"
        | "leftarrow" | "rightarrow" | "leftrightarrow" | "Rightarrow" | "mapsto"
        | "longrightarrow" | "implies" | "iff" | "models" | "vdash" | "dashv" | "asymp"
        | "bowtie" | "therefore" | "because" => AtomKind::Rel,
        _ => AtomKind::Ord,
    }
}

fn named_delim(n: &str) -> Result<char, Error> {
    let c = match n {
        "{" => '{',
        "}" => '}',
        "|" | "Vert" | "lVert" | "rVert" => '‖',
        "vert" | "lvert" | "rvert" => '|',
        "langle" => '⟨',
        "rangle" => '⟩',
        "lfloor" => '⌊',
        "rfloor" => '⌋',
        "lceil" => '⌈',
        "rceil" => '⌉',
        "backslash" => '\\',
        "uparrow" => '↑',
        "downarrow" => '↓',
        "Uparrow" => '⇑',
        "Downarrow" => '⇓',
        "updownarrow" => '↕',
        "Updownarrow" => '⇕',
        other => {
            return Err(Error::Unsupported {
                what: format!("delimiter {other}"),
            });
        }
    };
    Ok(c)
}

fn accent_char(kind: AccentKind) -> char {
    match kind {
        AccentKind::Hat | AccentKind::WideHat => 'ˆ',
        AccentKind::Check => 'ˇ',
        AccentKind::Breve => '˘',
        AccentKind::Acute => '´',
        AccentKind::Grave => '`',
        AccentKind::Tilde | AccentKind::WideTilde => '˜',
        AccentKind::Bar | AccentKind::Overline => '¯',
        AccentKind::Vec | AccentKind::Overrightarrow => '→',
        AccentKind::Overleftarrow => '←',
        AccentKind::Dot => '˙',
        AccentKind::Ddot => '¨',
        AccentKind::Dddot => '…',
        AccentKind::Ring => '˚',
        AccentKind::Not => '/',
        AccentKind::Underline
        | AccentKind::Overbrace
        | AccentKind::Underbrace
        | AccentKind::Cancel
        | AccentKind::BCancel
        | AccentKind::XCancel
        | AccentKind::Boxed => '^',
    }
}

fn matrix_delims(s: MatrixStyle) -> (Option<char>, Option<char>) {
    match s {
        MatrixStyle::Pmatrix => (Some('('), Some(')')),
        MatrixStyle::Bmatrix => (Some('['), Some(']')),
        MatrixStyle::Vmatrix => (Some('|'), Some('|')),
        MatrixStyle::VVmatrix => (Some('‖'), Some('‖')),
        MatrixStyle::BBmatrix => (Some('{'), Some('}')),
        MatrixStyle::Cases => (Some('{'), None),
        _ => (None, None),
    }
}
