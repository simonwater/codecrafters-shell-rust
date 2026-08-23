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

pub struct CommandResult {
    exit: bool,
    out: String,
    error: String,
}

impl CommandResult {
    pub fn new() -> Self {
        Self {
            exit: false,
            out: String::new(),
            error: String::new(),
        }
    }

    pub fn exit(mut self) -> Self {
        self.exit = true;
        self
    }

    pub fn success(mut self, out: String) -> Self {
        self.out = out;
        self
    }

    pub fn error(mut self, err: String) -> Self {
        self.error = err;
        self
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
                if let Err(e) = handle_output(&cmd_res, &redirects) {
                    eprintln!("{:#}", e);
                }

                if cmd_res.exit {
                    break;
                }
            }
        }
    }
}

fn handle_output(cmd_res: &CommandResult, redirects: &[Redirection]) -> Result<()> {
    let mut out_handled = false;
    let mut err_handled = false;

    for redirect in redirects {
        redirect.handle(cmd_res, &mut out_handled, &mut err_handled)?;
    }

    if !out_handled && !cmd_res.out.is_empty() {
        print!("{}", cmd_res.out);
    }

    if !err_handled && !cmd_res.error.is_empty() {
        print!("{}", cmd_res.error);
    }
    Ok(())
}

fn run_command(cmd: &str, args: &[&str]) -> Result<CommandResult> {
    let cmd_res = match cmd {
        "echo" => CommandResult::new().success(format!("{}\n", args.join(" "))),
        "type" => {
            if let Some(arg) = args.first() {
                let out = match my_which(arg) {
                    ProgramType::Builtin => format!("{arg} is a shell builtin\n"),
                    ProgramType::External(path) => format!("{} is {}\n", arg, path.display()),
                    _ => format!("{arg}: not found\n"),
                };
                CommandResult::new().success(out)
            } else {
                CommandResult::new()
            }
        }
        "pwd" => CommandResult::new().success(format!("{}\n", env::current_dir()?.display())),
        "cd" => {
            let mut cmd_res = CommandResult::new();
            if let Some(&arg) = args.first() {
                let result = if arg == "~" {
                    let home = env::var("HOME")?;
                    env::set_current_dir(home)
                } else {
                    env::set_current_dir(arg)
                };

                if let Err(_) = result {
                    cmd_res = cmd_res.success(format!("cd: {}: No such file or directory\n", arg));
                }
            }
            cmd_res
        }
        "exit" => CommandResult::new().exit(),
        _ => match which(cmd) {
            Ok(_) => {
                let output = Command::new(cmd).args(args).output()?;
                let out = String::from_utf8(output.stdout)?;
                let err = String::from_utf8(output.stderr)?;
                CommandResult::new().success(out).error(err)
            }
            Err(_) => CommandResult::new().success(format!("{cmd}: command not found\n")),
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
