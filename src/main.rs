#![allow(unused)]
mod lex;
mod parse;
mod run;
use std::{path::Path, sync::mpsc, time::Duration};

use notify::{Event, RecursiveMode, Watcher, recommended_watcher};
use notify_debouncer_full::{DebounceEventResult, new_debouncer, notify};

use crate::{
    parse::{BinaryOp, Expr, parse},
    run::Interpreter,
};

fn main() {
    use io::Write;
    use std::io;

    let (send, recv) = mpsc::channel();
    let mut debouncer = new_debouncer(Duration::from_millis(100), None, send).unwrap();
    debouncer
        .watch("./input.txt", RecursiveMode::NonRecursive)
        .unwrap();

    // repl
    loop {
        let mut input = std::fs::read_to_string(Path::new("./input.txt")).unwrap();
        // let mut input = String::new();
        // if let Err(error) = std::io::stdin().read_line(&mut input) {
        //     println!("error: {error}")
        // }

        println!("--------------------------------");
        match parse(input.as_str()) {
            Ok(items) => match Interpreter::new().run(items) {
                Ok(results) => {
                    for result in results {
                        println!("{result}");
                    }
                }
                Err(message) => {
                    println!("-- {message} --");
                }
            },
            Err(message) => {
                println!("-- {message} --");
            }
        }
        println!();
        std::io::stdout().flush().unwrap();

        recv.recv();
    }
}
