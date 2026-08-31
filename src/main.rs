use anyhow::{Context, Result};
use codecrafters_shell::{
    CompleterHelper, Environment, ShellCommand, ShellOutput, builtins, executables, jobs,
};
use os_pipe::{PipeReader, PipeWriter, pipe};
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
        // mut prev_out: Option<Stdio> = None;
        let mut prev_reader: Option<PipeReader> = None;
        let mut children: Vec<Child> = Vec::with_capacity(commands.len());
        for (i, &cmd_tokens) in commands.iter().enumerate() {
            // parse command
            let mut cmd = match ShellCommand::parse(cmd_tokens) {
                Ok(cmd) => cmd,
                Err(e) => {
                    eprintln!("{:#}", e);
                    continue;
                }
            };

            // execute
            let _is_first = i == 0;
            let is_last = i == commands.len() - 1;
            let (cur_reader, mut cur_writer) = if commands.len() > 0 && !is_last {
                let (r, w) = pipe().unwrap();
                (Some(r), Some(w))
            } else {
                (None, None)
            };

            let result = if builtins::is_builtin(cmd.name) {
                builtins::run_builtin(&mut prev_reader, &mut cur_writer, &mut cmd, &ctx)
            } else {
                run_external(&mut prev_reader, &mut cur_writer, &cmd, &mut children)
            };
            prev_reader = cur_reader;

            // handle error and continue or exit
            match result {
                Ok(output) => {
                    if output.exit {
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
    prev_reader: &mut Option<PipeReader>,
    cur_writer: &mut Option<PipeWriter>,
    shell_cmd: &ShellCommand,
    children: &mut Vec<Child>,
) -> Result<ShellOutput> {
    match which(shell_cmd.name) {
        Ok(_) => {
            let mut command = Command::new(shell_cmd.name);
            command.args(&shell_cmd.args);

            // 输入配置
            if let Some(prev) = prev_reader.take() {
                command.stdin(Stdio::from(prev));
            } else {
                command.stdin(Stdio::inherit());
            }

            // 错误配置
            if !shell_cmd.err_redirects.is_empty() {
                for r in &shell_cmd.err_redirects {
                    let file = r.redirect_file()?;
                    command.stderr(Stdio::from(file));
                }
            } else {
                command.stderr(Stdio::inherit());
            }

            // 输出配置
            if !shell_cmd.out_redirects.is_empty() {
                // 重定向优先级高于管道
                for r in &shell_cmd.out_redirects {
                    let file = r.redirect_file()?;
                    command.stdout(Stdio::from(file));
                }
            } else if let Some(writer) = cur_writer.take() {
                command.stdout(Stdio::from(writer));
            } else {
                command.stdout(Stdio::inherit());
            }

            let child = command.spawn()?;
            children.push(child);
        }
        Err(_) => {
            let output =
                ShellOutput::new().error(format!("{}: command not found\n", shell_cmd.name));
            handle_main_error(&output, shell_cmd)?;
        }
    }

    Ok(ShellOutput::null())
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

/// shell主进程的output
fn handle_main_error(output: &ShellOutput, shell_cmd: &ShellCommand) -> Result<()> {
    // 错误
    if !shell_cmd.err_redirects.is_empty() {
        for r in &shell_cmd.err_redirects {
            r.handle_err(&output)?;
        }
    } else if !output.err.is_empty() {
        eprint!("{}", output.err);
    }
    Ok(())
}
