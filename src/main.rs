mod lex;
mod parse;
use crate::lex::*;

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
            Ok(tokens) => {
                for token in tokens {
                    print!("{token} ");
                }
                println!("\n");
                std::io::stdout().flush().unwrap();
            }
            Err(message) => {
                eprintln!("-- {message} --");
                std::io::stderr().flush().unwrap();
            }
        }
    }
}
