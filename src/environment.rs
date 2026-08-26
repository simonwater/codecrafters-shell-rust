use std::collections::HashMap;

pub struct Environment {
    complete_regs: HashMap<String, String>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            complete_regs: HashMap::with_capacity(32),
        }
    }

    pub fn reg_complete(&mut self, cmd: String, content: String) {
        self.complete_regs.insert(cmd, content);
    }

    pub fn get_complete_reg(&self, cmd: &str) -> Option<&String> {
        self.complete_regs.get(cmd)
    }
}
