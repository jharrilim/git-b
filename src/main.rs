use algo::BranchItem;
use cli::{Args, Parser};
use display::{load, ColorOverrides, DisplayLayout};
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

    if let Some(branch) = args.branch.as_deref() {
        if branch == "-" {
            git_checkout_ago(1);
            return;
        }
        if let Some(n) = parse_checkout_ago(branch) {
            git_checkout_ago(n);
            return;
        }
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

    let layout = DisplayLayout::from_branches(&branches);
    let items: Vec<BranchItem> = branches
        .into_iter()
        .map(|b| BranchItem::new(b, layout, colors))
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

/// Parse `~N` (e.g. `~1`, `~2`) as the Nth previously checked-out branch.
fn parse_checkout_ago(s: &str) -> Option<u32> {
    let n: u32 = s.strip_prefix('~')?.parse().ok()?;
    (n >= 1).then_some(n)
}

fn git_checkout_ago(n: u32) {
    let spec = format!("@{{-{n}}}");
    let output = Command::new("git")
        .arg("checkout")
        .arg(&spec)
        .output()
        .expect("failed to execute process");
    print_git_output(&output);
    if !output.status.success() {
        std::process::exit(output.status.code().unwrap_or(1));
    }
}

fn print_git_output(output: &std::process::Output) {
    println!("{}", String::from_utf8_lossy(&output.stdout));
    println!("{}", String::from_utf8_lossy(&output.stderr));
}

#[cfg(test)]
mod tests {
    use super::parse_checkout_ago;

    #[test]
    fn parse_checkout_ago_valid() {
        assert_eq!(parse_checkout_ago("~1"), Some(1));
        assert_eq!(parse_checkout_ago("~2"), Some(2));
        assert_eq!(parse_checkout_ago("~10"), Some(10));
    }

    #[test]
    fn parse_checkout_ago_invalid() {
        assert_eq!(parse_checkout_ago("~0"), None);
        assert_eq!(parse_checkout_ago("~"), None);
        assert_eq!(parse_checkout_ago("~x"), None);
        assert_eq!(parse_checkout_ago("feature"), None);
        assert_eq!(parse_checkout_ago("-"), None);
    }
}
