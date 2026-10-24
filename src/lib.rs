//! "When" parses natural language date/time and produces computer friendly output structures.
//!
//! This library is inspired by <https://github.com/olebedev/when>
//!
//! ```
//! use when::Parser;
//!
//! let parser = Parser::new(chrono_tz::Europe::Moscow)
//!     .max_dist(5)
//!     .fuzzy_parse(true);
//! let results = parser.parse("tomorrow at 18:00");
//! assert_eq!(results.len(), 1);
//! assert!(results[0].is_ok());
//! ```
//!
//! Select the Russian parser explicitly:
//!
//! ```
//! use when::Parser;
//!
//! let parser = Parser::new(chrono_tz::Europe::Moscow)
//!     .parser(Box::new(&when::ru));
//! let results = parser.parse("завтра в 17");
//! assert_eq!(results.len(), 1);
//! assert!(results[0].is_ok());
//! ```

pub mod parser;
pub(crate) mod rules;

pub use parser::Parser;
pub use rules::en::parse as en;
pub use rules::errors::DateTimeError;
pub use rules::ru::parse as ru;
