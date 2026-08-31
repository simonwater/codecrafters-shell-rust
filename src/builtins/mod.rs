pub mod complete;

use crate::command::{CommandType, ShellCommand, ShellOutput};
use crate::environment::Environment;
use crate::jobs::JOB_MANAGER;
use anyhow::Result;
pub use complete::complete;
use std::env;
use std::process::Stdio;
use which::which;

pub fn run_builtin(
    _prev_stdout: &mut Option<Stdio>,
    shell_cmd: &mut ShellCommand,
    environment: &Environment,
) -> Result<ShellOutput> {
    let args = &shell_cmd.args;
    let mut output = match shell_cmd.name {
        "echo" => ShellOutput::new().success(format!("{}\n", args.join(" "))),
        "pwd" => ShellOutput::new().success(format!("{}\n", env::current_dir()?.display())),
        "exit" => ShellOutput::new().exit(),
        "complete" => complete(args, environment)?,
        "type" => my_type(args),
        "cd" => cd(args)?,
        "jobs" => jobs(args)?,
        _ => ShellOutput::new().success(format!("{}: command not found\n", shell_cmd.name)),
    };

    // 错误
    if !shell_cmd.err_redirects.is_empty() {
        for r in &shell_cmd.err_redirects {
            r.handle_err(&output)?;
        }
        shell_cmd.err_redirects.clear();
    } else if !output.err.is_empty() {
        eprint!("{}", output.err);
    }
    output.err.clear(); // 处理完

    // 输出
    if !shell_cmd.out_redirects.is_empty() {
        for r in &shell_cmd.out_redirects {
            r.handle_out(&output)?;
        }
        shell_cmd.out_redirects.clear();
    } else if !output.out.is_empty() {
        print!("{}", output.out);
    }
    output.out.clear(); // 处理完

    Ok(output)
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

pub fn cd(args: &[&str]) -> Result<ShellOutput> {
    let mut cmd_res = ShellOutput::new();
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

pub fn my_type(args: &[&str]) -> ShellOutput {
    if let Some(arg) = args.first() {
        let out = match my_which(arg) {
            CommandType::Builtin => format!("{arg} is a shell builtin\n"),
            CommandType::External(path) => format!("{} is {}\n", arg, path.display()),
            _ => format!("{arg}: not found\n"),
        };
        ShellOutput::new().success(out)
    } else {
        ShellOutput::null()
    }
}

pub fn jobs(_args: &[&str]) -> Result<ShellOutput> {
    let out = JOB_MANAGER.lock().unwrap().list_jobs(false)?;
    Ok(ShellOutput::new().success(out))
}
