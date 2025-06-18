#[allow(unused_imports)]
use std::io::{self, Write};

mod commands;
mod input;

fn main() {
    let builtin = ["echo", "exit", "type", "pwd", "cd"]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    loop {
        let (cmd, args) = input::input(&builtin);
        if cmd == "" {
            continue;
        }

        let (args, output_conf) = input::redirection(args);

        commands::command_handler(&cmd, &args, &builtin, output_conf);
    }
}
