//! Render backends: SVG (shipping), PNG and egui (later milestones).
//!
//! Pipeline: [`crate::layout::MathBox`] → this module. SVG is the v1 output.
//! PNG (`tiny-skia`) and egui shapes are optional backends. Until those
//! milestones land they return [`crate::Error::Unsupported`] — never a fake image.

pub mod egui;
pub mod png;
pub mod svg;

pub use egui::render_egui;
pub use png::{latex_to_png, render_png, PngOptions};
pub use svg::{latex_to_svg, render_svg, SvgOptions};
