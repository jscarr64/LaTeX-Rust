//! egui primitive backend. Milestone 10; optional feature `egui`.
//!
//! Emits UI shapes directly (no SVG intermediate) once the backend lands.
//! Until then every entry point is [`Error::Unsupported`](crate::Error::Unsupported).
//! Color channels are already resolved as [`Color::to_rgba8`](crate::Color::to_rgba8).

use crate::error::Error;
use crate::font::MathFont;
use crate::layout::MathBox;

/// Emit egui shapes for `tree`.
///
/// Returns [`Error::Unsupported`] until the `egui` backend is implemented.
pub fn render_egui(_tree: &MathBox, _font: &MathFont) -> Result<(), Error> {
    Err(Error::Unsupported {
        what: "egui renderer".into(),
    })
}

#[cfg(test)]
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
