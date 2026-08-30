use anyhow::{Context, Result};
use codecrafters_shell::{
    CommandResult, CompleterHelper, Environment, Redirection, builtins, executables, jobs, redirect,
};
use rustyline::config::Configurer;
use rustyline::{CompletionType, Editor, completion::FilenameCompleter, error::ReadlineError};
use std::process::Command;
use which::which;

fn main() {
    let mut commands = executables::get_path_executables();
    let builtins = vec!["echo".to_string(), "exit".to_string()];
    commands.extend(builtins);
    let ctx = Environment::new();
    let helper = CompleterHelper {
        file_completer: FilenameCompleter::new(),
        commands,
        env: &ctx,
    };
    let mut rl = Editor::new().unwrap();
    rl.set_completion_type(CompletionType::List);
    rl.set_bell_style(rustyline::config::BellStyle::Audible);
    rl.set_helper(Some(helper));

    loop {
        check_jobs();
        // read
        let input = match rl.readline("$ ") {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                println!("Aborted!");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        };
        rl.add_history_entry(input.as_str()).unwrap();

        // parse
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
        if parts.last().map(|s| s.as_str()) == Some("&") {
            let tokens = parts.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
            run_job(parts[0].as_str(), &tokens[1..tokens.len() - 1]).unwrap();
            continue;
        }

        let commands = parts.split(|token| token == "|");
        for (i, cmd_parts) in commands.enumerate() {
            let cmd = cmd_parts[0].as_str();
            let (args, redirects) = match redirect::parse_redirections(&cmd_parts[1..]) {
                Ok((args, redirects)) => (args, redirects),
                Err(e) => {
                    eprintln!("{:#}", e);
                    continue;
                }
            };

            // execute
            let _is_last = i == cmd_parts.len() - 1;
            let result = if builtins::is_builtin(cmd) {
                builtins::run_builtin(cmd, &args, &ctx)
            } else {
                run_external(cmd, &args)
            };

            // output
            match result {
                Err(e) => eprint!("{:#}", e),
                Ok(cmd_res) => {
                    if let Err(e) = handle_output(&cmd_res, &redirects) {
                        eprintln!("{:#}", e);
                    }

                    if cmd_res.exit {
                        return;
                    }
                }
            }
        }
    }
}

fn check_jobs() {
    match jobs::JOB_MANAGER.lock().unwrap().list_jobs(true) {
        Ok(out) => print!("{}", out),
        Err(e) => eprintln!("{:?}", e),
    }
}

fn run_external(cmd: &str, args: &[&str]) -> Result<CommandResult> {
    match which(cmd) {
        Ok(_) => {
            let output = Command::new(cmd).args(args).output()?;
            let out = String::from_utf8(output.stdout)?;
            let err = String::from_utf8(output.stderr)?;
            Ok(CommandResult::new().success(out).error(err))
        }
        Err(_) => Ok(CommandResult::new().success(format!("{cmd}: command not found\n"))),
    }
}

fn run_job(cmd: &str, args: &[&str]) -> Result<()> {
    match which(cmd) {
        Ok(_) => {
            let child = Command::new(cmd).args(args).spawn()?;
            let pid = child.id();
            let job_cmd = format!("{} {}", cmd, args.join(" "));
            let num = jobs::JOB_MANAGER.lock().unwrap().add_child(child, job_cmd);
            let out = format!("[{}] {}\n", num, pid);
            print!("{}", out);
        }
        Err(_) => print!("{cmd}: command not found\n"),
    };
    Ok(())
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
