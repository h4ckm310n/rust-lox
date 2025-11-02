use std::{cell::RefCell, collections::HashMap, fs, path::PathBuf, rc::Rc};
use walkdir::WalkDir;
use crate::{interpreter::Interpreter, parser::Parser, resolver::Resolver, scanner::Scanner};

pub struct Project {
    pub path: PathBuf,
    pub files: HashMap<PathBuf, String>
}

impl Project {
    pub fn new(path: PathBuf) -> Self {
        Self { 
            path: path, 
            files: HashMap::new() 
        }
    }

    pub fn collect_files(&mut self) {
        if self.is_lox_file(self.path.clone()) {
            // single file
            if let Ok(content) = fs::read_to_string(self.path.clone()) {
                self.files.insert(self.path.clone(), content);
            }
        }
        else if self.path.is_dir() {
            // dir
            for entry in WalkDir::new(self.path.clone()) {
                if entry.is_err() {
                    continue;
                }
                let path = entry.unwrap().into_path();
                if self.is_lox_file(path.clone()) && let Ok(content) = fs::read_to_string(path.clone()) {
                    self.files.insert(path, content);
                }
            }
        }
    }

    fn is_lox_file(&self, path: PathBuf) -> bool {
        path.is_file() && path.extension().is_some() && path.extension().unwrap() == "lox"
    }

    pub fn compile(&mut self) {
        for (path, content) in &self.files {
            //println!("file: {}", path.to_string_lossy());
            let mut scanner = Scanner::new(path.to_string_lossy().to_string(), content.clone());
            let tokens = scanner.scan_tokens();
            if *scanner.had_error.borrow() {
                continue;
            }
            let parser = Parser::new(path.to_string_lossy().to_string(), tokens);
            let stmts = parser.parse();
            if *parser.had_error.borrow() {
                continue;
            }
            let interpreter = Rc::new(RefCell::new(Interpreter::new()));
            let mut resolver = Resolver::new(interpreter.clone());
            resolver.resolve(&stmts);
            if *resolver.had_error.borrow() {
                continue;
            }
            interpreter.borrow_mut().interpret(&stmts);
        }
    }
}