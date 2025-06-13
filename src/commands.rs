use std::env;

fn cmd_echo(args: &Vec<String>) {
    println!("{}", args.join(" "));
}

fn cmd_type(args: &Vec<String>, builtin: &[&str]) {
    if args.len() == 0 {
        return;
    }

    for arg in args {
        if builtin.contains(&arg.as_str()) {
            println!("{} is a shell builtin", arg);
            continue;
        }

        if let Ok(path_var) = env::var("path") {
            let path_entries = env::split_paths(&path_var);

            for dir in path_entries {
                let full_path = dir.join(&arg);
                if full_path.exists() {
                    println!("{} is {}", arg, full_path.display());
                    return;
                }
            }
        } else {
            eprintln!("failed to get path variable");
        }

        println!("{}: not found", arg);
    }
}

fn cmd_pwd() {
    println!("{}", env::current_dir().unwrap().display());
}

fn cmd_cd(args: &Vec<String>) {
    let args_len = args.len();

    if args_len == 0 {
        return;
    }
    if args_len > 1 {
        println!("cd: too many arguments");
        return;
    }
    let path = match env::home_dir() {
        Some(home_path) => {
            let home_str = home_path.to_str().unwrap_or("");
            args[0].replace("~", home_str) // returns a new string
        }
        None => {
            println!("could not determine the home directory.");
            args[0].to_string() // still return something
        }
    };
    let new_dir = std::path::Path::new(&path);
    match env::set_current_dir(new_dir) {
        Ok(_) => {}
        Err(_) => {
            println!("cd: {}: No such file or directory", path)
        }
    };
}

pub fn command_handler(cmd: &str, args: &Vec<String>, builtin: &[&str]) {
    match cmd {
        "exit" => {
            std::process::exit(0);
        }
        "echo" => {
            cmd_echo(args);
        }
        "type" => {
            cmd_type(args, &builtin);
        }
        "pwd" => {
            cmd_pwd();
        }
        "cd" => {
            cmd_cd(args);
        }
        _ => {
            let mut command = std::process::Command::new(cmd);
            command.args(args);

            match command.status() {
                Ok(_) => {}
                Err(_) => {
                    println!("{}: command not found", cmd);
                }
            }
        }
    }
}
