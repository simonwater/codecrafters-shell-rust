use anyhow::{Result, bail};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use crate::CommandResult;

pub enum RedirectType {
    StdoutTruncate, // > 或 1> (覆盖标准输出)
    StderrTruncate, // 2> (覆盖标准错误)
}

pub struct Redirection {
    pub rtype: RedirectType,
    pub file_path: PathBuf,
}

impl Redirection {
    pub fn handle(
        &self,
        cmd_res: &CommandResult,
        out_handled: &mut bool,
        err_handled: &mut bool,
    ) -> Result<()> {
        match self.rtype {
            RedirectType::StdoutTruncate => {
                if !cmd_res.out.is_empty() {
                    let mut f = File::create(&self.file_path)?;
                    f.write_all(cmd_res.out.as_bytes())?;
                    *out_handled = true;
                }
            }
            RedirectType::StderrTruncate => {
                let mut f = File::create(&self.file_path)?;
                if !cmd_res.error.is_empty() {
                    f.write_all(cmd_res.error.as_bytes())?;
                    *err_handled = true;
                }
            }
        }
        Ok(())
    }
}

pub fn parse_redirections(tokens: &[String]) -> Result<(Vec<&str>, Vec<Redirection>)> {
    let mut redirects = Vec::new();
    let mut args: Vec<&str> = Vec::with_capacity(tokens.len());
    let mut iter = tokens.iter();
    while let Some(token) = iter.next() {
        let rtype = match token.as_str() {
            ">" | "1>" => Some(RedirectType::StdoutTruncate),
            "2>" => Some(RedirectType::StderrTruncate),
            _ => None,
        };

        if let Some(t) = rtype {
            if let Some(file) = iter.next() {
                let redirection = Redirection {
                    rtype: t,
                    file_path: PathBuf::from(file),
                };
                redirects.push(redirection);
            } else {
                bail!("parse error: miss redirect file name!");
            }
        } else {
            args.push(token);
        }
    }

    Ok((args, redirects))
}
