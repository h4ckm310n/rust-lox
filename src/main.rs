use std::{collections::HashMap, env, fs, path::PathBuf};

use rust_lox::vm::VM;

fn main() {
    let args: Vec<String> = env::args().collect();
    let files = collect_files(PathBuf::from(args[1].clone()));
    for (path, content) in files {
        let mut vm = VM::init();
        vm.interpret(path.to_string_lossy().to_string(), content.clone());
    }
}

fn collect_files(path: PathBuf) -> HashMap<PathBuf, String> {
    let mut files = HashMap::new();
    if is_lox_file(path.clone()) {
        if let Ok(content) = fs::read_to_string(path.clone()) {
            files.insert(path.clone(), content);
        }
    }
    else if path.is_dir() {
        if let Ok(dir) = fs::read_dir(path) {
            for entry in dir {
                if let Ok(entry) = entry && is_lox_file(entry.path()) && let Ok(content) = fs::read_to_string(entry.path()) {
                    files.insert(entry.path(), content);
                }
            }
        }
    }
    files
}

fn is_lox_file(path: PathBuf) -> bool {
    path.is_file() && path.extension().is_some() && path.extension().unwrap() == "lox"
}