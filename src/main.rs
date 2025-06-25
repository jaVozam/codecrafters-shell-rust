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
        let input_lines = input::input(&builtin);

        if input_lines.is_empty(){
            continue;
        }

        let (cmds, args) = input::split_inputs(input_lines);

        if cmds.len() == 1 {
            let (arg, output_conf) = input::redirection(args[0].clone());
            commands::command_handler(&cmds[0], &arg, &builtin, output_conf);
        }
        else {
            commands::run_pipeline(cmds, args, &builtin);
        }

    }
}
