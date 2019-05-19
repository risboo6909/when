# when    [![Build Status](https://travis-ci.com/risboo6909/when.svg?token=sEoRH24ki1j8CFisEvo5&branch=master)](https://travis-ci.com/risboo6909/when) [![codecov](https://codecov.io/gh/risboo6909/when/branch/master/graph/badge.svg)](https://codecov.io/gh/risboo6909/when)

A natural language date/time parser with pluggable rules

Inspired by https://github.com/olebedev/when

#### Examples

* **tonight at 11:10 pm**
* at **Friday afternoon**
* the deadline is **next tuesday 14:00**
* drop me a line **next wednesday at 2:25 p.m**
* it could be done at **11 am past tuesday**


#### Usage

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

#### How it works?

When uses Nom parsing library to parse input strings.

There is a *rules* folder which contains rules for various languages. Parser tries to apply all available rules for the given language one by one until the first match. Situation when several rules match the same part of the string is considered as an erroneous.

The library is also able to understand typos in words using Damerau-Levenshtein distance. Distances are defined inside the rules definitions. The longer word is the bigger distance is allowed. To disable fuzzy parsing, set the last argument to parser constructor to *true*, this will switch parser to *exact match only* mode.

#### Final note

This is my humble attempt to create a rust library. Although I tried to do my best, I know there are a lot of undiscovered bugs in there and my code is not at all perfect, so I will appreciate any help to make it more stable and better.
