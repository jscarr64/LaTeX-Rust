//! egui primitive backend. Milestone 10; optional feature `egui`.
//!
//! Without the feature every entry point is [`Error::Unsupported`](crate::Error::Unsupported).
//! With `features = ["egui"]` a [`MathBox`](crate::MathBox) becomes `egui::Shape`
//! meshes and rects — no SVG intermediate.

#[cfg(feature = "egui")]
mod emit;
#[cfg(feature = "egui")]
mod tessellate;

use crate::color::Color;
use crate::dim::Dim;
use crate::error::Error;
use crate::font::MathFont;
use crate::layout::MathBox;

/// Options for [`shapes`] / [`paint_egui`].
#[derive(Clone, Debug)]
pub struct EguiOptions {
    /// Em size in points (same meaning as [`crate::SvgOptions::font_size_pt`]).
    pub font_size_pt: Dim,
    /// Default glyph fill.
    pub color: Color,
    /// When using [`latex_to_shapes`], pick display vs text style.
    pub display: bool,
}

impl Default for EguiOptions {
    fn default() -> Self {
        Self {
            font_size_pt: Dim::from_i64(14),
            color: Color::rgb(0, 0, 0),
            display: false,
        }
    }
}

impl EguiOptions {
    /// 14 pt, black fill, text style.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

/// Emit egui shapes for `tree`.
///
/// Without `features = ["egui"]` this is [`Error::Unsupported`].
/// With the feature, use [`shapes`] for the real backend.
pub fn render_egui(tree: &MathBox, font: &MathFont) -> Result<(), Error> {
    #[cfg(not(feature = "egui"))]
    {
        let _ = (tree, font);
        return Err(Error::Unsupported {
            what: "egui renderer".into(),
        });
    }
    #[cfg(feature = "egui")]
    {
        let _ = shapes(tree, font, &EguiOptions::new(), egui::Pos2::ZERO, 1.0)?;
        Ok(())
    }
}

/// `MathBox` → egui shapes and the layout bounding rect.
///
/// `pixels_per_point` is egui's device pixel ratio. Zero or negative is
/// [`Error::InvalidOption`].
#[cfg(feature = "egui")]
pub fn shapes(
    tree: &MathBox,
    font: &MathFont,
    options: &EguiOptions,
    origin: egui::Pos2,
    pixels_per_point: f32,
) -> Result<(Vec<egui::Shape>, egui::Rect), Error> {
    emit::shapes(tree, font, options, origin, pixels_per_point)
}

/// Parse, lay out, and emit egui shapes.
#[cfg(feature = "egui")]
pub fn latex_to_shapes(
    latex: &str,
    font: &MathFont,
    options: &EguiOptions,
    origin: egui::Pos2,
    pixels_per_point: f32,
) -> Result<(Vec<egui::Shape>, egui::Rect), Error> {
    use crate::layout::{layout, MathStyle};
    use crate::parser::parse;
    let ast = parse(latex)?;
    let style = if options.display {
        MathStyle::Display
    } else {
        MathStyle::Text
    };
    let tree = layout(&ast, font, style)?;
    shapes(&tree, font, options, origin, pixels_per_point)
}

/// Paint shapes through an egui [`egui::Painter`].
#[cfg(feature = "egui")]
pub fn paint_egui(
    tree: &MathBox,
    font: &MathFont,
    options: &EguiOptions,
    painter: &egui::Painter,
    origin: egui::Pos2,
) -> Result<egui::Rect, Error> {
    let ppp = painter.ctx().pixels_per_point();
    let (shapes, rect) = shapes(tree, font, options, origin, ppp)?;
    painter.extend(shapes);
    Ok(rect)
}

#[cfg(all(test, not(feature = "egui")))]
mod tests {
    use super::*;
    use crate::font::MathFont;
    use crate::layout::MathBox;

    #[test]
    fn egui_is_unsupported() {
        let font = MathFont::stix_two_math().expect("STIX");
        let err = render_egui(&MathBox::empty(), &font).expect_err("egui");
        assert!(err.to_string().contains("egui"), "{err}");
    }
}
