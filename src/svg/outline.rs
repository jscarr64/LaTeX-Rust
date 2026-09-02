//! TrueType/CFF outline → SVG path `d` (font units, y-up).
//!
//! `ttf-parser` delivers outline points as hardware `f32`. Those bits are
//! converted immediately to [`Dim`](crate::Dim) via zenith-float `Ieee32` — they
//! are never used as layout arithmetic terminals.

use ttf_parser::{GlyphId, OutlineBuilder};

use crate::dim::Dim;
use crate::error::{Error, FontError};
use crate::font::MathFont;

struct PathBuilder {
    d: String,
}

fn coord(x: f32) -> String {
    Dim::from_ieee32_bits(x.to_bits()).to_svg_string()
}

impl OutlineBuilder for PathBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.d.push('M');
        self.d.push(' ');
        self.d.push_str(&coord(x));
        self.d.push(' ');
        self.d.push_str(&coord(y));
        self.d.push(' ');
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.d.push('L');
        self.d.push(' ');
        self.d.push_str(&coord(x));
        self.d.push(' ');
        self.d.push_str(&coord(y));
        self.d.push(' ');
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.d.push('Q');
        self.d.push(' ');
        self.d.push_str(&coord(x1));
        self.d.push(' ');
        self.d.push_str(&coord(y1));
        self.d.push(' ');
        self.d.push_str(&coord(x));
        self.d.push(' ');
        self.d.push_str(&coord(y));
        self.d.push(' ');
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.d.push('C');
        self.d.push(' ');
        self.d.push_str(&coord(x1));
        self.d.push(' ');
        self.d.push_str(&coord(y1));
        self.d.push(' ');
        self.d.push_str(&coord(x2));
        self.d.push(' ');
        self.d.push_str(&coord(y2));
        self.d.push(' ');
        self.d.push_str(&coord(x));
        self.d.push(' ');
        self.d.push_str(&coord(y));
        self.d.push(' ');
    }

    fn close(&mut self) {
        self.d.push('Z');
        self.d.push(' ');
    }
}

/// Outline of `glyph_id` as SVG path data, or [`FontError::MissingGlyph`] if empty.
pub fn glyph_path_d(font: &MathFont, glyph_id: u16) -> Result<String, Error> {
    let face = font.face()?;
    let mut b = PathBuilder { d: String::new() };
    let bbox = face.outline_glyph(GlyphId(glyph_id), &mut b);
    if bbox.is_none() && b.d.is_empty() {
        return Err(FontError::MissingGlyph { ch: '\u{FFFD}' }.into());
    }
    if b.d.ends_with(' ') {
        b.d.pop();
    }
    Ok(b.d)
}
