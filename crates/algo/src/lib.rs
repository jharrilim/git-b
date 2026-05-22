//! Weighted fuzzy matching for branches (name preferred over commit subject).

use std::borrow::Cow;

use parse::Branch;
use skim::prelude::*;
use skim::{MatchEngine, MatchEngineFactory, SkimItem};

/// Multiply name-field fuzzy scores so name matches outrank subject-only matches.
pub const NAME_WEIGHT: i32 = 1_000;

/// Bonus when the query fully matches the branch name (case-insensitive).
pub const EXACT_NAME_BONUS: i32 = 100_000;

/// Bonus when the branch name starts with the query (case-insensitive).
pub const PREFIX_NAME_BONUS: i32 = 50_000;

/// Wrapper for passing branches into Skim.
#[derive(Debug, Clone)]
pub struct BranchItem {
    branch: Branch,
    display: String,
    matching_ranges: [(usize, usize); 2],
}

impl BranchItem {
    pub fn new(branch: Branch) -> Self {
        let display = branch.display_line();
        let name_end = branch.name.len();
        let subject_start = name_end + 1 + branch.short_hash.len() + 1;
        let subject_end = display.len();
        Self {
            branch,
            display,
            matching_ranges: [(0, name_end), (subject_start, subject_end)],
        }
    }

    pub fn branch(&self) -> &Branch {
        &self.branch
    }
}

impl From<Branch> for BranchItem {
    fn from(branch: Branch) -> Self {
        Self::new(branch)
    }
}

impl SkimItem for BranchItem {
    fn text(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.display)
    }

    fn output(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.branch.name)
    }

    fn get_matching_ranges(&self) -> Option<&[(usize, usize)]> {
        Some(&self.matching_ranges)
    }
}

struct Scored<'a> {
    branch: &'a Branch,
    effective: i32,
}

fn name_bonuses(query: &str, name: &str) -> i32 {
    let q = query.to_lowercase();
    let n = name.to_lowercase();
    if q == n {
        EXACT_NAME_BONUS
    } else if n.starts_with(&q) {
        PREFIX_NAME_BONUS
    } else {
        0
    }
}

fn score_branch(query: &str, branch: &Branch, engine: &dyn MatchEngine) -> Option<i32> {
    let name_result = engine.match_item(&branch.name);
    let subject_result = engine.match_item(&branch.subject);

    match (name_result, subject_result) {
        (None, None) => None,
        (Some(name_match), None) => Some(
            name_match
                .rank
                .score
                .saturating_mul(NAME_WEIGHT)
                .saturating_add(name_bonuses(query, &branch.name)),
        ),
        (None, Some(subject_match)) => Some(subject_match.rank.score),
        (Some(name_match), Some(subject_match)) => {
            let name_score = name_match
                .rank
                .score
                .saturating_mul(NAME_WEIGHT)
                .saturating_add(name_bonuses(query, &branch.name));
            let subject_score = subject_match.rank.score;
            Some(name_score.max(subject_score))
        }
    }
}

/// Best matching branch by weighted score; tie-break by earlier index (committerdate order).
pub fn find_best<'a>(query: &str, branches: &'a [Branch]) -> Option<&'a Branch> {
    let factory = ExactOrFuzzyEngineFactory::builder().build();
    let engine = factory.create_engine(query);

    let mut best: Option<Scored<'a>> = None;

    for branch in branches {
        let Some(effective) = score_branch(query, branch, engine.as_ref()) else {
            continue;
        };

        if best.as_ref().is_none_or(|b| effective > b.effective) {
            best = Some(Scored { branch, effective });
        }
    }

    best.map(|s| s.branch)
}

/// Alias for direct CLI checkout (same ranking as [`find_best`]).
pub fn find_first_matching<'a>(query: &str, branches: &'a [Branch]) -> Option<&'a Branch> {
    find_best(query, branches)
}

#[cfg(test)]
mod tests {
    use super::*;
    use parse::Branch;

    fn branch(name: &str, subject: &str) -> Branch {
        Branch {
            name: name.into(),
            short_hash: "abc1234".into(),
            subject: subject.into(),
        }
    }

    #[test]
    fn name_match_beats_subject_only_on_other_branch() {
        let branches = vec![
            branch("feature/authentication", "minor cleanup"),
            branch("main", "authentication refactor for apps"),
        ];
        let best = find_best("authentication", &branches).unwrap();
        assert_eq!(best.name, "feature/authentication");
    }

    #[test]
    fn exact_name_beats_subject_match_elsewhere() {
        let branches = vec![
            branch("release", "ship release notes"),
            branch("main", "update release process"),
        ];
        let best = find_best("release", &branches).unwrap();
        assert_eq!(best.name, "release");
    }

    #[test]
    fn branch_item_output_is_checkout_name() {
        let item = BranchItem::new(branch("foo/bar", "subject"));
        assert_eq!(item.output(), "foo/bar");
        assert_eq!(item.text(), "foo/bar abc1234 subject");
    }

    #[test]
    fn branch_item_matching_ranges_name_before_subject() {
        let item = BranchItem::new(branch("my-branch", "my subject"));
        let ranges = item.get_matching_ranges().unwrap();
        assert_eq!(ranges[0], (0, "my-branch".len()));
        assert!(ranges[1].0 > ranges[0].1);
    }
}
