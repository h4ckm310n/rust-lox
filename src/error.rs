pub trait ErrorReporter {
    fn error(&self, start: usize, end: usize, error_content: String);
}