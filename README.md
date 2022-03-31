# lexer-generator

Lexer crate derived from Regex patterns with user customizeable tokens

# Example: Basic Tokenizing

Potential code one might use to lex tokens for a calculator

```key.json```:
```
{
    "literals": {
        "number": "[0-9]*(\\.[0-9]*){0, 1}",
        "subtract": "-",
        "add": "\\+",
        "divide": "/",
        "multiply": "\\*" 
    },
    "whitespace": "\n| |\r|\t"
}
```
```main.rs```:
```
let json: String = std::fs::read_to_string("key.json").unwrap();
let source: String = String::from("123 + 456 * 789");

let mut lexer = Lexer::from(json, source);
while !lexer.done() {
    println!("{}", lexer.next_token().unwrap());
}
```

```
number(123)
add(+)     
number(456)
multiply(*)
number(789)
```