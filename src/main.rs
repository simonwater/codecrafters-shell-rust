use anyhow::{Context, Result};
use codecrafters_shell::{
    CommandResult, CompleterHelper, Environment, Redirection, builtins, executables, redirect,
};
use rustyline::config::Configurer;
use rustyline::{CompletionType, Editor, completion::FilenameCompleter, error::ReadlineError};
use std::env;
#[allow(unused_imports)]
use std::io::{self, Write};
use std::process::Command;
use which::which;

fn main() {
    let mut commands = executables::get_path_executables();
    let builtins = vec!["echo".to_string(), "exit".to_string()];
    commands.extend(builtins);
    let helper = CompleterHelper {
        file_completer: FilenameCompleter::new(),
        commands,
    };
    let mut rl = Editor::new().unwrap();
    rl.set_completion_type(CompletionType::List);
    rl.set_helper(Some(helper));
    let mut ctx = Environment::new();

    loop {
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

        let cmd = parts[0].as_str();
        let (args, redirects) = match redirect::parse_redirections(&parts[1..]) {
            Ok((args, redirects)) => (args, redirects),
            Err(e) => {
                eprintln!("{:#}", e);
                continue;
            }
        };

        // execute
        match run_command(cmd, &args, &mut ctx) {
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

fn run_command(cmd: &str, args: &[&str], environment: &mut Environment) -> Result<CommandResult> {
    match cmd {
        "echo" => Ok(CommandResult::new().success(format!("{}\n", args.join(" ")))),
        "pwd" => Ok(CommandResult::new().success(format!("{}\n", env::current_dir()?.display()))),
        "exit" => Ok(CommandResult::new().exit()),
        "complete" => builtins::complete(args, environment),
        "type" => builtins::my_type(args),
        "cd" => builtins::cd(args),
        _ => match which(cmd) {
            Ok(_) => {
                let output = Command::new(cmd).args(args).output()?;
                let out = String::from_utf8(output.stdout)?;
                let err = String::from_utf8(output.stderr)?;
                Ok(CommandResult::new().success(out).error(err))
            }
            Err(_) => Ok(CommandResult::new().success(format!("{cmd}: command not found\n"))),
        },
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
