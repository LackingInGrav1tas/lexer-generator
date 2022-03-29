//! # lexer-generator
//! 
//! This crate is a small scale lexer package which is parsed from JSON
//! 
//! # Example: Basic Tokenizing
//! 
//! Potential code one might use to lex tokens for a calculator
//! 
//! ```key.json```:
//! ```
//! {
//!     "literals": {
//!         "number": "[0-9]*[0-9]",
//!         "subtract": "-",
//!         "add": "\\+",
//!         "divide": "/",
//!         "multiply": "\\*" 
//!     },
//!     "whitespace": "\n| |\r|\t"
//! }
//! ```
//! ```main.rs```:
//! ```
//! let json: String = std::fs::read_to_string("key.json").unwrap();
//! let source: String = String::from("123 + 456 * 789");
//! 
//! let mut lexer = Lexer::from(json, source);
//! // parsing, runtime, whatever one would want to do with their tokens
//! ```
//! 
//! ```
//! "123 + 456 * 789" -> Token("number", "123"), Token("add", "*"), Token("number", "456"), Token("multiply", "*"), Token("number", "789") // ignoring line position and the incremental nature of the lexer
//! ```

use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use regex::*;

#[derive(Serialize, Deserialize)]
struct RuleSet { // Parsed rule set from JSON file
    literals: HashMap<String, String>,
    whitespace: String
}

#[derive(Clone)]
struct RegexRuleSet { // Converting above into regex
    literals: HashMap<String, Regex>,
    whitespace: Regex
}

#[allow(dead_code)]
impl RegexRuleSet {
    fn from(ruleset: RuleSet) -> Self {
        Self {
            // list of literal values, operators, keywords, etc., "name" : "regex pattern"
            literals: {
                let mut hm: HashMap<String, Regex> = HashMap::new();
                for (k, v) in ruleset.literals {
                    hm.insert(k, Regex::new(&v).unwrap());
                }
                hm
            },
            whitespace: Regex::new(&ruleset.whitespace).unwrap()
        }
    }
    fn from_string(json: String) -> Self {
        Self::from(serde_json::from_str::<RuleSet>(&json).unwrap())
    }
}

#[derive(Clone)]
/// Tokens are parsed from source code, their types are defined by the Lexer's ruleset
pub struct Token {
    pub token_type: String,
    pub value: String,
    pub line: usize
}

#[allow(dead_code)]
impl Token {
    /// Returns true if token.token_type matches any of the types
    pub fn is<T: ToString>(&self, types: Vec<T>) -> bool {
        {
            let mut v = vec![];
            for t in types {
                v.push(t.to_string());
            }
            v
        }.contains(&self.token_type)
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.token_type, self.value)
    }
}

#[derive(Clone)]
/// Lexes tokens from source code based on JSON-parsed ruleset
/// # Example: 
/// ```
/// let mut lexer = Lexer::from(json, source);
/// while !lexer.done() {
///    println!("{}", lexer.next_token().unwrap());
///}
///```
///
pub struct Lexer {
    source: String,
    last_token: Option<Result<Token, ParsingError>>,
    cache: Option<Result<Token, ParsingError>>,
    rules: RegexRuleSet,
    line: usize
}

#[derive(Clone, Debug)]
pub enum ParsingError {
    EndOfFileError,
    UnrecognizedPatternError(String),
}

#[allow(dead_code)]
impl Lexer {
    /// Generates a lexer from JSON
    pub fn from(json: String, source: String) -> Self {
        Self {
            source: source,
            last_token: None,
            cache: None,
            rules: RegexRuleSet::from_string(json),
            line: 0
        }
    }

    /// Initializes lexer without JSON parsing
    pub fn from_args(literals: HashMap<String, String>, whitespace: String, source: String) -> Self {
        Self {
            source: source,
            last_token: None,
            cache: None,
            rules: RegexRuleSet::from(RuleSet { literals: literals, whitespace: whitespace } ),
            line: 0
        }
    }

    fn ch(&self) -> char {
        (&self.source).as_bytes()[0] as char
    }

    fn skip_whitespace(&mut self) {
        let mat = match self.rules.whitespace.find(&self.source) { Some(a) => (a.start() as i32, a.end() as i32), None => (-1, -1)};
        if mat.0 == 0 {
            for _i in mat.0..mat.1 {
                match self.source.remove(0) {
                    '\n' => self.line += 1,
                    _ => {}
                }
            }
        }
    }

    pub fn done(&self) -> bool {
        0 >= self.source.len()
    }

    fn get(& mut self) -> char {
        match self.source.remove(0) {
            c => {
                if c == '\n' { self.line += 1; }
                c
            }
        }
    }

    fn parse_next(&mut self) -> Result<Token, ParsingError> {
        self.skip_whitespace();
        if !self.done() {
            let mut name = String::new();
            let mut mat: (i32, i32) = (-1, -1);
            for (lit_type, pat) in &self.rules.literals {
                let new_mat = match pat.find(&self.source) {
                    Some(thing) => thing,
                    None => continue
                };
                if new_mat.start() == 0 && new_mat.end() as i32 > mat.1 {
                    mat = (new_mat.start() as i32, new_mat.end() as i32);
                    name = lit_type.clone();
                }
            }
            if mat.0 != 0 { // no patterns
                return Err(ParsingError::UnrecognizedPatternError(String::from(self.get())))
            }
            let mut lexeme = String::new();
            for _ in 0..mat.1 {
                lexeme.push(self.get());
            }
            return Ok(Token { token_type: name, value: lexeme, line: self.line });
        }
        Err(ParsingError::EndOfFileError)
    }

    /// Advances and returns the next token
    pub fn next_token(&mut self) -> Result<Token, ParsingError> {
        match self.cache.clone() {
            Some(token) => {
                self.cache = None;
                self.last_token = Some(token);
                self.last_token.clone().unwrap()
            }
            None => {
                self.last_token = Some(self.parse_next());
                self.last_token.clone().unwrap()
            }
        }
    }

    /// Returns the last token lexed
    pub fn current_token(&self) -> Option<Result<Token, ParsingError>> {
        self.last_token.clone()
    }

    /// Returns the next token to be lexed
    pub fn peek_next_token(&mut self) -> Option<Result<Token, ParsingError>> {
        self.cache = Some(self.next_token());
        self.cache.clone()
    }
}