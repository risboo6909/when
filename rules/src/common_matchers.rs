use crate::tokens::{IntWord, Token};
use num_traits::{FromPrimitive, Num};

pub(crate) fn match_num<T: Num + FromPrimitive>(token: Option<Token>) -> Option<T> {
    token.map_or(None, |t| match t {
        Token::IntWord(IntWord::One) => Some(T::one()),
        Token::IntWord(IntWord::Two) => Some(T::from_u32(2).unwrap()),
        Token::IntWord(IntWord::Three) => Some(T::from_u32(3).unwrap()),
        Token::IntWord(IntWord::Four) => Some(T::from_u32(4).unwrap()),
        Token::IntWord(IntWord::Five) => Some(T::from_u32(5).unwrap()),
        Token::IntWord(IntWord::Six) => Some(T::from_u32(6).unwrap()),
        Token::IntWord(IntWord::Seven) => Some(T::from_u32(7).unwrap()),
        Token::IntWord(IntWord::Eight) => Some(T::from_u32(8).unwrap()),
        Token::IntWord(IntWord::Nine) => Some(T::from_u32(9).unwrap()),
        Token::IntWord(IntWord::Ten) => Some(T::from_u32(10).unwrap()),
        Token::IntWord(IntWord::Eleven) => Some(T::from_u32(11).unwrap()),
        Token::IntWord(IntWord::Twelve) => Some(T::from_u32(12).unwrap()),
        Token::Number(n) => Some(T::from_usize(n).unwrap()),
        _ => None,
    })
}
