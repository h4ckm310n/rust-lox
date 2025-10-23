use std::{collections::HashMap, path::PathBuf, fs};
use walkdir::WalkDir;
use crate::{parser::Parser, scanner::Scanner};

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
            let mut scanner = Scanner::new(path.to_string_lossy().to_string(), content.clone());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(path.to_string_lossy().to_string(), tokens);
            parser.parse();
        }
    }
}