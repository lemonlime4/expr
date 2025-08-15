use std::{collections::HashMap, hash::Hash};

use crate::parse::{BinaryOp, Expr, Ident, TopLevelItem, UnaryOp};

// fn resolve_dependencies(expr: &Expr) -> Vec<Ident> {

// }

pub struct Interpreter {
    bindings: HashMap<Ident, f64>,
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
        match item {
            TopLevelItem::Expression(expr) => {
                self.results.push(self.evaluate(&expr)?);
            }
            TopLevelItem::Assignment { name, body } => {
                //
                if self.bindings.contains_key(&name) {
                    Err(format!("Cannot bind {name} twice"))?;
                }
                self.bindings.insert(name, self.evaluate(&body)?);
            }
        }
        Ok(())
    }

    fn evaluate(&self, expr: &Expr) -> Result<f64, String> {
        Ok(match expr {
            Expr::Lit(x) => *x,
            Expr::Variable(name) => match self.bindings.get(name) {
                Some(x) => *x,
                None => Err(format!("No binding {name} found"))?,
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
