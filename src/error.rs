//! Public error types. Unsupported input is never rendered.

use core::fmt;

/// Crate-level error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// Tokenizer or parser rejected the input.
    Parse(ParseError),
    /// Font bytes or a requested glyph could not be used.
    Font(FontError),
    /// A feature listed as out of scope, or not yet implemented.
    ///
    /// Callers must treat this as failure. The renderer does not invent output.
    Unsupported {
        /// Human-readable name of the missing feature or construct.
        what: String,
    },
}

/// Tokenizer / parser failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseError {
    /// Input ended with a stray `\` and no command character.
    TrailingBackslash,
}

/// Font loader or metric lookup failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FontError {
    /// Embedded or supplied bytes are not a usable OpenType face.
    InvalidFace,
    /// Character has no glyph in this face.
    MissingGlyph {
        /// Requested character.
        ch: char,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(e) => write!(f, "{e}"),
            Self::Font(e) => write!(f, "{e}"),
            Self::Unsupported { what } => write!(f, "unsupported: {what}"),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TrailingBackslash => f.write_str("trailing backslash"),
        }
    }
}

impl fmt::Display for FontError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFace => f.write_str("invalid OpenType face"),
            Self::MissingGlyph { ch } => write!(f, "missing glyph for {ch:?}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Self {
        Self::Parse(e)
    }
}

impl From<FontError> for Error {
    fn from(e: FontError) -> Self {
        Self::Font(e)
    }
}
