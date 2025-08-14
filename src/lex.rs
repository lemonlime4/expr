use std::fmt;

use ecow::EcoString;

// #[derive(Clone, Copy, PartialEq, Eq, Debug)]
// pub enum Operator {
//     Equals,
//     Plus,
//     Minus,
//     Dot,
//     Slash,
// }

// impl fmt::Display for Operator {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         use Operator::*;
//         write!(
//             f,
//             "{}",
//             match self {
//                 Equals => "=",
//                 Plus => "+",
//                 Minus => "-",
//                 Dot => "*",
//                 Slash => "/",
//             }
//         )
//     }
// }

// #[derive(Clone, Copy, PartialEq, Eq, Debug)]
// pub enum BracketSide {
//     Left,
//     Right,
// }

// #[derive(Clone, Copy, PartialEq, Eq, Debug)]
// pub enum BracketKind {
//     Paren,
// }

// #[derive(Clone, Copy, PartialEq, Eq, Debug)]
// pub struct Bracket {
//     side: BracketSide,
//     kind: BracketKind,
// }

// impl Bracket {
//     const LEFT_PAREN: Self = Self {
//         side: BracketSide::Left,
//         kind: BracketKind::Paren,
//     };
//     const RIGHT_PAREN: Self = Self {
//         side: BracketSide::Right,
//         kind: BracketKind::Paren,
//     };
// }

// impl fmt::Display for Bracket {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         use BracketKind::*;
//         use BracketSide::*;
//         write!(
//             f,
//             "{}",
//             match (self.side, self.kind) {
//                 (Left, Paren) => "(",
//                 (Right, Paren) => ")",
//             }
//         )
//     }
// }

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Token {
    Ident(EcoString),
    NumLit(EcoString),
    Newline,
    LeftParen,
    RightParen,
    Dot,

    Equals,
    Plus,
    Minus,
    Cdot,
    Slash,
}

// impl From<Operator> for Token<'_> {
//     fn from(op: Operator) -> Token<'static> {
//         Token::Op(op)
//     }
// }

// impl From<Bracket> for Token<'_> {
//     fn from(bracket: Bracket) -> Token<'static> {
//         Token::Bracket(bracket)
//     }
// }

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // match self {
        //     Ident(i) => write!(f, "{i}"),
        //     NumLit(l) => write!(f, "{l}"),
        //     Op(o) => write!(f, "{o}"),
        //     Bracket(b) => write!(f, "{b}"),
        //     Newline => write!(f, "\\n"),
        //     // Whitespace => write!(f, "\\s"),
        // }
        f.write_str(match self {
            Self::Ident(s) | Self::NumLit(s) => s,
            Self::Newline => "\\n",
            Self::LeftParen => "(",
            Self::RightParen => ")",
            Self::Dot => ".",
            Self::Equals => "=",
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

    fn next_char_if(&mut self, f: impl Fn(char) -> bool) -> Option<char> {
        let c = self.peek_char()?;
        if f(c) {
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
        self.next_char_if(|next| next == c).is_some()
    }

    fn one_or_more_greedy(&mut self, f: impl Fn(char) -> bool) -> bool {
        let mut found = false;
        while self.next_char_if(&f).is_some() {
            found = true;
        }
        found
    }

    fn next_char_while(&mut self, f: impl Fn(char) -> bool) {
        self.one_or_more_greedy(f);
    }

    fn read_digits(&mut self) -> bool {
        self.one_or_more_greedy(|c| c.is_ascii_digit())
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

    fn run(mut self) -> Result<Vec<Token>, String> {
        self.next_char_while(|c| c.is_ascii_whitespace());

        while let (start, Some(c)) = (self.pos, self.next_char()) {
            self.next_char_while(|c| c.is_ascii_whitespace());

            let token = match c {
                '\r' if self.next_char_exact('\n') => Token::Newline,
                '\n' => Token::Newline,
                ')' => Token::RightParen,
                '(' => Token::LeftParen,
                '=' => Token::Equals,
                '+' => Token::Plus,
                '-' => Token::Minus,
                '*' => Token::Cdot,
                '/' => Token::Slash,
                c if c.is_ascii_alphabetic() => {
                    self.next_char_while(|c| c.is_ascii_alphabetic());
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
                _ => Err(format!("Unexpected start of token '{c}'"))?,
            };
            println!("-- read {token:?}");
            self.tokens.push(token);
        }

        Ok(self.tokens)
    }
}

pub fn lex(input: &str) -> Result<Vec<Token>, String> {
    Lexer::new(input).run()
}
