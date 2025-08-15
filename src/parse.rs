// use std::rc::Rc;

use std::{
    cmp::{self, Ordering},
    fmt::{self, Binary},
};

use ecow::EcoVec;

use crate::lex::{Token, lex};

type Ident = ecow::EcoString;
// type Ident = Rc<str>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    // Multiply,
    DotProduct,
    // CrossProduct,
    Divide,
}

impl BinaryOp {
    fn binding_power(&self) -> u8 {
        match self {
            Self::Add => 1,
            Self::Subtract => 1,
            // Self::Multiply => 2,
            Self::DotProduct => 2,
            Self::Divide => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Negate,
    Plus,
}

#[derive(Debug)]
pub enum Expr {
    Lit(f64),
    Variable(Ident),
    Call {
        func: Ident,
        arg: Box<Expr>,
        args: EcoVec<Expr>,
    },
    UnOp {
        op: UnaryOp,
        arg: Box<Expr>,
    },
    BinOp {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

struct Parser {
    tokens: Vec<Token>,
}

impl Parser {
    pub fn new(mut tokens: Vec<Token>) -> Self {
        tokens.reverse();
        Self { tokens }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.last()
    }

    fn next(&mut self) -> Option<Token> {
        self.tokens.pop()
    }

    fn next_while<T>(&mut self, mut f: impl FnMut(&Token) -> Option<T>) -> Vec<T> {
        let mut result = Vec::new();
        while let Some(processed) = self.peek().and_then(&mut f) {
            self.next();
            result.push(processed);
        }
        result
    }

    // fn next_if(&mut self, f: impl Fn(&Token) -> bool) -> Option<Token> {
    //     if self.peek().is_some_and(f) {
    //         self.next()
    //     } else {
    //         None
    //     }
    // }

    pub fn parse(&mut self, last_op: Option<BinaryOp>) -> Result<Expr, String> {
        println!("parsing {:?}", self.tokens);
        // let negated = self.next_if(|t| *t == Token::Minus).is_some();
        let signs = self.next_while(|t| match t {
            Token::Minus => Some(true),
            Token::Plus => Some(false),
            _ => None,
        });
        let mut left = match self.next() {
            Some(Token::LeftParen) => {
                let inner = self.parse(None)?;
                if self.next() != Some(Token::RightParen) {
                    Err("Expected )")?;
                }
                inner
            }
            Some(Token::NumLit(s)) => Expr::Lit(s.parse().expect("Failed to parse float literal")),
            Some(Token::Ident(name)) => Expr::Variable(name),
            Some(t) => Err(format!("unknown token {t:?}"))?,
            None => Err("cannot parse empty expression")?,
        };
        for sign in signs {
            let op = match sign {
                true => UnaryOp::Negate,
                false => UnaryOp::Plus,
            };
            left = Expr::un_op(op, left);
        }
        loop {
            let op = match self.peek() {
                Some(Token::Plus) => BinaryOp::Add,
                Some(Token::Minus) => BinaryOp::Subtract,
                Some(Token::Cdot) => BinaryOp::DotProduct,
                Some(Token::Slash) => BinaryOp::Divide,
                _ => return Ok(left),
            };
            if Some(op.binding_power()) > last_op.map(|op| op.binding_power()) {
                self.next();
                println!("parse -- recursing -- {op:?}");
                let right = self.parse(Some(op))?;
                left = Expr::bin_op(op, left, right);
            } else {
                break Ok(left);
            }
        }
    }
}

pub fn parse(input: &str) -> Result<Expr, String> {
    let tokens = lex(input)?;
    Parser::new(tokens).parse(None)
}

impl Expr {
    pub fn un_op(op: UnaryOp, arg: Self) -> Self {
        Self::UnOp {
            op,
            arg: Box::new(arg),
        }
    }
    pub fn bin_op(op: BinaryOp, left: Self, right: Self) -> Self {
        Self::BinOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Lit(x) => write!(f, "{x}"),
            Self::Variable(s) => f.write_str(s),
            Self::Call { func, arg, args } => {
                func.fmt(f)?;
                f.write_str("(")?;
                arg.fmt(f)?;
                for arg in args.iter() {
                    f.write_str(" ,")?;
                    arg.fmt(f)?;
                }
                f.write_str(")")
            }
            Self::UnOp { op, arg } => match op {
                UnaryOp::Negate => write!(f, "-{arg}"),
                UnaryOp::Plus => write!(f, "+{arg}"),
            },
            Self::BinOp { op, left, right } => {
                // if *op == Op::Multiply {
                //     return write!(f, "{left} {right}");
                // }
                // let left_bp = match left.as_ref() {
                //     Self::BinOp { op, .. } => op.binding_power(),
                //     _ => u8::MAX,
                // };
                // let right_bp = match right.as_ref() {
                //     Self::BinOp { op, .. } => op.binding_power(),
                //     _ => u8::MAX,
                // };
                // let bp = op.binding_power();
                let op = match op {
                    BinaryOp::Add => '+',
                    BinaryOp::Subtract => '-',
                    // Op::Multiply => unreachable!(),
                    BinaryOp::DotProduct => '*',
                    BinaryOp::Divide => '/',
                };
                // match (left_bp > bp, right_bp > bp) {
                match (
                    matches!(left.as_ref(), Self::Lit(..) | Self::Variable(..)),
                    matches!(right.as_ref(), Self::Lit(..) | Self::Variable(..)),
                ) {
                    (true, true) => write!(f, "{left} {op} {right}"),
                    (true, false) => write!(f, "{left} {op} ({right})"),
                    (false, true) => write!(f, "({left}) {op} {right}"),
                    (false, false) => write!(f, "({left}) {op} ({right})"),
                }
            }
        }
    }
}
