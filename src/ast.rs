use crate::lex::Ident;
use std::fmt;

// #[derive(Clone, PartialEq, Eq, Debug)]
// pub enum Item {
//     Binding { name: Ident, def: Expr },
//     Value(Expr),
// }

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Expr {
    Var(Ident),
    Abs { bound: Ident, body: Box<Self> },
    App { func: Box<Self>, arg: Box<Self> },
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Var(name) => write!(f, "{name}"),
            Self::Abs { bound, body } => {
                write!(f, "\\{bound}.{body}")
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
