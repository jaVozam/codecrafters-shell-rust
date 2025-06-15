use std::env;
use std::fs;
use std::io::Write;

use crate::OutputConf;
use crate::OutputMode;

enum OutputMsgType {
    StdOut,
    StdErr,
}

struct OutputMsg {
    message: String,
    msg_type: OutputMsgType,
}

fn msg(message: String) -> OutputMsg {
    OutputMsg {
        message: message,
        msg_type: OutputMsgType::StdOut,
    }
}
fn err(message: String) -> OutputMsg {
    OutputMsg {
        message: message,
        msg_type: OutputMsgType::StdErr,
    }
}

fn cmd_echo(args: &Vec<String>) -> Option<OutputMsg> {
    return Some(msg(args.join(" ")));
}

fn cmd_type(arg: &String, builtin: &[&str]) -> Option<OutputMsg> {
    if arg.is_empty() {
        return None;
    }

    if builtin.contains(&arg.as_str()) {
        return Some(msg(format!("{} is a shell builtin", arg)));
    }

    if let Ok(path_var) = env::var("PATH") {
        let path_entries = env::split_paths(&path_var);

        for dir in path_entries {
            let full_path = dir.join(&arg);
            if full_path.exists() {
                return Some(msg(format!("{} is {}", arg, full_path.display())));
            }
        }
    } else {
        return Some(err("failed to get path variable".to_string()));
    }

    return Some(err(format!("{}: not found", arg)));
}

fn cmd_pwd() -> Option<OutputMsg> {
    return Some(msg(format!("{}", env::current_dir().unwrap().display())));
}

fn cmd_cd(args: &Vec<String>) -> Option<OutputMsg> {
    let args_len = args.len();

    if args_len == 0 {
        return None;
    }
    if args_len > 1 {
        return Some(err("cd: too many arguments".to_string()));
    }
    let path = match env::home_dir() {
        Some(home_path) => {
            let home_str = home_path.to_str().unwrap_or("");
            args[0].replace("~", home_str) // returns a new string
        }
        None => {
            eprintln!("could not determine the home directory.");
            args[0].to_string() // still return something
        }
    };
    let new_dir = std::path::Path::new(&path);
    match env::set_current_dir(new_dir) {
        Ok(_) => {}
        Err(_) => return Some(err(format!("cd: {}: No such file or directory", path))),
    };
    return None;
}

fn cmd_run(cmd: &str, args: &Vec<String>) -> Vec<Option<OutputMsg>> {
    let mut command = std::process::Command::new(cmd);
    command.args(args);

    match command.output() {
        Ok(output) => {
            let mut result = Vec::new();

            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !stdout.is_empty() {
                result.push(Some(msg(stdout)));
            }

            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            if !stderr.is_empty() {
                result.push(Some(err(stderr)));
            }

            result
        }
        Err(_) => {
            vec![Some(err(format!("{}: command not found", cmd)))]
        }
    }
}

fn write_to_file(path: &String, value: String) -> std::io::Result<()> {
    let mut file = fs::File::create(path)?;

    file.write_all(value.as_bytes())?;

    Ok(())
}

fn append_to_file(path: &String, value: String) -> std::io::Result<()> {
    let mut file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)?;

    file.write_all(value.as_bytes())?;

    Ok(())
}

fn output_handler(outputs: Vec<Option<OutputMsg>>, output_conf: OutputConf) {
    if output_conf.std_out != "" {
        append_to_file(&output_conf.std_out, "".to_string()).unwrap();
    }
    if output_conf.std_err != "" {
        append_to_file(&output_conf.std_err, "".to_string()).unwrap();
    }
    for output in outputs {
        match output {
            Some(value) => match value.msg_type {
                OutputMsgType::StdOut => match output_conf.std_out_mode {
                    OutputMode::Default => {
                        println!("{}", value.message);
                    }
                    OutputMode::File => {
                        write_to_file(&output_conf.std_out, value.message).unwrap();
                    }
                    OutputMode::FileAppend => {
                        append_to_file(&output_conf.std_out, value.message).unwrap();
                    }
                },
                OutputMsgType::StdErr => match output_conf.std_err_mode {
                    OutputMode::Default => {
                        eprintln!("{}", value.message);
                    }
                    OutputMode::File => {
                        write_to_file(&output_conf.std_err, value.message).unwrap();
                    }
                    OutputMode::FileAppend => {
                        append_to_file(&output_conf.std_err, value.message).unwrap();
                    }
                },
            },
            None => {}
        }
    }
}

pub fn command_handler(cmd: &str, args: &Vec<String>, builtin: &[&str], output_conf: OutputConf) {
    let mut outputs = Vec::new();
    match cmd {
        "exit" => {
            std::process::exit(0);
        }
        "echo" => {
            outputs.push(cmd_echo(args));
        }
        "type" => {
            for arg in args {
                outputs.push(cmd_type(arg, &builtin));
            }
        }
        "pwd" => {
            outputs.push(cmd_pwd());
        }
        "cd" => {
            outputs.push(cmd_cd(args));
        }
        _ => {
            let output = cmd_run(cmd, args);
            for value in output {
                outputs.push(value);
            } 
        },
    }

    output_handler(outputs, output_conf);
}
