use anyhow::{Context, Result};
use codecrafters_shell::{
    CommandResult, CompleterHelper, Environment, ShellCommand, builtins, executables, jobs,
};
use rustyline::config::Configurer;
use rustyline::{CompletionType, Editor, completion::FilenameCompleter, error::ReadlineError};
use std::process::{Child, Command, Stdio};
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

        // parse tokens
        let tokens = match shell_words::split(&input).context("command line parse error.") {
            Ok(tokens) => tokens,
            Err(e) => {
                eprintln!("{:#}", e);
                continue;
            }
        };
        if tokens.is_empty() {
            continue;
        }

        // parse job
        if tokens.last().map(|s| s.as_str()) == Some("&") {
            let str_tokens = tokens.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
            run_job(tokens[0].as_str(), &str_tokens[1..str_tokens.len() - 1]).unwrap();
            continue;
        }

        let commands = tokens
            .split(|token| token == "|")
            .collect::<Vec<&[String]>>();
        let mut prev_out: Option<Stdio> = None;
        let mut children: Vec<Child> = Vec::with_capacity(commands.len());
        for (i, &cmd_tokens) in commands.iter().enumerate() {
            // parse command
            let cmd = match ShellCommand::parse(cmd_tokens) {
                Ok(cmd) => cmd,
                Err(e) => {
                    eprintln!("{:#}", e);
                    continue;
                }
            };

            // execute
            let is_first = i == 0;
            let is_last = i == commands.len() - 1;
            let result = if builtins::is_builtin(cmd.name) {
                builtins::run_builtin(cmd.name, &cmd.args, &ctx)
            } else {
                run_external(&mut prev_out, &cmd, is_first, is_last, &mut children)
            };

            match result {
                Ok(cmd_res) => {
                    if let Err(e) = handle_output(&cmd_res, &cmd, is_last) {
                        eprintln!("{:#}", e);
                    }

                    if cmd_res.exit {
                        return;
                    }
                }
                Err(e) => {
                    eprint!("{:#}", e);
                    continue;
                }
            }
        }

        for mut child in children {
            if let Err(e) = child.wait() {
                eprintln!("{:#}", e);
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

fn run_external(
    prev_stdout: &mut Option<Stdio>,
    shell_cmd: &ShellCommand,
    is_first: bool,
    is_last: bool,
    children: &mut Vec<Child>,
) -> Result<CommandResult> {
    match which(shell_cmd.name) {
        Ok(_) => {
            let mut command = Command::new(shell_cmd.name);
            command.args(&shell_cmd.args);

            // 输入
            if is_first {
                command.stdin(Stdio::inherit());
            } else if let Some(prev) = prev_stdout.take() {
                command.stdin(prev);
            } else {
                command.stdin(Stdio::piped());
            }

            // 错误
            if !shell_cmd.err_redirects.is_empty() {
                for r in &shell_cmd.err_redirects {
                    let file = r.redirect_file()?;
                    command.stderr(Stdio::from(file));
                }
            } else {
                command.stderr(Stdio::inherit());
            }

            // 输出
            if !shell_cmd.out_redirects.is_empty() {
                // 重定向优先级高于管道
                for r in &shell_cmd.out_redirects {
                    let file = r.redirect_file()?;
                    command.stdout(Stdio::from(file));
                }
            } else if is_last {
                command.stdout(Stdio::inherit());
            } else {
                command.stdout(Stdio::piped());
            }

            let mut child = command.spawn()?;
            if !is_last {
                if let Some(stdout) = child.stdout.take() {
                    *prev_stdout = Some(Stdio::from(stdout));
                }
            }
            children.push(child);

            Ok(CommandResult::new())
        }
        Err(_) => {
            Ok(CommandResult::new().success(format!("{}: command not found\n", shell_cmd.name)))
        }
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

fn handle_output(cmd_res: &CommandResult, cmd: &ShellCommand, is_last: bool) -> Result<()> {
    // handle out
    if !cmd.out_redirects.is_empty() {
        for r in &cmd.out_redirects {
            r.handle(cmd_res)?;
        }
    } else if is_last && !cmd_res.out.is_empty() {
        print!("{}", cmd_res.out);
    }

    // handle error
    if !cmd.err_redirects.is_empty() {
        for r in &cmd.err_redirects {
            r.handle(cmd_res)?;
        }
    } else if !cmd_res.error.is_empty() {
        print!("{}", cmd_res.error);
    }
    Ok(())
}
