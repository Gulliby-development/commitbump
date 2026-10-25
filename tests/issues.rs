use std::fs;
use std::io::Cursor;

use commitbump::run;

const USAGE: &str = "Usage: commitbump --current [v]X.Y.Z [--format version|changelog|json|why] [--date YYYY-MM-DD] [--scope NAME] [--issues] [--strict]";

const STREAM: &str = concat!(
    "fix(db): close the pool\n",
    "\n",
    "Closes #12\n",
    "Fixes #34\n",
    "Closes #12\n",
    "----\n",
    "feat: add paging\n",
    "\n",
    "Resolves #7\n",
    "----\n",
    "feat(api)!: drop the v1 endpoints\n",
    "\n",
    "BREAKING CHANGE: v1 endpoints are removed\n",
    "Closes #5\n",
    "----\n",
    "docs: update readme\n",
    "\n",
    "Closes #99\n",
);

const STREAM_ISSUES_OUTPUT: &str = concat!(
    "## 2.0.0 - 2026-03-01\n",
    "\n",
    "### Breaking changes\n",
    "\n",
    "- **api:** v1 endpoints are removed (#5)\n",
    "\n",
    "### Features\n",
    "\n",
    "- add paging (#7)\n",
    "- **api:** drop the v1 endpoints (#5)\n",
    "\n",
    "### Fixes\n",
    "\n",
    "- **db:** close the pool (#12, #34)\n",
);

const STREAM_PLAIN_OUTPUT: &str = concat!(
    "## 2.0.0 - 2026-03-01\n",
    "\n",
    "### Breaking changes\n",
    "\n",
    "- **api:** v1 endpoints are removed\n",
    "\n",
    "### Features\n",
    "\n",
    "- add paging\n",
    "- **api:** drop the v1 endpoints\n",
    "\n",
    "### Fixes\n",
    "\n",
    "- **db:** close the pool\n",
);

const ISSUES_ERROR: &str = "commitbump: --issues requires --format changelog\n";

fn invoke(args: &[&str], input: &[u8]) -> (i32, String, String) {
    let args = args
        .iter()
        .map(|argument| (*argument).to_owned())
        .collect::<Vec<_>>();
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let status = run(&args, &mut stdin, &mut stdout, &mut stderr);
    (
        status,
        String::from_utf8(stdout).unwrap(),
        String::from_utf8(stderr).unwrap(),
    )
}

fn issues_changelog(input: &str) -> (i32, String, String) {
    invoke(
        &[
            "--current",
            "1.4.2",
            "--format",
            "changelog",
            "--date",
            "2026-03-01",
            "--issues",
        ],
        input.as_bytes(),
    )
}

fn patch_changelog(bullet: &str) -> (i32, String, String) {
    (
        0,
        format!("## 1.4.3 - 2026-03-01\n\n### Fixes\n\n{bullet}\n"),
        String::new(),
    )
}

#[test]
fn stream_with_issues_prints_the_readme_output() {
    assert_eq!(
        issues_changelog(STREAM),
        (0, STREAM_ISSUES_OUTPUT.to_owned(), String::new())
    );
}

#[test]
fn stream_without_issues_has_no_suffix() {
    assert_eq!(
        invoke(
            &[
                "--current",
                "1.4.2",
                "--format",
                "changelog",
                "--date",
                "2026-03-01"
            ],
            STREAM.as_bytes()
        ),
        (0, STREAM_PLAIN_OUTPUT.to_owned(), String::new())
    );
}

#[test]
fn accepts_the_three_words_trailing_blanks_and_both_line_endings() {
    assert_eq!(
        issues_changelog("fix: a\n\nCloses #1 \t\nFixes #999999 \t\nResolves #10 \t\n"),
        patch_changelog("- a (#1, #999999, #10)")
    );

    let lf = issues_changelog("fix: a\n\nCloses #5\n");
    let crlf = issues_changelog("fix: a\r\n\r\nCloses #5\r\n");
    assert_eq!(lf, crlf);
    assert_eq!(lf, patch_changelog("- a (#5)"));
}

#[test]
fn lines_of_any_other_shape_are_not_footers() {
    let input = concat!(
        "fix: a\n",
        "\n",
        "closes #1\n",
        "FIXES #6\n",
        " Closes #3\n",
        "Closes  #4\n",
        "Closes #0\n",
        "Closes #012\n",
        "Closes #1000000\n",
        "Closes 12\n",
        "Closes #12x\n",
        "Closes #1, #2\n",
        "Closes #-1\n",
        "Closes #\n",
        "Fixed #5\n",
        "Refs #7\n",
    );
    assert_eq!(issues_changelog(input), patch_changelog("- a"));
    assert_eq!(
        issues_changelog("fix: closes #8, Closes #9\n"),
        patch_changelog("- closes #8, Closes #9")
    );
}

#[test]
fn keeps_each_number_once_in_footer_order() {
    assert_eq!(
        issues_changelog("fix: a\n\nFixes #3\nCloses #1\nResolves #3\nCloses #2\nFixes #1\n"),
        patch_changelog("- a (#3, #1, #2)")
    );
}

#[test]
fn every_bullet_of_a_breaking_feature_gets_the_numbers() {
    let input = concat!(
        "feat(ui): new menu\n",
        "\n",
        "BREAKING CHANGE: menu keys moved\n",
        "BREAKING-CHANGE: menu has no icons\n",
        "Fixes #8\n",
    );
    assert_eq!(
        issues_changelog(input),
        (
            0,
            concat!(
                "## 2.0.0 - 2026-03-01\n",
                "\n",
                "### Breaking changes\n",
                "\n",
                "- **ui:** menu keys moved (#8)\n",
                "- **ui:** menu has no icons (#8)\n",
                "\n",
                "### Features\n",
                "\n",
                "- **ui:** new menu (#8)\n",
            )
            .to_owned(),
            String::new()
        )
    );
}

#[test]
fn scope_filter_applies_before_the_numbers() {
    assert_eq!(
        invoke(
            &[
                "--current",
                "1.4.2",
                "--format",
                "changelog",
                "--date",
                "2026-03-01",
                "--issues",
                "--scope",
                "db"
            ],
            STREAM.as_bytes()
        ),
        patch_changelog("- **db:** close the pool (#12, #34)")
    );
}

#[test]
fn commits_outside_the_changelog_are_ignored() {
    assert_eq!(
        issues_changelog("docs: only docs\n\nCloses #3\n"),
        (1, "no release\n".to_owned(), String::new())
    );
}

#[test]
fn issues_argument_errors_follow_the_check_order() {
    assert_eq!(
        invoke(&["--current", "1.4.2", "--issues"], b"fix: a\n"),
        (2, String::new(), ISSUES_ERROR.to_owned())
    );
    assert_eq!(
        invoke(
            &["--current", "1.4.2", "--issues", "--format", "version"],
            b"fix: a\n"
        ),
        (2, String::new(), ISSUES_ERROR.to_owned())
    );
    assert_eq!(
        invoke(
            &["--current", "1.4.2", "--issues", "--format", "json"],
            b"fix: a\n"
        ),
        (2, String::new(), ISSUES_ERROR.to_owned())
    );
    assert_eq!(
        invoke(
            &["--current", "1.4.2", "--issues", "--format", "why"],
            b"fix: a\n"
        ),
        (2, String::new(), ISSUES_ERROR.to_owned())
    );

    assert_eq!(
        invoke(
            &["--current", "1.4.2", "--issues", "--date", "2026-03-01"],
            b"fix: a\n"
        ),
        (
            2,
            String::new(),
            "commitbump: --date requires --format changelog\n".to_owned()
        )
    );
    assert_eq!(
        invoke(
            &["--current", "1.4.2", "--format", "changelog", "--issues"],
            b"fix: a\n"
        ),
        (
            2,
            String::new(),
            "commitbump: --format changelog requires --date\n".to_owned()
        )
    );
    assert_eq!(
        invoke(&["--current", "bad", "--issues"], b"fix: a\n"),
        (
            2,
            String::new(),
            "commitbump: invalid version \"bad\"\n".to_owned()
        )
    );
}

#[test]
fn help_and_readme_document_the_option() {
    assert_eq!(
        invoke(&["--help"], b""),
        (0, format!("{USAGE}\n"), String::new())
    );

    let readme = fs::read_to_string("README.md")
        .expect("README.md should be readable")
        .replace("\r\n", "\n");
    let fence = "`".repeat(3);
    let block = format!("{fence}text\n{STREAM_ISSUES_OUTPUT}{fence}");
    assert!(readme.contains(USAGE));
    assert!(readme.contains("## Issue references"));
    assert!(readme.contains("| `--issues` | Add the issue numbers of"));
    assert!(readme.contains(&block));
}
