use std::collections::{HashMap, HashSet};

use crate::{
    builtins::BuiltinFunction,
    parse::{ArgList, BinaryOp, Expr, Ident, TopLevelItem, UnaryOp},
};

use anyhow::{Result, bail};
use ecow::EcoString;
use strum::IntoEnumIterator;

#[derive(Debug, Clone)]
enum Binding {
    Value(f64),
    Function { args: ArgList<Ident>, body: Expr },
    Builtin(BuiltinFunction),
}

#[derive(Debug)]
pub struct Interpreter {
    bindings: HashMap<Ident, Binding>,
}

#[derive(Default)]
pub struct Output {
    pub constants: Vec<(Option<Ident>, f64)>,
    pub single_var_functions: Vec<(Ident, Expr)>,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut bindings = HashMap::new();
        for builtin in BuiltinFunction::iter() {
            bindings.insert(
                EcoString::from(builtin.to_string()),
                Binding::Builtin(builtin),
            );
        }
        Self { bindings }
    }

    pub fn run(&mut self, items: Vec<TopLevelItem>) -> Result<Output> {
        let mut output = Output::default();
        for item in items {
            self.add_item(item, &mut output)?;
        }
        Ok(output)
    }

    pub fn add_item(&mut self, item: TopLevelItem, output: &mut Output) -> Result<()> {
        // eprintln!("running {item}");
        match item {
            TopLevelItem::Expression(expr) => {
                if expr.contains_var("x") {
                    output
                        .single_var_functions
                        .push((EcoString::inline("x"), expr));
                } else {
                    let value = self.evaluate(&expr, &HashMap::new())?;
                    output.constants.push((None, value));
                }
            }
            TopLevelItem::Assignment { name, body } => {
                if self.bindings.contains_key(&name) {
                    bail!("Cannot define variable '{name}' as this name is already bound");
                }
                let value = self.evaluate(&body, &HashMap::new())?;
                output.constants.push((Some(name.clone()), value));
                self.bindings.insert(name, Binding::Value(value));
            }
            TopLevelItem::FunctionDef { name, args, body } => {
                if self.bindings.contains_key(&name) {
                    bail!("Cannot define function '{name}' as this name is already bound");
                }
                for arg in args.iter() {
                    if self.bindings.contains_key(arg) {
                        bail!("Cannot use argument '{arg}' as this name is already bound")
                    }
                }
                if body.contains_func(name.as_str()) {
                    bail!("Function '{name}' cannot recursively call itself");
                }
                self.bindings.insert(name, Binding::Function { args, body });
            }
        }
        Ok(())
    }

    pub fn evaluate(&self, expr: &Expr, arg_map: &HashMap<Ident, f64>) -> Result<f64> {
        Ok(match expr {
            Expr::Lit(x) => *x,
            Expr::Variable(name) => match arg_map.get(name) {
                Some(x) => *x,
                None => match self.bindings.get(name) {
                    Some(Binding::Value(x)) => *x,
                    Some(Binding::Function { .. } | Binding::Builtin(_)) => {
                        bail!("'{name}' is a function and not a variable")
                    }
                    None => bail!("Binding '{name}' not defined"),
                },
            },
            Expr::Call { func, args } => match self.bindings.get(func) {
                Some(Binding::Builtin(builtin)) => {
                    let head = args.first().unwrap();
                    let mut evaluated_args = ArgList::from_head(self.evaluate(head, arg_map)?);
                    for arg in &args[1..] {
                        evaluated_args.push(self.evaluate(arg, arg_map)?);
                    }
                    builtin.call(evaluated_args)?
                }

                Some(Binding::Function {
                    args: arg_names,
                    body,
                }) => {
                    // eprintln!("evaluating {func}");
                    if arg_names.len() != args.len() {
                        bail!(
                            "Cannot pass {} arguments to a function taking {} arguments",
                            args.len(),
                            arg_names.len(),
                        );
                    }

                    let mut new_arg_map = HashMap::new();
                    for (arg_name, arg) in std::iter::zip(arg_names.iter(), args.iter()) {
                        let evaluated = self.evaluate(arg, arg_map)?;
                        new_arg_map.insert(arg_name.clone(), evaluated);
                    }
                    self.evaluate(body, &new_arg_map)?
                }
                Some(Binding::Value(_)) => {
                    bail!("Cannot call '{func}' as it is not a function")
                }
                None => bail!("Function '{func}' not defined"),
            },
            Expr::UnOp { op, arg } => op.evaluate(self.evaluate(arg, arg_map)?),
            Expr::BinOp { op, left, right } => op.evaluate(
                self.evaluate(left, arg_map)?,
                self.evaluate(right, arg_map)?,
            ),
        })
    }
}

impl Expr {
    fn contains_var(&self, needle: &str) -> bool {
        match self {
            Self::Lit(_) => false,
            Self::Variable(name) => name == needle,
            Self::UnOp { arg, .. } => arg.contains_var(needle),
            Self::BinOp { left, right, .. } => {
                left.contains_var(needle) || right.contains_var(needle)
            }
            Self::Call { args, .. } => args.iter().any(|arg| arg.contains_var(needle)),
        }
    }

    fn contains_func(&self, needle: &str) -> bool {
        match self {
            Self::Lit(_) => false,
            Self::Variable(_) => false,
            Self::UnOp { arg, .. } => arg.contains_func(needle),
            Self::BinOp { left, right, .. } => {
                left.contains_func(needle) || right.contains_func(needle)
            }
            Self::Call { func, args } => {
                func == needle || args.iter().any(|arg| arg.contains_var(needle))
            }
        }
    }
}

enum SingleVarFunction {
    Lit(f64),
    Var,
}

impl UnaryOp {
    pub fn evaluate(&self, arg: f64) -> f64 {
        match self {
            Self::Negate => -arg,
            Self::Plus => arg,
        }
    }
}

impl BinaryOp {
    pub fn evaluate(&self, left: f64, right: f64) -> f64 {
        match self {
            Self::Add => left + right,
            Self::Subtract => left - right,
            Self::DotProduct => left * right,
            Self::Divide => left / right,
            Self::Power => left.powf(right),
        }
    }
}
