use std::fs;

const USAGE: &str = "Usage: commitbump --current X.Y.Z [--format version|changelog|json] [--date YYYY-MM-DD] [--strict]";

const WORKED_CHANGELOG: &str = concat!(
    "## 2.0.0 - 2026-03-01\n",
    "\n",
    "### Breaking changes\n",
    "\n",
    "- **api:** v1 endpoints are removed\n",
    "\n",
    "### Features\n",
    "\n",
    "- **api:** drop the v1 endpoints\n",
    "- add --json flag\n",
    "\n",
    "### Fixes\n",
    "\n",
    "- handle empty input",
);

#[test]
fn readme_documents_the_cli_and_exact_worked_changelog() {
    let readme = fs::read_to_string("README.md").expect("README.md should be readable");

    for section in [
        "Install",
        "Usage",
        "Header grammar",
        "Bump rules",
        "Output",
        "Exit codes",
        "Example",
        "Development",
    ] {
        assert!(
            readme.contains(&format!("## {section}\n")),
            "README.md is missing the {section} section"
        );
    }

    for required_text in [
        "cargo install --path .",
        "git log v1.4.2..HEAD --format='%B%n----' | commitbump --current 1.4.2",
        "cargo build",
        "cargo test",
        "cargo fmt --check",
        "cargo clippy -- -D warnings",
    ] {
        assert!(
            readme.contains(required_text),
            "README.md is missing {required_text:?}"
        );
    }

    assert!(
        readme.contains(USAGE),
        "README.md is missing the usage line"
    );
    assert_eq!(WORKED_CHANGELOG.lines().count(), 14);

    let fence = String::from(char::from(96)).repeat(3);
    let fenced_changelog = format!("{fence}text\n{WORKED_CHANGELOG}\n{fence}");
    assert!(
        readme.contains(&format!("\n{fenced_changelog}\n")),
        "README.md must contain the exact unindented worked-example changelog in a fenced block"
    );
}
