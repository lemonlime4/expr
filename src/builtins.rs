use anyhow::{Result, bail};
use strum_macros::{EnumIter, IntoStaticStr};

use crate::parse::ArgList;

#[derive(Debug, Clone, Copy, EnumIter, IntoStaticStr)]
pub enum BuiltinFunction {
    Sqrt,
    Abs,
    Log,
    Ln,
    Exp,
    Sin,
    Cos,
    Tan,
    Atan,
    Min,
    Max,
    Mod,
}

impl BuiltinFunction {
    pub fn call(&self, args: ArgList<f64>) -> Result<f64> {
        macro_rules! call_unary {
            ($func:expr) => {
                match args.as_slice() {
                    [x] => $func(*x),
                    _ => bail!("{self} takes 1 argument but got {}", args.len()),
                }
            };
        }
        macro_rules! call_binary {
            ($func:expr) => {
                match args.as_slice() {
                    [x, y] => $func(*x, *y),
                    _ => bail!("{self} takes 2 arguments but got {}", args.len()),
                }
            };
        }
        Ok(match self {
            Self::Sqrt => call_unary!(f64::sqrt),
            Self::Abs => call_unary!(f64::abs),
            Self::Log => call_unary!(f64::log10),
            Self::Ln => call_unary!(f64::ln),
            Self::Exp => call_unary!(f64::exp),
            Self::Sin => call_unary!(f64::sin),
            Self::Cos => call_unary!(f64::cos),
            Self::Tan => call_unary!(f64::tan),
            Self::Atan => match args.as_slice() {
                [x] => x.atan(),
                [y, x] => y.atan2(*x),
                _ => bail!("atan takes 1 or 2 arguments but got {}", args.len()),
            },
            Self::Min => call_binary!(f64::min),
            Self::Max => call_binary!(f64::max),
            Self::Mod => call_binary!(|x: f64, y: f64| x - y * (x / y).floor()),
        })
    }
}

impl std::fmt::Display for BuiltinFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s: &'static str = self.into();
        f.write_str(s.to_ascii_lowercase().as_str())
    }
}
