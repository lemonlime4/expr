use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Operator {
    Equals,
    Plus,
    Minus,
    Dot,
    Slash,
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Operator::*;
        write!(
            f,
            "{}",
            match self {
                Equals => "=",
                Plus => "+",
                Minus => "-",
                Dot => "*",
                Slash => "/",
            }
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BracketSide {
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BracketKind {
    Paren,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Bracket {
    side: BracketSide,
    kind: BracketKind,
}

impl Bracket {
    const LEFT_PAREN: Self = Self {
        side: BracketSide::Left,
        kind: BracketKind::Paren,
    };
    const RIGHT_PAREN: Self = Self {
        side: BracketSide::Right,
        kind: BracketKind::Paren,
    };
}

impl fmt::Display for Bracket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use BracketKind::*;
        use BracketSide::*;
        write!(
            f,
            "{}",
            match (self.side, self.kind) {
                (Left, Paren) => "(",
                (Right, Paren) => ")",
            }
        )
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Token {
    Ident(String),
    Lit(String),
    Op(Operator),
    Bracket(Bracket),
    Newline,
    // Whitespace,
}

impl From<Operator> for Token {
    fn from(op: Operator) -> Token {
        Token::Op(op)
    }
}

impl From<Bracket> for Token {
    fn from(bracket: Bracket) -> Token {
        Token::Bracket(bracket)
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Token::*;
        match self {
            Ident(i) => write!(f, "{i}"),
            Lit(l) => write!(f, "{l}"),
            Op(o) => write!(f, "{o}"),
            Bracket(b) => write!(f, "{b}"),
            Newline => write!(f, "\\n"),
            // Whitespace => write!(f, "\\s"),
        }
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

    fn next_char_while(&mut self, f: impl Fn(char) -> bool) {
        while self.next_char_if(&f).is_some() {}
    }

    fn run(mut self) -> Result<Vec<Token>, String> {
        loop {
            self.next_char_while(|c| c.is_ascii_whitespace());
            let start = self.pos;
            let c = self.next_char();
            if c.is_none() {
                break Ok(self.tokens);
            }
            let c = c.unwrap();
            let token: Token = match c {
                '=' => Operator::Equals.into(),
                '+' => Operator::Plus.into(),
                '-' => Operator::Minus.into(),
                '*' => Operator::Dot.into(),
                '/' => Operator::Slash.into(),
                '(' => Bracket::LEFT_PAREN.into(),
                ')' => Bracket::RIGHT_PAREN.into(),
                '\n' => Token::Newline,
                c if c.is_ascii_alphabetic() => {
                    self.next_char_while(|c| c.is_ascii_alphabetic());
                    Token::Ident(self.input[start..self.pos].to_string())
                }
                c if c.is_digit(10) => {
                    self.next_char_while(|c| c.is_digit(10));
                    Token::Lit(self.input[start..self.pos].to_string())
                }
                _ => Err(format!("unknown token '{c}'"))?,
            };
            println!("-- read {token:?}");
            self.tokens.push(token);
        }
    }
}

pub fn lex(input: &str) -> Result<Vec<Token>, String> {
    Lexer::new(input).run()
}
