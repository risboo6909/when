# when [![CI](https://github.com/risboo6909/when/actions/workflows/ci.yml/badge.svg)](https://github.com/risboo6909/when/actions/workflows/ci.yml)

A natural language date/time parser with pluggable rules

Inspired by https://github.com/olebedev/when

Check out how it works interactively at https://risboo6909.org/when/ or at https://tarpaha.github.io/when/

#### Examples

See the [English](#english-expressions) and [Russian](#russian-expressions) tables
below for supported expressions. The parser also finds dates and times inside
longer text, such as `drop me a line next wednesday at 2:25 p.m`.

As a demonstration, check out a simple HTTP web [server](https://github.com/risboo6909/when-http) which parses natural language date/time using this library.

#### Key features

* No regular expressions in language rules.
* Has two modes, one with typos recognition and another one which works with exact matches only.

#### Requirements and development

Rust 1.85 or newer (edition 2024). The library uses Nom 8, Chrono 0.4,
chrono-tz 0.10 and strsim 0.11. See [CHANGELOG.md](CHANGELOG.md) for migration notes.

```sh
cargo build --locked
cargo test --locked
cargo fmt --all -- --check
```

Cargo.lock is tracked to keep development and CI builds reproducible.

#### Usage

```rust
use when::Parser;

fn main() {
    let parser = Parser::new(chrono_tz::Europe::Moscow)
        .max_dist(5)
        .fuzzy_parse(true);
    println!("{:?}", parser.parse("today at 18:00"));
}
```

English is the default language. Select the Russian parser with `when::ru`:

```rust
use when::Parser;

let parser = Parser::new(chrono_tz::Europe::Moscow)
    .parser(Box::new(&when::ru));
println!("{:?}", parser.parse("завтра в 17"));
```

#### English expressions

| Kind | Examples |
| --- | --- |
| Relative dates | `today`, `yesterday`, `tomorrow`, `now` |
| Clock times | `20`, `at 17`, `17:30`, `5pm`, `12 am`, `tomorrow at 17` |
| Times of day | `tomorrow morning`, `next Friday afternoon`, `this evening`, `tonight`, `noon` |
| Weekdays | `next monday`, `last friday`, `this sunday`, `next wednesday` |
| Calendar dates | `5 may`, `May 5th`, `first of January`, `twenty seventh of April` |
| Numeric dates | `20/12/2026`, `20/12` |
| Intervals | `in two hours`, `three days ago`, `in 20 seconds`, `within 5 minutes` |
| Half-units | `half an hour ago`, `half a day ago`, `in half of year` |

English numeric dates use day/month/year order with `/` separators. Month-name
dates use the reference's year. Standalone hours such as `20`
mean 20:00; `tomorrow at 17` means tomorrow at 17:00.

#### Russian expressions

| Kind | Examples |
| --- | --- |
| Relative dates | `сегодня`, `вчера`, `завтра`, `послезавтра`, `позавчера` |
| Clock times | `в 17`, `17:30`, `в семнадцать`, `в пять вечера`, `в 12 ночи` |
| Times of day | `завтра утром`, `сегодня днём`, `вечером`, `в полдень`, `в полночь` |
| Weekdays | `в понедельник`, `во вторник`, `в следующую пятницу`, `в прошлый четверг` |
| Calendar dates | `5 мая`, `первого января`, `31 декабря 2026 года`, `1 янв. 2026 г.` |
| Numeric dates | `20.12.2026`, `20/12/2026`, `20-12-2026`, `2026-12-20`, `20.12` |
| Intervals | `через два часа`, `три дня назад`, `через 2 часа 30 минут`, `через год и два месяца` |
| Half-units | `через полчаса`, `через полтора часа`, `через полторы минуты`, `через полгода` |

The parser accepts common inflections and cardinal numbers written as words;
ordinal day names are also supported. Matching is case-insensitive, and `ё` and `е`
are equivalent, including in exact mode. `.fuzzy_parse(false)` disables typo
recognition; short keywords and number words require exact spellings in both modes.
Unqualified times of day use 08:00 for `утром`, 15:00 for `днём`, 18:00 for
`вечером`, and 23:00 for `ночью`.

Dates without a year use the reference's local year. Explicit years require at
least four digits. An unqualified weekday is its next occurrence, including today;
`следующий` selects a strictly future occurrence and `прошлый` a strictly past one.
`в эту среду` refers to the current week and returns an ambiguity error when that
day has already passed.

`завтра` advances the local calendar date, while `через 24 часа` adds elapsed time.
Months and years are calendar intervals and preserve the day of the month;
an impossible result, such as January 31 plus one month, returns an error.
Nonexistent or ambiguous local clock times at timezone transitions also return errors.
Omitted seconds and fractional seconds follow the same precision rules as English.

The shared merge distance is measured in bytes. To extract separate dates joined
by `и`, use `.max_dist(3)`, for example with `завтра в 17 и послезавтра в 18`.

#### How it works?

English rules use Nom combinators. Russian rules tokenize the original UTF-8
input and produce the same date/time contexts for the shared merge and timezone logic.

Language rules live in `src/rules`. English rules are applied independently;
overlapping matches produce an intersection error. Russian rules consume complete
expressions before their individual words and numbers. Both parsers return byte
offsets for the shared date/time merge logic.

The library is also able to understand typos in words using Damerau-Levenshtein distance. Distances are defined inside the rules definitions. The longer word is the bigger distance is allowed. To disable fuzzy parsing, call `.fuzzy_parse(false)` on the parser builder.

Each rule has its own start and end match boundaries and a merge distance. If two matches are less than threshold away from each other then they are merged together.

For example:

```
Today 21:50 and tomorrow 22:00
-----------     --------------
```

First two terms **Today** and **21:5O** will be merged together as the distance between them is less than or equal to 5. So the final result will be as if there were one match which captures the term **Today 21:50** as the whole. The same thing will happen with the next two terms (**tomorrow** and **22:00**), despite the fact that they match by different rules, rules results will be merged together because the distance between matches is less than or equal to 5.

#### Final note

This is my humble attempt to create a rust library. Although I tried to do my best, I know there are a lot of undiscovered bugs in there and my code is not at all perfect, so I will appreciate any help to make it more stable and better.
