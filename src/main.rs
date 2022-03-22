mod lib;

fn main() {
    let argv: Vec<String> = std::env::args().collect();
    let json: String = std::fs::read_to_string(argv[1].clone()).unwrap();
    let source: String = std::fs::read_to_string(argv[2].clone()).unwrap();

    let mut lexer = lib::Lexer::from(json, source);
    while !lexer.done() {
        println!("{}", lexer.next_token().unwrap());
    }
}
