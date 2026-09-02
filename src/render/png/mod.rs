//! PNG raster backend. Milestone 9; optional feature `png` pulls in `tiny-skia`.
//!
//! Without the feature every entry point is [`Error::Unsupported`](crate::Error::Unsupported).
//! Color channels use [`Color::to_rgba8`](crate::Color::to_rgba8).

#[cfg(feature = "png")]
mod raster;

use crate::color::Color;
use crate::dim::Dim;
use crate::error::Error;
use crate::font::MathFont;
use crate::layout::MathBox;
use crate::parser::parse;

/// Canvas behind the expression.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PngBackground {
    /// Fully transparent (default).
    Transparent,
    /// Opaque white.
    White,
    /// Opaque custom fill.
    Color(Color),
}

/// Options for [`render_png`].
#[derive(Clone, Debug)]
pub struct PngOptions {
    /// Em size in points (same meaning as [`crate::SvgOptions::font_size_pt`]).
    pub font_size_pt: Dim,
    /// Raster resolution in dots per inch. Must be in `(0, 2400]`.
    pub dpi: Dim,
    /// Default glyph fill.
    pub color: Color,
    /// Canvas behind the glyphs.
    pub background: PngBackground,
    /// When using [`latex_to_png`], pick display vs text style.
    pub display: bool,
}

impl Default for PngOptions {
    fn default() -> Self {
        Self {
            font_size_pt: Dim::from_i64(12),
            dpi: Dim::from_i64(144),
            color: Color::rgb(0, 0, 0),
            background: PngBackground::Transparent,
            display: false,
        }
    }
}

impl PngOptions {
    /// 12 pt, 144 dpi, black fill, transparent background, text style.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

/// Parse, lay out, and rasterize `latex` to PNG.
pub fn latex_to_png(latex: &str, font: &MathFont, options: &PngOptions) -> Result<Vec<u8>, Error> {
    let ast = parse(latex)?;
    #[cfg(not(feature = "png"))]
    {
        let _ = (font, options, ast);
        return Err(Error::Unsupported {
            what: "png renderer".into(),
        });
    }
    #[cfg(feature = "png")]
    {
        use crate::layout::{layout, MathStyle};
        let style = if options.display {
            MathStyle::Display
        } else {
            MathStyle::Text
        };
        let tree = layout(&ast, font, style)?;
        render_png(&tree, font, options)
    }
}

/// Rasterize a laid-out [`MathBox`] to PNG bytes.
pub fn render_png(tree: &MathBox, font: &MathFont, options: &PngOptions) -> Result<Vec<u8>, Error> {
    #[cfg(not(feature = "png"))]
    {
        let _ = (tree, font, options);
        Err(Error::Unsupported {
            what: "png renderer".into(),
        })
    }
    #[cfg(feature = "png")]
    {
        raster::render(tree, font, options)
    }
}

#[cfg(all(test, not(feature = "png")))]
mod tests {
    use super::*;
    use crate::font::MathFont;
    use crate::layout::MathBox;

    #[test]
    fn png_is_unsupported() {
        let font = MathFont::stix_two_math().expect("STIX");
        let err = render_png(&MathBox::empty(), &font, &PngOptions::new()).expect_err("png");
        assert!(err.to_string().contains("png"), "{err}");
        let err = latex_to_png("x", &font, &PngOptions::new()).expect_err("png latex");
        assert!(err.to_string().contains("png"), "{err}");
    }
}
