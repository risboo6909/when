type ApplyFunc = Fn(&str) -> ApplyResult;

struct Rule {
    name: String,
    func: ApplyFunc,
}

impl Rule {

    pub(crate) fn new(name: String, func: ApplyFunc) -> Self {
        Rule {
            name,
            func,
        }
    }

}

#[derive(Default)]
struct ApplyResult {
    not_parsed: Option<String>,
    similarity: f64,
}
