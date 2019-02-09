type ApplyFunc = Fn(&str) -> ApplyResult;

#[derive(Default)]
struct ApplyResult {
    not_parsed: Option<String>,
    similarity: f64,
}
