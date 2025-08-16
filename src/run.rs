use std::{collections::HashMap, hash::Hash};

use crate::parse::{ArgList, BinaryOp, Expr, Ident, TopLevelItem, UnaryOp};

#[derive(Clone)]
enum Binding {
    Value(f64),
    Function { args: ArgList<Ident>, body: Expr },
}

pub struct Interpreter {
    bindings: HashMap<Ident, Binding>,
    results: Vec<f64>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
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
        println!("running {item}");
        match item {
            TopLevelItem::Expression(expr) => {
                let value = self.evaluate(&expr)?;
                self.results.push(value);
            }
            TopLevelItem::Assignment { name, body } => {
                //
                if self.bindings.contains_key(&name) {
                    Err(format!(
                        "Cannot define variable '{name}' as this name is already bound"
                    ))?;
                }
                let value = self.evaluate(&body)?;
                self.bindings.insert(name, Binding::Value(value));
            }
            TopLevelItem::FunctionDef { name, args, body } => {
                //
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
                self.bindings.insert(name, Binding::Function { args, body });
            }
        }
        Ok(())
    }

    // TODO 1 remove the mut and change bindings to RefCell
    fn evaluate(&mut self, expr: &Expr) -> Result<f64, String> {
        Ok(match expr {
            Expr::Lit(x) => *x,
            Expr::Variable(name) => match self.bindings.get(name) {
                Some(Binding::Value(x)) => *x,
                Some(Binding::Function { .. }) => {
                    Err(format!("'{name}' is a function and not a variable"))?
                }
                None => Err(format!("Binding '{name}' not defined"))?,
            },
            Expr::Call { func, args } => match self.bindings.clone().get(func) {
                Some(Binding::Function {
                    args: arg_names,
                    body,
                }) => {
                    if arg_names.len() != args.len() {
                        Err(format!(
                            "Cannot pass {} arguments to a function taking {} arguments",
                            args.len(),
                            arg_names.len(),
                        ))?;
                    }

                    for (arg_name, arg) in std::iter::zip(arg_names.iter(), args.iter()) {
                        let evaluated = self.evaluate(arg)?;
                        // TODO 1 mutable access here
                        self.bindings
                            .insert(arg_name.clone(), Binding::Value(evaluated));
                    }
                    let result = self.evaluate(body)?;
                    for arg_name in arg_names.iter() {
                        self.bindings.remove(arg_name).unwrap();
                    }
                    result
                }
                Some(Binding::Value(x)) => {
                    Err(format!("Cannot call '{func}' as it is not a function"))?
                }
                None => Err(format!("Function '{func}' not defined"))?,
            },
            Expr::UnOp { op, arg } => {
                let arg = self.evaluate(arg)?;
                match op {
                    UnaryOp::Negate => -arg,
                    UnaryOp::Plus => arg,
                }
            }
            Expr::BinOp { op, left, right } => {
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;
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
