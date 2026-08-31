use crate::redirect::{self, Redirection};
use anyhow::Result;
use std::path::PathBuf;

pub enum CommandType {
    Builtin,
    External(PathBuf),
    Unknown,
}

pub struct ShellOutput {
    pub exit: bool,
    pub out: String,
    pub err: String,
}

impl ShellOutput {
    pub fn new() -> Self {
        Self {
            exit: false,
            out: String::new(),
            err: String::new(),
        }
    }

    pub fn null() -> Self {
        Self {
            exit: false,
            out: String::new(),
            err: String::new(),
        }
    }

    pub fn exit(mut self, val: bool) -> Self {
        self.exit = val;
        self
    }

    pub fn success(mut self, out: String) -> Self {
        self.out = out;
        self
    }

    pub fn error(mut self, err: String) -> Self {
        self.err = err;
        self
    }
}

pub struct ShellCommand<'a> {
    pub name: &'a str,
    pub args: Vec<&'a str>,
    pub out_redirects: Vec<Redirection>,
    pub err_redirects: Vec<Redirection>,
}

impl<'a> ShellCommand<'a> {
    pub fn parse(cmd_tokens: &'a [String]) -> Result<Self> {
        let name = cmd_tokens[0].as_str();
        let (args, out_redirects, err_redirects) = redirect::parse_redirections(&cmd_tokens[1..])?;
        let command = Self {
            name,
            args,
            out_redirects,
            err_redirects,
        };
        Ok(command)
    }
}
