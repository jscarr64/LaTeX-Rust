//! LaTeX-Rust: pure Rust LaTeX math renderer.
//!
//! Milestone 2 parses LaTeX math into a typed [`MathNode`] AST. Milestone 1
//! still provides layout dimensions, box composition, STIX Two Math metrics,
//! the tokenizer, the math symbol catalog, and color-model resolution.
//! Unsupported constructs return [`Error`] — never a fake render.
//!
//! No hardware `f32` / `f64` is used in layout arithmetic.
//!
//! ```
//! use latex_rust::{parse, tokenize, lookup, MathFont, Dim, MathBox};
//!
//! let ast = parse(r"\frac{1}{2}").expect("parse");
//! assert_eq!(ast.gold(), r#"(frac (atom Ord "1") (atom Ord "2"))"#);
//!
//! let tokens = tokenize(r"\frac{1}{2}").expect("tokens");
//! assert!(!tokens.is_empty());
//! assert_eq!(lookup(r"\alpha").unwrap().glyph, "α");
//!
//! let font = MathFont::stix_two_math().expect("STIX Two Math");
//! assert_eq!(font.units_per_em(), 1000);
//!
//! let packed = MathBox::hpack(vec![
//!     MathBox::rule(Dim::one(), Dim::zero(), Dim::zero()),
//!     MathBox::rule(Dim::ratio(1, 2), Dim::zero(), Dim::zero()),
//! ]);
//! assert_eq!(packed.width, Dim::ratio(3, 2));
//! ```

#![deny(missing_docs)]
#![deny(clippy::float_arithmetic)]
#![deny(clippy::undocumented_unsafe_blocks)]

mod color;
mod dim;
mod error;
mod font;
mod layout;
mod parser;
mod symbols;

pub use color::{named_color, parse_color_spec, Color, ColorTable};
pub use dim::{Dim, DIM_PREC};
pub use error::{Error, FontError, ParseError};
pub use font::{
    GlyphMetrics, MathFont, STIX_TWO_MATH_NAME, STIX_TWO_MATH_OTF, STIX_TWO_MATH_SHA256,
};
pub use layout::{BoxContent, MathBox};
pub use parser::{
    format_tokens, parse, parse_with_colors, preprocess, tokenize, AccentKind, AtomKind, Delimiter,
    IntegralKind, MathNode, MatrixStyle, PhantomKind, SpaceKind, TextStyle, Token,
};
pub use symbols::{category_count, lookup, symbols, SymbolEntry, SymbolKind};
