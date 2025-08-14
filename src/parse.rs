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

    pub fn parse(&mut self, last_op: Option<BinaryOp>) -> Result<Expr, String> {
        let mut left = match self.next() {
            Some(Token::NumLit(s)) => Expr::Lit(s.parse().expect("Failed to parse float literal")),
            Some(Token::Ident(name)) => Expr::Variable(name),
            _ => Err("cannot parse empty expression")?,
        };
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
    pub fn bin_op(op: BinaryOp, left: Expr, right: Expr) -> Self {
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
