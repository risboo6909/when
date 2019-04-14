use crate::tokens::{IntWord, Ordinals, Token};
use num_traits::{FromPrimitive, Num};

pub(crate) fn match_num<T: Num + FromPrimitive>(token: Option<Token>) -> Option<T> {
    token.map_or(None, |t| match t {
        Token::IntWord(IntWord::One) => Some(T::one()),
        Token::IntWord(IntWord::Two) => Some(T::from_u8(2).unwrap()),
        Token::IntWord(IntWord::Three) => Some(T::from_u8(3).unwrap()),
        Token::IntWord(IntWord::Four) => Some(T::from_u8(4).unwrap()),
        Token::IntWord(IntWord::Five) => Some(T::from_u8(5).unwrap()),
        Token::IntWord(IntWord::Six) => Some(T::from_u8(6).unwrap()),
        Token::IntWord(IntWord::Seven) => Some(T::from_u8(7).unwrap()),
        Token::IntWord(IntWord::Eight) => Some(T::from_u8(8).unwrap()),
        Token::IntWord(IntWord::Nine) => Some(T::from_u8(9).unwrap()),
        Token::IntWord(IntWord::Ten) => Some(T::from_u8(10).unwrap()),
        Token::IntWord(IntWord::Eleven) => Some(T::from_u8(11).unwrap()),
        Token::IntWord(IntWord::Twelve) => Some(T::from_u8(12).unwrap()),
        Token::Number(n) => Some(T::from_i32(n).unwrap()),
        _ => None,
    })
}

pub(crate) fn match_ordinal<T: Num + FromPrimitive>(token: Option<Token>) -> Option<T> {
    token.map_or(None, |t| match t {
        Token::Ordinals(Ordinals::First) => Some(T::one()),
        Token::Ordinals(Ordinals::Second) => Some(T::from_u8(2).unwrap()),
        Token::Ordinals(Ordinals::Third) => Some(T::from_u8(3).unwrap()),
        Token::Ordinals(Ordinals::Fourth) => Some(T::from_u8(4).unwrap()),
        Token::Ordinals(Ordinals::Fifth) => Some(T::from_u8(5).unwrap()),
        Token::Ordinals(Ordinals::Sixth) => Some(T::from_u8(6).unwrap()),
        Token::Ordinals(Ordinals::Seventh) => Some(T::from_u8(7).unwrap()),
        Token::Ordinals(Ordinals::Eighth) => Some(T::from_u8(8).unwrap()),
        Token::Ordinals(Ordinals::Ninth) => Some(T::from_u8(9).unwrap()),
        Token::Ordinals(Ordinals::Tenth) => Some(T::from_u8(10).unwrap()),
        Token::Ordinals(Ordinals::Eleventh) => Some(T::from_u8(11).unwrap()),
        Token::Ordinals(Ordinals::Twelfth) => Some(T::from_u8(12).unwrap()),
        Token::Ordinals(Ordinals::Thirteenth) => Some(T::from_u8(13).unwrap()),
        Token::Ordinals(Ordinals::Fourteenth) => Some(T::from_u8(14).unwrap()),
        Token::Ordinals(Ordinals::Fifteenth) => Some(T::from_u8(15).unwrap()),
        Token::Ordinals(Ordinals::Sixteenth) => Some(T::from_u8(16).unwrap()),
        Token::Ordinals(Ordinals::Seventeenth) => Some(T::from_u8(17).unwrap()),
        Token::Ordinals(Ordinals::Eighteenth) => Some(T::from_u8(18).unwrap()),
        Token::Ordinals(Ordinals::Nineteenth) => Some(T::from_u8(19).unwrap()),
        Token::Ordinals(Ordinals::Twentieth) => Some(T::from_u8(20).unwrap()),
        Token::Ordinals(Ordinals::TwentyFirst) => Some(T::from_u8(21).unwrap()),
        Token::Ordinals(Ordinals::TwentySecond) => Some(T::from_u8(22).unwrap()),
        Token::Ordinals(Ordinals::TwentyThird) => Some(T::from_u8(23).unwrap()),
        Token::Ordinals(Ordinals::TwentyFourth) => Some(T::from_u8(24).unwrap()),
        Token::Ordinals(Ordinals::TwentyFifth) => Some(T::from_u8(25).unwrap()),
        Token::Ordinals(Ordinals::TwentySixth) => Some(T::from_u8(26).unwrap()),
        Token::Ordinals(Ordinals::TwentySeventh) => Some(T::from_u8(27).unwrap()),
        Token::Ordinals(Ordinals::TwentyEighth) => Some(T::from_u8(28).unwrap()),
        Token::Ordinals(Ordinals::TwentyNinth) => Some(T::from_u8(29).unwrap()),
        Token::Ordinals(Ordinals::Thirtieth) => Some(T::from_u8(30).unwrap()),
        Token::Ordinals(Ordinals::ThirtiethFirst) => Some(T::from_u8(31).unwrap()),
        _ => None,
    })
}
