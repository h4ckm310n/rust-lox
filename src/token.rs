pub const KEYWORDS: [(&str, TokenType); 18] = [
   ("and", TokenType::And), ("or", TokenType::Or),
   ("true", TokenType::True), ("false", TokenType::False),
   ("if", TokenType::If), ("else", TokenType::Else), ("for", TokenType::For), ("while", TokenType::While),
   ("print", TokenType::Print), ("return", TokenType::Return), ("super", TokenType::Super), ("this", TokenType::This),
   ("var", TokenType::Var), ("class", TokenType::Class), ("fun", TokenType::Fun), ("nil", TokenType::Nil),
   ("break", TokenType::Break), ("continue", TokenType::Continue)
];

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Token {
    pub text: String,
    pub start: usize,
    pub end: usize,
    pub token_type: TokenType,
    pub literal: Option<Literal>
}

#[derive(PartialEq, Clone, Eq, Hash)]
pub enum TokenType {
    LeftParen, RightParen,
    LeftSuqareBracket, RightSquareBracket,
    LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus, Semicolon, Slash, Star,
    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,
    Question, Colon,
    Identifier, String, Number,
    And, Or, True, False, If, Else, For, While, Break, Continue,
    Print, Return, Super, This,
    Var, Class, Fun, Nil,
    Eof
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum Literal {
    Bool(bool),
    String(String),
    Number(String),
    Nil
}
