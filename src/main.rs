use std::env;
#[allow(unused_imports)]
use std::io::{self, Write};

fn cmd_echo(args: &Vec<String>) {
    println!("{}", args.join(" "));
}

fn cmd_type(args: &Vec<String>, builtin: &[&str]) {
    if args.len() == 0 {
        return;
    }

    for arg in args {
        if builtin.contains(&arg.as_str()) {
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

fn cmd_cd(args: &Vec<String>) {
    let args_len = args.len();

    if args_len == 0 {
        return;
    }
    if args_len > 1 {
        println!("cd: too many arguments");
        return;
    }
    let path = match env::home_dir() {
        Some(home_path) => {
            let home_str = home_path.to_str().unwrap_or("");
            args[0].replace("~", home_str) // Returns a new String
        }
        None => {
            println!("Could not determine the home directory.");
            args[0].to_string() // Still return something
        }
    };
    let new_dir = std::path::Path::new(&path);
    match env::set_current_dir(new_dir) {
        Ok(_) => {}
        Err(_) => {
            println!("{}: No such file or directory", path)
        }
    };
}

fn command_handler(cmd: &str, args: &Vec<String>, builtin: &[&str]) {
    match cmd {
        "exit" => {
            std::process::exit(0);
        }
        "echo" => {
            cmd_echo(args);
        }
        "type" => {
            cmd_type(args, &builtin);
        }
        "pwd" => {
            cmd_pwd();
        }
        "cd" => {
            cmd_cd(args);
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

fn input() -> String {
    print!("$ ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    loop {
        let mut input_end = false;

        for char in input.chars() {
            if char == '\'' {
                input_end = !input_end;
            }
        }
        if !input_end {
            break;
        }
        print!("> ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();
    }
    input
}

fn parse_input(input: String) -> (String, Vec<String>) {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    for char in input.chars() {
        match char {
            '\'' => {
                in_quotes = !in_quotes;
            }
            ' ' => {
                if !in_quotes {
                    if !current.is_empty() {
                        result.push(current.clone());
                        current.clear();
                    }
                } else {
                    current.push(char);
                }
            }
            _ => current.push(char),
        }
    }

    if !current.is_empty() {
        result.push(current);
    }

    let cmd = result.remove(0);

    let args = result;

    return (cmd, args);
}

fn main() {
    loop {
        let builtin = ["echo", "exit", "type", "pwd", "cd"];

        let input = input().trim().to_string();

        if input.is_empty() {
            continue;
        }

        let (cmd, args) = parse_input(input);

        command_handler(&cmd, &args, &builtin);
    }
}
