//! PNG raster backend. Milestone 9; optional feature `png` will pull in `tiny-skia`.
//!
//! Until that milestone, every entry point is [`Error::Unsupported`](crate::Error::Unsupported).

use crate::color::Color;
use crate::dim::Dim;
use crate::error::Error;
use crate::font::MathFont;
use crate::layout::MathBox;
use crate::parser::parse;

/// Options for [`render_png`].
#[derive(Clone, Debug)]
pub struct PngOptions {
    /// Em size in points (same meaning as [`crate::SvgOptions::font_size_pt`]).
    pub font_size_pt: Dim,
    /// Raster resolution in dots per inch.
    pub dpi: Dim,
    /// Default glyph fill.
    pub color: Color,
}

impl Default for PngOptions {
    fn default() -> Self {
        Self {
            font_size_pt: Dim::from_i64(12),
            dpi: Dim::from_i64(96),
            color: Color::rgb(0, 0, 0),
        }
    }
}

impl PngOptions {
    /// 12 pt, 96 dpi, black fill.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

/// Parse, lay out, and rasterize `latex` to PNG.
///
/// Returns [`Error::Unsupported`] until the `png` backend is implemented.
pub fn latex_to_png(latex: &str, font: &MathFont, options: &PngOptions) -> Result<Vec<u8>, Error> {
    let _ast = parse(latex)?;
    let _ = (font, options);
    Err(Error::Unsupported {
        what: "png renderer".into(),
    })
}

/// Rasterize a laid-out [`MathBox`] to PNG bytes.
///
/// Returns [`Error::Unsupported`] until the `png` backend is implemented.
pub fn render_png(
    _tree: &MathBox,
    _font: &MathFont,
    _options: &PngOptions,
) -> Result<Vec<u8>, Error> {
    Err(Error::Unsupported {
        what: "png renderer".into(),
    })
}

#[cfg(test)]
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
