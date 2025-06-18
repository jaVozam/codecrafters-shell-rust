#[allow(unused_imports)]
use std::io::{self, Write};

mod commands;
mod input;

fn main() {
    let builtin = ["echo".to_string(), "exit".to_string(), "type".to_string(), "pwd".to_string(), "cd".to_string()].to_vec();


    loop {

        let (cmd, args) = input::input(&builtin);
        if cmd == "" {
            continue;
        }
        
        let (args, output_conf) = input::redirection(args);

        commands::command_handler(&cmd, &args, &builtin, output_conf);
    }
}
