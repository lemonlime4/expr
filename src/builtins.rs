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
    Asin,
    Acos,
    Atan,
    Min,
    Max,
    Mod,
    Sgn,
}

impl BuiltinFunction {
    pub fn call(&self, args: ArgList<f64>) -> Result<f64> {
        macro_rules! unary {
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
            Self::Sqrt => unary!(f64::sqrt),
            Self::Abs => unary!(f64::abs),
            Self::Log => unary!(f64::log10),
            Self::Ln => unary!(f64::ln),
            Self::Exp => unary!(f64::exp),
            Self::Sin => unary!(f64::sin),
            Self::Cos => unary!(f64::cos),
            Self::Asin => unary!(f64::asin),
            Self::Acos => unary!(f64::acos),
            Self::Tan => unary!(f64::tan),
            Self::Atan => match args.as_slice() {
                [x] => x.atan(),
                [y, x] => y.atan2(*x),
                _ => bail!("atan takes 1 or 2 arguments but got {}", args.len()),
            },
            Self::Min => {
                if args.is_empty() {
                    bail!("min cannot take no arguments")
                }
                args.iter().fold(f64::INFINITY, |x, &y| {
                    if x.is_nan() || y.is_nan() {
                        f64::NAN
                    } else {
                        x.min(y)
                    }
                })
            }
            Self::Max => {
                if args.is_empty() {
                    bail!("min cannot take no arguments")
                }
                args.iter().fold(-f64::INFINITY, |x, &y| {
                    if x.is_nan() || y.is_nan() {
                        f64::NAN
                    } else {
                        x.max(y)
                    }
                })
            }
            Self::Mod => call_binary!(|x, y| x - y * f64::floor(x / y)),
            Self::Sgn => unary!(|x| {
                use std::cmp::Ordering::*;
                match f64::partial_cmp(&x, &0.0) {
                    Some(Less) => -1.0,
                    Some(Equal) => x,
                    Some(Greater) => 1.0,
                    None => x,
                }
            }),
        })
    }
}

impl std::fmt::Display for BuiltinFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s: &'static str = self.into();
        f.write_str(s.to_ascii_lowercase().as_str())
    }
}
