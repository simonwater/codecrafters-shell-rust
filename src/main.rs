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
    out: Option<String>,
    error: Option<String>,
}

impl CommandResult {
    fn new(exit: bool, out: Option<String>, error: Option<String>) -> Self {
        Self { exit, out, error }
    }

    fn proceed(out: Option<String>, error: Option<String>) -> Self {
        Self {
            exit: false,
            out,
            error,
        }
    }

    fn exit() -> Self {
        Self {
            exit: true,
            out: None,
            error: None,
        }
    }

    fn ok() -> Self {
        Self {
            exit: false,
            out: None,
            error: None,
        }
    }

    fn success(out: String) -> Self {
        Self {
            exit: false,
            out: Some(out),
            error: None,
        }
    }

    fn error(err: String) -> Self {
        Self {
            exit: false,
            out: None,
            error: Some(err),
        }
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

    if !out_handled {
        if let Some(msg) = cmd_res.out.as_ref() {
            print!("{msg}");
        }
    }

    if !err_handled {
        if let Some(msg) = cmd_res.error.as_ref() {
            print!("{msg}");
        }
    }
    Ok(())
}

fn run_command(cmd: &str, args: &[&str]) -> Result<CommandResult> {
    let cmd_res = match cmd {
        "echo" => CommandResult::success(format!("{}\n", args.join(" "))),
        "type" => {
            if let Some(arg) = args.first() {
                let out = match my_which(arg) {
                    ProgramType::Builtin => format!("{arg} is a shell builtin\n"),
                    ProgramType::External(path) => format!("{} is {}\n", arg, path.display()),
                    _ => format!("{arg}: not found\n"),
                };
                CommandResult::success(out)
            } else {
                CommandResult::ok()
            }
        }
        "pwd" => CommandResult::success(format!("{}\n", env::current_dir()?.display())),
        "cd" => {
            let mut out: Option<String> = None;
            if let Some(&arg) = args.first() {
                let result = if arg == "~" {
                    let home = env::var("HOME")?;
                    env::set_current_dir(home)
                } else {
                    env::set_current_dir(arg)
                };

                if let Err(_) = result {
                    out = Some(format!("cd: {}: No such file or directory\n", arg));
                }
            }
            CommandResult::proceed(out, None)
        }
        "exit" => CommandResult::exit(),
        _ => match which(cmd) {
            Ok(_) => {
                let output = Command::new(cmd).args(args).output()?;
                let out = Some(String::from_utf8(output.stdout)?);
                let err = Some(String::from_utf8(output.stderr)?);
                CommandResult::proceed(out, err)
            }
            Err(_) => CommandResult::success(format!("{cmd}: command not found\n")),
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
