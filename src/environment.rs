use crate::CompleterHelper;
use rustyline::{Editor, history::FileHistory};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

type _RcEditor = Rc<RefCell<Editor<CompleterHelper, FileHistory>>>;
type _VarsCell = RefCell<HashMap<String, String>>;

pub struct Environment {
    editor: Editor<CompleterHelper, FileHistory>,
    vars: HashMap<String, String>,
}

impl Environment {
    pub fn new(editor: Editor<CompleterHelper, FileHistory>) -> Self {
        Environment {
            editor,
            vars: HashMap::with_capacity(32),
        }
    }

    pub fn get_editor(&self) -> &Editor<CompleterHelper, FileHistory> {
        &self.editor
    }

    pub fn get_editor_mut(&mut self) -> &mut Editor<CompleterHelper, FileHistory> {
        &mut self.editor
    }

    pub fn add_variable(&mut self, name: String, value: String) {
        self.vars.insert(name, value);
    }

    pub fn get_variable(&self, name: &str) -> Option<&String> {
        self.vars.get(name)
    }
}
