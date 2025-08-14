#![allow(unused)]
mod lex;
mod parse;
use crate::{
    lex::*,
    parse::{parse, parse_tokens},
};

fn main() {
    use io::Write;
    use std::io;
    loop {
        let mut input = String::new();
        if let Err(error) = std::io::stdin().read_line(&mut input) {
            println!("error: {error}")
        }
        input = input.trim().to_string();

        match lex(&input) {
            Ok(tokens) => match parse_tokens(&tokens) {
                Ok(expr) => {
                    println!("{expr:?}");
                    // println!("{expr}");
                }
                Err(message) => {
                    eprintln!("-- message -- parse error");
                }
            },
            Err(message) => {
                eprintln!("-- {message} -- lex error");
            }
        }
        println!("\n");
        std::io::stdout().flush().unwrap();
        std::io::stderr().flush().unwrap();
    }
}
