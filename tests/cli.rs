use std::io::Cursor;

use commitbump::run;

const USAGE: &str = "Usage: commitbump --current X.Y.Z [--format version|changelog|json] [--date YYYY-MM-DD] [--scope NAME] [--strict]\n";

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

fn assert_cli_error(args: &[&str], input: &[u8], message: &str) {
    let (status, stdout, stderr) = invoke(args, input);
    assert_eq!(status, 2);
    assert_eq!(stdout, "");
    assert_eq!(stderr, format!("commitbump: {message}\n"));
}

#[test]
fn help_is_only_available_as_the_single_argument() {
    assert_eq!(
        invoke(&["--help"], b""),
        (0, USAGE.to_owned(), String::new())
    );
    assert_cli_error(
        &["--current", "1.0.0", "--help"],
        b"",
        "unknown argument \"--help\"",
    );
}

#[test]
fn argument_errors_stop_left_to_right_and_options_consume_values_verbatim() {
    assert_cli_error(
        &["--unknown", "--current"],
        b"",
        "unknown argument \"--unknown\"",
    );
    assert_cli_error(&["--format"], b"", "missing value for --format");
    assert_cli_error(&["--date"], b"", "missing value for --date");
    assert_cli_error(
        &["--current", "--strict"],
        b"",
        "invalid version \"--strict\"",
    );
}

#[test]
fn validation_errors_are_exact_and_follow_the_required_order() {
    let cases: &[(&[&str], &[u8], &str)] = &[
        (&[], b"", "--current is required"),
        (&["--current"], b"", "missing value for --current"),
        (
            &["--current", "1.0.0", "--date"],
            b"",
            "missing value for --date",
        ),
        (
            &["--current", "1.2", "--format", "invalid", "--date", "bad"],
            b"",
            "invalid version \"1.2\"",
        ),
        (
            &["--current", "1.2.3", "--format", "invalid"],
            b"",
            "invalid format \"invalid\"",
        ),
        (
            &["--current", "1.2.3", "--date", "2026-01-01"],
            b"",
            "--date requires --format changelog",
        ),
        (
            &["--current", "1.2.3", "--format", "changelog"],
            b"",
            "--format changelog requires --date",
        ),
        (
            &[
                "--current",
                "1.2.3",
                "--format",
                "changelog",
                "--date",
                "2026-13-01",
            ],
            b"",
            "invalid date \"2026-13-01\"",
        ),
        (
            &["--current", "invalid", "--format", "changelog"],
            &[0xff],
            "invalid version \"invalid\"",
        ),
    ];

    for (args, input, message) in cases {
        assert_cli_error(args, input, message);
    }
}

#[test]
fn repeated_options_keep_their_last_value() {
    let (status, stdout, stderr) = invoke(
        &[
            "--current",
            "not-a-version",
            "--current",
            "1.2.3",
            "--format",
            "invalid",
            "--format",
            "changelog",
            "--date",
            "invalid",
            "--date",
            "2026-02-31",
        ],
        b"fix: repair",
    );

    assert_eq!(status, 0);
    assert_eq!(stdout, "## 1.2.4 - 2026-02-31\n\n### Fixes\n\n- repair\n");
    assert_eq!(stderr, "");
}

#[test]
fn invalid_utf8_stdin_is_reported_without_output() {
    assert_cli_error(
        &["--current", "1.0.0"],
        &[0xff, 0xfe],
        "input is not valid UTF-8",
    );
}

#[test]
fn bom_crlf_and_empty_separators_are_handled_without_a_final_separator() {
    let input = "\u{feff}\r\n----\r\n----\r\nfeat: first\r\n----\r\n \t\r\n----\r\nfix: second";
    let (status, stdout, stderr) = invoke(&["--current", "1.0.0"], input.as_bytes());

    assert_eq!(status, 0);
    assert_eq!(stdout, "1.1.0\n");
    assert_eq!(stderr, "");
}

#[test]
fn strict_mode_numbers_only_counted_commits_and_suppresses_release_output() {
    let input = "\n \t\n----\nfeat: accepted\n----\n \t\n----\nMerge branch 'main'\t ";
    assert_cli_error(
        &["--current", "1.0.0", "--strict"],
        input.as_bytes(),
        "commit 2: not a conventional commit: \"Merge branch 'main'\"",
    );
}

#[test]
fn strict_errors_preserve_leading_header_whitespace_and_trim_only_the_end() {
    assert_cli_error(
        &["--current", "1.0.0", "--strict"],
        b" feat: x \t",
        "commit 1: not a conventional commit: \" feat: x\"",
    );
}

#[test]
fn empty_and_ignored_input_report_no_release() {
    for input in ["", "----\n \t\n----\nMerge branch 'main'"] {
        assert_eq!(
            invoke(&["--current", "1.0.0"], input.as_bytes()),
            (1, "no release\n".to_owned(), String::new())
        );
    }
}

#[test]
fn scope_filter_selects_exact_commits_for_each_output_format() {
    let input = "feat(api)!: drop the v1 endpoints\n----\nfix(db): close the pool\n----\nfeat: add --json flag";
    let cases: &[(&[&str], i32, &str)] = &[
        (&["--current", "1.4.2"], 0, "2.0.0\n"),
        (&["--current", "1.4.2", "--scope", "api"], 0, "2.0.0\n"),
        (&["--current", "1.4.2", "--scope", "db"], 0, "1.4.3\n"),
        (&["--current", "1.4.2", "--scope", "ap"], 1, "no release\n"),
        (&["--current", "1.4.2", "--scope", "ui"], 1, "no release\n"),
        (
            &["--current", "1.4.2", "--scope", "ui", "--format", "json"],
            1,
            "{\"release\":false,\"current\":\"1.4.2\",\"next\":null,\"bump\":null,\"commits\":0,\"breaking\":0,\"features\":0,\"fixes\":0,\"performance\":0}\n",
        ),
        (
            &["--current", "1.4.2", "--scope", "api", "--format", "json"],
            0,
            "{\"release\":true,\"current\":\"1.4.2\",\"next\":\"2.0.0\",\"bump\":\"major\",\"commits\":1,\"breaking\":1,\"features\":1,\"fixes\":0,\"performance\":0}\n",
        ),
        (
            &[
                "--current",
                "1.4.2",
                "--scope",
                "db",
                "--format",
                "changelog",
                "--date",
                "2026-03-01",
            ],
            0,
            "## 1.4.3 - 2026-03-01\n\n### Fixes\n\n- **db:** close the pool\n",
        ),
    ];

    for (args, expected_status, expected_stdout) in cases {
        let (status, stdout, stderr) = invoke(args, input.as_bytes());
        assert_eq!(status, *expected_status, "args: {args:?}");
        assert_eq!(stdout, *expected_stdout, "args: {args:?}");
        assert_eq!(stderr, "", "args: {args:?}");
    }
}

#[test]
fn scope_validation_and_precedence_errors_are_exact() {
    for scope in ["Api", "a_b", "a b", "a.b", ""] {
        assert_cli_error(
            &["--current", "1.2.3", "--scope", scope],
            b"",
            &format!("invalid scope \"{scope}\""),
        );
    }

    assert_cli_error(
        &["--current", "1.2.3", "--scope"],
        b"",
        "missing value for --scope",
    );
    assert_cli_error(
        &["--current", "1.2.3", "--scope=api"],
        b"",
        "unknown argument \"--scope=api\"",
    );
    assert_cli_error(
        &["--current", "1.2.3", "--format", "bad", "--scope", "Api"],
        b"",
        "invalid format \"bad\"",
    );
    assert_cli_error(
        &[
            "--current",
            "1.2.3",
            "--scope",
            "Api",
            "--date",
            "2026-01-01",
        ],
        b"",
        "invalid scope \"Api\"",
    );
}

#[test]
fn repeated_scope_values_use_the_last_value() {
    let input = "feat(api): add paging\n----\nfix(db): close the pool";
    for args in [
        &["--current", "1.0.0", "--scope", "api", "--scope", "db"][..],
        &["--current", "1.0.0", "--scope", "Api", "--scope", "db"][..],
    ] {
        assert_eq!(
            invoke(args, input.as_bytes()),
            (0, "1.0.1\n".to_owned(), String::new())
        );
    }
}

#[test]
fn strict_scope_filter_still_validates_every_input_commit() {
    let input = "feat(api): add paging\n----\nfix(db): close the pool\n----\nMerge branch 'main'";
    assert_cli_error(
        &["--current", "1.4.2", "--scope", "api", "--strict"],
        input.as_bytes(),
        "commit 3: not a conventional commit: \"Merge branch 'main'\"",
    );
}

#[test]
fn scope_names_accept_the_allowed_characters_at_64_bytes() {
    let scope = format!("{}9-z", "a".repeat(61));
    let input = format!("feat({scope}): boundary scope");
    let (status, stdout, stderr) = invoke(
        &["--current", "1.0.0", "--scope", scope.as_str()],
        input.as_bytes(),
    );

    assert_eq!(status, 0);
    assert_eq!(stdout, "1.1.0\n");
    assert_eq!(stderr, "");
}

#[test]
fn scope_filter_selects_readme_example_for_api_changelog_and_db_json() {
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
        "----\n",
        "fix(db): close the pool\n",
        "----\n",
        "feat(api): add paging\n",
    );

    let (status, stdout, stderr) = invoke(
        &[
            "--current",
            "1.4.2",
            "--scope",
            "api",
            "--format",
            "changelog",
            "--date",
            "2026-03-01",
        ],
        input.as_bytes(),
    );
    assert_eq!(status, 0);
    assert_eq!(
        stdout,
        "## 2.0.0 - 2026-03-01\n\n### Breaking changes\n\n- **api:** v1 endpoints are removed\n\n### Features\n\n- **api:** drop the v1 endpoints\n- **api:** add paging\n"
    );
    assert_eq!(stderr, "");

    let (status, stdout, stderr) = invoke(
        &["--current", "1.4.2", "--scope", "db", "--format", "json"],
        input.as_bytes(),
    );
    assert_eq!(status, 0);
    assert_eq!(
        stdout,
        "{\"release\":true,\"current\":\"1.4.2\",\"next\":\"1.4.3\",\"bump\":\"patch\",\"commits\":1,\"breaking\":0,\"features\":0,\"fixes\":1,\"performance\":0}\n"
    );
    assert_eq!(stderr, "");
}
