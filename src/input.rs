use rustyline::Helper;
use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::Editor;


struct ShellCompleter {
    commands: Vec<String>
}

impl Completer for ShellCompleter {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        _pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let completions = self.commands.clone();
        let mut candidates = Vec::new();

        for completion in completions {
            if completion.starts_with(line) {
                candidates.push(completion);
            }
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
pub fn input(builtin: &Vec<String>) -> (String, Vec<String>) {
    let completer = ShellCompleter {
        commands: builtin.to_owned()
    };

    // Create editor with completer
    let mut rl = Editor::new().expect("Failed to create rustyline Editor");
    rl.set_helper(Some(completer));


    let mut input = String::new();
    let mut result = Vec::new();

    loop {
        // Show "$ " prompt for first line, then "> " for multiline continuation
        let prompt = if input.is_empty() { "$ " } else { "> " };

        let line = rl.readline(prompt).unwrap_or_else(|_| "".to_string());
        input.push_str(&line);
        input.push('\n'); // keep newlines as part of input if you want multiline support

        // Your existing parsing logic with input.trim()
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

        // If all quotes are balanced, break loop, else continue multiline input
        if !in_s_quotes && !in_d_quotes {
            break;
        }
    }

    if result.is_empty() {
        return ("".to_string(), vec![]);
    }

    let cmd = result.remove(0);
    let args = result;

    (cmd, args)
}

pub enum OutputMode {
    Default,
    File,
    FileAppend,
}

pub struct OutputConf {
    pub std_out: String,
    pub std_out_mode: OutputMode,
    pub std_err: String,
    pub std_err_mode: OutputMode,
}

pub fn redirection(mut args: Vec<String>) -> (Vec<String>, OutputConf){
    let mut output_conf = OutputConf {
        std_out: "".to_string(),
        std_out_mode: OutputMode::Default,
        std_err: "".to_string(),
        std_err_mode: OutputMode::Default,
    };

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "1>" | ">" => {
                if i + 1 < args.len() {
                    output_conf.std_out = args[i + 1].clone();
                    output_conf.std_out_mode = OutputMode::File;
                    args.remove(i + 1);
                } else {
                    eprintln!("syntax error");
                }
                args.remove(i);
            }
            "2>" => {
                if i + 1 < args.len() {
                    output_conf.std_err = args[i + 1].clone();
                    output_conf.std_err_mode = OutputMode::File;
                    args.remove(i + 1);
                } else {
                    eprintln!("syntax error");
                }
                args.remove(i);
            }
            "1>>" | ">>" => {
                if i + 1 < args.len() {
                    output_conf.std_out = args[i + 1].clone();
                    output_conf.std_out_mode = OutputMode::FileAppend;
                    args.remove(i + 1);
                } else {
                    eprintln!("syntax error");
                }
                args.remove(i);
            }
            "2>>" => {
                if i + 1 < args.len() {
                    output_conf.std_err = args[i + 1].clone();
                    output_conf.std_err_mode = OutputMode::FileAppend;
                    args.remove(i + 1);
                } else {
                    eprintln!("syntax error");
                }
                args.remove(i);
            }
            _ => {}
        }
        i += 1;
    }

    (args, output_conf)
}
