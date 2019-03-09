//! "When" parses natural language date/time and produces computer friendly output structures.
//!
//! This library is inspired by https://github.com/olebedev/when

pub mod parser;

pub use rules::en::parse as en;
pub use parser::Parser;
