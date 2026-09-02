//! Render backends: SVG (shipping), PNG (`tiny-skia`, feature `png`), and egui
//! (Milestone 10).
//!
//! Pipeline: [`crate::layout::MathBox`] → this module. SVG is the v1 output.
//! PNG is optional. Without `features = ["png"]` the PNG entry points return
//! [`crate::Error::Unsupported`].

pub mod egui;
pub mod png;
pub mod svg;

pub use egui::render_egui;
pub use png::{latex_to_png, render_png, PngBackground, PngOptions};
pub use svg::{latex_to_svg, render_svg, SvgOptions};
