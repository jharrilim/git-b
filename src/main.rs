use clap::Parser;
use skim::prelude::*;
use std::process::Command;

mod cli;

fn main() {
    let args = cli::Args::parse();

    let branches = branch_names();

    let options = match args.branch {
        Some(branch) => SkimOptionsBuilder::default()
            .multi(false)
            .query(branch)
            .no_sort(true)
            .build()
            .unwrap(),
        None => SkimOptionsBuilder::default()
            .multi(false)
            .no_sort(true)
            .build()
            .unwrap(),
    };

    match Skim::run_items(options, branches) {
        Ok(output) => {
            if output.is_abort {
                return;
            }
            // This indexer should be safe, as they should have always had picked
            // one option at this point.
            let selected_item = &output.selected_items[0];
            let line = selected_item.output();
            let branch = line.split_whitespace().next();
            match branch {
                Some(branch) => {
                    git_checkout(branch.to_string());
                }
                None => {
                    println!("Branch name invalid, somehow. Report this:\n'{}'", line);
                }
            }
        }
        Err(e) => eprintln!("skim error: {e}"),
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
        .arg("-v")
        .arg("--sort=-committerdate")
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
