use std::str::Chars;

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    Identifier,           // 标识符
    Literal(LiteralKind), // 字符串字面量，如 `"Hello, World!"`
    Dot,                  // 点 `.`
    Comment,              // 注释
    Comma,                // 逗号 `,`
    Colon,                // 冒号 `:`
    LParen,               // 左括号 `(`
    RParen,               // 右括号 `)`
    LBracket,             // 左中括号 `[`
    RBracket,             // 右中括号 `]`
    Plus,                 // 加号 `+`
    Minus,                // 减号 `-`
    Star,                 // 星号 `*`
    Pound,                // 井号 `#`
    Bang,                 // 叹号 `!`
    Percent,              // 百分号 '%'
    Slash,                // 斜杠 `/`
    Whitespace,           // 空白字符
    Unknown,              // 未知字符
    Eof,
}
#[derive(Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub len: u32,
}

impl Token {
    pub fn new(kind: TokenKind, len: u32) -> Self {
        Token { kind, len }
    }
}
#[derive(Debug, PartialEq)]
pub enum LiteralKind {
    Int(IntBase),
    Float,
    Char,
    String,
}
#[derive(Debug, PartialEq)]
pub enum IntBase {
    Binary,
    Octal,
    Decimal,
    Hex,
}
pub(crate) const EOF_CHAR: char = '\0';

pub struct Cursor<'a> {
    len_remaining: usize,
    chars: Chars<'a>,
}

impl<'a> Cursor<'a> {
    pub fn new(input: &'a str) -> Self {
        Cursor {
            len_remaining: input.len(),
            chars: input.chars(),
        }
    }

    pub fn as_str(&self) -> &'a str {
        self.chars.as_str()
    }

    pub fn first(&self) -> char {
        self.chars.clone().next().unwrap_or(EOF_CHAR)
    }

    pub fn second(&self) -> char {
        let mut chars = self.chars.clone();
        chars.next();
        chars.next().unwrap_or(EOF_CHAR)
    }
    pub fn bump(&mut self) -> Option<char> {
        self.chars.next()
    }
    pub fn eat_while<F>(&mut self, mut f: F)
    where
        F: FnMut(char) -> bool,
    {
        while f(self.first()) && !self.is_eof() {
            self.bump();
        }
    }
    pub fn is_eof(&self) -> bool {
        self.first() == EOF_CHAR
    }
    fn position(&self) -> u32 {
        (self.len_remaining - self.chars.as_str().len()) as u32
    }
    fn reset_position(&mut self) {
        self.len_remaining = self.chars.as_str().len()
    }
}

fn is_id_continue(c: char) -> bool {
    c.is_ascii_alphabetic() || c.is_ascii_digit() || c == '_' || c == '@'
}
fn is_id_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

impl Cursor<'_> {
    fn whitespace(&mut self) {
        self.eat_while(char::is_whitespace);
    }
    fn number_literal(&mut self, fiest_char: char) -> TokenKind {
        if fiest_char == '0' {
            match self.first() {
                'b' | 'B' => {
                    self.bump();
                    self.eat_decimal();
                    return TokenKind::Literal(LiteralKind::Int(IntBase::Binary));
                }
                'x' | 'X' => {
                    self.bump();
                    self.eat_hex();
                    return TokenKind::Literal(LiteralKind::Int(IntBase::Hex));
                }
                'f' | 'F' => {
                    self.bump();
                    self.eat_while(|c| {
                        c.is_digit(10) || c == '.' || c == 'e' || c == 'E' || c == '+' || c == '-'
                    });
                    return TokenKind::Literal(LiteralKind::Float);
                }
                '1'..'9' => {
                    self.eat_decimal();
                    return TokenKind::Literal(LiteralKind::Int(IntBase::Octal));
                }
                '.' | 'e' | 'E' => {}
                _ => {}
            }
        }
        self.eat_decimal();
        match self.first() {
            '.' => {
                self.bump();
                self.eat_decimal();
                match self.first() {
                    'e' | 'E' => {
                        self.bump();
                        self.eat_float_exponent();
                    }
                    _ => {}
                }
                TokenKind::Literal(LiteralKind::Float)
            }
            'e' | 'E' => {
                self.bump();
                self.eat_float_exponent();
                TokenKind::Literal(LiteralKind::Float)
            }
            _ => TokenKind::Literal(LiteralKind::Int(IntBase::Decimal)),
        }
    }

    fn eat_decimal(&mut self) {
        self.eat_while(|c| c.is_digit(10));
    }
    fn eat_float_exponent(&mut self) {
        if self.first() == '-' || self.first() == '+' {
            self.bump();
        }
        self.eat_decimal();
    }
    fn eat_hex(&mut self) {
        self.eat_while(|c| c.is_digit(16));
    }

    fn string_literal(&mut self) {
        while let Some(c) = self.bump() {
            match c {
                '"' => break,
                '\\' if self.first() == '\\' || self.first() == '"' => {
                    self.bump();
                }
                _ => {}
            }
        }
    }
    fn char_literal(&mut self) {
        while let Some(c) = self.bump() {
            match c {
                '\'' => break,
                '\\' if self.first() == '\\' || self.first() == '\'' => {
                    self.bump();
                }
                _ => {}
            }
        }
    }
    fn identifier(&mut self) {
        self.eat_while(is_id_continue);
    }
    fn line_comment(&mut self) -> TokenKind {
        self.eat_while(|c| c != '\n');
        TokenKind::Comment
    }
    fn block_comment(&mut self) -> TokenKind {
        self.bump();
        while !self.is_eof() {
            if self.first() == '*' && self.second() == '/' {
                self.bump();
                self.bump();
                break;
            }
            self.bump();
        }
        TokenKind::Comment
    }
    pub fn advance_token(&mut self) -> Token {
        let first_char = match self.bump() {
            Some(c) => c,
            None => return Token::new(TokenKind::Eof, 0),
        };

        let token_kind = match first_char {
            '/' => match self.first() {
                '/' => self.line_comment(),
                '*' => self.block_comment(),
                _ => TokenKind::Slash,
            },
            c if c.is_whitespace() => {
                self.whitespace();
                TokenKind::Whitespace
            }
            c @ '0'..='9' => self.number_literal(c),
            '"' => {
                self.string_literal();
                TokenKind::Literal(LiteralKind::String)
            }
            '\'' => {
                self.char_literal();
                TokenKind::Literal(LiteralKind::Char)
            }
            c if is_id_start(c) => {
                self.identifier();
                TokenKind::Identifier
            }
            ',' => TokenKind::Comma,
            ':' => TokenKind::Colon,
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => TokenKind::Star,
            '#' => TokenKind::Pound,
            '!' => TokenKind::Bang,
            '%' => TokenKind::Percent,
            '.' => TokenKind::Dot,
            _ => TokenKind::Unknown,
        };

        let len = self.position();
        let res = Token::new(token_kind, len);
        self.reset_position();
        res
    }
}
pub fn tokenize(input: &str) -> impl Iterator<Item = Token> + '_ {
    let mut cursor = Cursor::new(input);
    std::iter::from_fn(move || {
        let token = cursor.advance_token();
        if token.kind != TokenKind::Eof {
            Some(token)
        } else {
            None
        }
    })
}
