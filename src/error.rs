pub trait ErrorReporter {
    fn error(&mut self, start: usize, end: usize, error_content: String);
}