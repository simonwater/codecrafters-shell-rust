use anyhow::Result;
use rustyline::completion::{Completer, FilenameCompleter, Pair, longest_common_prefix};
use rustyline::error::ReadlineError;
use rustyline::highlight::{CmdKind, Highlighter};
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};
use std::borrow::Cow;
use std::io;
use std::process::Command;

use crate::COMPS_MANAGER;

pub struct CompleterHelper {
    // 内置的文件路径补全器
    pub file_completer: FilenameCompleter,
    // 自定义的关键字命令列表
    pub commands: Vec<String>,
}

/// 2. 为 Helper 实现 Completer 特质以支持 Tab 补全
impl Completer for CompleterHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let (start_pos, canditates) = self.candidates(line, pos, ctx)?;
        let prefix = &line[start_pos..pos];
        if let Some(lcp) = longest_common_prefix(&canditates) {
            if lcp.len() > prefix.len() {
                let candidates = vec![Pair {
                    display: lcp.to_string(),
                    replacement: lcp.to_string(),
                }];
                return Ok((start_pos, candidates));
            }
        }
        Ok((start_pos, canditates))
    }
}

impl CompleterHelper {
    fn candidates(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let mut candidates = Vec::new();
        let input = &line[..pos];
        for cmd in &self.commands {
            if cmd.starts_with(input) {
                candidates.push(Pair {
                    display: cmd.clone(),
                    replacement: format!("{} ", cmd),
                });
            }
        }

        // 注册的脚本
        if candidates.is_empty() {
            let (start_pos, prog_candidates) = self
                .programmable_complete(line, pos)
                .map_err(|e| ReadlineError::Io(io::Error::other(e)))?;
            if !prog_candidates.is_empty() {
                return Ok((start_pos, prog_candidates));
            }
        }

        // 如果没有匹配到自定义命令，或者包含路径分隔符，尝试回退到文件路径补全
        if candidates.is_empty() || input.contains('/') || input.contains('\\') {
            if let Ok((start_pos, file_candidates)) = self.file_completer.complete(line, pos, ctx) {
                let canditates = file_candidates
                    .into_iter()
                    .map(|mut pair| {
                        if !pair.replacement.ends_with('/') && !pair.replacement.ends_with('\\') {
                            // 文件
                            pair.replacement.push(' ');
                        } else {
                            // 目录
                            pair.display.push('/');
                        }
                        pair
                    })
                    .collect();
                return Ok((start_pos, canditates));
            }
        }

        Ok((0, candidates))
    }

    fn programmable_complete(&self, line: &str, pos: usize) -> Result<(usize, Vec<Pair>)> {
        let empty = String::new();
        let input = &line[..pos];
        let parts = shell_words::split(input).unwrap_or_else(|_| vec![input.to_string()]);
        let mut iter = parts.iter();
        let cmd = &parts[0];
        if parts.len() == 1 {
            // 仅有命令自身一个词时直接消耗掉，不让后面的last取到
            // 超过一个词时，命令自身可能会被赋给last_second，需要保留
            iter.next();
        }
        if let Some(script_path) = COMPS_MANAGER.lock().unwrap().get_complete_reg(cmd) {
            let mut iter = iter.rev();
            let (last, last_second) =
                (iter.next().unwrap_or(&empty), iter.next().unwrap_or(&empty));

            let output = Command::new(&script_path)
                .args(vec![cmd, last, last_second])
                .env("COMP_LINE", line)
                .env("COMP_POINT", pos.to_string())
                .output()?;
            let out = String::from_utf8(output.stdout)?;
            let _err = String::from_utf8(output.stderr)?;

            if !out.is_empty() {
                let candidates = out
                    .lines()
                    .filter(|&line| !line.is_empty() && line.starts_with(last))
                    .map(|line| Pair {
                        display: format!("{}", line.trim()),
                        replacement: format!("{} ", line.trim()),
                    })
                    .collect();
                return Ok((input.len() - last.len(), candidates));
            }
        }

        Ok((0, vec![]))
    }
}

impl Hinter for CompleterHelper {
    type Hint = String;

    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<Self::Hint> {
        None
    }
}

impl Highlighter for CompleterHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        Cow::Borrowed(line)
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _kind: CmdKind) -> bool {
        false
    }
}

impl Validator for CompleterHelper {}

impl Helper for CompleterHelper {}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn t() {
        let output = Command::new("./test").output().unwrap();
        let out = String::from_utf8(output.stdout).unwrap();
        let err = String::from_utf8(output.stderr).unwrap();

        for (i, line) in out.lines().enumerate() {
            println!("out line{}: {}", i, line);
        }
        eprintln!("err: {}", err);
    }
}
