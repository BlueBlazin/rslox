#[derive(Debug)]
pub struct Token {
    pub tok_type: TokenType,
    pub line: usize,
}

#[derive(Debug, PartialEq)]
pub enum TokenType {
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    Bang,
    BangEq,
    Equal,
    EqualEq,
    Greater,
    GreaterEq,
    Less,
    LessEq,

    Ident(String),
    Str(String),
    Num(f64),

    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
}

impl TokenType {
    pub fn precedence(&self) -> usize {
        match self {
            TokenType::Equal => 1,
            TokenType::Or => 2,
            TokenType::And => 3,
            TokenType::EqualEq | TokenType::BangEq => 4,
            TokenType::Less | TokenType::LessEq | TokenType::Greater | TokenType::GreaterEq => 5,
            TokenType::Plus | TokenType::Minus => 6,
            TokenType::Star | TokenType::Slash => 7,
            TokenType::Bang => 8,
            TokenType::Dot => 9,
            _ => 0,
        }
    }
}
