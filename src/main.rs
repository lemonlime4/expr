mod lex;
mod parse;
use crate::parse::parse;

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
                io::stdout().flush().unwrap();
            }
            Err(message) => {
                eprintln!("-- {message} --");
                io::stderr().flush().unwrap();
            }
        }
    }
}
