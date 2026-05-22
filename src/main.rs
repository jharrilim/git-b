use cli::{Args, Parser};
use skim::prelude::*;
use std::process::Command;

fn main() {
    let args = Args::parse();

    if let Some(name) = args.new_branch {
        git_checkout_new(name);
        return;
    }

    let branches = branch_names();

    if let Some(query) = args.branch {
        match find_first_matching_branch(&branches, &query) {
            Some(branch) => git_checkout(branch),
            None => {
                eprintln!("no branch matching '{query}'");
                std::process::exit(1);
            }
        }
        return;
    }

    let options = SkimOptionsBuilder::default()
        .multi(false)
        .no_sort(true)
        .build()
        .unwrap();

    match Skim::run_items(options, branches) {
        Ok(output) => {
            if output.is_abort {
                return;
            }
            let selected_item = &output.selected_items[0];
            let line = selected_item.output();
            match branch_name_from_line(&line) {
                Some(branch) => git_checkout(branch),
                None => {
                    println!("Branch name invalid, somehow. Report this:\n'{line}'");
                }
            }
        }
        Err(e) => eprintln!("skim error: {e}"),
    }
}

fn find_first_matching_branch(branches: &[String], query: &str) -> Option<String> {
    let factory = ExactOrFuzzyEngineFactory::builder().build();
    let engine = factory.create_engine(query);

    for line in branches {
        if engine.match_item(line).is_some() {
            return branch_name_from_line(line);
        }
    }
    None
}

fn branch_name_from_line(line: &str) -> Option<String> {
    line.split_whitespace().next().map(str::to_string)
}

fn git_checkout(branch: String) {
    let output = Command::new("git")
        .arg("checkout")
        .arg(&branch)
        .output()
        .expect("failed to execute process");
    print_git_output(&output);
}

fn git_checkout_new(branch: String) {
    let output = Command::new("git")
        .arg("checkout")
        .arg("-b")
        .arg(&branch)
        .output()
        .expect("failed to execute process");
    print_git_output(&output);
}

fn print_git_output(output: &std::process::Output) {
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
