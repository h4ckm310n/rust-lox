use std::{collections::HashMap, env, path::PathBuf};
use rust_lox::project::Project;


fn main() {
    let args: Vec<String> = env::args().collect();
    let mut project = Project::new(PathBuf::from(args[1].clone()));
    project.collect_files();
    project.compile();
}