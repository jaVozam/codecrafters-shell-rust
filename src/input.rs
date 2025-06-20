use std::collections::HashSet;
use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::history::DefaultHistory;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::Editor;
use rustyline::Helper;

struct ShellCompleter {
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

        if candidates.len() == 1{
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
pub fn input(builtin: &Vec<String>) -> (String, Vec<String>) {
    let mut executables = HashSet::new();

    if let Ok(path_var) = std::env::var("PATH") {
        let path_entries = std::env::split_paths(&path_var);

        for dir in path_entries {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();

                    if path.is_file() {
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            if let Ok(metadata) = path.metadata() {
                                if metadata.permissions().mode() & 0o111 != 0 {
                                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                        executables.insert(name.to_string());
                                    }
                                }
                            }
                        }

                        #[cfg(windows)]
                        {
                            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                                let ext = ext.to_ascii_lowercase();
                                if ext == "exe" || ext == "bat" || ext == "cmd" {
                                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                        executables.push(name.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    executables.extend(builtin.clone());
    let completer = ShellCompleter {
        commands: executables,
    };

    let config = rustyline::Config::builder().completion_type(rustyline::CompletionType::List).build();

    let mut rl = Editor::<ShellCompleter, DefaultHistory>::with_config(config).expect("Failed to create rustyline Editor");
    rl.set_helper(Some(completer));

    let mut input = String::new();
    let mut result = Vec::new();

    loop {
        let prompt = if input.is_empty() { "$ " } else { "> " };

        let line = rl.readline(prompt).unwrap_or_else(|_| "".to_string());
        input = line;
        input.push('\n');

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

pub fn redirection(mut args: Vec<String>) -> (Vec<String>, OutputConf) {
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
