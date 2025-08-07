use std::fmt;

pub fn lex(input: &str) -> Result<Vec<Token>, String> {
    Lexer::new(input).run()
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
            let token = match c {
                // '\n' => Token::Newline,
                '\\' => Token::Lambda,
                '.' => Token::Dot,
                '(' => Token::LeftP,
                ')' => Token::RightP,
                '=' => Token::Equals,
                c if c.is_ascii_alphabetic() => {
                    self.next_char_while(|c| c.is_ascii_alphabetic());
                    Token::Ident(self.input[start..self.pos].to_string())
                }
                _ => Err(format!("unknown start of token '{c}'"))?,
            };
            // println!("-- read {token:?}");
            self.tokens.push(token);
        }
    }
}

pub type Ident = String;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Token {
    // Newline,
    Lambda,
    Dot,
    LeftP,
    RightP,
    Equals,
    Ident(Ident),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // Newline => write!(f, "\\n"),
            Token::Lambda => write!(f, "\\"),
            Token::Dot => write!(f, "."),
            Token::LeftP => write!(f, "("),
            Token::RightP => write!(f, ")"),
            Token::Equals => write!(f, "="),
            Token::Ident(ident) => write!(f, "{ident}"),
        }
    }
}
