#[allow(unused_imports)]
use std::io::{self, Write};

mod commands;

fn input() -> String {
    print!("$ ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    loop {
        let mut in_s_quotes = false;
        let mut in_d_quotes = false;
        for char in input.chars() {
            if char == '\'' && !in_d_quotes {
                in_s_quotes = !in_s_quotes;
            }
            if char == '"' && !in_s_quotes {
                in_d_quotes = !in_d_quotes
            }
        }
        if !in_s_quotes && !in_d_quotes {
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
    let mut in_s_quotes = false;
    let mut in_d_quotes = false;
    let mut escape = false;
    for char in input.chars() {
        match char {
            '\'' => {
                if !in_d_quotes && !escape {
                    in_s_quotes = !in_s_quotes;
                } else {
                    current.push(char);
                    escape = false;
                }
            }
            '"' => {
                if !in_s_quotes && !escape {
                    in_d_quotes = !in_d_quotes;
                } else {
                    current.push(char);
                    escape = false;
                }
            }
            '\\' => {
                if in_s_quotes && in_d_quotes {
                    current.push(char);
                } else {
                    escape = true;
                }
            }
            ' ' => {
                if !in_s_quotes && !in_d_quotes && !escape {
                    if !current.is_empty() {
                        result.push(current.clone());
                        current.clear();
                    }
                } else {
                    current.push(char);
                    escape = false;
                }
            }
            _ => {current.push(char);
                escape = false;
            }
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

        commands::command_handler(&cmd, &args, &builtin);
    }
}
