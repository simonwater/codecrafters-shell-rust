#[allow(unused_imports)]
use std::io::{self, Write};
use which::which;

fn main() {
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
                eprintln!("{args}");
            }
            "type" => {
                let arg = args.next().unwrap();
                if is_builtin(arg) {
                    eprintln!("{arg} is a shell builtin");
                } else if let Ok(path) = which(arg) {
                    eprintln!("{} is {}", arg, path.display());
                } else {
                    eprintln!("{arg}: not found");
                }
            }
            "exit" => break,
            _ => eprintln!("{cmd}: command not found"),
        }
    }
}

fn is_builtin(s: &str) -> bool {
    match s {
        "echo" | "type" | "exit" => true,
        _ => false,
    }
}
