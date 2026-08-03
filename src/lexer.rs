use std::cmp::PartialEq;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Fn,
    Ret,
    If,
    Else,
    Elif,
    While,
    For,
    And,
    Or,
    Not,
    Equal,
    DoubleEqual,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterEqual,
    LessEqual,
    Var,
    Immut,
    In,
    Try,
    Catch,
    As,
    Break,
    Continue,

    Plus,
    Minus,
    Star,
    Slash,
    Comma,
    Period,
    DoublePeriod,
    Colon,
    Semicolon,
    Percent,

    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,

    Ident(String),
    IntLit(i64),
    FloatLit(f64),
    StrLit(String),
    BoolLit(bool),

    Eof,
}

pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(code: &'a str) -> Lexer<'a> {
        Lexer {
            chars: code.chars().peekable(),
        }
    }

    pub fn tokenize(mut self) -> Vec<Token> {
        let mut tokens: Vec<Token> = vec![];
        loop {
            let token: Token = self.next_token();
            if token == Token::Eof {
                break;
            }
            tokens.push(token);
        }
        tokens
    }

    fn next_token(&mut self) -> Token {
        loop {
            self.skip_whitespace();
            let c = match self.chars.next() {
                Some(c) => c,
                None => return Token::Eof,
            };
            match c {
                ',' => return Token::Comma,
                ':' => return Token::Colon,
                ';' => return Token::Semicolon,
                '(' => return Token::LeftParen,
                ')' => return Token::RightParen,
                '[' => return Token::LeftBracket,
                ']' => return Token::RightBracket,
                '{' => return Token::LeftBrace,
                '}' => return Token::RightBrace,
                '+' => return Token::Plus,
                '*' => return Token::Star,
                '-' => return Token::Minus,
                '/' => return Token::Slash,
                '%' => return Token::Percent,
                '>' => return Token::GreaterThan,
                '<' => return Token::LessThan,
                '.' => {
                    return if self.chars.peek() == Some(&'.') {
                        self.chars.next();
                        Token::DoublePeriod
                    } else {
                        Token::Period
                    };
                }
                '!' if matches!(self.chars.peek(), Some('!') | Some('/') | Some('=')) => {
                    if self.chars.peek() == Some(&'=') {
                        self.chars.next();
                        return Token::NotEqual;
                    }
                    if self.chars.peek() == Some(&'!') {
                        loop {
                            if matches!(self.chars.next(), None | Some('\n')) {
                                break;
                            }
                        }
                        continue;
                    }
                    loop {
                        match self.chars.next() {
                            None => break,
                            Some('/') if self.chars.peek() == Some(&'!') => {
                                self.chars.next();
                                break;
                            }
                            _ => {}
                        }
                    }
                    continue;
                }
                '=' => {
                    return match self.chars.peek() {
                        Some('=') => {
                            self.chars.next();
                            Token::DoubleEqual
                        }
                        Some('<') => {
                            self.chars.next();
                            Token::LessEqual
                        }
                        Some('>') => {
                            self.chars.next();
                            Token::GreaterEqual
                        }
                        _ => Token::Equal,
                    };
                }
                '"' => return self.next_string(),
                c if c.is_alphabetic() => return self.next_word(&c),
                c if c.is_ascii_digit() => return self.next_number(&c),
                _ => panic!("Unexpected character: {}", c),
            }
        }
    }

    fn next_word(&mut self, curr: &char) -> Token {
        let mut word: String = curr.to_string();

        while let Some(&c) = self.chars.peek() {
            if c.is_alphanumeric() || c == '_' {
                self.chars.next();
                word.push(c);
            } else {
                break;
            }
        }

        match word.as_str() {
            "fn" => Token::Fn,
            "ret" => Token::Ret,
            "if" => Token::If,
            "else" => Token::Else,
            "elif" => Token::Elif,
            "while" => Token::While,
            "for" => Token::For,
            "and" => Token::And,
            "or" => Token::Or,
            "not" => Token::Not,
            "var" => Token::Var,
            "immut" => Token::Immut,
            "in" => Token::In,
            "try" => Token::Try,
            "catch" => Token::Catch,
            "as" => Token::As,
            "break" => Token::Break,
            "continue" => Token::Continue,
            "true" => Token::BoolLit(true),
            "false" => Token::BoolLit(false),
            _ => Token::Ident(word),
        }
    }

    fn next_number(&mut self, curr: &char) -> Token {
        let mut number: String = curr.to_string();

        while let Some(&c) = self.chars.peek() {
            if c.is_ascii_digit() {
                self.chars.next();
                number.push(c);
            } else {
                break;
            }
        }

        if self.chars.peek() == Some(&'.') {
            let mut lookahead = self.chars.clone();
            lookahead.next();
            if lookahead.peek() != Some(&'.') {
                self.chars.next();
                number.push('.');
                while let Some(&c) = self.chars.peek() {
                    if c.is_ascii_digit() {
                        self.chars.next();
                        number.push(c);
                    } else {
                        break;
                    }
                }
            }
        }

        if number.contains('.') {
            Token::FloatLit(number.parse().unwrap())
        } else {
            Token::IntLit(number.parse().unwrap())
        }
    }

    fn next_string(&mut self) -> Token {
        let mut string: String = String::new();
        while let Some(c) = self.chars.next() {
            if c == '"' {
                break;
            }
            string.push(c);
        }
        Token::StrLit(string)
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.chars.peek() {
            if c.is_whitespace() {
                self.chars.next();
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::{Lexer, Token};

    #[test]
    fn test_next_token() {
        let lexer: Lexer = Lexer::new("!! hi\n!/h/! !=");
        assert_eq!(lexer.tokenize(), vec![Token::NotEqual]);
    }
}
