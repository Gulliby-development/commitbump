use std::fs;
use std::io::Cursor;

use commitbump::run;

const README_EXAMPLE: &str = "{\"release\":true,\"current\":\"1.4.2\",\"next\":\"2.0.0\",\"bump\":\"major\",\"commits\":4,\"breaking\":1,\"features\":2,\"fixes\":1,\"performance\":0}";

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

fn assert_json(args: &[&str], input: &[u8], expected_status: i32, expected_json: &str) {
    assert_eq!(
        invoke(args, input),
        (expected_status, format!("{expected_json}\n"), String::new())
    );
}

fn assert_error(args: &[&str], input: &[u8], message: &str) {
    assert_eq!(
        invoke(args, input),
        (2, String::new(), format!("commitbump: {message}\n"))
    );
}

#[test]
fn emits_the_readme_worked_example_as_one_json_line() {
    let input = concat!(
        "feat(api)!: drop the v1 endpoints\n",
        "\n",
        "BREAKING CHANGE: v1 endpoints are removed\n",
        "----\n",
        "fix: handle empty input\n",
        "----\n",
        "Merge branch 'main'\n",
        "----\n",
        "feat: add --json flag\n",
        "----\n",
        "docs: update readme\n",
    );

    assert_json(
        &["--current", "1.4.2", "--format", "json"],
        input.as_bytes(),
        0,
        README_EXAMPLE,
    );
}

#[test]
fn uses_the_zero_major_bump_matrix_for_json() {
    assert_json(
        &["--current", "0.9.1", "--format", "json"],
        concat!(
            "feat(api)!: drop the v1 endpoints\n",
            "\n",
            "BREAKING CHANGE: v1 endpoints are removed\n",
            "----\n",
            "fix: handle empty input\n",
            "----\n",
            "Merge branch 'main'\n",
            "----\n",
            "feat: add --json flag\n",
            "----\n",
            "docs: update readme\n",
        )
        .as_bytes(),
        0,
        "{\"release\":true,\"current\":\"0.9.1\",\"next\":\"0.10.0\",\"bump\":\"minor\",\"commits\":4,\"breaking\":1,\"features\":2,\"fixes\":1,\"performance\":0}",
    );
}

#[test]
fn counts_fix_and_performance_commits_for_a_patch_release() {
    assert_json(
        &["--current", "1.4.2", "--format", "json"],
        b"fix: a\n----\nperf: b",
        0,
        "{\"release\":true,\"current\":\"1.4.2\",\"next\":\"1.4.3\",\"bump\":\"patch\",\"commits\":2,\"breaking\":0,\"features\":0,\"fixes\":1,\"performance\":1}",
    );
}

#[test]
fn counts_header_and_body_breaking_markers_but_not_empty_notes() {
    assert_json(
        &["--current", "1.4.2", "--format", "json"],
        concat!(
            "fix: body breaking\n",
            "BREAKING CHANGE: remove old behavior\n",
            "----\n",
            "fix: empty note\n",
            "BREAKING-CHANGE:   \n",
            "----\n",
            "feat!: header breaking\n",
        )
        .as_bytes(),
        0,
        "{\"release\":true,\"current\":\"1.4.2\",\"next\":\"2.0.0\",\"bump\":\"major\",\"commits\":3,\"breaking\":2,\"features\":1,\"fixes\":2,\"performance\":0}",
    );
}

#[test]
fn reports_no_release_with_accurate_counts_and_for_empty_input() {
    assert_json(
        &["--current", "1.4.2", "--format", "json"],
        b"docs: x\n----\nchore: y",
        1,
        "{\"release\":false,\"current\":\"1.4.2\",\"next\":null,\"bump\":null,\"commits\":2,\"breaking\":0,\"features\":0,\"fixes\":0,\"performance\":0}",
    );
    assert_json(
        &["--current", "1.4.2", "--format", "json"],
        b"",
        1,
        "{\"release\":false,\"current\":\"1.4.2\",\"next\":null,\"bump\":null,\"commits\":0,\"breaking\":0,\"features\":0,\"fixes\":0,\"performance\":0}",
    );
}

#[test]
fn allows_a_next_version_component_to_exceed_the_input_limit() {
    assert_json(
        &["--current", "999999.999999.999999", "--format", "json"],
        b"feat: add capability",
        0,
        "{\"release\":true,\"current\":\"999999.999999.999999\",\"next\":\"999999.1000000.0\",\"bump\":\"minor\",\"commits\":1,\"breaking\":0,\"features\":1,\"fixes\":0,\"performance\":0}",
    );
}

#[test]
fn the_last_format_option_controls_the_output() {
    assert_json(
        &[
            "--current",
            "1.4.2",
            "--format",
            "version",
            "--format",
            "json",
        ],
        b"feat: add capability",
        0,
        "{\"release\":true,\"current\":\"1.4.2\",\"next\":\"1.5.0\",\"bump\":\"minor\",\"commits\":1,\"breaking\":0,\"features\":1,\"fixes\":0,\"performance\":0}",
    );
    assert_eq!(
        invoke(
            &[
                "--current",
                "1.4.2",
                "--format",
                "json",
                "--format",
                "version",
            ],
            b"feat: add capability",
        ),
        (0, "1.5.0\n".to_owned(), String::new())
    );
}

#[test]
fn reports_json_format_errors_without_stdout() {
    assert_error(
        &[
            "--current",
            "1.4.2",
            "--format",
            "json",
            "--date",
            "2026-03-01",
        ],
        b"feat: add capability",
        "--date requires --format changelog",
    );
    assert_error(
        &["--current", "1.4.2", "--format", "json", "--strict"],
        b"x",
        "commit 1: not a conventional commit: \"x\"",
    );
    assert_error(
        &["--current", "1.4.2", "--format=json"],
        b"feat: add capability",
        "unknown argument \"--format=json\"",
    );
    assert_error(
        &["--current", "1.4.2", "--format", "JSON"],
        b"feat: add capability",
        "invalid format \"JSON\"",
    );
    assert_error(&["--format", "json"], b"", "--current is required");
    assert_error(
        &["--current", "invalid", "--format", "json"],
        &[0xff],
        "invalid version \"invalid\"",
    );
    assert_error(
        &["--current", "1.4.2", "--format", "json"],
        &[0xff],
        "input is not valid UTF-8",
    );
}

#[test]
fn documents_the_json_output_contract() {
    let readme = fs::read_to_string("README.md").expect("README.md should be readable");
    let usage = "Usage: commitbump --current X.Y.Z [--format version|changelog|json] [--date YYYY-MM-DD] [--strict]";
    let fence = String::from(char::from(96)).repeat(3);
    let example_block = format!("{fence}json\n{README_EXAMPLE}\n{fence}");

    assert!(readme.contains("\n## JSON output\n"));
    assert!(readme.contains(usage));
    assert!(readme.contains(&example_block));
}
