use anyhow::{Result, bail};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use crate::ShellOutput;

type OutRedirects = Vec<Redirection>;
type ErrRedirects = Vec<Redirection>;

#[derive(Clone, Copy)]
pub enum RedirectType {
    StdoutTruncate, // > 或 1> (覆盖标准输出)
    StdoutAppend,   // >> 或 1>> (追加标准输出)
    StderrTruncate, // 2> (覆盖标准错误)
    StderrAppend,   // 2>> (追加标准错误)
}

pub struct Redirection {
    pub rtype: RedirectType,
    pub file_path: PathBuf,
}

impl Redirection {
    pub fn redirect_file(&self) -> Result<File> {
        let file = match self.rtype {
            RedirectType::StdoutTruncate | RedirectType::StderrTruncate => {
                File::create(&self.file_path)?
            }
            RedirectType::StdoutAppend | RedirectType::StderrAppend => File::options()
                .write(true)
                .create(true)
                .append(true)
                .open(&self.file_path)?,
        };
        Ok(file)
    }

    pub fn handle_out(&self, output: &ShellOutput) -> Result<()> {
        match self.rtype {
            RedirectType::StdoutTruncate | RedirectType::StdoutAppend => {
                let mut f = self.redirect_file()?; // 无论out是否存在都创建文件
                if !output.out.is_empty() {
                    f.write_all(output.out.as_bytes())?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn handle_err(&self, output: &ShellOutput) -> Result<()> {
        match self.rtype {
            RedirectType::StderrTruncate | RedirectType::StderrAppend => {
                let mut f = self.redirect_file()?; // 无论err是否存在都创建文件
                if !output.err.is_empty() {
                    f.write_all(output.err.as_bytes())?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}

pub fn parse_redirections(tokens: &[String]) -> Result<(Vec<&str>, OutRedirects, ErrRedirects)> {
    let mut out_redirects = Vec::new();
    let mut err_redirects = Vec::new();
    let mut args: Vec<&str> = Vec::with_capacity(tokens.len());
    let mut iter = tokens.iter();
    while let Some(token) = iter.next() {
        let rtype = match token.as_str() {
            ">" | "1>" => Some(RedirectType::StdoutTruncate),
            ">>" | "1>>" => Some(RedirectType::StdoutAppend),
            "2>" => Some(RedirectType::StderrTruncate),
            "2>>" => Some(RedirectType::StderrAppend),
            _ => None,
        };

        if let Some(t) = rtype {
            if let Some(file) = iter.next() {
                let redirect = Redirection {
                    rtype: t,
                    file_path: PathBuf::from(file),
                };
                match t {
                    RedirectType::StdoutAppend | RedirectType::StdoutTruncate => {
                        out_redirects.push(redirect)
                    }
                    RedirectType::StderrAppend | RedirectType::StderrTruncate => {
                        err_redirects.push(redirect)
                    }
                };
            } else {
                bail!("parse error: miss redirect file name!");
            }
        } else {
            args.push(token);
        }
    }

    Ok((args, out_redirects, err_redirects))
}
