use std::cell::RefCell;
use crate::error::ErrorReporter;
use crate::token::*;

pub struct Scanner {
    file_path: String,
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    pub had_error: RefCell<bool>,
    errors: RefCell<Vec<(usize, usize, String)>>
}

impl Scanner {
    pub fn new(file_path: String, source: String) -> Self {
        Self {
            file_path: file_path,
            source: source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            had_error: RefCell::new(false),
            errors: RefCell::new(Vec::new())
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_end() {
            self.start = self.current;
            self.scan_token();
        }
        self.add_token(TokenType::Eof, None);
        self.tokens.clone()
    }

    fn scan_token(&mut self) {
        let c = self.next_char();
        match c {
            //single char
            '(' => { self.add_token(TokenType::LeftParen, None); }
            ')' => { self.add_token(TokenType::RightParen, None); }
            '[' => { self.add_token(TokenType::LeftSuqareBracket, None); }
            ']' => { self.add_token(TokenType::RightSquareBracket, None); }
            '{' => { self.add_token(TokenType::LeftBrace, None); }
            '}' => { self.add_token(TokenType::RightBrace, None); }
            ',' => { self.add_token(TokenType::Comma, None); }
            '.' => { self.add_token(TokenType::Dot, None); }
            '-' => { self.add_token(TokenType::Minus, None); }
            '+' => { self.add_token(TokenType::Plus, None); }
            ';' => { self.add_token(TokenType::Semicolon, None); }
            '*' => { self.add_token(TokenType::Star, None); }
            '?' => { self.add_token(TokenType::Question, None); }
            ':' => { self.add_token(TokenType::Colon, None); }

            // equal
            '!' => {
                if self.is_match('=') {
                    self.add_token(TokenType::BangEqual, None);
                } else {
                    self.add_token(TokenType::Bang, None);
                }
            }
            '=' => {if self.is_match('=') {
                    self.add_token(TokenType::EqualEqual, None);
                } else {
                    self.add_token(TokenType::Equal, None);
                }}
            '<' => {
                if self.is_match('=') {
                    self.add_token(TokenType::LessEqual, None);
                } else {
                    self.add_token(TokenType::Less, None);
                }
            }
            '>' => {
                if self.is_match('=') {
                    self.add_token(TokenType::GreaterEqual, None);
                } else {
                    self.add_token(TokenType::Greater, None);
                }
            }

            // comment or div
            '/' => {
                if self.is_match('/') {
                    while self.peek() != '\n' && !self.is_end() {
                        self.next_char();
                    }
                } else if self.is_match('*') {
                    loop {
                        if self.is_end() {
                            break;
                        }
                        if self.is_match('*') {
                            if self.is_match('/') || self.is_end() {
                                break;
                            }
                        }
                        self.next_char();
                    }
                } else {
                    self.add_token(TokenType::Slash, None);
                }
            }

            // space
            ' ' | '\r' | '\t' | '\n' => {}

            // string
            '"' => {
                self.scan_string();
            }
            _ => {
                // number
                if c.is_ascii_digit() {
                    self.scan_number();
                }
                // identifier
                else if c.is_ascii_alphabetic() || self.peek() == '_' {
                    self.scan_identifier();
                }
                else {
                    self.error(self.start, self.current, "Unexpected character.".to_string());
                }
            }
        }
    }

    fn scan_string(&mut self) {
        while self.peek() != '"' && !self.is_end() {
            self.next_char();
        }
        if self.is_end() {
            // error, no " at the end
            self.error(self.start, self.current-1, "Unterminated string.".to_string());
            return;
        }
        // close "
        self.next_char();
        let value = self.source.get(self.start+1..self.current-1);
        self.add_token(TokenType::String, value.map(|v|Literal::String(v.to_string())));
    }

    fn scan_number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.next_char();
        }
        // fractional part
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.next_char();
            while self.peek().is_ascii_digit() {
                self.next_char();
            }
        }

        self.add_token(TokenType::Number, self.source.get(self.start..self.current).map(|v|Literal::Number(v.to_string())));
    }

    fn scan_identifier(&mut self) {
        while self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
            self.next_char();
        }

        // check keyword
        let text = self.source.get(self.start..self.current).unwrap();
        for (keyword, keyword_type) in KEYWORDS {
            if keyword == text {
                self.add_token(keyword_type, None);
                return;
            }
        }
        self.add_token(TokenType::Identifier, None);
    }

    fn next_char(&mut self) -> char {
        let c = self.source.chars().nth(self.current);
        self.current += 1;
        c.unwrap()
    }

    fn peek(&mut self) -> char {
        if self.is_end() {
            return '\0';
        }
        let c = self.source.chars().nth(self.current);
        c.unwrap()
    }

    fn peek_next(&mut self) -> char {
        if self.current+1 >= self.source.len() {
            return '\0';
        }
        let c = self.source.chars().nth(self.current+1);
        c.unwrap()
    }

    fn add_token(&mut self, token_type: TokenType, literal: Option<Literal>) {
        let text = self.source.get(self.start..self.current);
        self.tokens.push(Token { text: text.unwrap().to_string(), start: self.start, end: self.current-1, token_type: token_type, literal: literal });
    }

    fn is_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn is_match(&mut self, expected: char) -> bool {
        if self.is_end() {
            return false;
        }
        if self.source.chars().nth(self.current).unwrap() != expected {
            return false;
        }
        self.current += 1;
        true
    }
}

impl ErrorReporter for Scanner {
    fn error(&self, start: usize, end: usize, error_content: String) {
        *self.had_error.borrow_mut() = true;
        println!("Scanner error: {} {} {} {}", self.file_path, start, end, error_content);
        self.errors.borrow_mut().push((start, end, error_content));
    }
}