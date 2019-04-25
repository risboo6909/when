use when;

#[test]
fn test_parsers() {
    let mut parser =
        when::parser::Parser::new(Box::new(when::en), chrono_tz::Europe::Moscow, 5, false);
    let res = parser.recognize("Call me next mnday at 6P.m.");
    println!("integration1: {:?}", res);

    let mut parser =
        when::parser::Parser::new(Box::new(when::en), chrono_tz::Europe::Moscow, 8, false);
    let res = parser.recognize("please call me tomorrow at 20p.m.");
    println!("integration2: {:?}", res);
}
