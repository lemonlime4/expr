use std::fmt;

use crate::lex::{Ident, Token, lex};

pub fn parse(input: &str) -> Result<Expr<Ident>, String> {
    Parser::new(lex(input)?).run()
}

struct Parser {
    tokens: Vec<Token>,
}

impl Parser {
    fn new(mut tokens: Vec<Token>) -> Self {
        tokens.reverse();
        Self { tokens }
    }

    fn next_token(&mut self) -> Option<Token> {
        self.tokens.pop()
    }

    // fn peek_token(&self) -> Option<&Token> {
    //     self.tokens.last()
    // }

    fn run(&mut self) -> Result<Expr<Ident>, String> {
        let mut result = None;
        let mut add_segment = |expr| {
            if let Some(func) = result.take() {
                result = Some(Expr::App {
                    func: Box::new(func),
                    arg: Box::new(expr),
                });
            } else {
                result = Some(expr);
            }
        };

        while let Some(token) = self.next_token() {
            match token {
                Token::Lambda => {
                    let bound = match self.next_token() {
                        Some(Token::Ident(name)) => name.clone(),
                        _ => Err("Expected variable")?,
                    };
                    if self.next_token() != Some(Token::Dot) {
                        Err("Expected dot")?;
                    }
                    let body = Box::new(self.run()?);
                    add_segment(Expr::Abs { bound, body });
                }
                Token::Ident(name) => add_segment(Expr::Var(name)),
                Token::LeftP => {
                    let inner = self.run()?;
                    if self.next_token() != Some(Token::RightP) {
                        Err("Expected )")?;
                    }
                    add_segment(inner);
                }
                Token::RightP => {
                    self.tokens.push(Token::RightP);
                    break;
                }
                Token::Dot => Err("Unexpected token .")?,
                Token::Equals => Err("Equals not implemented yet")?,
            }
        }
        result.ok_or("Cannot parse nothing".into())
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Expr<V> {
    Var(V),
    Abs { bound: V, body: Box<Self> },
    App { func: Box<Self>, arg: Box<Self> },
}

impl<V: ExprVarDisplay> fmt::Display for Expr<V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Var(x) => x.fmt_normal(f),
            Self::Abs { bound, body } => {
                f.write_str("\\")?;
                bound.fmt_lambda_bound(f)?;
                f.write_str(".")?;
                body.fmt(f)
            }
            Self::App { func, arg } => {
                let left_paren = matches!(**func, Self::Abs { .. });
                let right_paren = !matches!(**arg, Self::Var(..));
                match (left_paren, right_paren) {
                    (false, false) => write!(f, "{func} {arg}"),
                    (true, false) => write!(f, "({func}) {arg}"),
                    (false, true) => write!(f, "{func} ({arg})"),
                    (true, true) => write!(f, "({func}) ({arg})"),
                }
            }
        }
    }
}

pub trait ExprVarDisplay {
    fn fmt_normal(&self, f: &mut fmt::Formatter) -> fmt::Result;
    fn fmt_lambda_bound(&self, f: &mut fmt::Formatter) -> fmt::Result;
}

impl ExprVarDisplay for Ident {
    fn fmt_normal(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self)
    }
    fn fmt_lambda_bound(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt_normal(f)
    }
}

impl<V> From<V> for Expr<V> {
    fn from(var: V) -> Expr<V> {
        Expr::Var(var)
    }
}
