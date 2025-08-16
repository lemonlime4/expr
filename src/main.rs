#![allow(unused)]
mod lex;
mod parse;
mod run;
use std::{io::Write, path::Path, sync::mpsc, time::Duration};

use notify::{Event, RecursiveMode, Watcher, recommended_watcher};
use notify_debouncer_full::{DebounceEventResult, new_debouncer, notify};

use crate::{
    parse::{ArgList, BinaryOp, Expr, parse},
    run::Interpreter,
};

fn main() {
    let (send, recv) = mpsc::channel();
    let mut debouncer = new_debouncer(Duration::from_millis(100), None, send).unwrap();
    if let Err(err) = debouncer.watch("./input.txt", RecursiveMode::NonRecursive) {
        println!("`input.txt` was not found in current directory");
        println!("{err:?}");
        return;
    }

    // use parse::example_fn;
    // eprintln!("{}", example_fn());

    // repl
    loop {
        println!("--------------------------------");

        match std::fs::read_to_string(Path::new("./input.txt")) {
            Ok(input) => {
                run_and_print(input.as_str());
            }
            Err(err) => {
                println!("Couldn't read from input.txt");
                println!("{err:?}")
            }
        };
        std::io::stdout().flush().unwrap();

        recv.recv();
    }
}

fn run_and_print(input: &str) {
    match parse(input) {
        Ok(items) => match Interpreter::new().run(items) {
            Ok(results) => {
                for result in results {
                    println!("{result}");
                }
            }
            Err(message) => {
                println!("Error -- {message}");
            }
        },
        Err(message) => {
            println!("Parse error -- {message}");
        }
    }
    println!();
}
