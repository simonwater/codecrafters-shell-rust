pub mod complete;

use crate::command::{CommandResult, CommandType};
use crate::environment::Environment;
use crate::jobs::JOB_MANAGER;
use anyhow::Result;
pub use complete::complete;
use std::env;
use which::which;

pub fn run_builtin(cmd: &str, args: &[&str], environment: &Environment) -> Result<CommandResult> {
    match cmd {
        "echo" => Ok(CommandResult::new().success(format!("{}\n", args.join(" ")))),
        "pwd" => Ok(CommandResult::new().success(format!("{}\n", env::current_dir()?.display()))),
        "exit" => Ok(CommandResult::new().exit()),
        "complete" => complete(args, environment),
        "type" => my_type(args),
        "cd" => cd(args),
        "jobs" => jobs(args),
        _ => Ok(CommandResult::new().success(format!("{cmd}: command not found\n"))),
    }
}

pub fn is_builtin(s: &str) -> bool {
    match s {
        "echo" | "type" | "exit" | "pwd" | "cd" | "complete" | "jobs" => true,
        _ => false,
    }
}

fn my_which(cmd: &str) -> CommandType {
    if is_builtin(cmd) {
        CommandType::Builtin
    } else if let Ok(path) = which(cmd) {
        CommandType::External(path)
    } else {
        CommandType::Unknown
    }
}

pub fn cd(args: &[&str]) -> Result<CommandResult> {
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
    Ok(cmd_res)
}

pub fn my_type(args: &[&str]) -> Result<CommandResult> {
    if let Some(arg) = args.first() {
        let out = match my_which(arg) {
            CommandType::Builtin => format!("{arg} is a shell builtin\n"),
            CommandType::External(path) => format!("{} is {}\n", arg, path.display()),
            _ => format!("{arg}: not found\n"),
        };
        Ok(CommandResult::new().success(out))
    } else {
        Ok(CommandResult::new())
    }
}

pub fn jobs(_args: &[&str]) -> Result<CommandResult> {
    let out = JOB_MANAGER.lock().unwrap().list_jobs(false)?;
    Ok(CommandResult::new().success(out))
}
