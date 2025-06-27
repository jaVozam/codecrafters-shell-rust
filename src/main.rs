#[allow(unused_imports)]
use std::io::{self, Write};

mod commands;
mod input;

use input::input;
use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::history::DefaultHistory;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::Editor;
use rustyline::Helper;
use std::collections::HashSet;

pub struct ShellCompleter {
    commands: HashSet<String>,
}

impl Completer for ShellCompleter {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        _pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let mut candidates = Vec::new();

        for completion in &self.commands {
            if completion.starts_with(line) {
                candidates.push(completion.clone());
            }
        }

        candidates.sort();

        if candidates.len() == 1 {
            candidates[0].push(' ');
        }

        Ok((0, candidates))
    }
}

impl Helper for ShellCompleter {}

impl Validator for ShellCompleter {
    fn validate(&self, _ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        Ok(ValidationResult::Valid(None))
    }
}

impl Highlighter for ShellCompleter {}

impl Hinter for ShellCompleter {
    type Hint = String;
}

fn main() {
    let builtin = ["echo", "exit", "type", "pwd", "cd", "history"]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    let mut executables = input::get_executables();
    executables.extend(builtin.clone());
    let completer = ShellCompleter {
        commands: executables,
    };

    let config = rustyline::Config::builder()
        .completion_type(rustyline::CompletionType::List)
        .build();

    let mut rl = Editor::<ShellCompleter, DefaultHistory>::with_config(config)
        .expect("Failed to create rustyline Editor");
    rl.set_helper(Some(completer));

    let histfile = std::env::var("HISTFILE").ok();
    
    if let Some(path) = histfile {
        rl.load_history(&path).ok();
    }

    loop {
        let input_lines = input::input(&mut rl);

        if input_lines.is_empty() {
            continue;
        }

        let (cmds, args) = input::split_inputs(input_lines);

        if cmds.len() == 1 {
            let (arg, output_conf) = input::redirection(args[0].clone());
            commands::command_handler(&cmds[0], &arg, &builtin, output_conf, &mut rl);
        } else {
            commands::run_pipeline(cmds, args, &builtin, &mut rl);
        }
    }
}
