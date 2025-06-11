#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        // Uncomment this block to pass the first stage
        print!("$ ");
        io::stdout().flush().unwrap();

        // Wait for user input
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        let (cmd, args) = match input.split_once(' ') {
            Some((cmd, args)) => (cmd, args),
            None => (input.as_str(), ""),
        };

        match cmd {
            "exit" => {
                if args == "" {
                    break;
                }
                std::process::exit(args.parse().unwrap());
            }
            "echo" => {
                println!("{}", args)
            }
            _ => {
                println!("{}: command not found", input.trim());
            }
        }
    }
}
