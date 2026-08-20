#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    print!("$ ");
    io::stdout().flush().unwrap();

    let mut input = String::with_capacity(32);
    io::stdin().read_line(&mut input).unwrap();
    let cmd = input.trim();
    println!("{cmd}: command not found")
}
