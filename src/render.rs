use crate::parse::{Commit, CommitKind};
use crate::version::{Bump, Version};

pub fn summary_json(
    current: Version,
    prefixed: bool,
    release: Option<(Version, Bump)>,
    commits: &[Commit],
) -> String {
    let commits_count = commits.len();
    let breaking_count = commits.iter().filter(|commit| commit.breaking).count();
    let features_count = commits
        .iter()
        .filter(|commit| commit.kind == CommitKind::Feat)
        .count();
    let fixes_count = commits
        .iter()
        .filter(|commit| commit.kind == CommitKind::Fix)
        .count();
    let performance_count = commits
        .iter()
        .filter(|commit| commit.kind == CommitKind::Perf)
        .count();

    let current = display_version(current, prefixed);
    match release {
        Some((next, bump)) => format!(
            "{{\"release\":true,\"current\":\"{current}\",\"next\":\"{}\",\"bump\":\"{}\",\"commits\":{commits_count},\"breaking\":{breaking_count},\"features\":{features_count},\"fixes\":{fixes_count},\"performance\":{performance_count}}}",
            display_version(next, prefixed),
            bump_name(bump)
        ),
        None => format!(
            "{{\"release\":false,\"current\":\"{current}\",\"next\":null,\"bump\":null,\"commits\":{commits_count},\"breaking\":{breaking_count},\"features\":{features_count},\"fixes\":{fixes_count},\"performance\":{performance_count}}}"
        ),
    }
}

fn bump_name(bump: Bump) -> &'static str {
    match bump {
        Bump::Major => "major",
        Bump::Minor => "minor",
        Bump::Patch => "patch",
    }
}

pub fn changelog(next: Version, prefixed: bool, date: &str, commits: &[Commit]) -> String {
    let mut lines = vec![format!("## {} - {date}", display_version(next, prefixed))];

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

fn display_version(version: Version, prefixed: bool) -> String {
    if prefixed {
        format!("v{version}")
    } else {
        version.to_string()
    }
}
