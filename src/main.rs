use skim::prelude::*;
use std::{io::BufReader, process::Command};

fn main() {
    if std::env::args().any(|arg| arg == "--version") {
        println!("git-b {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    let branches = branch_names().join("\n");
    let b = Box::leak(branches.into_boxed_str());
    let reader = BufReader::new(b.as_bytes());

    let options = SkimOptionsBuilder::default().multi(false).build().unwrap();

    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(reader);

    match Skim::run_with(&options, Some(items)) {
        Some(output) => {
            if output.is_abort {
                return;
            }
            let selected_item = &output.selected_items[0];
            let branch = selected_item.output();
            git_checkout(branch.to_string());
        }
        None => {}
    }
}

fn git_checkout(branch: String) {
    let output = Command::new("git")
        .arg("checkout")
        .arg(branch)
        .output()
        .expect("failed to execute process");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    println!("{}", String::from_utf8_lossy(&output.stderr));
}

fn branch_names() -> Vec<String> {
    let mut branch_names = Vec::new();
    let output = Command::new("git")
        .arg("branch")
        .output()
        .expect("failed to execute process");
    let output = String::from_utf8(output.stdout).unwrap();
    for line in output.lines() {
        let line = line.trim();
        if line.starts_with('*') {
            continue;
        }
        branch_names.push(line.to_string());
    }
    branch_names
}
