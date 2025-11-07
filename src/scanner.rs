pub struct Scanner {
    file_path: String,
    source: Vec<char>,
    start: usize,
    current: usize,
    line: usize
}

impl Scanner {
    pub fn init(file_path: String, source: String) -> Self {
        Self {
            file_path: file_path,
            source: source.chars().collect(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan(&mut self) {
        loop {
            let token = self.scan_token();
            if let Ok(token) = token {
                if token.token_type == TokenType::Eof {
                    break;
                }
            }
        }
    }

    fn scan_token(&mut self) -> Result<Token, (&str, usize)> {
        self.skip_whitespace();
        self.start = self.current;
        if self.is_at_end() { return Ok(self.make_token(TokenType::Eof)); }
        let c = self.advance();
        if c.is_ascii_alphabetic() || c == '_' { return Ok(self.scan_identifier()); }
        if c.is_ascii_digit() { return Ok(self.scan_number()); }
        match c {
            //single char
            '(' => { return Ok(self.make_token(TokenType::LeftParen)); }
            ')' => { return Ok(self.make_token(TokenType::RightParen)); }
            '{' => { return Ok(self.make_token(TokenType::LeftBrace)); }
            '}' => { return Ok(self.make_token(TokenType::RightBrace)); }
            ',' => { return Ok(self.make_token(TokenType::Comma)); }
            '.' => { return Ok(self.make_token(TokenType::Dot)); }
            '-' => { return Ok(self.make_token(TokenType::Minus)); }
            '+' => { return Ok(self.make_token(TokenType::Plus)); }
            ';' => { return Ok(self.make_token(TokenType::Semicolon)); }
            '*' => { return Ok(self.make_token(TokenType::Star)); }

            // equal
            '!' => {
                if self.is_match('=') {
                    return Ok(self.make_token(TokenType::BangEqual));
                } else {
                    return Ok(self.make_token(TokenType::Bang));
                }
            }
            '=' => {
                if self.is_match('=') {
                    return Ok(self.make_token(TokenType::EqualEqual));
                } else {
                    return Ok(self.make_token(TokenType::Equal));
                }}
            '<' => {
                if self.is_match('=') {
                    return Ok(self.make_token(TokenType::LessEqual));
                } else {
                    return Ok(self.make_token(TokenType::Less));
                }
            }
            '>' => {
                if self.is_match('=') {
                    return Ok(self.make_token(TokenType::GreaterEqual));
                } else {
                    return Ok(self.make_token(TokenType::Greater));
                }
            }

            // string
            '"' => {
                return self.scan_string();
            }
            _ => ()
        }
        Err(("Unexpected character.", self.line))
    }

    fn make_token(&self, token_type: TokenType) -> Token {
        let token = Token {
            token_type: token_type,
            start: self.start,
            lenght: self.current - self.start,
            line: self.line,
        };
        token
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source[self.current-1]
    }

    fn is_match(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source[self.current] != expected {
            return false;
        }
        self.current += 1;
        true
    }

    fn skip_whitespace(&mut self) {
        loop {
            let c = self.peek();
            match c {
                ' ' | '\r' | '\t' => { self.advance(); }
                '\n' => {
                    self.line += 1;
                    self.advance();
                }
                '/' => {
                    if self.peek_next() == '/' {
                        while self.peek() != '\n' && !self.is_at_end() {
                            self.advance();
                        }
                    } else {
                        return;
                    }
                }
                _ => { return; }
            }
        }
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source[self.current]
    }

    fn peek_next(&self) -> char {
        if self.current+1 >= self.source.len() {
            return '\0';
        }
        self.source[self.current+1]
    }

    fn scan_string(&mut self) -> Result<Token, (&str, usize)> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        if self.is_at_end() {
            // error, no " at the end
            return Err(("Unterminated string.", self.line));
        }
        // close "
        self.advance();
        Ok(self.make_token(TokenType::String))
    }

    fn scan_number(&mut self) -> Token {
        while self.peek().is_ascii_digit() {
            self.advance();
        }
        // fractional part
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        self.make_token(TokenType::Number)
    }

    fn scan_identifier(&mut self) -> Token {
        while self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
            self.advance();
        }

        // check keyword
        /*let text = self.source.get(self.start..self.current).unwrap();
        for (keyword, keyword_type) in KEYWORDS {
            if keyword == text {
                self.add_token(keyword_type, None);
                return;
            }
        }*/
        self.make_token(self.get_identifier_type())
    }

    fn get_identifier_type(&self) -> TokenType {
        match self.source[self.start] {
            'a' => return self.check_keyword(1, 2, "nd", TokenType::And),
            'c' => return self.check_keyword(1, 4, "lass", TokenType::Class),
            'e' => return self.check_keyword(1, 3, "lse", TokenType::Else),
            'f' => {
                if self.current - self.start > 1 {
                    match self.source[self.start+1] {
                        'a' => return self.check_keyword(2, 3, "lse", TokenType::False),
                        'o' => return self.check_keyword(2, 1, "r", TokenType::For),
                        'u' => return self.check_keyword(2, 1, "n", TokenType::Fun),
                        _ => ()
                    };
                }
            },
            'i' => return self.check_keyword(1, 1, "f", TokenType::If),
            'n' => return self.check_keyword(1, 2, "il", TokenType::Nil),
            'o' => return self.check_keyword(1, 1, "r", TokenType::Or),
            'p' => return self.check_keyword(1, 4, "rint", TokenType::Print),
            'r' => return self.check_keyword(1, 5, "eturn", TokenType::Return),
            's' => return self.check_keyword(1, 4, "uper", TokenType::Super),
            't' => {
                if self.current - self.start > 1 {
                    match self.source[self.start+1] {
                        'h' => return self.check_keyword(2, 2, "is", TokenType::This),
                        'r' => return self.check_keyword(2, 2, "ue", TokenType::True),
                        _ => ()
                    };
                }
            },
            'v' => return self.check_keyword(1, 2, "ar", TokenType::Var),
            'w' => return self.check_keyword(1, 4, "hile", TokenType::While),
            _ => ()
        }
        return TokenType::Identifier
    }

    fn check_keyword(&self, start: usize, length: usize, rest: &str, token_type: TokenType) -> TokenType {
        if self.current - self.start == start + length &&
           self.source[self.start+start..self.start+length] == rest.chars().collect::<Vec<char>>() {
            return token_type;
        }
        TokenType::Identifier
    }
}

#[derive(PartialEq, Clone, Eq, Hash)]
pub struct Token {
    token_type: TokenType,
    start: usize,
    lenght: usize,
    line: usize
}

#[derive(PartialEq, Clone, Eq, Hash)]
pub enum TokenType {
    LeftParen, RightParen,
    LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus, Semicolon, Slash, Star,
    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,
    Identifier, String, Number,
    And, Or, True, False, If, Else, For, While,
    Print, Return, Super, This,
    Var, Class, Fun, Nil,
    Eof
}