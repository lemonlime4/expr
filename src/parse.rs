// use std::rc::Rc;

use std::fmt::{self, Binary};

use ecow::EcoVec;

use crate::lex::{Token, lex};

type Ident = ecow::EcoString;
// type Ident = Rc<str>;

#[derive(Debug, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    // Multiply,
    DotProduct,
    // CrossProduct,
    Divide,
}

impl BinaryOp {
    fn binding_power(&self) -> u16 {
        match self {
            Self::Add => 1,
            Self::Subtract => 1,
            // Self::Multiply => 2,
            Self::DotProduct => 2,
            Self::Divide => 2,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
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

pub fn parse_tokens(tokens: &[Token]) -> Result<Expr, String> {
    Err("")?
}

pub fn parse(input: &str) -> Result<Expr, String> {
    let tokens = lex(input)?;
    parse_tokens(&tokens)
}

// impl fmt::Display for Expr {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match self {
//             Self::Lit(x) => write!(f, "{x}"),
//             Self::Variable(s) => f.write_str(s),
//             Self::Call { func, arg, args } => {
//                 func.fmt(f)?;
//                 f.write_str("(")?;
//                 arg.fmt(f)?;
//                 for arg in args.iter() {
//                     f.write_str(" ,")?;
//                     arg.fmt(f)?;
//                 }
//                 f.write_str(")")
//             }
//             Self::UnOp { op, arg } => match op {
//                 UnaryOp::Negate => write!(f, "-{arg}"),
//             },
//             Self::BinOp { op, left, right } => {
//                 // if *op == Op::Multiply {
//                 //     write!(f, "{left} {right}")
//                 // } else {
//                 let precedence =
//                 let op = match op {
//                     BinaryOp::Add => '+',
//                     BinaryOp::Subtract => '-',
//                     // Op::Multiply => unreachable!(),
//                     BinaryOp::DotProduct => '*',
//                     BinaryOp::Divide => '/',
//                 };
//                 write!(f, "{left} {op} {right}")
//                 // }
//             }
//         }
//     }
// }
