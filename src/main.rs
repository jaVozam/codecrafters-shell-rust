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

        let (cmd, args) = match input.trim().split_once(' ') {
            Some((cmd, args)) => (cmd, args.split(' ').collect()),
            None => (input.as_str(), Vec::new()),
        };

        match cmd {
            "exit" => {
                std::process::exit(0);
            }
            "echo" => {
                println!("{}", args.join(" "));
            }
            "type" => {
                let builtin = ["echo", "exit", "type"];

                if args.len() == 0 {
                    return;
                }

                for arg in args {
                    if builtin.contains(&arg) {
                        println!("{} is a shell builtin", arg);
                    } else {
                        println!("{}: not found", arg);
                    }
                }
            }
            _ => {
                println!("{}: command not found", input.trim());
            }
        }
    }
}
