//! Horizontal and vertical box composition. TeX-faithful style rules come later.

use crate::dim::Dim;
use crate::error::Error;
use crate::font::MathFont;

/// What a box contains. Milestone 1 is composition only; glyphs are metric boxes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BoxContent {
    /// Empty box (strut or placeholder).
    Empty,
    /// Solid rule (fraction bar, vinculum). No glyph.
    Rule,
    /// A single character whose metrics came from the math font.
    Glyph {
        /// Character.
        ch: char,
        /// OpenType glyph id.
        glyph_id: u16,
    },
    /// Horizontal list. Width is the sum of children.
    HList(Vec<MathBox>),
    /// Vertical list. Height/depth stack on the baseline of the first box.
    VList(Vec<MathBox>),
}

/// TeX-style box: width, height above baseline, depth below, italic correction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MathBox {
    /// Width.
    pub width: Dim,
    /// Height above the baseline.
    pub height: Dim,
    /// Depth below the baseline.
    pub depth: Dim,
    /// Italic correction.
    pub italic: Dim,
    /// Payload.
    pub content: BoxContent,
}

impl MathBox {
    /// Zero-size empty box.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            width: Dim::zero(),
            height: Dim::zero(),
            depth: Dim::zero(),
            italic: Dim::zero(),
            content: BoxContent::Empty,
        }
    }

    /// Rule with explicit dimensions (fraction bar, strut).
    #[must_use]
    pub fn rule(width: Dim, height: Dim, depth: Dim) -> Self {
        Self {
            width,
            height,
            depth,
            italic: Dim::zero(),
            content: BoxContent::Rule,
        }
    }

    /// Box from a font glyph. Errors if the face has no glyph for `ch`.
    pub fn from_glyph(font: &MathFont, ch: char) -> Result<Self, Error> {
        let g = font.glyph(ch)?;
        Ok(Self {
            width: g.advance,
            height: g.height,
            depth: g.depth,
            italic: Dim::zero(),
            content: BoxContent::Glyph {
                ch,
                glyph_id: g.glyph_id,
            },
        })
    }

    /// Pack boxes in a row. Width sums; height and depth are maxima.
    #[must_use]
    pub fn hpack(children: Vec<Self>) -> Self {
        let mut width = Dim::zero();
        let mut height = Dim::zero();
        let mut depth = Dim::zero();
        for c in &children {
            width = &width + &c.width;
            height = height.max(&c.height);
            depth = depth.max(&c.depth);
        }
        Self {
            width,
            height,
            depth,
            italic: Dim::zero(),
            content: BoxContent::HList(children),
        }
    }

    /// Pack boxes in a column, first child on the baseline.
    ///
    /// Subsequent children sit below the previous (height + depth stacked).
    #[must_use]
    pub fn vpack(children: Vec<Self>) -> Self {
        if children.is_empty() {
            return Self::empty();
        }
        let mut width = Dim::zero();
        let height = children[0].height.clone();
        let mut depth = children[0].depth.clone();
        for c in children.iter().skip(1) {
            width = width.max(&c.width);
            depth = &depth + &c.height;
            depth = &depth + &c.depth;
        }
        width = width.max(&children[0].width);
        Self {
            width,
            height,
            depth,
            italic: Dim::zero(),
            content: BoxContent::VList(children),
        }
    }
}
