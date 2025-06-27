use std::env;
use std::fs;
use std::io::Write;

use crate::input;
use crate::ShellCompleter;

use input::OutputConf;
use input::OutputMode;
use rustyline::history::DefaultHistory;
use rustyline::Editor;

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

fn cmd_type(arg: &String, builtin: &Vec<String>) -> Option<OutputMsg> {
    if arg.is_empty() {
        return None;
    }

    if builtin.contains(&arg) {
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
use std::fmt::Write as FmtWrite;

fn cmd_history(
    rl: &mut Editor<ShellCompleter, DefaultHistory>,
    args: &Vec<String>,
) -> Option<OutputMsg> {
    let mut history = String::new();

    let history_len = rl.history().iter().count();

    let mut n: usize = history_len;
    if !args.is_empty() {
        if args[0] == "-r" {
            rl.load_history(&args[1]).ok();
            return None;
        } else if args[0] == "-w" {
            let file = std::fs::File::create(&args[1]).unwrap();
            let mut writer = std::io::BufWriter::new(file);
            for entry in rl.history().iter() {
                writeln!(writer, "{}", entry).unwrap();
            }
            return None;
        } else if args[0] == "-a" {
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&args[1]).unwrap();
            for entry in rl.history().iter() {
                writeln!(file, "{}", entry).ok();
            }
        } else {
            n = args[0].parse().expect("Not a valid number");
        }
    }

    let start = if history_len > n { history_len - n } else { 0 };

    for (i, entry) in rl.history().iter().skip(start).enumerate() {
        writeln!(&mut history, "\t{} {}", start + i + 1, entry).unwrap();
    }
    Some(msg(history.trim_end().to_string()))
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
                        write_to_file(&output_conf.std_out, value.message + "\n").unwrap();
                    }
                    OutputMode::FileAppend => {
                        append_to_file(&output_conf.std_out, value.message + "\n").unwrap();
                    }
                },
                OutputMsgType::StdErr => match output_conf.std_err_mode {
                    OutputMode::Default => {
                        eprintln!("{}", value.message);
                    }
                    OutputMode::File => {
                        write_to_file(&output_conf.std_err, value.message + "\n").unwrap();
                    }
                    OutputMode::FileAppend => {
                        append_to_file(&output_conf.std_err, value.message + "\n").unwrap();
                    }
                },
            },
            None => {}
        }
    }
}

fn run_builtin(
    cmd: &str,
    args: &Vec<String>,
    builtin: &Vec<String>,
    rl: &mut Editor<ShellCompleter, DefaultHistory>,
) -> Vec<Option<OutputMsg>> {
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
                outputs.push(cmd_type(arg, builtin));
            }
        }
        "pwd" => {
            outputs.push(cmd_pwd());
        }
        "cd" => {
            outputs.push(cmd_cd(args));
        }
        "history" => {
            outputs.push(cmd_history(rl, args));
        }
        _ => {}
    }

    outputs
}

use std::os::unix::io::RawFd;

fn write_outputs_to_fd(outputs: Vec<Option<OutputMsg>>, out_fd: RawFd) {
    let mut writer = unsafe { std::fs::File::from_raw_fd(out_fd) };

    for output in outputs {
        if let Some(msg) = output {
            writeln!(writer, "{}", msg.message).unwrap();
        }
    }
}

use nix::unistd::{close, pipe};
use std::os::unix::io::FromRawFd;
use std::process::{Command, Stdio};

pub fn run_pipeline(
    cmds: Vec<String>,
    args: Vec<Vec<String>>,
    builtin: &Vec<String>,
    rl: &mut Editor<ShellCompleter, DefaultHistory>,
) {
    let mut children = Vec::new();
    let mut prev_read: Option<RawFd> = None;

    for i in 0..cmds.len() {
        let is_last = i == cmds.len() - 1;

        let (read_fd, write_fd) = if !is_last {
            let (r, w) = pipe().expect("pipe failed");
            (Some(r), Some(w))
        } else {
            (None, None)
        };

        let cmd_name = &cmds[i];
        let filtered_args: Vec<String> = args[i]
            .iter()
            .filter(|arg| !arg.is_empty())
            .cloned()
            .collect();

        if builtin.contains(cmd_name) {
            let output = run_builtin(cmd_name, &filtered_args, &builtin, rl);
            if let Some(wfd) = write_fd {
                write_outputs_to_fd(output, wfd);
                close(wfd).ok();
            } else {
                for out in output {
                    if let Some(msg) = out {
                        println!("{}", msg.message);
                    }
                }
            }
        } else {
            let mut cmd = Command::new(cmd_name);

            if !filtered_args.is_empty() {
                cmd.args(&filtered_args);
            }

            if let Some(fd) = prev_read {
                let stdin = unsafe { Stdio::from_raw_fd(fd) };
                cmd.stdin(stdin);
            }

            if let Some(wfd) = write_fd {
                let stdout = unsafe { Stdio::from_raw_fd(wfd) };
                cmd.stdout(stdout);
            }

            let child = cmd.spawn().expect(&format!("Failed to run {}", cmd_name));
            children.push(child);
        }

        if let Some(wfd) = write_fd {
            close(wfd).ok();
        }

        if let Some(rfd) = prev_read {
            close(rfd).ok();
        }

        prev_read = read_fd;
    }

    for mut child in children {
        child.wait().expect("Failed to wait for child");
    }
}

pub fn command_handler(
    cmd: &str,
    args: &Vec<String>,
    builtin: &Vec<String>,
    output_conf: OutputConf,
    rl: &mut Editor<ShellCompleter, DefaultHistory>,
) {
    let mut outputs = Vec::new();

    if builtin.contains(&cmd.to_string()) {
        outputs = run_builtin(cmd, args, builtin, rl);
    } else {
        let output = cmd_run(cmd, args);
        for value in output {
            outputs.push(value);
        }
    }

    output_handler(outputs, output_conf);
}
