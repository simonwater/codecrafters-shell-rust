pub struct CommandResult {
    pub exit: bool,
    pub out: String,
    pub error: String,
}

impl CommandResult {
    pub fn new() -> Self {
        Self {
            exit: false,
            out: String::new(),
            error: String::new(),
        }
    }

    pub fn exit(mut self) -> Self {
        self.exit = true;
        self
    }

    pub fn success(mut self, out: String) -> Self {
        self.out = out;
        self
    }

    pub fn error(mut self, err: String) -> Self {
        self.error = err;
        self
    }
}
