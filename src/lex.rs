use std::fmt;

use anyhow::{Result, bail};
use ecow::EcoString;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Token {
    Ident(EcoString),
    NumLit(EcoString),
    Newline,
    Assign,
    LeftParen,
    RightParen,
    Comma,
    Dot,

    Plus,
    Minus,
    Cdot,
    Slash,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::Ident(s) | Self::NumLit(s) => s,
            Self::Newline => "\\n",
            Self::LeftParen => "(",
            Self::RightParen => ")",
            Self::Comma => ",",
            Self::Dot => ".",
            Self::Assign => ":=",
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Cdot => "*",
            Self::Slash => "/",
        })
    }
}

struct Lexer<'a> {
    input: &'a str,
    pos: usize,
    tokens: Vec<Token>,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        assert!(input.is_ascii());
        Self {
            input,
            pos: 0,
            tokens: Vec::new(),
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn next_char_if(&mut self, f: impl Fn(&char) -> bool) -> Option<char> {
        let c = self.peek_char()?;
        if f(&c) {
            self.pos += 1;
            Some(c)
        } else {
            None
        }
    }

    fn next_char(&mut self) -> Option<char> {
        self.next_char_if(|_| true)
    }

    fn next_char_exact(&mut self, c: char) -> bool {
        self.next_char_if(|&next| next == c).is_some()
    }

    fn one_or_more_greedy(&mut self, f: impl Fn(&char) -> bool) -> bool {
        let mut found = false;
        while self.next_char_if(&f).is_some() {
            found = true;
        }
        found
    }

    fn next_char_while(&mut self, f: impl Fn(&char) -> bool) {
        self.one_or_more_greedy(f);
    }

    fn read_digits(&mut self) -> bool {
        self.one_or_more_greedy(char::is_ascii_digit)
    }

    fn read_number_unsigned(&mut self) -> bool {
        if self.read_digits() {
            self.next_char_exact('.');
            self.read_digits();
            return true;
        }
        let start = self.pos;
        if self.next_char_exact('.') {
            if self.read_digits() {
                true
            } else {
                self.pos = start;
                false
            }
        } else {
            false
        }
    }

    fn run(mut self) -> Result<Vec<Token>> {
        fn is_whitespace(c: &char) -> bool {
            c.is_ascii_whitespace() && !"\r\n".contains(*c)
        }
        self.next_char_while(is_whitespace);

        while let (start, Some(c)) = (self.pos, self.next_char()) {
            let token = match c {
                '\r' if self.next_char_exact('\n') => Token::Newline,
                '\n' => Token::Newline,
                ':' if self.next_char_exact('=') => Token::Assign,
                ')' => Token::RightParen,
                '(' => Token::LeftParen,
                ',' => Token::Comma,
                '+' => Token::Plus,
                '-' => Token::Minus,
                '*' => Token::Cdot,
                '/' => Token::Slash,
                c if c.is_ascii_alphabetic() => {
                    self.next_char_while(char::is_ascii_alphabetic);
                    Token::Ident(EcoString::from(&self.input[start..self.pos]))
                }
                '0'..='9' => {
                    self.pos = start;
                    self.read_number_unsigned();
                    Token::NumLit(EcoString::from(&self.input[start..self.pos]))
                }
                '.' => {
                    if self.read_digits() {
                        Token::NumLit(EcoString::from(&self.input[start..self.pos]))
                    } else {
                        Token::Dot
                    }
                }
                _ => bail!("Unknown character {c:?}"),
            };
            self.tokens.push(token);
            self.next_char_while(is_whitespace);
        }

        Ok(self.tokens)
    }
}

pub fn lex(input: &str) -> Result<Vec<Token>> {
    Lexer::new(input).run()
}
