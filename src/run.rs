use std::{collections::HashMap, hash::Hash};

use crate::{
    builtins::BuiltinFunction,
    parse::{ArgList, BinaryOp, Expr, Ident, TopLevelItem, UnaryOp},
};

use ecow::EcoString;
use strum::IntoEnumIterator;

#[derive(Clone)]
enum Binding {
    Value(f64),
    Function { args: ArgList<Ident>, body: Expr },
    Builtin(BuiltinFunction),
}

pub struct Interpreter {
    bindings: HashMap<Ident, Binding>,
    arg_bindings: HashMap<Ident, f64>,
    results: Vec<f64>,
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
        Self {
            bindings,
            arg_bindings: HashMap::new(),
            results: Vec::new(),
        }
    }

    pub fn run(mut self, items: Vec<TopLevelItem>) -> Result<Vec<f64>, String> {
        for item in items {
            self.add_item(item)?;
        }
        Ok(self.results)
    }

    fn add_item(&mut self, item: TopLevelItem) -> Result<(), String> {
        eprintln!("running {item}");
        match item {
            TopLevelItem::Expression(expr) => {
                let value = self.evaluate(&expr, &HashMap::new())?;
                self.results.push(value);
            }
            TopLevelItem::Assignment { name, body } => {
                //
                if self.bindings.contains_key(&name) {
                    Err(format!(
                        "Cannot define variable '{name}' as this name is already bound"
                    ))?;
                }
                let value = self.evaluate(&body, &HashMap::new())?;
                self.bindings.insert(name, Binding::Value(value));
            }
            TopLevelItem::FunctionDef { name, args, body } => {
                if self.bindings.contains_key(&name) {
                    Err(format!(
                        "Cannot define function '{name}' as this name is already bound"
                    ))?;
                }
                for arg in args.iter() {
                    if self.bindings.contains_key(arg) {
                        Err(format!(
                            "Cannot use argument '{arg}' as this name is already bound"
                        ))?
                    }
                }
                // let body = self.pre_evaluate(&args, body);
                self.bindings.insert(name, Binding::Function { args, body });
            }
        }
        Ok(())
    }

    fn evaluate(&self, expr: &Expr, arg_map: &HashMap<Ident, f64>) -> Result<f64, String> {
        Ok(match expr {
            Expr::Lit(x) => *x,
            Expr::Variable(name) => match arg_map.get(name) {
                Some(x) => *x,
                None => match self.bindings.get(name) {
                    Some(Binding::Value(x)) => *x,
                    Some(Binding::Function { .. } | Binding::Builtin(_)) => {
                        Err(format!("'{name}' is a function and not a variable"))?
                    }
                    None => Err(format!("Binding '{name}' not defined"))?,
                },
            },
            Expr::Call { func, args } => match self.bindings.get(func) {
                Some(Binding::Builtin(builtin)) => Err("Builtin functions not yet implemented")?,

                Some(Binding::Function {
                    args: arg_names,
                    body,
                }) => {
                    // eprintln!("evaluating {func}");
                    if arg_names.len() != args.len() {
                        Err(format!(
                            "Cannot pass {} arguments to a function taking {} arguments",
                            args.len(),
                            arg_names.len(),
                        ))?;
                    }

                    let mut new_arg_map = HashMap::new();
                    for (arg_name, arg) in std::iter::zip(arg_names.iter(), args.iter()) {
                        let evaluated = self.evaluate(arg, arg_map)?;
                        new_arg_map.insert(arg_name.clone(), evaluated);
                    }
                    self.evaluate(body, &new_arg_map)?
                }
                Some(Binding::Value(x)) => {
                    Err(format!("Cannot call '{func}' as it is not a function"))?
                }
                None => Err(format!("Function '{func}' not defined"))?,
            },
            Expr::UnOp { op, arg } => {
                let arg = self.evaluate(arg, arg_map)?;
                match op {
                    UnaryOp::Negate => -arg,
                    UnaryOp::Plus => arg,
                }
            }
            Expr::BinOp { op, left, right } => {
                let left = self.evaluate(left, arg_map)?;
                let right = self.evaluate(right, arg_map)?;
                match op {
                    BinaryOp::Add => left + right,
                    BinaryOp::Subtract => left - right,
                    BinaryOp::DotProduct => left * right,
                    BinaryOp::Divide => left / right,
                }
            }
        })
    }
}
