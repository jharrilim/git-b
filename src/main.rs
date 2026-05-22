use algo::BranchItem;
use cli::{Args, Parser};
use display::{load, ColorOverrides};
use skim::prelude::*;
use std::process::Command;

fn main() {
    let args = Args::parse();

    let colors = load(&ColorOverrides {
        disabled: args.no_color.then_some(true),
        triple: args.color.clone(),
        ..Default::default()
    });

    if let Some(name) = args.new_branch {
        git_checkout_new(name);
        return;
    }

    let branches = match parse::list_branches() {
        Ok(branches) => branches,
        Err(e) => {
            eprintln!("git-b: {e}");
            std::process::exit(1);
        }
    };

    if let Some(query) = args.branch {
        match algo::find_first_matching(&query, &branches) {
            Some(branch) => git_checkout(branch.name.clone()),
            None => {
                eprintln!("no branch matching '{query}'");
                std::process::exit(1);
            }
        }
        return;
    }

    let items: Vec<BranchItem> = branches
        .into_iter()
        .map(|b| BranchItem::new(b, colors))
        .collect();

    let options = SkimOptionsBuilder::default()
        .multi(false)
        .no_sort(true)
        .build()
        .unwrap();

    match Skim::run_items(options, items) {
        Ok(output) => {
            if output.is_abort {
                return;
            }
            let selected_item = &output.selected_items[0];
            git_checkout(selected_item.output().to_string());
        }
        Err(e) => eprintln!("skim error: {e}"),
    }
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
