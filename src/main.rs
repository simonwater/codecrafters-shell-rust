use anyhow::{Context, Result};
use std::env;
#[allow(unused_imports)]
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use which::which;

pub enum ProgramType {
    Builtin,
    External(PathBuf),
    Unknown,
}

pub enum RunState {
    Exit,
    Continue,
}

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::with_capacity(32);
        io::stdin().read_line(&mut input).unwrap();

        match run(input) {
            Err(e) => eprintln!("{:#}", e),
            Ok(state) => match state {
                RunState::Continue => continue,
                RunState::Exit => return,
            },
        }
    }
}

fn run(input: String) -> Result<RunState> {
    let parts = shell_words::split(&input).context("command line parse error.")?;
    if parts.is_empty() {
        return Ok(RunState::Continue);
    }
    let cmd = parts[0].as_str();
    let args = &parts[1..];

    match cmd {
        "echo" => println!("{}", args.join(" ")),
        "type" => {
            if !args.is_empty() {
                let arg = &args[0];
                match my_which(arg) {
                    ProgramType::Builtin => println!("{arg} is a shell builtin"),
                    ProgramType::External(path) => println!("{} is {}", arg, path.display()),
                    _ => println!("{arg}: not found"),
                }
            }
        }
        "pwd" => println!("{}", env::current_dir()?.display()),
        "cd" => {
            if !args.is_empty() {
                let arg = &args[0];
                let result = if arg == "~" {
                    let home = env::var("HOME")?;
                    env::set_current_dir(home)
                } else {
                    env::set_current_dir(arg)
                };
                result.unwrap_or_else(|_| println!("cd: {}: No such file or directory", arg));
            }
        }
        "exit" => return Ok(RunState::Exit),
        _ => match which(cmd) {
            Ok(_) => {
                let output = Command::new(cmd).args(args).output()?;
                if output.status.success() {
                    print!("{}", String::from_utf8(output.stdout)?);
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    eprintln!("execute fail! exit code: {:?}", output.status.code());
                    eprintln!("error info:\n{}", stderr);
                }
            }
            Err(_) => println!("{cmd}: command not found"),
        },
    }
    Ok(RunState::Continue)
}

fn my_which(cmd: &str) -> ProgramType {
    if is_builtin(cmd) {
        ProgramType::Builtin
    } else if let Ok(path) = which(cmd) {
        ProgramType::External(path)
    } else {
        ProgramType::Unknown
    }
}

fn is_builtin(s: &str) -> bool {
    match s {
        "echo" | "type" | "exit" | "pwd" | "cd" => true,
        _ => false,
    }
}
