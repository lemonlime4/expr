use std::{
    cmp::{self, Ordering},
    fmt::{self, Binary},
    hint::unreachable_unchecked,
    slice,
};

use ecow::EcoString;

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
        args: ArgList<Box<Expr>>,
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
pub struct ArgList<T> {
    pub head: T,
    pub tail: Vec<T>,
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
        println!("parsing expr {:?}", self.tokens);
        let signs = self.consume_while(|t| match t {
            Token::Minus => Some(true),
            Token::Plus => Some(false),
            _ => None,
        });
        let mut left = match self.next() {
            Some(Token::LeftParen) => {
                let inner = self.parse_expr(None)?;
                if self.next() != Some(Token::RightParen) {
                    Err("Expected )")?;
                }
                inner
            }
            Some(Token::NumLit(s)) => Expr::Lit(s.parse().expect("Failed to parse float literal")),
            Some(Token::Ident(name)) => match self.peek() {
                Some(Token::LeftParen) => {
                    // TODO parse comma separated list
                    self.next();
                    let arg = self.parse_expr(None)?;
                    if self.next() != Some(Token::RightParen) {
                        Err("Expected )")?;
                    }
                    Expr::Call {
                        func: name,
                        args: ArgList {
                            head: Box::new(arg),
                            tail: Vec::new(),
                        },
                    }
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

    pub fn read_assignment_head(&mut self) -> Option<(Ident, Option<ArgList<Ident>>)> {
        match self.tokens.as_slice() {
            [.., Token::Assign, Token::Ident(_)] => {
                let name = match self.next().unwrap() {
                    Token::Ident(name) => name,
                    _ => unreachable!(),
                };
                self.next();
                Some((name, None))
            }
            [
                ..,
                Token::Assign,
                Token::RightParen,
                Token::Ident(_),
                Token::LeftParen,
                Token::Ident(_),
            ] => {
                let name = match self.next().unwrap() {
                    Token::Ident(name) => name,
                    _ => unreachable!(),
                };
                self.next();
                let arg = match self.next().unwrap() {
                    Token::Ident(name) => name,
                    _ => unreachable!(),
                };
                self.next();
                self.next();

                Some((
                    name,
                    Some(ArgList {
                        head: arg,
                        tail: Vec::new(),
                    }),
                ))
            }
            _ => None,
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

            let assigned_name = self.read_assignment_head();
            println!("-- {:?}", self.tokens);

            let body = self.parse_expr(None)?;
            items.push(match assigned_name {
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
            Self::FunctionDef { name, args, body } => write!(f, "{name}{args} := {body}"),
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
            Self::Call { func, args } => write!(f, "{func}{args}"),
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

impl<T> ArgList<T> {
    pub fn iter(&self) -> ArgListIter<'_, T> {
        ArgListIter {
            head: Some(&self.head),
            tail: self.tail.iter(),
        }
    }

    pub fn len(&self) -> usize {
        1 + self.tail.len()
    }
}
pub struct ArgListIter<'a, T> {
    head: Option<&'a T>,
    tail: slice::Iter<'a, T>,
}

impl<'a, T> std::iter::Iterator for ArgListIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(head) = self.head {
            self.head = None;
            Some(head)
        } else {
            self.tail.next()
        }
    }
    // TODO find out if i have to implement more methods
}

impl<T: fmt::Display> fmt::Display for ArgList<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}", self.head)?;
        for arg in self.tail.iter() {
            write!(f, ", {arg}")?;
        }
        write!(f, ")")
    }
}

pub fn example_fn() -> TopLevelItem {
    TopLevelItem::FunctionDef {
        name: "f".into(),
        args: ArgList {
            head: "x".into(),
            tail: vec!["y".into(), "z".into()],
        },
        body: Parser::new(lex("x * y + z / x / y").unwrap())
            .parse_expr(None)
            .unwrap(),
    }
}
