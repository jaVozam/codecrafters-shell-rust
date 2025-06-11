use std::env;
#[allow(unused_imports)]
use std::io::{self, Write};

fn cmd_echo(args: &[&str]) {
    println!("{}", args.join(" "));
}

fn cmd_type(args: &[&str], builtin: &[&str]) {
    if args.len() == 0 {
        return;
    }

    for arg in args {
        if builtin.contains(&arg) {
            println!("{} is a shell builtin", arg);
            continue;
        }

        if let Ok(path_var) = env::var("PATH") {
            let path_entries = env::split_paths(&path_var);

            for dir in path_entries {
                let full_path = dir.join(&arg);
                if full_path.exists() {
                    println!("{} is {}", arg, full_path.display());
                    return;
                }
            }
        } else {
            eprintln!("Failed to get PATH variable");
        }

        println!("{}: not found", arg);
    }
}

fn cmd_pwd() {
    println!("{}", env::current_dir().unwrap().display());
}

fn main() {
    loop {
        // Uncomment this block to pass the first stage
        print!("$ ");
        io::stdout().flush().unwrap();

        // Wait for user input
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        let builtin = ["echo", "exit", "type", "pwd"];

        let (cmd, args) = match input.trim().split_once(' ') {
            Some((cmd, args)) => (cmd, args.split(' ').collect()),
            None => (input.trim(), Vec::new()),
        };

        match cmd {
            "exit" => {
                std::process::exit(0);
            }
            "echo" => {
                cmd_echo(&args);
            }
            "type" => {
                cmd_type(&args, &builtin);
            }
            "pwd" => {
                cmd_pwd();
            }
            _ => {
                let mut command = std::process::Command::new(cmd);
                command.args(args);

                match command.status() {
                    Ok(_) => {}
                    Err(_) => {
                        println!("{}: command not found", cmd);
                    }
                }
            }
        }
    }
}
