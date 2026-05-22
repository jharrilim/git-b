//! Weighted fuzzy matching for branches (name preferred over commit subject).

use std::borrow::Cow;
use std::collections::HashSet;

use display::{colored_line, DisplayColors, DisplayLayout, FieldRanges, Line};
use parse::Branch;
use skim::prelude::*;
use skim::{DisplayContext, MatchEngine, MatchEngineFactory, Matches, SkimItem};

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
    field_ranges: FieldRanges,
    colors: DisplayColors,
}

impl BranchItem {
    pub fn new(branch: Branch, layout: DisplayLayout, colors: DisplayColors) -> Self {
        let display = layout.format_line(&branch);
        let matching_ranges = layout.matching_ranges(&display);
        let field_ranges = layout.field_ranges();
        Self {
            branch,
            display,
            matching_ranges,
            field_ranges,
            colors,
        }
    }

    pub fn branch(&self) -> &Branch {
        &self.branch
    }
}

impl From<Branch> for BranchItem {
    fn from(branch: Branch) -> Self {
        let layout = DisplayLayout::from_branches(std::slice::from_ref(&branch));
        Self::new(branch, layout, DisplayColors::default())
    }
}

fn highlight_chars(text: &str, matches: &Matches) -> HashSet<usize> {
    match matches {
        Matches::None => HashSet::new(),
        Matches::CharIndices(indices) => indices.iter().copied().collect(),
        Matches::CharRange(start, end) => (*start..*end).collect(),
        Matches::ByteRange(start, end) => {
            let mut set = HashSet::new();
            let mut ci = 0;
            for (bi, _) in text.char_indices() {
                if bi >= *start && bi < *end {
                    set.insert(ci);
                }
                ci += 1;
            }
            set
        }
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

    fn display(&self, context: DisplayContext) -> Line<'_> {
        if !self.colors.enabled {
            return context.to_line(Cow::Borrowed(&self.display));
        }
        let highlight = highlight_chars(&self.display, &context.matches);
        colored_line(
            &self.display,
            self.field_ranges,
            self.colors,
            context.base_style,
            context.matched_style,
            &highlight,
        )
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
    use display::{DisplayColors, DisplayLayout};
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
        let b = branch("foo/bar", "subject");
        let layout = DisplayLayout::from_branches(&[b.clone()]);
        let item = BranchItem::new(b, layout, DisplayColors::disabled());
        assert_eq!(item.output(), "foo/bar");
        assert_eq!(item.text(), "foo/bar abc1234 subject");
    }

    #[test]
    fn branch_item_matching_ranges_name_before_subject() {
        let b = branch("my-branch", "my subject");
        let layout = DisplayLayout::from_branches(&[b.clone()]);
        let item = BranchItem::new(b, layout, DisplayColors::disabled());
        let ranges = item.get_matching_ranges().unwrap();
        assert_eq!(ranges[0], (0, "my-branch".len()));
        assert!(ranges[1].0 > ranges[0].1);
    }

    #[test]
    fn colored_display_when_enabled() {
        let b = branch("main", "init");
        let layout = DisplayLayout::from_branches(&[b.clone()]);
        let item = BranchItem::new(b, layout, DisplayColors::default());
        assert!(item.colors.enabled);
        let line = item.display(DisplayContext::default());
        assert!(!line.spans.is_empty());
    }
}
