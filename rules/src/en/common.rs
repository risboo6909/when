use crate::{Dist, TokenDesc, tokens::{Token, IntWord, Priority, When}};

use nom::{alt, call, apply, named_args, many_till, tuple, take, types::CompleteStr};

define!(one: (Token::IntWord(IntWord::One), Priority(3)), "one", Dist(0));
define!(two: (Token::IntWord(IntWord::Two), Priority(3)), "two", Dist(0));
define!(three: (Token::IntWord(IntWord::Three), Priority(3)), "three", Dist(1));
define!(four: (Token::IntWord(IntWord::Four), Priority(3)), "four", Dist(1));
define!(five: (Token::IntWord(IntWord::Five), Priority(3)), "five", Dist(1));
define!(six: (Token::IntWord(IntWord::Six), Priority(3)), "six", Dist(0));
define!(seven: (Token::IntWord(IntWord::Seven), Priority(3)), "seven", Dist(1));
define!(eight: (Token::IntWord(IntWord::Eight), Priority(3)), "eight", Dist(1));
define!(nine: (Token::IntWord(IntWord::Nine), Priority(3)), "nine", Dist(1));
define!(ten: (Token::IntWord(IntWord::Ten), Priority(3)), "ten", Dist(0));
define!(eleven: (Token::IntWord(IntWord::Eleven), Priority(3)), "eleven", Dist(1));
define!(twelve: (Token::IntWord(IntWord::Twelve), Priority(3)), "twelve", Dist(1));

combine!(int_word => one | two | three | four | five | six | seven | eight | nine | ten
                         | eleven | twelve);
