# Changelog

## Unreleased

- Recognize hours without minutes after `at`, such as `tomorrow at 17`, while giving structured dates, times and durations priority.

## 0.2.0 (2026-10-23)

- Move to Rust edition 2024, requiring Rust 1.85 or newer.
- Migrate from Nom 4 macros and `CompleteStr` to Nom 8 function combinators and `&str`.
- Update Chrono, chrono-tz and strsim; use `chrono::Duration` for relative time.
- Remove the time, failure, tuple and unused num-traits direct dependencies.
- Implement `std::error::Error` for `DateTimeError`, preserving its variants and messages.
- Replace deprecated Chrono constructors in tests and use UTC fixtures.
- Track Cargo.lock for reproducible development builds and CI.
- Replace Travis CI with GitHub Actions checks for Rust 1.85, stable Rust and formatting.
- Correct 12 AM to midnight and 12 PM to noon, including dotted suffixes.
- Build complete dates and times atomically to avoid panics at month and year boundaries.
- Return errors for invalid dates and timezone gaps/folds; preserve exact instants for relative durations.
- Recognize standalone hours (`0` through `23`) as `HH:00`, preserving the local reference date (issue #1).

### Migration

Applications should update their chrono-tz dependency to 0.10. `Parser` usage remains
the same. `DateTimeError` now implements the standard error trait instead of deriving
`failure::Fail`; error variants and serialized fields are unchanged.
