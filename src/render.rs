use crate::parse::{Commit, CommitKind};
use crate::version::Version;

pub fn changelog(next: Version, date: &str, commits: &[Commit]) -> String {
    let mut lines = vec![format!("## {next} - {date}")];

    append_group(
        &mut lines,
        "Breaking changes",
        commits
            .iter()
            .filter(|commit| commit.breaking)
            .flat_map(|commit| {
                commit
                    .notes
                    .iter()
                    .map(move |note| bullet(commit.scope.as_deref(), note))
            }),
    );
    append_group(
        &mut lines,
        "Features",
        commits
            .iter()
            .filter(|commit| commit.kind == CommitKind::Feat)
            .map(|commit| bullet(commit.scope.as_deref(), &commit.description)),
    );
    append_group(
        &mut lines,
        "Fixes",
        commits
            .iter()
            .filter(|commit| commit.kind == CommitKind::Fix)
            .map(|commit| bullet(commit.scope.as_deref(), &commit.description)),
    );
    append_group(
        &mut lines,
        "Performance",
        commits
            .iter()
            .filter(|commit| commit.kind == CommitKind::Perf)
            .map(|commit| bullet(commit.scope.as_deref(), &commit.description)),
    );

    lines.join("\n")
}

fn append_group(lines: &mut Vec<String>, title: &str, bullets: impl IntoIterator<Item = String>) {
    let bullets: Vec<String> = bullets.into_iter().collect();
    if bullets.is_empty() {
        return;
    }

    lines.push(String::new());
    lines.push(format!("### {title}"));
    lines.push(String::new());
    lines.extend(bullets);
}

fn bullet(scope: Option<&str>, text: &str) -> String {
    match scope {
        Some(scope) => format!("- **{scope}:** {text}"),
        None => format!("- {text}"),
    }
}
