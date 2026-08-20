#[allow(unused_imports)]
use std::io::{self, Write};
use std::process::Command;

fn main() {
    print!("$ ");
    io::stdout().flush().unwrap();

    let mut input = String::with_capacity(32);
    io::stdin().read_line(&mut input).unwrap();
    let cmd = input.trim();

    Command::new(cmd).spawn().unwrap();
}
