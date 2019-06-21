use super::tokens::{IntWord, Ordinals, Token};

pub(crate) fn match_num(token: Option<Token>) -> Option<i32> {
    token.and_then(|t| match t {
        Token::IntWord(IntWord::One) => Some(1),
        Token::IntWord(IntWord::Two) => Some(2),
        Token::IntWord(IntWord::Three) => Some(3),
        Token::IntWord(IntWord::Four) => Some(4),
        Token::IntWord(IntWord::Five) => Some(5),
        Token::IntWord(IntWord::Six) => Some(6),
        Token::IntWord(IntWord::Seven) => Some(7),
        Token::IntWord(IntWord::Eight) => Some(8),
        Token::IntWord(IntWord::Nine) => Some(9),
        Token::IntWord(IntWord::Ten) => Some(10),
        Token::IntWord(IntWord::Eleven) => Some(11),
        Token::IntWord(IntWord::Twelve) => Some(12),
        Token::Number(n) => Some(n),
        _ => None,
    })
}

pub(crate) fn match_ordinal(token: Option<Token>) -> Option<i32> {
    token.and_then(|t| match t {
        Token::Ordinals(Ordinals::First) => Some(1),
        Token::Ordinals(Ordinals::Second) => Some(2),
        Token::Ordinals(Ordinals::Third) => Some(3),
        Token::Ordinals(Ordinals::Fourth) => Some(4),
        Token::Ordinals(Ordinals::Fifth) => Some(5),
        Token::Ordinals(Ordinals::Sixth) => Some(6),
        Token::Ordinals(Ordinals::Seventh) => Some(7),
        Token::Ordinals(Ordinals::Eighth) => Some(8),
        Token::Ordinals(Ordinals::Ninth) => Some(9),
        Token::Ordinals(Ordinals::Tenth) => Some(10),
        Token::Ordinals(Ordinals::Eleventh) => Some(11),
        Token::Ordinals(Ordinals::Twelfth) => Some(12),
        Token::Ordinals(Ordinals::Thirteenth) => Some(13),
        Token::Ordinals(Ordinals::Fourteenth) => Some(14),
        Token::Ordinals(Ordinals::Fifteenth) => Some(15),
        Token::Ordinals(Ordinals::Sixteenth) => Some(16),
        Token::Ordinals(Ordinals::Seventeenth) => Some(17),
        Token::Ordinals(Ordinals::Eighteenth) => Some(18),
        Token::Ordinals(Ordinals::Nineteenth) => Some(19),
        Token::Ordinals(Ordinals::Twentieth) => Some(20),
        Token::Ordinals(Ordinals::TwentyFirst) => Some(21),
        Token::Ordinals(Ordinals::TwentySecond) => Some(22),
        Token::Ordinals(Ordinals::TwentyThird) => Some(23),
        Token::Ordinals(Ordinals::TwentyFourth) => Some(24),
        Token::Ordinals(Ordinals::TwentyFifth) => Some(25),
        Token::Ordinals(Ordinals::TwentySixth) => Some(26),
        Token::Ordinals(Ordinals::TwentySeventh) => Some(27),
        Token::Ordinals(Ordinals::TwentyEighth) => Some(28),
        Token::Ordinals(Ordinals::TwentyNinth) => Some(29),
        Token::Ordinals(Ordinals::Thirtieth) => Some(30),
        Token::Ordinals(Ordinals::ThirtiethFirst) => Some(31),
        _ => None,
    })
}
