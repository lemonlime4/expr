#![allow(unused)]
mod lex;
mod parse;
use crate::parse::{BinaryOp, Expr, parse};

fn main() {
    use io::Write;
    use std::io;
    println!(
        "{}",
        Expr::bin_op(
            BinaryOp::DotProduct,
            Expr::Lit(1.0),
            Expr::bin_op(BinaryOp::Add, Expr::Lit(1.0), Expr::Lit(1.0))
        )
    );
    loop {
        let mut input = String::new();
        if let Err(error) = std::io::stdin().read_line(&mut input) {
            println!("error: {error}")
        }
        input = input.trim().to_string();

        match parse(&input) {
            Ok(expr) => {
                println!("{expr}");
                // println!("{expr}");
            }
            Err(message) => {
                eprintln!("-- {message} --");
            }
        }
        println!("\n");
        std::io::stdout().flush().unwrap();
        std::io::stderr().flush().unwrap();
    }
}
