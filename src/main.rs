mod lib;

use lib::*;

impl Token {
    fn get_precedence(&self) -> usize {
        match self.token_type.as_str() {
            "add" => 1,
            "subtract" => 1,
            "divide" => 2,
            "multiply" => 2,
            _ => panic!()
        }
    }
}

enum StackObject {
    Number(i32),
    Operator(Token)
}

fn main() {
    let argv: Vec<String> = std::env::args().collect();
    let json: String = std::fs::read_to_string(argv[1].clone()).unwrap();
    let source: String = std::fs::read_to_string(argv[2].clone()).unwrap();

    let mut lexer = lib::Lexer::from(json, source);
    let mut output_stack: Vec<StackObject> = vec![];
    let mut operator_stack: Vec<Token> = vec![];
    while !lexer.done() {
        if lexer.next_token().unwrap().is(vec!["number"]) {
            output_stack.push(StackObject::Number(lexer.current_token_x().value.parse::<i32>().unwrap()));
        } else if lexer.current_token_x().is(vec!["add", "subtract", "divide", "multiply"]) {
            if operator_stack.len() > 0 && operator_stack.last().unwrap().get_precedence() > lexer.current_token_x().get_precedence() {
                operator_stack.reverse();
                for t in &operator_stack {
                    output_stack.push(StackObject::Operator(t.clone()));
                }
                operator_stack = vec![];
            } else {
                operator_stack.push(lexer.current_token_x())
            }
        } else {
            panic!()
        }
    }
    operator_stack.reverse();
    for t in &operator_stack {
        output_stack.push(StackObject::Operator(t.clone()));
    }

    for obj in &output_stack {
        print!("{} ", match obj {
            StackObject::Number(n) => n.to_string(),
            StackObject::Operator(t) => t.value.clone()
        });
    }

    println!();

    let mut stack: Vec<i32> = vec![];
    for obj in output_stack {
        match obj {
            StackObject::Number(n) => {
                stack.push(n);
            }
            StackObject::Operator(token) => {
                match token.value.as_str() {
                    "+" => {
                        let b = stack.pop().unwrap();
                        let a = stack.pop().unwrap();
                        stack.push(a + b);
                    }
                    "-" => {
                        let b = stack.pop().unwrap();
                        let a = stack.pop().unwrap();
                        stack.push(a - b);
                    }
                    "/" => {
                        let b = stack.pop().unwrap();
                        let a = stack.pop().unwrap();
                        stack.push(a / b);
                    }
                    "*" => {
                        let b = stack.pop().unwrap();
                        let a = stack.pop().unwrap();
                        stack.push(a * b);
                    }
                    _ => panic!()
                }
            }
        }
    }
    println!("final output: {}", stack.pop().unwrap());
}
