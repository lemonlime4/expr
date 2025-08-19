use std::collections::HashMap;

use strum_macros::EnumIter;

use crate::parse::{ArgList, Ident};

#[derive(Debug, Clone, Copy, EnumIter)]
pub enum BuiltinFunction {
    Sqrt,
    Abs,
    Sin,
    Cos,
    Tan,
    Atan,
}

impl std::fmt::Display for BuiltinFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(match self {
            Self::Sqrt => "sqrt",
            Self::Abs => "abs",
            Self::Sin => "sin",
            Self::Cos => "cos",
            Self::Tan => "tan",
            Self::Atan => "atan",
        })
    }
}
