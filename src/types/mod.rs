#[derive(Debug, Clone, PartialEq)]
pub struct Configuration {
    pub options: Vec<Vec<String>>,
    pub command: String,
}

impl Configuration {
    pub fn new() -> Configuration {
        Self {
            options: Vec::new(),
            command: String::new(),
        }
    }
    pub fn clear(&mut self) {
        self.options.clear();
        self.command.clear();
    }
}

#[derive(Clone)]
pub enum GeneralOption<T> where T: Clone {
    None,
    Some(T),
}


#[derive(Clone)]
pub enum TimeOrdering {
    LT,
    GT,
    GE,
    LE,
    EQ,
    NE
}