# when    [![Build Status](https://travis-ci.com/risboo6909/when.svg?token=sEoRH24ki1j8CFisEvo5&branch=master)](https://travis-ci.com/risboo6909/when)
A natural language date/time parser with pluggable rules

Inspired by https://github.com/olebedev/when

#### Example usage:

```Rust
use chrono_tz;
use when;

fn main() {
    // first argument - is a specific parser (only english is available for the moment), then a timezone, maximum merge
    // distance and whether an exact match must be used (false - enables fuzzy matching)
    let parser = when::parser::Parser::new(Box::new(when::en), chrono_tz::Europe::Moscow, 3, false);
    println!("{:?}", parser.recognize("today 18p.m."));
}
```

Only english language is supported yet. More languages may be added in future.

#### Final note

This is my humble attempt to create a rust library. Although I tried to do my best, I know there are a lot of undiscovered bugs in there and my code is not at all perfect, so I will appreciate any help to make it more stable and better.
