use failure::Fail;

pub(crate) const AMBIGUOUS: u32 = 0;
pub(crate) const UNKNOWN: u32 = 1;
pub(crate) const EMPTY: u32 = 2;
pub(crate) const OUT_OF_BOUNDS: u32 = 3;

#[derive(Debug, Fail, PartialEq)]
pub enum DateTimeError {

    #[fail(display = "can't parse time unambiguously: {}", msg)]
    AmbiguousTime {
        msg: String,
    },

}
