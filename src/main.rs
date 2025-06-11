#[allow(unused_imports)]
use std::io::{self, Write};
use std::process;

fn main() {
    loop {
        // Uncomment this block to pass the first stage
        print!("$ ");
        io::stdout().flush().unwrap();

        // Wait for user input
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        if input.trim() == "exit 0" {
            process::exit(0);
        }

        println!("{}: command not found", input.trim());
    }
}
