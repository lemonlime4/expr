use crate::parse::ExprVarDisplay;
use crate::{lex::Ident, parse::Expr};
use std::collections::HashMap;
use std::fmt;
use std::ops::RangeFrom;

pub type TypeId = u32;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TypedVar {
    type_id: TypeId,
    name: Ident,
}

pub type TypedExpr = Expr<TypedVar>;

pub fn type_check(expr: Expr<Ident>) -> Result<TypedExpr, String> {
    let mut next_type_id = 0..;
    let mut type_map = HashMap::new();
    assign_types(expr, &mut next_type_id, &mut type_map)
}

fn assign_types(
    expr: Expr<Ident>,
    next_type_id: &mut RangeFrom<TypeId>,
    type_map: &mut HashMap<Ident, TypeId>,
) -> Result<TypedExpr, String> {
    Ok(match expr {
        Expr::Var(name) => {
            if let Some(&type_id) = type_map.get(&name) {
                Expr::Var(TypedVar { type_id, name })
            } else {
                let type_id = next_type_id.next().unwrap();
                type_map.insert(name.clone(), type_id);
                Expr::Var(TypedVar { type_id, name })
            }
        }
        Expr::Abs { bound, body } => {
            let type_id = next_type_id.next().unwrap();
            let original_id = type_map.insert(bound.clone(), type_id);
            // println!("binding parameter {bound}");
            // println!("original: {original_id:?}");
            let body = assign_types(*body, next_type_id, type_map)?;
            // println!("unbound parameter {bound}");
            if let Some(original_id) = original_id {
                type_map.insert(bound.clone(), original_id);
            } else {
                type_map.remove(&bound);
            }

            Expr::Abs {
                bound: TypedVar {
                    type_id,
                    name: bound,
                },
                body: Box::new(body),
            }
        }
        Expr::App { func, arg } => Expr::App {
            func: Box::new(assign_types(*func, next_type_id, type_map)?),
            arg: Box::new(assign_types(*arg, next_type_id, type_map)?),
        },
    })
}

impl ExprVarDisplay for TypedVar {
    fn fmt_normal(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // f.write_str(&self.name)
        self.fmt_lambda_bound(f)
    }
    fn fmt_lambda_bound(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.name, self.type_id)
    }
}
