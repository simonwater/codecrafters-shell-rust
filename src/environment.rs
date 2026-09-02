use crate::CompleterHelper;
use rustyline::{Editor, history::FileHistory};
use std::cell::RefCell;
use std::rc::Rc;

type RcEditor = Rc<RefCell<Editor<CompleterHelper, FileHistory>>>;

pub struct Environment {
    editor: RcEditor,
}

impl Environment {
    pub fn new(editor: RcEditor) -> Self {
        Environment { editor }
    }

    pub fn get_editor_ref(&self) -> &RcEditor {
        &self.editor
    }

    pub fn get_editor_mut(&mut self) -> &mut RcEditor {
        &mut self.editor
    }
}
