use crate::scanner::Scanner;

pub fn compile(file_path: String, source: String) {
    let mut scanner = Scanner::init(file_path.clone(), source.clone());
    scanner.scan();
}