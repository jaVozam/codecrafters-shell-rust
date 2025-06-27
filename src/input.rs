use rustyline::history::DefaultHistory;
use rustyline::Editor;
use std::collections::HashSet;
use crate::ShellCompleter;

pub fn get_executables() -> HashSet<String> {
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
    executables
}

pub fn input(rl: &mut Editor<ShellCompleter, DefaultHistory>) -> Vec<String> {

    let mut input = String::new();
    let mut result = Vec::new();

    let mut in_s_quotes = false;
    let mut in_d_quotes = false;
    loop {
        let prompt = if input.is_empty() { "$ " } else { "> " };

        let line = rl.readline(prompt).unwrap_or_else(|_| "".to_string());

        if !line.is_empty() {
            rl.add_history_entry(&line).ok();
        }

        if !input.is_empty() {
            input.clear();
            input.push('\n');
        }
        input += line.trim();

        let mut current = String::new();
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

    result
}

pub fn split_inputs(mut input_lines: Vec<String>) -> (Vec<String>, Vec<Vec<String>>) {
    let mut cmds: Vec<String> = Vec::new();
    let mut cmd_args = Vec::new();
    let mut args = Vec::new();

    cmds.push(input_lines.remove(0));

    let mut last_el_is_pipe = false;

    for el in input_lines {
        match el.as_str() {
            "|" => {
                args.push(cmd_args.clone());
                cmd_args.clear();
                cmd_args.push("".to_string());
                last_el_is_pipe = true;
            }
            _ => {
                if last_el_is_pipe {
                    cmds.push(el);
                    last_el_is_pipe = false;
                } else {
                    cmd_args.push(el);
                }
            }
        }
    }

    if !cmd_args.is_empty() {
        args.push(cmd_args);
    }

    if args.is_empty() {
        args.push(vec![])
    }

    (cmds, args)
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
