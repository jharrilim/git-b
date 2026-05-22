//! Load and parse git branch metadata.

use std::io;
use std::process::Command;

/// A local branch with its latest commit metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Branch {
    pub name: String,
    pub short_hash: String,
    pub subject: String,
}

impl Branch {
    /// Format like `git branch -v` for display in the picker.
    pub fn display_line(&self) -> String {
        format!("{} {} {}", self.name, self.short_hash, self.subject)
    }
}

/// Parse a tab-separated `for-each-ref` line: `name\thash\tsubject`.
pub fn parse_line(line: &str) -> Option<Branch> {
    let line = line.trim_end();
    if line.is_empty() {
        return None;
    }

    let (name, rest) = line.split_once('\t')?;
    let (short_hash, subject) = rest.split_once('\t')?;

    Some(Branch {
        name: name.to_string(),
        short_hash: short_hash.to_string(),
        subject: subject.to_string(),
    })
}

/// List local branches sorted by committer date (newest first), excluding the current branch.
pub fn list_branches() -> io::Result<Vec<Branch>> {
    let current = current_branch()?;

    let output = Command::new("git")
        .args([
            "for-each-ref",
            "--format=%(refname:short)\t%(objectname:short)\t%(subject)",
            "refs/heads/",
            "--sort=-committerdate",
        ])
        .output()?;

    if !output.status.success() {
        return Err(io::Error::other(format!(
            "git for-each-ref failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let stdout = String::from_utf8(output.stdout).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let mut branches = Vec::new();
    for line in stdout.lines() {
        let Some(branch) = parse_line(line) else {
            continue;
        };
        if current.as_deref() == Some(branch.name.as_str()) {
            continue;
        }
        branches.push(branch);
    }

    Ok(branches)
}

fn current_branch() -> io::Result<Option<String>> {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .output()?;

    if !output.status.success() {
        return Err(io::Error::other(format!(
            "git branch --show-current failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let name = String::from_utf8(output.stdout).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let name = name.trim();
    if name.is_empty() {
        Ok(None)
    } else {
        Ok(Some(name.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_line() {
        let branch = parse_line("main\tabc1234\tInitial commit").unwrap();
        assert_eq!(branch.name, "main");
        assert_eq!(branch.short_hash, "abc1234");
        assert_eq!(branch.subject, "Initial commit");
    }

    #[test]
    fn parse_subject_with_tabs() {
        let branch = parse_line("feature/foo\tdeadbeef\tfix\tthing\tnow").unwrap();
        assert_eq!(branch.name, "feature/foo");
        assert_eq!(branch.short_hash, "deadbeef");
        assert_eq!(branch.subject, "fix\tthing\tnow");
    }

    #[test]
    fn parse_subject_with_spaces() {
        let branch = parse_line("release/1.0\t1111111\tmerge pull request #42").unwrap();
        assert_eq!(branch.subject, "merge pull request #42");
    }

    #[test]
    fn display_line_matches_branch_v_style() {
        let branch = Branch {
            name: "main".into(),
            short_hash: "abc1234".into(),
            subject: "Initial commit".into(),
        };
        assert_eq!(branch.display_line(), "main abc1234 Initial commit");
    }

    #[test]
    fn parse_empty_line_returns_none() {
        assert!(parse_line("").is_none());
        assert!(parse_line("   ").is_none());
    }

    #[test]
    fn parse_malformed_line_returns_none() {
        assert!(parse_line("only-name").is_none());
        assert!(parse_line("name\thash-only").is_none());
    }
}
