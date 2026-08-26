use std::{cell::RefCell, collections::HashMap};

pub struct Environment {
    complete_regs: RefCell<HashMap<String, String>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            complete_regs: RefCell::new(HashMap::with_capacity(32)),
        }
    }

    pub fn reg_complete(&self, cmd: String, content: String) {
        self.complete_regs.borrow_mut().insert(cmd, content);
    }

    pub fn get_complete_reg(&self, cmd: &str) -> Option<String> {
        self.complete_regs.borrow().get(cmd).cloned()
    }
}
