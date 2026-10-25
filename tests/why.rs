use std::io::Cursor;

use commitbump::run;

const STREAM: &[u8] = b"feat(api)!: drop the v1 endpoints\n----\nfix(db): close the pool\n----\nfeat: add --json flag\n";

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

fn assert_why(args: &[&str], input: &[u8], expected_status: i32, expected: &str) {
    let mut full = args.to_vec();
    full.extend(["--format", "why"]);
    assert_eq!(
        invoke(&full, input),
        (expected_status, expected.to_owned(), String::new())
    );
}

#[test]
fn major_release_names_the_breaking_commit_and_keeps_the_v_prefix() {
    assert_why(
        &["--current", "1.4.2"],
        STREAM,
        0,
        "next     2.0.0\nbump     major\nbecause  feat(api)!: drop the v1 endpoints\n",
    );
    assert_why(
        &["--current", "v1.4.2"],
        STREAM,
        0,
        "next     v2.0.0\nbump     major\nbecause  feat(api)!: drop the v1 endpoints\n",
    );
}

#[test]
fn scope_filter_applies_before_the_reasons() {
    assert_why(
        &["--current", "1.4.2", "--scope", "db"],
        STREAM,
        0,
        "next     1.4.3\nbump     patch\nbecause  fix(db): close the pool\n",
    );
}

#[test]
fn no_release_prints_no_release_and_returns_one() {
    assert_why(
        &["--current", "1.4.2", "--scope", "ui"],
        STREAM,
        1,
        "no release\n",
    );
    assert_why(
        &["--current", "1.0.0"],
        b"docs: only docs\n----\nchore: tidy\n",
        1,
        "no release\n",
    );
}

#[test]
fn only_commits_of_the_release_level_are_reasons() {
    assert_why(
        &["--current", "1.4.2"],
        b"fix: a\n----\nfeat: b\n----\nchore: c\n----\nfeat(ui): d\n----\nMerge branch 'main'\n",
        0,
        "next     1.5.0\nbump     minor\nbecause  feat: b\nbecause  feat(ui): d\n",
    );
}

#[test]
fn feat_is_patch_level_below_one() {
    assert_why(
        &["--current", "0.3.0"],
        b"feat: add x\n----\nfix: y\n",
        0,
        "next     0.3.1\nbump     patch\nbecause  feat: add x\nbecause  fix: y\n",
    );
}

#[test]
fn breaking_is_minor_level_below_one() {
    assert_why(
        &["--current", "0.3.0"],
        b"feat: add x\n----\nfeat!: drop y\n----\nfix: z\n",
        0,
        "next     0.4.0\nbump     minor\nbecause  feat!: drop y\n",
    );
}

#[test]
fn more_than_five_reasons_are_summarized() {
    assert_why(
        &["--current", "1.0.0"],
        b"fix: item 1\n----\nfix: item 2\n----\nfix: item 3\n----\nfix: item 4\n----\nfix: item 5\n----\nfix: item 6\n----\nfix: item 7\n",
        0,
        "next     1.0.1\nbump     patch\nbecause  fix: item 1\nbecause  fix: item 2\nbecause  fix: item 3\nbecause  fix: item 4\nbecause  fix: item 5\nmore     2\n",
    );
}

#[test]
fn trailing_spaces_and_tabs_are_trimmed_from_the_header() {
    assert_why(
        &["--current", "1.4.2"],
        b"fix: trailing blanks \t\n",
        0,
        "next     1.4.3\nbump     patch\nbecause  fix: trailing blanks\n",
    );
}

#[test]
fn breaking_change_note_makes_the_header_a_major_reason() {
    assert_why(
        &["--current", "2.0.0"],
        b"feat: a\n\nBREAKING CHANGE: b\n----\nfix: c\n",
        0,
        "next     3.0.0\nbump     major\nbecause  feat: a\n",
    );
}

#[test]
fn date_is_still_rejected_with_the_why_format() {
    assert_eq!(
        invoke(&["--help"], b""),
        (
            0,
            "Usage: commitbump --current [v]X.Y.Z [--format version|changelog|json|why] [--date YYYY-MM-DD] [--scope NAME] [--issues] [--strict]\n".to_owned(),
            String::new()
        )
    );
    assert_eq!(
        invoke(&["--current", "1.4.2", "--format", "xml"], b""),
        (
            2,
            String::new(),
            "commitbump: invalid format \"xml\"\n".to_owned()
        )
    );
    assert_eq!(
        invoke(
            &[
                "--current",
                "1.4.2",
                "--date",
                "2026-03-01",
                "--format",
                "why"
            ],
            b""
        ),
        (
            2,
            String::new(),
            "commitbump: --date requires --format changelog\n".to_owned()
        )
    );
}
