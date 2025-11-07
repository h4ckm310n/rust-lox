pub type Value = f64;

pub fn print_value(value: Value) {
    print!("{value}");
}

pub struct ValueArray {
    pub values: Vec<Value>
}

impl ValueArray {
    pub fn new() -> Self {
        Self {
            values: Vec::new()
        }
    }

    pub fn write(&mut self, value: Value) {
        self.values.push(value);
    }

    pub fn free(&mut self) {
        self.values.clear();
    }

    pub fn count(&self) -> usize {
        self.values.len()
    }
}