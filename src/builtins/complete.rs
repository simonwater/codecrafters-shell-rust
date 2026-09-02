use crate::command::ShellOutput;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

pub static COMPS_MANAGER: LazyLock<Mutex<CompleteRepo>> =
    LazyLock::new(|| Mutex::new(CompleteRepo::new()));

pub struct CompleteRepo {
    complete_regs: HashMap<String, String>,
}

impl CompleteRepo {
    fn new() -> Self {
        CompleteRepo {
            complete_regs: HashMap::with_capacity(32),
        }
    }

    pub fn reg_complete(&mut self, cmd: String, content: String) {
        self.complete_regs.insert(cmd, content);
    }

    pub fn remove_complete_reg(&mut self, cmd: &str) {
        self.complete_regs.remove(cmd);
    }

    pub fn get_complete_reg(&self, cmd: &str) -> Option<String> {
        self.complete_regs.get(cmd).cloned()
    }
}

pub fn complete(args: &[&str]) -> Result<ShellOutput> {
    let mut iter = args.iter();
    if let Some(&first) = iter.next() {
        match first {
            "-p" => {
                if let Some(&cmd) = iter.next() {
                    let res = if let Some(content) =
                        COMPS_MANAGER.lock().unwrap().get_complete_reg(cmd)
                    {
                        let msg = format!("complete -C '{}' {}\n", content, cmd);
                        ShellOutput::new().success(msg)
                    } else {
                        ShellOutput::new()
                            .error(format!("complete: {}: no completion specification\n", cmd))
                    };
                    return Ok(res);
                }
            }
            "-r" => {
                if let Some(&cmd) = iter.next() {
                    COMPS_MANAGER.lock().unwrap().remove_complete_reg(cmd);
                }
            }
            "-C" => {
                if let (Some(&content), Some(&cmd)) = (iter.next(), iter.next()) {
                    COMPS_MANAGER
                        .lock()
                        .unwrap()
                        .reg_complete(cmd.to_string(), content.to_string());
                }
            }
            _ => {}
        }
    }
    Ok(ShellOutput::null())
}
