use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use regex::Regex;

#[derive(Serialize, Deserialize)]
struct RuleSet { // Parsed rule set from JSON file
    keywords: Vec<String>,
    literal_regex: HashMap<String, Vec<String>>,
    operators: HashMap<String, String>,
    operator_start: String,
    operator_halt: String,
    whitespace: String
}

struct RegexRuleSet { // Converting above into regex
    keywords: Vec<String>,
    literal_regex: HashMap<String, Vec<Regex>>,
    operators: HashMap<String, String>,
    operator_start: Regex,
    operator_halt: Regex,
    whitespace: Regex
}

impl RegexRuleSet {
    pub fn from(json: String) -> Self {
        let ruleset = serde_json::from_str::<RuleSet>(&json).unwrap(); // serde deserializing
        Self {
            // list of keywords
            keywords: ruleset.keywords,

            // list of literal values, "type name" : [ "regex for start", "regex for end" ]
            literal_regex: {
                let mut hm: HashMap<String, Vec<Regex>> = HashMap::new();
                for (k, v) in ruleset.literal_regex {
                    hm.insert(k, {
                        let mut vec = vec![];
                        for pat in v { vec.push(Regex::new(&pat).unwrap()); }
                        vec
                    });
                }
                hm
            },
            operators: {
                ruleset.operators
            },
            operator_start: Regex::new(&ruleset.operator_start).unwrap(),
            operator_halt: Regex::new(&ruleset.operator_halt).unwrap(),
            whitespace: Regex::new(&ruleset.whitespace).unwrap()
        }
    }
}

#[derive(Clone)]
pub struct Token {
    pub token_type: String,
    pub value: String,
    pub line: usize
}

impl Token {
    pub fn is(&self, types: Vec<String>) -> bool {
        types.contains(&self.token_type)
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.token_type, self.value)
    }
}

pub struct Lexer {
    source: String,
    last_token: Option<Token>,
    cache: Option<Token>,
    rules: RegexRuleSet,
    line: usize
}

impl Lexer {
    pub fn from(json: String, source: String) -> Self {
        Self {
            source: source,
            last_token: None,
            cache: None,
            rules: RegexRuleSet::from(json),
            line: 0
        }
    }

    fn ch(&self) -> char {
        (&self.source).as_bytes()[0] as char
    }

    fn skip_whitespace(&mut self) {
        while self.rules.whitespace.is_match(&String::from(self.ch())) {
            self.source.remove(0);
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

    fn parse_next(&mut self) -> Option<Token> {
        self.skip_whitespace();
        let mut lexeme = String::new();
        if !self.done() {
            lexeme.push(self.get());

            // lexing operator
            if self.rules.operator_start.is_match(&lexeme) {
                while !self.done() && !self.rules.operator_halt.is_match(&String::from(self.ch())) {
                    lexeme.push(self.get());
                }
                self.rules.operators.get(&lexeme).expect(&format!("'{}' is not a valid operator", lexeme));
                return Some(
                    Token {
                        token_type: String::from("operator"),
                        value: lexeme,
                        line: self.line
                    }
                );
            }

            // lexing literal
            for (literal_type, patterns) in self.rules.literal_regex.clone() {
                if patterns.get(0).unwrap().is_match(&lexeme) { // matches start
                    while !self.done() && !patterns.get(1).unwrap().is_match(&String::from(self.ch())) { // while it isn't halt
                        lexeme.push(self.get());
                    }
                    return if self.rules.keywords.contains(&lexeme) {
                        Some(
                            Token {
                                token_type: String::from("keyword"),
                                value: lexeme,
                                line: self.line
                            }
                        )
                    } else {
                        Some(
                            Token {
                                token_type: literal_type,
                                value: lexeme,
                                line: self.line
                            }
                        )
                    };
                }
            }

            // something else
            while !self.done() && !self.rules.whitespace.is_match(&String::from(self.ch())) {
                lexeme.push(self.get());
            }
            if self.rules.keywords.contains(&lexeme) {
                return Some(
                    Token {
                        token_type: String::from("keyword"),
                        value: lexeme,
                        line: self.line
                    }
                )
            } else {
                panic!("{}", format!("no known pattern exists for {}", lexeme))
            }
        }
        None
    }

    pub fn next_token(&mut self) -> Option<Token> {
        match self.cache.clone() {
            Some(token) => {
                self.cache = None;
                self.last_token = Some(token);
                self.last_token.clone()
            }
            None => {
                self.last_token = self.parse_next();
                self.last_token.clone()
            }
        }
    }

    pub fn current_token(&self) -> Token {
        match &self.last_token {
            Some(t) => t.clone(),
            None => panic!("no current token")
        }
    }

    pub fn peek_next_token(&mut self) -> Option<Token> {
        self.cache = self.next_token();
        self.cache.clone()
    }
}