use std::io::Cursor;

use commitbump::run;

const WORKED_EXAMPLE: &str = "feat(api)!: drop the v1 endpoints\n\nBREAKING CHANGE: v1 endpoints are removed\n----\nfix: handle empty input\n----\nMerge branch 'main'\n----\nfeat: add --json flag\n----\ndocs: update readme";

fn invoke(args: &[&str], input: &str) -> (i32, String, String) {
    let args = args
        .iter()
        .map(|argument| (*argument).to_owned())
        .collect::<Vec<_>>();
    let mut stdin = Cursor::new(input.as_bytes());
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let status = run(&args, &mut stdin, &mut stdout, &mut stderr);
    (
        status,
        String::from_utf8(stdout).unwrap(),
        String::from_utf8(stderr).unwrap(),
    )
}

#[test]
fn worked_example_renders_the_exact_changelog_and_respects_zero_major_rules() {
    assert_eq!(
        invoke(&["--current", "1.4.2"], WORKED_EXAMPLE),
        (0, "2.0.0\n".to_owned(), String::new())
    );
    assert_eq!(
        invoke(&["--current", "0.9.1"], WORKED_EXAMPLE),
        (0, "0.10.0\n".to_owned(), String::new())
    );

    let expected = concat!(
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
        "- handle empty input\n",
    );
    assert_eq!(
        invoke(
            &[
                "--current",
                "1.4.2",
                "--format",
                "changelog",
                "--date",
                "2026-03-01",
            ],
            WORKED_EXAMPLE
        ),
        (0, expected.to_owned(), String::new())
    );
    assert_eq!(
        invoke(&["--current", "1.4.2", "--strict"], WORKED_EXAMPLE),
        (
            2,
            String::new(),
            "commitbump: commit 3: not a conventional commit: \"Merge branch 'main'\"\n".to_owned()
        )
    );
}

#[test]
fn bump_levels_follow_major_and_zero_major_precedence() {
    let cases = [
        (&["--current", "1.4.2"][..], "fix: repair", "1.4.3"),
        (&["--current", "1.4.2"][..], "feat: add option", "1.5.0"),
        (
            &["--current", "1.4.2"][..],
            "chore!: change contract",
            "2.0.0",
        ),
        (
            &["--current", "1.4.2"][..],
            "fix: repair\n----\nfeat: add option\n----\nchore!: change contract",
            "2.0.0",
        ),
        (
            &["--current", "0.4.2"][..],
            "fix: repair\n----\nfeat: add option",
            "0.4.3",
        ),
        (
            &["--current", "0.2.3"][..],
            "chore!: change contract",
            "0.3.0",
        ),
        (&["--current", "0.9.1"][..], "feat: add option", "0.9.2"),
        (
            &["--current", "999999.0.0"][..],
            "feat!: cross version limit",
            "1000000.0.0",
        ),
        (
            &["--current", "0.9.1"][..],
            "fix: repair\n----\nperf: faster",
            "0.9.2",
        ),
    ];

    for (args, input, expected) in cases {
        assert_eq!(
            invoke(args, input),
            (0, format!("{expected}\n"), String::new()),
            "current args: {args:?}; input: {input:?}"
        );
    }
}

#[test]
fn changelog_orders_all_groups_and_preserves_scopes_notes_and_breaking_features() {
    let input = concat!(
        "feat(api)!: drop v1\n",
        "BREAKING CHANGE: remove old endpoint\n",
        "BREAKING-CHANGE: rename response field\n",
        "----\n",
        "feat: add a flag\n",
        "----\n",
        "fix: handle empty input\n",
        "----\n",
        "perf(db): cache queries",
    );
    let expected = concat!(
        "## 2.0.0 - 2026-03-01\n",
        "\n",
        "### Breaking changes\n",
        "\n",
        "- **api:** remove old endpoint\n",
        "- **api:** rename response field\n",
        "\n",
        "### Features\n",
        "\n",
        "- **api:** drop v1\n",
        "- add a flag\n",
        "\n",
        "### Fixes\n",
        "\n",
        "- handle empty input\n",
        "\n",
        "### Performance\n",
        "\n",
        "- **db:** cache queries\n",
    );

    assert_eq!(
        invoke(
            &[
                "--current",
                "1.4.2",
                "--format",
                "changelog",
                "--date",
                "2026-03-01",
            ],
            input
        ),
        (0, expected.to_owned(), String::new())
    );
}

#[test]
fn changelog_preserves_repeated_breaking_notes() {
    let input = concat!(
        "feat(api)!: replace the API\n",
        "BREAKING CHANGE: same change\n",
        "BREAKING-CHANGE: same change",
    );
    let expected = concat!(
        "## 2.0.0 - 2026-03-01\n",
        "\n",
        "### Breaking changes\n",
        "\n",
        "- **api:** same change\n",
        "- **api:** same change\n",
        "\n",
        "### Features\n",
        "\n",
        "- **api:** replace the API\n",
    );

    assert_eq!(
        invoke(
            &[
                "--current",
                "1.4.2",
                "--format",
                "changelog",
                "--date",
                "2026-03-01",
            ],
            input
        ),
        (0, expected.to_owned(), String::new())
    );
}

#[test]
fn patch_and_performance_changelog_groups_keep_their_exact_spacing() {
    let input = "fix: a\n----\nperf(db): faster";
    let expected = concat!(
        "## 1.4.3 - 2026-03-01\n",
        "\n",
        "### Fixes\n",
        "\n",
        "- a\n",
        "\n",
        "### Performance\n",
        "\n",
        "- **db:** faster\n",
    );

    assert_eq!(
        invoke(
            &[
                "--current",
                "1.4.2",
                "--format",
                "changelog",
                "--date",
                "2026-03-01",
            ],
            input
        ),
        (0, expected.to_owned(), String::new())
    );
}

#[test]
fn empty_or_non_bumping_commits_produce_no_release_in_both_formats() {
    assert_eq!(
        invoke(&["--current", "1.4.2"], ""),
        (1, "no release\n".to_owned(), String::new())
    );
    assert_eq!(
        invoke(
            &["--current", "1.4.2"],
            "docs: update readme\n----\nchore: cleanup"
        ),
        (1, "no release\n".to_owned(), String::new())
    );
    assert_eq!(
        invoke(
            &[
                "--current",
                "1.4.2",
                "--format",
                "changelog",
                "--date",
                "2026-03-01",
            ],
            "docs: update readme\n----\nchore: cleanup"
        ),
        (1, "no release\n".to_owned(), String::new())
    );
}
