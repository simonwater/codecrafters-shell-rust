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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::with_capacity(32);
        io::stdin().read_line(&mut input).unwrap();
        let mut parts = input.trim().split_whitespace();
        let cmd = parts.next().unwrap();
        let mut args = parts;

        match cmd {
            "echo" => {
                let args = args.collect::<Vec<&str>>().join(" ");
                println!("{args}");
            }
            "type" => {
                let arg = args.next().unwrap();
                match my_which(arg) {
                    ProgramType::Builtin => println!("{arg} is a shell builtin"),
                    ProgramType::External(path) => println!("{} is {}", arg, path.display()),
                    _ => println!("{arg}: not found"),
                }
            }
            "pwd" => {
                println!("{}", std::env::current_dir()?.display());
            }
            "exit" => break,
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
    }
    Ok(())
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
        "echo" | "type" | "exit" => true,
        _ => false,
    }
}
