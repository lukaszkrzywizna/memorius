//! Branch and commit history read from Git's own files (`.git/logs/HEAD`),
//! without starting a `git` process.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// What `.git/logs/HEAD` says about one repository or worktree.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitHistory {
    /// `(time_ms, from, to)` for each branch switch, oldest first.
    switches: Vec<(i64, String, String)>,
    /// `(time_ms, subject)` for each commit made here, oldest first.
    pub commits: Vec<(i64, String)>,
    /// The branch checked out now (`HEAD` file).
    head: Option<String>,
}

impl GitHistory {
    /// The branch that was checked out at `time_ms`.
    pub fn branch_at(&self, time_ms: i64) -> Option<&str> {
        match self.switches.iter().rposition(|(at, _, _)| *at <= time_ms) {
            Some(index) => Some(self.switches[index].2.as_str()),
            None => self
                .switches
                .first()
                .map(|(_, from, _)| from.as_str())
                .or(self.head.as_deref()),
        }
    }

    /// `(time_ms, subject)` of the commits made from `from_ms` to `to_ms`.
    pub fn commits_between(&self, from_ms: i64, to_ms: i64) -> impl Iterator<Item = (i64, &str)> {
        self.commits
            .iter()
            .filter(move |(at, _)| (from_ms..=to_ms).contains(at))
            .map(|(at, subject)| (*at, subject.as_str()))
    }
}

/// Histories by working folder, reread only when `.git/logs/HEAD` changes.
/// Repositories are few; the cache resets if it ever holds more than
/// [`CACHE_LIMIT`].
#[derive(Default)]
pub struct GitHistories {
    by_folder: HashMap<PathBuf, (Option<SystemTime>, GitHistory)>,
}

const CACHE_LIMIT: usize = 256;

impl GitHistories {
    pub fn get(&mut self, folder: &Path) -> Option<&GitHistory> {
        let git_dir = git_dir(folder)?;
        let modified = fs::metadata(git_dir.join("logs").join("HEAD"))
            .and_then(|meta| meta.modified())
            .ok();
        let fresh = self
            .by_folder
            .get(folder)
            .is_some_and(|(cached, _)| *cached == modified);
        if !fresh {
            if self.by_folder.len() >= CACHE_LIMIT {
                self.by_folder.clear();
            }
            self.by_folder
                .insert(folder.to_path_buf(), (modified, read_history(&git_dir)));
        }
        self.by_folder.get(folder).map(|(_, history)| history)
    }
}

/// The Git directory for a working folder: walks up to the `.git` entry and
/// follows a worktree's `gitdir:` file.
pub fn git_dir(folder: &Path) -> Option<PathBuf> {
    let mut current = Some(folder);
    while let Some(dir) = current {
        let dot_git = dir.join(".git");
        if dot_git.is_dir() {
            return Some(dot_git);
        }
        if let Ok(text) = fs::read_to_string(&dot_git) {
            let target = text.strip_prefix("gitdir:")?.trim();
            return Some(dir.join(target));
        }
        current = dir.parent();
    }
    None
}

pub fn read_history(git_dir: &Path) -> GitHistory {
    let log = fs::read_to_string(git_dir.join("logs").join("HEAD")).unwrap_or_default();
    let head = fs::read_to_string(git_dir.join("HEAD")).unwrap_or_default();
    parse_history(&log, &head)
}

pub(crate) fn parse_history(log: &str, head: &str) -> GitHistory {
    let mut history = GitHistory {
        head: head
            .trim()
            .strip_prefix("ref: refs/heads/")
            .map(str::to_owned),
        ..GitHistory::default()
    };
    for line in log.lines() {
        let Some((identity, message)) = line.split_once('\t') else {
            continue;
        };
        // "<old> <new> <name> <email> <seconds> <zone>"
        let mut fields = identity.rsplitn(3, ' ');
        let _zone = fields.next();
        let Some(seconds) = fields.next().and_then(|s| s.parse::<i64>().ok()) else {
            continue;
        };
        let time_ms = seconds * 1_000;
        if let Some(moves) = message.strip_prefix("checkout: moving from ")
            && let Some((from, to)) = moves.split_once(" to ")
        {
            history
                .switches
                .push((time_ms, from.to_owned(), to.to_owned()));
        } else if let Some(branch) = message
            .strip_prefix("rebase (finish): returning to refs/heads/")
            .or_else(|| message.strip_prefix("rebase -i (finish): returning to refs/heads/"))
        {
            let from = history.branch_at(time_ms).unwrap_or_default().to_owned();
            history.switches.push((time_ms, from, branch.to_owned()));
        } else if message.starts_with("commit")
            && let Some((_, subject)) = message.split_once(": ")
        {
            history.commits.push((time_ms, subject.to_owned()));
        }
    }
    history
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOG: &str = "\
0000 aaaa Ann <a@x> 1000 +0200\tcommit (initial): Start
aaaa bbbb Ann <a@x> 2000 +0200\tcheckout: moving from main to claude/int-290-order-sync
bbbb cccc Ann <a@x> 3000 +0200\tcommit: [INT-290] Skip unchanged employees
cccc dddd Ann <a@x> 4000 +0200\tcheckout: moving from claude/int-290-order-sync to main
dddd eeee Ann <a@x> 5000 +0200\trebase (finish): returning to refs/heads/codex/is-61-catalog
";

    #[test]
    fn knows_the_branch_at_each_time() {
        let history = parse_history(LOG, "ref: refs/heads/codex/is-61-catalog\n");

        assert_eq!(history.branch_at(1_500_000), Some("main"));
        assert_eq!(
            history.branch_at(2_000_000),
            Some("claude/int-290-order-sync")
        );
        assert_eq!(history.branch_at(4_500_000), Some("main"));
        assert_eq!(history.branch_at(9_000_000), Some("codex/is-61-catalog"));
    }

    #[test]
    fn falls_back_to_head_without_switches() {
        let history = parse_history("", "ref: refs/heads/claude/sel-54\n");

        assert_eq!(history.branch_at(0), Some("claude/sel-54"));
    }

    #[test]
    fn lists_commit_subjects_in_a_time_range() {
        let history = parse_history(LOG, "");

        assert_eq!(
            history
                .commits_between(2_500_000, 3_500_000)
                .collect::<Vec<_>>(),
            [(3_000_000, "[INT-290] Skip unchanged employees")]
        );
    }

    #[test]
    fn follows_a_worktree_gitdir_file() {
        let root = tempfile::tempdir().unwrap();
        let worktree = root.path().join("wt");
        let nested = worktree.join("src").join("app");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(worktree.join(".git"), "gitdir: /repo/.git/worktrees/wt\n").unwrap();

        assert_eq!(
            git_dir(&nested),
            Some(std::path::PathBuf::from("/repo/.git/worktrees/wt"))
        );
    }
}
