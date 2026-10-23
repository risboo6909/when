# when [![CI](https://github.com/risboo6909/when/actions/workflows/ci.yml/badge.svg)](https://github.com/risboo6909/when/actions/workflows/ci.yml)

A natural language date/time parser with pluggable rules

Inspired by https://github.com/olebedev/when

Check out how it works interactively at https://risboo6909.org/when/ or at https://tarpaha.github.io/when/

#### Examples

* **tonight at 11:10 pm**
* at **Friday afternoon**
* the deadline is **next tuesday 14:00**
* drop me a line **next wednesday at 2:25 p.m**
* it could be done at **11 am past tuesday**
* **20** (standalone hour, interpreted as 20:00 on the reference date)

As a demonstration, check out a simple HTTP web [server](https://github.com/risboo6909/when-http) which parses natural language date/time using this library.

#### Key features

* No regexp in rules definition. All rules are defined using rust macros.
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

Only english language is supported yet. More languages may be added in future.

#### How it works?

When uses Nom parsing library to parse input strings.

There is a *rules* folder which contains rules for various languages. The parser applies each available rule for the selected language across the input and sorts the matches by position. Situation when several rules match the same part of the string is considered as an erroneous.

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
