use crate::tokens::{IntWord, Token};

pub(crate) fn match_num(token: Option<Token>) -> Option<i64> {
    token.map_or(None, |t| match t {
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
        Token::Number(n) => Some(n as i64),
        _ => None,
    })
}
