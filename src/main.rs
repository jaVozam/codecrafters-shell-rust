#[allow(unused_imports)]
use std::io::{self, Write};

mod commands;

fn input() -> (String, Vec<String>) {
    print!("$ ");
    io::stdout().flush().unwrap();

    let mut input = String::new();

    let mut result = Vec::new();
    loop {
        io::stdin().read_line(&mut input).unwrap();

        let mut current = String::new();
        let mut in_s_quotes = false;
        let mut in_d_quotes = false;
        let mut escape = false;
        for char in input.trim().chars() {
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
                        if escape && in_d_quotes {
                            current.pop();
                        }
                        current.push(char);
                        escape = false;
                    }
                }
                '\\' => {
                    if in_s_quotes {
                        current.push(char);
                    } else if in_d_quotes && escape {
                        escape = false;
                    } else if !escape && in_d_quotes {
                        current.push(char);
                        escape = true;
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
                _ => {
                    current.push(char);
                    escape = false;
                }
            }
        }

        if !current.is_empty() {
            result.push(current);
        }
        if !in_s_quotes && !in_d_quotes {
            break;
        }

        input.clear();
        print!("> ");
        io::stdout().flush().unwrap();
    }

    if result.is_empty() {
        return ("".to_string(), vec!());
    }

    let cmd = result.remove(0);

    let args = result;

    return (cmd, args);
}

fn main() {
    loop {
        let builtin = ["echo", "exit", "type", "pwd", "cd"];

        let (cmd, args) = input();

        if cmd == "" {
            continue;
        }

        commands::command_handler(&cmd, &args, &builtin);
    }
}
