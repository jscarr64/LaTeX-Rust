//! TrueType outline → triangle mesh in font units (y-up).
//!
//! Curve flattening and ear clipping use [`Dim`](crate::Dim) only. `f32` from
//! `ttf-parser` is converted to [`Dim`] immediately.

use ttf_parser::{GlyphId, OutlineBuilder};

use crate::dim::Dim;
use crate::error::{Error, FontError};
use crate::font::MathFont;

/// Cached tessellation: font-unit vertices (y-up) and triangle indices.
#[derive(Clone, Debug)]
pub(super) struct GlyphTris {
    pub vertices: Vec<(Dim, Dim)>,
    pub indices: Vec<u32>,
}

pub(super) fn tessellate(font: &MathFont, glyph_id: u16) -> Result<GlyphTris, Error> {
    let face = font.face()?;
    let mut b = ContourBuilder::new();
    if face.outline_glyph(GlyphId(glyph_id), &mut b).is_none() {
        return Err(FontError::MissingGlyph { ch: '\u{FFFD}' }.into());
    }
    b.close_current();
    let contours = b.contours;
    if contours.is_empty() {
        return Err(FontError::MissingGlyph { ch: '\u{FFFD}' }.into());
    }
    let polys = combine_contours(contours)?;
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for poly in polys {
        let tris = earcut(&poly)?;
        let base = vertices.len() as u32;
        vertices.extend(poly);
        for [a, b, c] in tris {
            indices.push(base + a);
            indices.push(base + b);
            indices.push(base + c);
        }
    }
    if indices.is_empty() {
        return Err(Error::Unsupported {
            what: "glyph tessellation produced no triangles".into(),
        });
    }
    Ok(GlyphTris { vertices, indices })
}

struct ContourBuilder {
    contours: Vec<Vec<(Dim, Dim)>>,
    current: Vec<(Dim, Dim)>,
    last: Option<(Dim, Dim)>,
}

impl ContourBuilder {
    fn new() -> Self {
        Self {
            contours: Vec::new(),
            current: Vec::new(),
            last: None,
        }
    }

    fn pt(x: f32, y: f32) -> (Dim, Dim) {
        (
            Dim::from_ieee32_bits(x.to_bits()),
            Dim::from_ieee32_bits(y.to_bits()),
        )
    }

    fn push(&mut self, p: (Dim, Dim)) {
        if let Some(last) = self.current.last() {
            if last.0.eq_dim(&p.0) && last.1.eq_dim(&p.1) {
                return;
            }
        }
        self.current.push(p);
        self.last = Some(self.current[self.current.len() - 1].clone());
    }

    fn close_current(&mut self) {
        if self.current.len() >= 3 {
            if let (Some(first), Some(last)) = (self.current.first(), self.current.last()) {
                if first.0.eq_dim(&last.0) && first.1.eq_dim(&last.1) {
                    self.current.pop();
                }
            }
            if self.current.len() >= 3 {
                self.contours.push(std::mem::take(&mut self.current));
            } else {
                self.current.clear();
            }
        } else {
            self.current.clear();
        }
        self.last = None;
    }
}

impl OutlineBuilder for ContourBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.close_current();
        let p = Self::pt(x, y);
        self.push(p);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.push(Self::pt(x, y));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let Some(p0) = self.last.clone() else {
            return;
        };
        let p1 = Self::pt(x1, y1);
        let p2 = Self::pt(x, y);
        flatten_quad(&p0, &p1, &p2, self);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let Some(p0) = self.last.clone() else {
            return;
        };
        let p1 = Self::pt(x1, y1);
        let p2 = Self::pt(x2, y2);
        let p3 = Self::pt(x, y);
        flatten_cubic(&p0, &p1, &p2, &p3, self);
    }

    fn close(&mut self) {
        self.close_current();
    }
}

const FLAT_STEPS: i64 = 8;

fn flatten_quad(p0: &(Dim, Dim), p1: &(Dim, Dim), p2: &(Dim, Dim), b: &mut ContourBuilder) {
    let n = Dim::from_i64(FLAT_STEPS);
    for i in 1..=FLAT_STEPS {
        let t = Dim::from_i64(i) / &n;
        b.push(eval_quad(p0, p1, p2, &t));
    }
}

fn flatten_cubic(
    p0: &(Dim, Dim),
    p1: &(Dim, Dim),
    p2: &(Dim, Dim),
    p3: &(Dim, Dim),
    b: &mut ContourBuilder,
) {
    let n = Dim::from_i64(FLAT_STEPS);
    for i in 1..=FLAT_STEPS {
        let t = Dim::from_i64(i) / &n;
        b.push(eval_cubic(p0, p1, p2, p3, &t));
    }
}

fn lerp(a: &(Dim, Dim), b: &(Dim, Dim), t: &Dim) -> (Dim, Dim) {
    let omt = Dim::one() - t;
    (&(&a.0 * &omt) + &(&b.0 * t), &(&a.1 * &omt) + &(&b.1 * t))
}

fn eval_quad(p0: &(Dim, Dim), p1: &(Dim, Dim), p2: &(Dim, Dim), t: &Dim) -> (Dim, Dim) {
    let a = lerp(p0, p1, t);
    let b = lerp(p1, p2, t);
    lerp(&a, &b, t)
}

fn eval_cubic(
    p0: &(Dim, Dim),
    p1: &(Dim, Dim),
    p2: &(Dim, Dim),
    p3: &(Dim, Dim),
    t: &Dim,
) -> (Dim, Dim) {
    let a = lerp(p0, p1, t);
    let b = lerp(p1, p2, t);
    let c = lerp(p2, p3, t);
    let d = lerp(&a, &b, t);
    let e = lerp(&b, &c, t);
    lerp(&d, &e, t)
}

fn signed_area(poly: &[(Dim, Dim)]) -> Dim {
    let n = poly.len();
    let mut a = Dim::zero();
    for i in 0..n {
        let j = (i + 1) % n;
        a = &a + &(&(&poly[i].0 * &poly[j].1) - &(&poly[j].0 * &poly[i].1));
    }
    a
}

fn is_pos(d: &Dim) -> bool {
    matches!(d.cmp(&Dim::zero()), Some(core::cmp::Ordering::Greater))
}

fn is_neg(d: &Dim) -> bool {
    matches!(d.cmp(&Dim::zero()), Some(core::cmp::Ordering::Less))
}

fn dist2(a: &(Dim, Dim), b: &(Dim, Dim)) -> Dim {
    let dx = &a.0 - &b.0;
    let dy = &a.1 - &b.1;
    &(&dx * &dx) + &(&dy * &dy)
}

fn point_in_poly(poly: &[(Dim, Dim)], p: &(Dim, Dim)) -> bool {
    let mut inside = false;
    let n = poly.len();
    for i in 0..n {
        let a = &poly[i];
        let b = &poly[(i + 1) % n];
        let a_below = !is_pos(&(&a.1 - &p.1));
        let b_above = is_pos(&(&b.1 - &p.1));
        let b_below = !is_pos(&(&b.1 - &p.1));
        let a_above = is_pos(&(&a.1 - &p.1));
        if (a_below && b_above) || (b_below && a_above) {
            let dy = &b.1 - &a.1;
            if dy.is_zero() {
                continue;
            }
            let t = &(&p.1 - &a.1) / &dy;
            let xint = &a.0 + &(&t * &(&b.0 - &a.0));
            if is_pos(&(&xint - &p.0)) {
                inside = !inside;
            }
        }
    }
    inside
}

fn combine_contours(contours: Vec<Vec<(Dim, Dim)>>) -> Result<Vec<Vec<(Dim, Dim)>>, Error> {
    if contours.is_empty() {
        return Ok(Vec::new());
    }
    let areas: Vec<Dim> = contours.iter().map(|c| signed_area(c)).collect();
    let mut outer_idx = Vec::new();
    let mut hole_idx = Vec::new();
    for (i, a) in areas.iter().enumerate() {
        if a.is_zero() {
            continue;
        }
        if is_neg(a) {
            // TrueType often uses CW outers (negative in y-up). Treat the
            // first non-zero contour's sign as outer.
            hole_idx.push(i);
        } else {
            outer_idx.push(i);
        }
    }
    // If everything went into holes, flip the classification.
    if outer_idx.is_empty() && !hole_idx.is_empty() {
        outer_idx = hole_idx;
        hole_idx = Vec::new();
    }
    // Reclassify: largest |area| contours of the majority sign are outers.
    if outer_idx.is_empty() {
        return Ok(contours);
    }
    let outer_sign_pos = is_pos(&areas[outer_idx[0]]);
    outer_idx.clear();
    hole_idx.clear();
    for (i, a) in areas.iter().enumerate() {
        if a.is_zero() {
            continue;
        }
        let pos = is_pos(a);
        if pos == outer_sign_pos {
            outer_idx.push(i);
        } else {
            hole_idx.push(i);
        }
    }
    let mut outers: Vec<Vec<(Dim, Dim)>> =
        outer_idx.into_iter().map(|i| contours[i].clone()).collect();
    for h in hole_idx {
        let hole = &contours[h];
        let Some(pt) = hole.first() else {
            continue;
        };
        let mut host = 0usize;
        let mut found = false;
        for (oi, outer) in outers.iter().enumerate() {
            if point_in_poly(outer, pt) {
                host = oi;
                found = true;
                break;
            }
        }
        if !found {
            // Attach to the outer with largest |area|.
            let mut best = Dim::zero();
            for (oi, outer) in outers.iter().enumerate() {
                let aa = signed_area(outer).abs();
                if is_pos(&(&aa - &best)) {
                    best = aa;
                    host = oi;
                }
            }
        }
        insert_hole(&mut outers[host], hole);
    }
    Ok(outers)
}

fn insert_hole(outer: &mut Vec<(Dim, Dim)>, hole: &[(Dim, Dim)]) {
    if hole.len() < 3 || outer.len() < 3 {
        return;
    }
    let mut best_i = 0usize;
    let mut best_j = 0usize;
    let mut best_d = dist2(&outer[0], &hole[0]);
    for (i, op) in outer.iter().enumerate() {
        for (j, hp) in hole.iter().enumerate() {
            let d = dist2(op, hp);
            if matches!(d.cmp(&best_d), Some(core::cmp::Ordering::Less)) {
                best_d = d;
                best_i = i;
                best_j = j;
            }
        }
    }
    let mut spliced = Vec::with_capacity(outer.len() + hole.len() + 2);
    spliced.extend_from_slice(&outer[..=best_i]);
    let hn = hole.len();
    for k in 0..hn {
        spliced.push(hole[(best_j + k) % hn].clone());
    }
    spliced.push(hole[best_j].clone());
    spliced.push(outer[best_i].clone());
    spliced.extend_from_slice(&outer[best_i + 1..]);
    *outer = spliced;
}

fn cross(a: &(Dim, Dim), b: &(Dim, Dim), c: &(Dim, Dim)) -> Dim {
    let bax = &b.0 - &a.0;
    let bay = &b.1 - &a.1;
    let cax = &c.0 - &a.0;
    let cay = &c.1 - &a.1;
    &(&bax * &cay) - &(&bay * &cax)
}

fn triangle_contains(a: &(Dim, Dim), b: &(Dim, Dim), c: &(Dim, Dim), p: &(Dim, Dim)) -> bool {
    let c1 = cross(a, b, p);
    let c2 = cross(b, c, p);
    let c3 = cross(c, a, p);
    (is_pos(&c1) && is_pos(&c2) && is_pos(&c3)) || (is_neg(&c1) && is_neg(&c2) && is_neg(&c3))
}

fn earcut(poly: &[(Dim, Dim)]) -> Result<Vec<[u32; 3]>, Error> {
    let n0 = poly.len();
    if n0 < 3 {
        return Ok(Vec::new());
    }
    let area = signed_area(poly);
    let ccw = is_pos(&area);
    let mut idx: Vec<usize> = (0..n0).collect();
    if !ccw {
        idx.reverse();
    }
    let mut tris = Vec::new();
    let mut guard = 0usize;
    let max_guard = n0 * n0 + 8;
    while idx.len() > 3 {
        guard += 1;
        if guard > max_guard {
            return Err(Error::Unsupported {
                what: "glyph tessellation".into(),
            });
        }
        let m = idx.len();
        let mut clipped = false;
        for i in 0..m {
            let prev = idx[(i + m - 1) % m];
            let cur = idx[i];
            let next = idx[(i + 1) % m];
            let a = &poly[prev];
            let b = &poly[cur];
            let c = &poly[next];
            // Convex in CCW polygon: cross > 0.
            if !is_pos(&cross(a, b, c)) {
                continue;
            }
            let mut empty = true;
            for (k, &vi) in idx.iter().enumerate() {
                if k == (i + m - 1) % m || k == i || k == (i + 1) % m {
                    continue;
                }
                if triangle_contains(a, b, c, &poly[vi]) {
                    empty = false;
                    break;
                }
            }
            if !empty {
                continue;
            }
            tris.push([prev as u32, cur as u32, next as u32]);
            idx.remove(i);
            clipped = true;
            break;
        }
        if !clipped {
            return Err(Error::Unsupported {
                what: "glyph tessellation".into(),
            });
        }
    }
    if idx.len() == 3 {
        tris.push([idx[0] as u32, idx[1] as u32, idx[2] as u32]);
    }
    Ok(tris)
}
