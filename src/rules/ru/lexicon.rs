use strsim::damerau_levenshtein;

pub(super) type Words = &'static [(i32, &'static [&'static str])];

#[derive(Debug)]
pub(super) struct Lexeme<'a> {
    pub text: &'a str,
    pub word: String,
    pub start: usize,
    pub end: usize,
}

pub(super) fn tokenize(input: &str) -> Vec<Lexeme<'_>> {
    fn class(c: char) -> u8 {
        if c.is_ascii_digit() {
            1
        } else if c.is_alphabetic() || c == '_' {
            2
        } else {
            0
        }
    }
    let mut result = Vec::new();
    let mut chars = input.char_indices().peekable();
    while let Some((start, c)) = chars.next() {
        if c.is_whitespace() {
            continue;
        }
        let kind = class(c);
        let mut end = start + c.len_utf8();
        if kind != 0 {
            while let Some(&(index, next)) = chars.peek() {
                if class(next) != kind {
                    break;
                }
                end = index + next.len_utf8();
                chars.next();
            }
        }
        let text = &input[start..end];
        result.push(Lexeme {
            text,
            word: text.to_lowercase().replace('ё', "е"),
            start,
            end,
        });
    }
    result
}

pub(super) fn word<'a>(tokens: &'a [Lexeme<'_>], index: usize) -> &'a str {
    tokens.get(index).map_or("", |token| token.word.as_str())
}

pub(super) fn keyword(value: &str, exact: bool, choices: Words) -> Option<i32> {
    // Exact spellings always win over fuzzy variants of other words.
    for &(number, spellings) in choices {
        if spellings.contains(&value) {
            return Some(number);
        }
    }
    if exact || !value.chars().all(char::is_alphabetic) {
        return None;
    }
    let mut best = None;
    let mut distance = usize::MAX;
    let mut tied = false;
    let value_length = value.chars().count();
    for &(number, spellings) in choices {
        for &spelling in spellings {
            let length = spelling.chars().count();
            let limit = if length >= 8 {
                2
            } else if length >= 5 {
                1
            } else {
                0
            };
            if limit == 0 || value_length.abs_diff(length) > limit {
                continue;
            }
            let candidate = damerau_levenshtein(value, spelling);
            if candidate > limit {
                continue;
            }
            if candidate < distance {
                best = Some(number);
                distance = candidate;
                tied = false;
            } else if candidate == distance && best != Some(number) {
                tied = true;
            }
        }
    }
    if tied { None } else { best }
}

pub(super) fn integer(tokens: &[Lexeme<'_>], index: usize) -> Option<(usize, i32)> {
    let sign = match word(tokens, index) {
        "-" => -1,
        "+" => 1,
        _ => 0,
    };
    let end = integer_end(tokens, index)?;
    let value = tokens[end - 1]
        .text
        .parse::<i64>()
        .ok()?
        .checked_mul(if sign == 0 { 1 } else { sign })?;
    Some((end, i32::try_from(value).ok()?))
}

pub(super) fn integer_end(tokens: &[Lexeme<'_>], index: usize) -> Option<usize> {
    let position = index + usize::from(matches!(word(tokens, index), "-" | "+"));
    let token = tokens.get(position)?;
    token
        .text
        .bytes()
        .all(|byte| byte.is_ascii_digit())
        .then_some(position + 1)
}

pub(super) fn number(tokens: &[Lexeme<'_>], index: usize, ordinal: bool) -> Option<(usize, i32)> {
    if let Some(value) = integer(tokens, index) {
        return Some(value);
    }
    let first = word(tokens, index);
    if ordinal {
        if let Some(value) = keyword(first, true, ORDINALS) {
            return Some((index + 1, value));
        }
    }
    let mut value = keyword(first, true, NUMBERS)?;
    let mut end = index + 1;
    if value >= 100 {
        if let Some(next) = keyword(word(tokens, end), true, NUMBERS).filter(|&n| n > 0 && n < 100)
        {
            value += next;
            end += 1;
        }
    }
    if (20..=90).contains(&(value % 100)) && value % 10 == 0 {
        let next = keyword(word(tokens, end), true, NUMBERS).or_else(|| {
            if ordinal {
                keyword(word(tokens, end), true, ORDINALS)
            } else {
                None
            }
        });
        if let Some(next) = next.filter(|&n| (1..=9).contains(&n)) {
            value += next;
            end += 1;
        }
    }
    Some((end, value))
}

pub(super) const DAYS: Words = &[
    (-2, &["позавчера"]),
    (-1, &["вчера"]),
    (0, &["сегодня", "сейчас"]),
    (1, &["завтра"]),
    (2, &["послезавтра"]),
];

pub(super) const DAYTIMES: Words = &[
    (8, &["утро", "утром", "утра"]),
    (15, &["день", "днем", "дня"]),
    (18, &["вечер", "вечером", "вечера"]),
    (23, &["ночь", "ночью", "ночи"]),
    (12, &["полдень", "полудня"]),
    (0, &["полночь", "полуночи"]),
];

pub(super) const WEEKDAYS: Words = &[
    (0, &["понедельник", "понедельника", "пн"]),
    (1, &["вторник", "вторника", "вт"]),
    (2, &["среда", "среду", "среды", "ср"]),
    (3, &["четверг", "четверга", "чт"]),
    (4, &["пятница", "пятницу", "пятницы", "пт"]),
    (5, &["суббота", "субботу", "субботы", "сб"]),
    (6, &["воскресенье", "воскресенья", "вс"]),
];

pub(super) const DIRECTIONS: Words = &[
    (
        1,
        &[
            "следующий",
            "следующую",
            "следующее",
            "следующем",
            "следующей",
        ],
    ),
    (
        -1,
        &[
            "прошлый",
            "прошлую",
            "прошлое",
            "прошлом",
            "прошлой",
            "предыдущий",
            "предыдущую",
        ],
    ),
    (0, &["этот", "эту", "это", "этом", "этой"]),
];

pub(super) const MONTHS: Words = &[
    (1, &["январь", "января", "январе", "янв"]),
    (2, &["февраль", "февраля", "феврале", "фев"]),
    (3, &["март", "марта", "марте", "мар"]),
    (4, &["апрель", "апреля", "апреле", "апр"]),
    (5, &["май", "мая", "мае"]),
    (6, &["июнь", "июня", "июне", "июн"]),
    (7, &["июль", "июля", "июле", "июл"]),
    (8, &["август", "августа", "августе", "авг"]),
    (9, &["сентябрь", "сентября", "сентябре", "сен", "сент"]),
    (10, &["октябрь", "октября", "октябре", "окт"]),
    (11, &["ноябрь", "ноября", "ноябре", "ноя", "нояб"]),
    (12, &["декабрь", "декабря", "декабре", "дек"]),
];

// Values are seconds for elapsed units and negative month counts for calendars.
pub(super) const UNITS: Words = &[
    (1, &["секунда", "секунду", "секунды", "секунд", "сек"]),
    (60, &["минута", "минуту", "минуты", "минут", "мин"]),
    (3600, &["час", "часа", "часов"]),
    (86400, &["день", "дня", "дней", "сутки", "суток"]),
    (604800, &["неделя", "неделю", "недели", "недель"]),
    (-1, &["месяц", "месяца", "месяцев"]),
    (-12, &["год", "года", "годов", "лет"]),
];

const NUMBERS: Words = &[
    (0, &["ноль", "нуль"]),
    (1, &["один", "одна", "одно", "одну", "одного"]),
    (2, &["два", "две", "двух"]),
    (3, &["три", "трех"]),
    (4, &["четыре", "четырех"]),
    (5, &["пять"]),
    (6, &["шесть"]),
    (7, &["семь"]),
    (8, &["восемь"]),
    (9, &["девять"]),
    (10, &["десять"]),
    (11, &["одиннадцать"]),
    (12, &["двенадцать"]),
    (13, &["тринадцать"]),
    (14, &["четырнадцать"]),
    (15, &["пятнадцать"]),
    (16, &["шестнадцать"]),
    (17, &["семнадцать"]),
    (18, &["восемнадцать"]),
    (19, &["девятнадцать"]),
    (20, &["двадцать"]),
    (30, &["тридцать"]),
    (40, &["сорок"]),
    (50, &["пятьдесят"]),
    (60, &["шестьдесят"]),
    (70, &["семьдесят"]),
    (80, &["восемьдесят"]),
    (90, &["девяносто"]),
    (100, &["сто"]),
    (200, &["двести"]),
    (300, &["триста"]),
    (400, &["четыреста"]),
    (500, &["пятьсот"]),
    (600, &["шестьсот"]),
    (700, &["семьсот"]),
    (800, &["восемьсот"]),
    (900, &["девятьсот"]),
];

const ORDINALS: Words = &[
    (1, &["первый", "первое", "первого"]),
    (2, &["второй", "второе", "второго"]),
    (3, &["третий", "третье", "третьего"]),
    (4, &["четвертый", "четвертое", "четвертого"]),
    (5, &["пятый", "пятое", "пятого"]),
    (6, &["шестой", "шестое", "шестого"]),
    (7, &["седьмой", "седьмое", "седьмого"]),
    (8, &["восьмой", "восьмое", "восьмого"]),
    (9, &["девятый", "девятое", "девятого"]),
    (10, &["десятый", "десятое", "десятого"]),
    (11, &["одиннадцатое", "одиннадцатого"]),
    (12, &["двенадцатое", "двенадцатого"]),
    (13, &["тринадцатое", "тринадцатого"]),
    (14, &["четырнадцатое", "четырнадцатого"]),
    (15, &["пятнадцатое", "пятнадцатого"]),
    (16, &["шестнадцатое", "шестнадцатого"]),
    (17, &["семнадцатое", "семнадцатого"]),
    (18, &["восемнадцатое", "восемнадцатого"]),
    (19, &["девятнадцатое", "девятнадцатого"]),
    (20, &["двадцатое", "двадцатого"]),
    (30, &["тридцатое", "тридцатого"]),
];
