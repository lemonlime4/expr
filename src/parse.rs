use std::{
    cmp::{self, Ordering},
    fmt::{self, Binary},
    hint::unreachable_unchecked,
    slice,
};

use ecow::{EcoString, EcoVec, eco_vec};

use crate::lex::{Token, lex};

pub type Ident = ecow::EcoString;

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

#[derive(Debug, Clone)]
pub enum Expr {
    Lit(f64),
    Variable(Ident),
    Call {
        func: Ident,
        args: ArgList<Expr>,
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

pub enum TopLevelItem {
    Expression(Expr),
    Assignment {
        name: Ident,
        body: Expr,
    },
    FunctionDef {
        name: Ident,
        args: ArgList<Ident>,
        body: Expr,
    },
}

#[derive(Debug, Clone)]
pub struct ArgList<T>(Vec<T>);

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

    fn next_if(&mut self, f: impl Fn(&Token) -> bool) -> Option<Token> {
        if self.peek().is_some_and(f) {
            self.next()
        } else {
            None
        }
    }

    fn consume_while<T>(&mut self, mut f: impl FnMut(&Token) -> Option<T>) -> Vec<T> {
        let mut result = Vec::new();
        while let Some(processed) = self.peek().and_then(&mut f) {
            self.next();
            result.push(processed);
        }
        result
    }

    pub fn parse_expr(&mut self, last_op: Option<BinaryOp>) -> Result<Expr, String> {
        // eprintln!("parsing expr {:?}", self.tokens);
        let signs = self.consume_while(|t| match t {
            Token::Minus => Some(true),
            Token::Plus => Some(false),
            _ => None,
        });
        let mut left = match self.next() {
            Some(Token::LeftParen) => {
                let inner = self.parse_expr(None)?;
                match self.next() {
                    Some(Token::RightParen) => {}
                    Some(token) => Err(format!("{token} is not allowed here"))?,
                    None => Err("Unclosed parenthesis")?,
                }
                inner
            }
            Some(Token::NumLit(s)) => Expr::Lit(s.parse().expect("Failed to parse float literal")),
            Some(Token::Ident(name)) => match self.peek() {
                Some(Token::LeftParen) => {
                    self.next();
                    if self.peek() == Some(&Token::RightParen) {
                        Err(format!("Cannot call {name} with no arguments"))?;
                    }
                    let mut args = ArgList::from_head(self.parse_expr(None)?);
                    loop {
                        match self.next() {
                            Some(Token::RightParen) => break,
                            Some(Token::Comma) => {}
                            Some(token) => Err(format!("Expected comma but got {token}"))?,
                            None => Err("Unclosed parenthesis")?,
                        }
                        args.push(self.parse_expr(None)?);
                    }
                    Expr::Call { func: name, args }
                }
                _ => Expr::Variable(name),
            },
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
                _ => break Ok(left),
            };
            if Some(op.binding_power()) > last_op.map(|op| op.binding_power()) {
                self.next();
                let right = self.parse_expr(Some(op))?;
                left = Expr::bin_op(op, left, right);
            } else {
                break Ok(left);
            }
        }
    }

    pub fn parse(&mut self) -> Result<Vec<TopLevelItem>, String> {
        let mut items = Vec::new();
        loop {
            if self.peek().is_none() {
                break;
            }

            // parse top level item
            // println!("parsing item {:?}", self.tokens);

            let assignment = if let Some(pos) = self
                .tokens
                .iter()
                .rev()
                .take_while(|t| **t != Token::Newline)
                .position(|t| *t == Token::Assign)
            {
                let mut tokens = self.tokens.drain(self.tokens.len() - pos..).rev();
                // println!("{:?}", tokens.collect::<Vec<_>>());
                let name = match tokens.next() {
                    Some(Token::Ident(name)) => name,
                    Some(token) => Err(format!("Unknown token {token} in assignment"))?,
                    None => Err("Empty assignment")?,
                };
                let args = match tokens.next() {
                    Some(Token::LeftParen) => {
                        let mut args = ArgList::from_head(match tokens.next() {
                            Some(Token::Ident(name)) => name,
                            Some(token) => Err("Unexpected token {token} in argument list")?,
                            None => Err("Unclosed argument list")?,
                        });
                        loop {
                            match tokens.next() {
                                Some(Token::Comma) => {}
                                Some(Token::RightParen) => break,
                                Some(token) => {
                                    Err(format!("Unknown token {token} in argument list"))?
                                }
                                None => Err("Unclosed argument list")?,
                            }
                            match tokens.next() {
                                Some(Token::Ident(name)) => args.push(name),
                                Some(token) => Err(format!("Expected parameter but got {token}"))?,
                                None => Err("Unclosed argument list")?,
                            }
                        }
                        Some(args)
                    }
                    Some(token) => Err(format!("Unknown token {token} in assignment"))?,
                    None => None,
                };
                drop(tokens);
                assert_eq!(Some(Token::Assign), self.tokens.pop()); // pop Token::Assign
                Some((name, args))
            } else {
                None
            };

            // eprintln!("parsed assignment {assignment:?}");
            // eprintln!("parsing body {:?}", self.tokens);
            let body = self.parse_expr(None)?;
            items.push(match assignment {
                Some((name, Some(args))) => TopLevelItem::FunctionDef { name, args, body },
                Some((name, None)) => TopLevelItem::Assignment { name, body },
                None => TopLevelItem::Expression(body),
            });

            match self.peek() {
                Some(Token::Newline) => {
                    while self.peek() == Some(&Token::Newline) {
                        self.next();
                    }
                }
                None => break,
                Some(t) => Err(format!("Expected newline but got {t}"))?,
            }
        }
        Ok(items)
    }
}

pub fn parse(input: &str) -> Result<Vec<TopLevelItem>, String> {
    let tokens = lex(input)?;
    Parser::new(tokens).parse()
}

impl fmt::Display for TopLevelItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Expression(expr) => write!(f, "{expr}"),
            Self::Assignment { name, body } => write!(f, "{name} := {body}"),
            Self::FunctionDef { name, args, body } => write!(f, "{name}({args}) := {body}"),
        }
    }
}
impl Expr {
    pub fn un_op(op: UnaryOp, arg: Self) -> Self {
        let arg = Box::new(arg);
        Self::UnOp { op, arg }
    }
    pub fn bin_op(op: BinaryOp, left: Self, right: Self) -> Self {
        let left = Box::new(left);
        let right = Box::new(right);
        Self::BinOp { op, left, right }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Lit(x) => write!(f, "{x}"),
            Self::Variable(s) => write!(f, "{s}"),
            Self::Call { func, args } => write!(f, "{func}({args})"),
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
                    matches!(
                        left.as_ref(),
                        Self::Lit(..) | Self::Variable(..) | Self::Call { .. }
                    ),
                    matches!(
                        right.as_ref(),
                        Self::Lit(..) | Self::Variable(..) | Self::Call { .. }
                    ),
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

impl<T> ArgList<T> {
    pub fn from_head(head: T) -> Self {
        Self(vec![head])
    }
    pub fn push(&mut self, value: T) {
        self.0.push(value)
    }
}

impl<T> std::ops::Deref for ArgList<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: fmt::Display> fmt::Display for ArgList<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut it = self.0.iter();
        write!(f, "{}", it.next().unwrap())?;
        for arg in it {
            write!(f, ", {arg}")?;
        }
        Ok(())
    }
}
