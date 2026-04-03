

///
/// Raw representation of command configuration
/// * options - raw local keys
/// * command - command
///
#[derive(Debug, Clone, PartialEq)]
pub struct Configuration {
    pub options: Vec<Vec<String>>,
    pub command: String,
}

impl Configuration {

    ///
    /// Make object
    ///
    pub fn new() -> Configuration {
        Self {
            options: Vec::new(),
            command: String::new(),
        }
    }

    ///
    /// Clearing object fields
    ///
    pub fn clear(&mut self) {
        self.options.clear();
        self.command.clear();
    }
}


///
/// Representation of key`s value
///
#[derive(Clone)]
pub enum Key {
    None,
    Basic,
    Value(String),
    Three((String, f64, String))
}

