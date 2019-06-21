//! "When" parses natural language date/time and produces computer friendly output structures.
//!
//! This library is inspired by https://github.com/olebedev/when

pub mod parser;
pub(crate) mod rules;

pub use parser::Parser;
pub use rules::en::parse as en;
pub use rules::errors::DateTimeError;
