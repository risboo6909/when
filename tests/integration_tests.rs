use when;

#[test]
fn test_add_parser() {
    let mut parser = when::parser::Parser::new(false);
    parser.add_lang(Box::new(when::en));
    let res = parser.recognize("Call me next mnday at 6P.m.");
    println!("{:?}", res);
}
