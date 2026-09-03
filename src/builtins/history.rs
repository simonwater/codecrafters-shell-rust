use std::path::Path;

use crate::command::ShellOutput;
use crate::environment::Environment;
use anyhow::Result;
use rustyline::history::{FileHistory, History};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};

pub(crate) fn history(args: &[&str], environment: &mut Environment) -> Result<ShellOutput> {
    let editor = environment.get_editor_mut();
    let history = editor.history_mut();
    let mut iter = args.iter();
    if let Some(&arg1) = iter.next() {
        let output = match arg1 {
            "-r" => {
                if let Some(&arg2) = iter.next() {
                    let path = Path::new(arg2);
                    history.load(path)?;
                }
                ShellOutput::null()
            }
            "-w" => {
                if let Some(&arg2) = iter.next() {
                    save_all_history(history, arg2)?;
                }
                ShellOutput::null()
            }
            "-a" => {
                if let Some(&arg2) = iter.next() {
                    let path = Path::new(arg2);
                    history.append(path)?;
                    // 兼容测试机直接读文件进行验证的情况
                    strip_v2_header_if_exists(path)?;
                }
                ShellOutput::null()
            }
            arg1 => {
                let len = history.len();
                let recent = str::parse::<usize>(arg1).unwrap_or(len).min(len);
                get_recent_historys(history, recent)
            }
        };
        return Ok(output);
    } else {
        let output = get_recent_historys(history, history.len());
        return Ok(output);
    }
}

fn get_recent_historys(history: &FileHistory, recent: usize) -> ShellOutput {
    let mut recs = String::with_capacity(128);
    let len = history.len();
    let skip_cnt = len - recent;
    let iter = history.iter().skip(skip_cnt);
    for (i, rec) in iter.enumerate() {
        let line = format!("  {} {}\n", i + 1 + skip_cnt, rec);
        recs.push_str(&line);
    }
    return ShellOutput::new().success(recs);
}

/// 检查并移除历史文件第一行的 :#V2 Header
fn strip_v2_header_if_exists<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(());
    }

    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut first_line = String::new();

    // 1. 读取第一行
    if reader.read_line(&mut first_line)? > 0 {
        let trimmed = first_line.trim_start_matches('\u{feff}').trim(); // 兼容 UTF-8 BOM

        // 判断是否包含 Rustyline 的 V2 Header
        if trimmed.starts_with(":#V2") || trimmed.starts_with("#V2") {
            // 2. 将后续的所有纯文本内容全部读取到内存中
            let mut remainder = Vec::new();
            reader.read_to_end(&mut remainder)?;

            // 3. 覆写原文件（去除 Header）
            let mut file = OpenOptions::new().write(true).truncate(true).open(path)?;
            file.write_all(&remainder)?;
            file.flush()?;
        }
    }

    Ok(())
}

pub fn save_all_history<P: AsRef<Path>>(history: &FileHistory, path: P) -> Result<()> {
    let contents = history
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<&str>>()
        .join("\n")
        + "\n";
    fs::write(path, contents)?;
    Ok(())
}
