use rustyline::completion::{Completer, FilenameCompleter, Pair};

use rustyline::highlight::{CmdKind, Highlighter};
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};
use std::borrow::Cow;

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

        // 如果没有匹配到自定义命令，或者包含路径分隔符，尝试回退到文件路径补全
        if candidates.is_empty() || input.contains('/') || input.contains('\\') {
            if let Ok((start_pos, file_candidates)) = self.file_completer.complete(line, pos, ctx) {
                let canditates = file_candidates
                    .into_iter()
                    .map(|mut pair| {
                        if !pair.replacement.ends_with('/') && !pair.replacement.ends_with('\\') {
                            pair.replacement.push(' ');
                        }
                        pair
                    })
                    .collect();
                return Ok((start_pos, canditates));
            }
        }

        Ok((0, candidates))
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
