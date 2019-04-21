use when;

#[test]
fn test_add_parser() {
    let mut parser =
        when::parser::Parser::new(Box::new(when::en), chrono_tz::Europe::Moscow, 5, false);
    let res = parser.recognize("Call me next mnday at 6P.m.");
    println!("integration: {:?}", res);
}
