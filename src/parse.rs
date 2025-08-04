use crate::lex::*;

pub enum Expr {
    BinOp(Operator, Box<Expr>, Box<Expr>),
    Lit(f64),
}

impl Expr {
    fn eval(&self) -> f64 {
        use Expr::*;
        match self {
            Lit(x) => *x,
            BinOp(op, left, right) => {
                let left = left.eval();
                let right = right.eval();
                use Operator::*;
                match op {
                    Plus => left + right,
                    Minus => left - right,
                    Dot => left * right,
                    Slash => left / right,
                    Equals => (left == right) as u8 as f64,
                }
            }
        }
    }
}

// pub fn parse_tokens(tokens: Vec<Token>) -> Expr {}
