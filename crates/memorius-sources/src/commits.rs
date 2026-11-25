//! The user's own commits on this Mac, from each repository's reflogs (the
//! main checkout and every worktree). Reflogs hold exactly the commits made
//! here, with hash, author e-mail, time and subject; nothing reads diffs.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::hash::BuildHasher;
use std::path::{Path, PathBuf};
use std::process::Command;

use memorius_core::timeline::DayCommit;
use serde::Serialize;
use ts_rs::TS;

use crate::git::{git_dir, parse_history};

/// How deep the root folder is searched for repositories.
const ROOT_DEPTH: usize = 3;
/// Folders never worth entering while looking for repositories.
const SKIPPED: [&str; 4] = ["node_modules", "target", "bin", "obj"];

/// The repositories (by their shared git directory, so a repository and
/// its worktrees count once) under `roots` and around `folders`.
pub fn repositories(roots: &[PathBuf], folders: &[PathBuf]) -> Vec<PathBuf> {
    let mut found: HashSet<PathBuf> = HashSet::new();
    for root in roots {
        search(root, ROOT_DEPTH, &mut found);
    }
    for folder in folders {
        if let Some(dir) = git_dir(folder) {
            found.insert(common_dir(&dir));
        }
    }
    let mut repositories: Vec<PathBuf> = found.into_iter().collect();
    repositories.sort();
    repositories
}

fn search(dir: &Path, depth: usize, found: &mut HashSet<PathBuf>) {
    let dot_git = dir.join(".git");
    if dot_git.is_dir() {
        found.insert(dot_git.canonicalize().unwrap_or(dot_git));
        return;
    }
    if depth == 0 {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with('.') || SKIPPED.contains(&name.as_ref()) {
            continue;
        }
        if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
            search(&entry.path(), depth - 1, found);
        }
    }
}

/// Folders holding two or more of `repositories` that no root covers yet,
/// most repositories first: good roots to suggest.
pub fn suggested_roots(repositories: &[PathBuf], roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut counts: HashMap<PathBuf, usize> = HashMap::new();
    for repository in repositories {
        // `<parent>/<repository>/.git` → `<parent>`.
        if let Some(parent) = repository.parent().and_then(Path::parent)
            && !roots.iter().any(|root| parent.starts_with(root))
        {
            *counts.entry(parent.to_path_buf()).or_default() += 1;
        }
    }
    let mut suggested: Vec<(PathBuf, usize)> = counts
        .into_iter()
        .filter(|(_, count)| *count >= 2)
        .collect();
    suggested.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    suggested.into_iter().map(|(parent, _)| parent).collect()
}

/// A worktree's git directory points to the repository's shared one.
fn common_dir(git_dir: &Path) -> PathBuf {
    fs::read_to_string(git_dir.join("commondir"))
        .map(|common| {
            let common = Path::new(common.trim());
            if common.is_absolute() {
                common.to_path_buf()
            } else {
                git_dir.join(common)
            }
        })
        .ok()
        .and_then(|common| common.canonicalize().ok())
        .unwrap_or_else(|| git_dir.to_path_buf())
}

/// The user's e-mail addresses from `git config`: the global one and each
/// repository's own.
pub fn user_emails(repositories: &[PathBuf]) -> HashSet<String> {
    let email = |args: &[&str]| {
        Command::new("git")
            .args(args)
            .output()
            .ok()
            .filter(|output| output.status.success())
            .map(|output| {
                String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .to_lowercase()
            })
            .filter(|email| !email.is_empty())
    };
    let mut emails: HashSet<String> = email(&["config", "--global", "--get", "user.email"])
        .into_iter()
        .collect();
    for repository in repositories {
        let git_dir = repository.display().to_string();
        emails.extend(email(&[
            "--git-dir",
            &git_dir,
            "config",
            "--get",
            "user.email",
        ]));
    }
    emails
}

/// The commits by `emails` in one repository's reflogs, oldest first; a
/// commit later amended counts only in its amended form.
pub fn read_commits<S: BuildHasher>(
    repository: &Path,
    emails: &HashSet<String, S>,
) -> Vec<DayCommit> {
    let name = repository_name(repository);
    let mut logs = vec![repository.to_path_buf()];
    if let Ok(entries) = fs::read_dir(repository.join("worktrees")) {
        logs.extend(entries.flatten().map(|entry| entry.path()));
    }
    let mut commits: BTreeMap<String, DayCommit> = BTreeMap::new();
    let mut amended: HashSet<String> = HashSet::new();
    for git_dir in logs {
        let log = fs::read_to_string(git_dir.join("logs").join("HEAD")).unwrap_or_default();
        let head = fs::read_to_string(git_dir.join("HEAD")).unwrap_or_default();
        parse_commits(&log, &head, &name, emails, &mut commits, &mut amended);
    }
    let mut kept: Vec<DayCommit> = commits
        .into_values()
        .filter(|commit| !amended.contains(&commit.hash))
        .collect();
    kept.sort_by_key(|commit| commit.at_ms);
    kept
}

fn repository_name(repository: &Path) -> String {
    let folder = if repository.file_name().is_some_and(|name| name == ".git") {
        repository.parent().unwrap_or(repository)
    } else {
        repository
    };
    folder
        .file_name()
        .map_or_else(String::new, |name| name.to_string_lossy().into_owned())
}

fn parse_commits<S: BuildHasher>(
    log: &str,
    head: &str,
    repo: &str,
    emails: &HashSet<String, S>,
    commits: &mut BTreeMap<String, DayCommit>,
    amended: &mut HashSet<String>,
) {
    let history = parse_history(log, head);
    for line in log.lines() {
        // "<old> <new> <name> <email> <seconds> <zone>\t<message>"
        let Some((identity, message)) = line.split_once('\t') else {
            continue;
        };
        if !message.starts_with("commit") {
            continue;
        }
        let mut hashes = identity.splitn(3, ' ');
        let (Some(old), Some(new)) = (hashes.next(), hashes.next()) else {
            continue;
        };
        // A reflog is a plain file anyone can edit; keep only real ids.
        if !is_object_id(new) {
            continue;
        }
        let email = identity
            .split_once('<')
            .and_then(|(_, rest)| rest.split_once('>'))
            .map(|(email, _)| email.to_lowercase());
        let seconds = identity
            .rsplit(' ')
            .nth(1)
            .and_then(|seconds| seconds.parse::<i64>().ok());
        let (Some(email), Some(seconds), Some((_, subject))) =
            (email, seconds, message.split_once(": "))
        else {
            continue;
        };
        if message.starts_with("commit (amend)") {
            amended.insert(old.to_owned());
        }
        if !emails.contains(&email) {
            continue;
        }
        let at_ms = seconds * 1_000;
        commits.insert(
            new.to_owned(),
            DayCommit {
                hash: new.to_owned(),
                at_ms,
                subject: subject.to_owned(),
                repo: repo.to_owned(),
                branch: history.branch_at(at_ms).map(str::to_owned),
            },
        );
    }
}

/// What a commit holds beyond its subject, read from git on hover; nothing
/// of it is stored.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CommitDetails {
    /// The message under the subject, if any.
    pub body: String,
    pub files: u32,
    pub insertions: u32,
    pub deletions: u32,
    /// The most changed files, most first.
    pub top_files: Vec<String>,
    /// The commit on GitHub when the repository's origin is there.
    pub web_url: Option<String>,
}

/// Files changed shown by name.
const TOP_FILES: usize = 3;

/// A git object id: 40 (SHA-1) or 64 (SHA-256) hex digits.
pub fn is_object_id(text: &str) -> bool {
    matches!(text.len(), 40 | 64) && text.bytes().all(|byte| byte.is_ascii_hexdigit())
}

pub fn commit_details(git_dir: &Path, hash: &str) -> Result<CommitDetails, String> {
    if !is_object_id(hash) {
        return Err("Not a commit id".into());
    }
    let git = |args: &[&str]| -> Result<String, String> {
        let output = Command::new("git")
            .arg("--git-dir")
            .arg(git_dir)
            .args(args)
            .output()
            .map_err(|error| format!("Cannot run git: {error}"))?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).trim().to_owned())
        }
    };
    // The id can never be read as an option.
    let message = git(&["show", "-s", "--format=%B", "--end-of-options", hash])?;
    let numstat = git(&["show", "--numstat", "--format=", "--end-of-options", hash])?;
    let remote = git(&["config", "--get", "remote.origin.url"]).unwrap_or_default();
    let mut details = numstat_details(&numstat);
    details.body = message
        .split_once('\n')
        .map(|(_, body)| body.trim().to_owned())
        .unwrap_or_default();
    details.web_url = github_commit_url(remote.trim(), hash);
    Ok(details)
}

/// Sums `git show --numstat` lines ("added\tdeleted\tpath"; "-" for binary).
fn numstat_details(numstat: &str) -> CommitDetails {
    let mut files: Vec<(u32, String)> = Vec::new();
    let mut details = CommitDetails::default();
    for line in numstat.lines() {
        let mut fields = line.splitn(3, '\t');
        let (Some(added), Some(deleted), Some(path)) =
            (fields.next(), fields.next(), fields.next())
        else {
            continue;
        };
        let added = added.parse::<u32>().unwrap_or(0);
        let deleted = deleted.parse::<u32>().unwrap_or(0);
        details.files += 1;
        details.insertions += added;
        details.deletions += deleted;
        files.push((added + deleted, path.to_owned()));
    }
    files.sort_by(|(a, _), (b, _)| b.cmp(a));
    details.top_files = files
        .into_iter()
        .take(TOP_FILES)
        .map(|(_, path)| path)
        .collect();
    details
}

/// `https://github.com/owner/repo/commit/<hash>` from an origin such as
/// `git@github.com:owner/repo.git` or `https://github.com/owner/repo`.
fn github_commit_url(remote: &str, hash: &str) -> Option<String> {
    let path = remote
        .strip_prefix("git@github.com:")
        .or_else(|| remote.strip_prefix("https://github.com/"))
        .or_else(|| remote.strip_prefix("ssh://git@github.com/"))?;
    let path = path.trim_end_matches('/').trim_end_matches(".git");
    (path.split('/').count() == 2).then(|| format!("https://github.com/{path}/commit/{hash}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOG: &str = "\
0000000000000000000000000000000000000000 aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa Ann <ann@x.com> 1000 +0200\tcommit (initial): Start
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa Ann <ann@x.com> 2000 +0200\tcheckout: moving from main to claude/int-290
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb Ann <ann@x.com> 3000 +0200\tcommit: [INT-290] Skip unchanged
bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb cccccccccccccccccccccccccccccccccccccccc Ann <Ann@X.com> 3500 +0200\tcommit (amend): [INT-290] Skip unchanged rows
cccccccccccccccccccccccccccccccccccccccc dddddddddddddddddddddddddddddddddddddddd Bob <bob@y.com> 4000 +0200\tcommit: Someone else's commit
dddddddddddddddddddddddddddddddddddddddd eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee Ann <ann@x.com> 5000 +0200\trebase (pick): Replayed, not new work
";

    static A: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| "aaaa".repeat(10));
    static C: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| "cccc".repeat(10));

    #[test]
    fn keeps_the_users_commits_with_their_branch_and_drops_amended_ones() {
        let dir = tempfile::tempdir().unwrap();
        let git = dir.path().join("memorius").join(".git");
        fs::create_dir_all(git.join("logs")).unwrap();
        fs::write(git.join("logs").join("HEAD"), LOG).unwrap();
        fs::write(git.join("HEAD"), "ref: refs/heads/main\n").unwrap();
        let emails = HashSet::from(["ann@x.com".to_owned()]);

        let commits = read_commits(&git, &emails);

        assert_eq!(
            commits
                .iter()
                .map(|c| (
                    c.hash.as_str(),
                    c.subject.as_str(),
                    c.branch.as_deref(),
                    c.repo.as_str()
                ))
                .collect::<Vec<_>>(),
            [
                (A.as_str(), "Start", Some("main"), "memorius"),
                (
                    C.as_str(),
                    "[INT-290] Skip unchanged rows",
                    Some("claude/int-290"),
                    "memorius"
                ),
            ]
        );
    }

    #[test]
    fn finds_repositories_once_through_the_root_and_a_worktree() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("repos");
        let git = root.join("Shop").join(".git");
        let worktree_git = git.join("worktrees").join("wt");
        fs::create_dir_all(&worktree_git).unwrap();
        fs::write(worktree_git.join("commondir"), "../..\n").unwrap();
        let worktree = dir.path().join("elsewhere").join("wt");
        fs::create_dir_all(&worktree).unwrap();
        fs::write(
            worktree.join(".git"),
            format!("gitdir: {}\n", worktree_git.display()),
        )
        .unwrap();

        let found = repositories(&[root], &[worktree.join("src").join("a.fs")]);

        assert_eq!(found.len(), 1);
        assert_eq!(repository_name(&found[0]), "Shop");
    }

    #[test]
    fn suggests_folders_with_several_repositories_not_searched_yet() {
        let repo = |path: &str| PathBuf::from(path).join(".git");
        let found = [
            repo("/home/ann/code/a"),
            repo("/home/ann/code/b"),
            repo("/home/ann/work/c"),
            repo("/home/ann/work/d"),
            repo("/home/ann/work/e"),
            repo("/home/ann/one-off"),
        ];

        assert_eq!(
            suggested_roots(&found, &[]),
            [
                PathBuf::from("/home/ann/work"),
                PathBuf::from("/home/ann/code")
            ]
        );
        assert_eq!(
            suggested_roots(&found, &[PathBuf::from("/home/ann/work")]),
            [PathBuf::from("/home/ann/code")]
        );
    }

    #[test]
    fn only_hex_object_ids_count_as_commits() {
        assert!(is_object_id(&"a".repeat(40)));
        assert!(is_object_id(&"0".repeat(64)));
        assert!(!is_object_id("--output=/tmp/x"));
        assert!(!is_object_id(&"g".repeat(40)));
        assert!(commit_details(Path::new("/nonexistent"), "--output=/tmp/x").is_err());
    }

    #[test]
    fn sums_the_changes_and_names_the_biggest_files() {
        let details = numstat_details("3\t1\tsrc/a.rs\n-\t-\ticon.png\n40\t2\tsrc/b.rs\n");

        assert_eq!(
            (details.files, details.insertions, details.deletions),
            (3, 43, 3)
        );
        assert_eq!(details.top_files, ["src/b.rs", "src/a.rs", "icon.png"]);
    }

    #[test]
    fn links_a_github_origin_to_the_commit() {
        assert_eq!(
            github_commit_url("git@github.com:lukaszkrzywizna/memorius.git", "abc").as_deref(),
            Some("https://github.com/lukaszkrzywizna/memorius/commit/abc")
        );
        assert_eq!(
            github_commit_url("https://github.com/acme/shop", "abc").as_deref(),
            Some("https://github.com/acme/shop/commit/abc")
        );
        assert_eq!(github_commit_url("https://dev.azure.com/x/y", "abc"), None);
    }
}
