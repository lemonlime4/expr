mod lex;
mod parse;
mod types;
use crate::{parse::parse, types::type_check};

fn main() {
    use std::io;
    use std::io::Write;
    loop {
        let mut input = String::new();
        if let Err(error) = io::stdin().read_line(&mut input) {
            println!("error: {error}")
        }
        input = input.trim().to_string();

        match parse(&input) {
            Ok(expr) => {
                println!("{expr:#?}");
                println!("{expr}   -- parsed");

                match type_check(expr) {
                    Ok(typed) => println!("{typed}   -- typed"),
                    Err(message) => eprintln!("-- {message} --"),
                };
            }
            Err(message) => {
                eprintln!("-- {message} --");
            }
        }
        println!("");
        io::stdout().flush().unwrap();
        io::stderr().flush().unwrap();
    }
}
