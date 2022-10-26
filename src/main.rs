use clap::Parser;
use skim::prelude::*;
use std::{io::BufReader, process::Command};

mod cli;

fn main() {
    let args = cli::Args::parse();

    let branches = branch_names().join("\n");

    // SkimItemReader#of_bufread requires a 'static lifetime on this.
    // Leaking it prevents it from being destroyed and thus makes it 'static.
    let b = Box::leak(branches.into_boxed_str());
    let reader = BufReader::new(b.as_bytes());

    let options = match args.branch {
        Some(branch) => {
            let branch = Box::leak(branch.into_boxed_str());

            SkimOptionsBuilder::default()
                .multi(false)
                .query(Some(branch))
                .nosort(true)
                .build()
                .unwrap()
        }
        None => SkimOptionsBuilder::default().multi(false).nosort(true).build().unwrap(),
    };

    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(reader);

    match Skim::run_with(&options, Some(items)) {
        Some(output) => {
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
