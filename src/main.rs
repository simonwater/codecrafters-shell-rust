use anyhow::{Context, Result};
use std::env;
#[allow(unused_imports)]
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use which::which;

use crate::redirect::Redirection;

pub mod redirect;

pub enum ProgramType {
    Builtin,
    External(PathBuf),
    Unknown,
}

pub enum RunState {
    Out(String),
    Error(String),
    None,
}

pub struct CommandResult {
    exit: bool,
    state: RunState,
}

impl CommandResult {
    fn new(exit: bool, state: RunState) -> Self {
        Self { exit, state }
    }

    fn exit(state: RunState) -> Self {
        Self::new(true, state)
    }

    fn proceed(state: RunState) -> Self {
        Self::new(false, state)
    }
}

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut input = String::with_capacity(32);
        io::stdin().read_line(&mut input).unwrap();

        let parts = match shell_words::split(&input).context("command line parse error.") {
            Ok(parts) => parts,
            Err(e) => {
                eprintln!("{:#}", e);
                continue;
            }
        };
        if parts.is_empty() {
            continue;
        }

        let cmd = parts[0].as_str();
        let (args, redirects) = match redirect::parse_redirections(&parts[1..]) {
            Ok((args, redirects)) => (args, redirects),
            Err(e) => {
                eprintln!("{:#}", e);
                continue;
            }
        };

        match run_command(cmd, &args) {
            Err(e) => eprint!("{:#}", e),
            Ok(cmd_res) => {
                if let Err(e) = handle_output(cmd_res.state, &redirects) {
                    eprintln!("{:#}", e);
                }

                if cmd_res.exit {
                    break;
                }
            }
        }
    }
}

fn handle_output(state: RunState, redirects: &[Redirection]) -> Result<()> {
    let mut out_handled = false;
    let mut err_handled = false;

    for redirect in redirects {
        redirect.handle(&state, &mut out_handled, &mut err_handled)?;
    }

    match state {
        RunState::Out(msg) => {
            if !out_handled {
                print!("{msg}");
            }
        }
        RunState::Error(msg) => {
            if !err_handled {
                eprint!("{msg}");
            }
        }
        _ => {}
    };
    Ok(())
}

fn run_command(cmd: &str, args: &[&str]) -> Result<CommandResult> {
    let cmd_res = match cmd {
        "echo" => CommandResult::proceed(RunState::Out(format!("{}\n", args.join(" ")))),
        "type" => {
            if let Some(arg) = args.first() {
                let out = match my_which(arg) {
                    ProgramType::Builtin => format!("{arg} is a shell builtin\n"),
                    ProgramType::External(path) => format!("{} is {}\n", arg, path.display()),
                    _ => format!("{arg}: not found\n"),
                };
                CommandResult::proceed(RunState::Out(out))
            } else {
                CommandResult::proceed(RunState::None)
            }
        }
        "pwd" => CommandResult::proceed(RunState::Out(format!(
            "{}\n",
            env::current_dir()?.display()
        ))),
        "cd" => {
            let mut state = RunState::None;
            if let Some(&arg) = args.first() {
                let result = if arg == "~" {
                    let home = env::var("HOME")?;
                    env::set_current_dir(home)
                } else {
                    env::set_current_dir(arg)
                };

                if let Err(_) = result {
                    state = RunState::Out(format!("cd: {}: No such file or directory\n", arg));
                }
            }
            CommandResult::proceed(state)
        }
        "exit" => CommandResult::exit(RunState::None),
        _ => match which(cmd) {
            Ok(_) => {
                let output = Command::new(cmd).args(args).output()?;
                if output.status.success() {
                    CommandResult::proceed(RunState::Out(String::from_utf8(output.stdout)?))
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let msg = format!("{}", stderr);
                    CommandResult::proceed(RunState::Error(msg))
                }
            }
            Err(_) => CommandResult::proceed(RunState::Out(format!("{cmd}: command not found\n"))),
        },
    };
    Ok(cmd_res)
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
