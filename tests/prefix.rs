use std::fs;
use std::io::Cursor;

use commitbump::run;

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

#[test]
fn preserves_prefix_in_version_format() {
    let input = concat!(
        "feat(api)!: drop the v1 endpoints\n",
        "----\n",
        "fix(db): close the pool\n",
        "----\n",
        "feat: add --json flag\n",
    );
    let cases: &[(&[&str], i32, &str)] = &[
        (&["--current", "v1.4.2"], 0, "v2.0.0\n"),
        (&["--current", "1.4.2"], 0, "2.0.0\n"),
        (&["--current", "v0.9.1"], 0, "v0.10.0\n"),
    ];

    for (args, expected_status, expected_stdout) in cases {
        assert_eq!(
            invoke(args, input.as_bytes()),
            (
                *expected_status,
                (*expected_stdout).to_owned(),
                String::new()
            ),
            "args: {args:?}"
        );
    }
}

#[test]
fn preserves_prefix_in_changelog_heading() {
    let input = "fix(db): close the pool";
    let prefixed = invoke(
        &[
            "--current",
            "v1.4.2",
            "--format",
            "changelog",
            "--date",
            "2026-03-01",
            "--scope",
            "db",
        ],
        input.as_bytes(),
    );
    let unprefixed = invoke(
        &[
            "--current",
            "1.4.2",
            "--format",
            "changelog",
            "--date",
            "2026-03-01",
            "--scope",
            "db",
        ],
        input.as_bytes(),
    );

    assert_eq!(
        prefixed,
        (
            0,
            "## v1.4.3 - 2026-03-01\n\n### Fixes\n\n- **db:** close the pool\n".to_owned(),
            String::new()
        )
    );
    assert_eq!(
        unprefixed,
        (
            0,
            "## 1.4.3 - 2026-03-01\n\n### Fixes\n\n- **db:** close the pool\n".to_owned(),
            String::new()
        )
    );
}

#[test]
fn preserves_prefix_in_json_current_and_next_fields() {
    let input = "fix(db): close the pool";
    let prefixed = invoke(
        &["--current", "v1.4.2", "--format", "json", "--scope", "db"],
        input.as_bytes(),
    );
    let unprefixed = invoke(
        &["--current", "1.4.2", "--format", "json", "--scope", "db"],
        input.as_bytes(),
    );

    assert_eq!(
        prefixed,
        (
            0,
            "{\"release\":true,\"current\":\"v1.4.2\",\"next\":\"v1.4.3\",\"bump\":\"patch\",\"commits\":1,\"breaking\":0,\"features\":0,\"fixes\":1,\"performance\":0}\n".to_owned(),
            String::new()
        )
    );
    assert_eq!(
        unprefixed,
        (
            0,
            "{\"release\":true,\"current\":\"1.4.2\",\"next\":\"1.4.3\",\"bump\":\"patch\",\"commits\":1,\"breaking\":0,\"features\":0,\"fixes\":1,\"performance\":0}\n".to_owned(),
            String::new()
        )
    );
}

#[test]
fn keeps_no_release_output_unchanged() {
    assert_eq!(
        invoke(&["--current", "v1.4.2"], b"docs: update guide"),
        (1, "no release\n".to_owned(), String::new())
    );
    assert_eq!(
        invoke(&["--current", "1.4.2"], b"docs: update guide"),
        (1, "no release\n".to_owned(), String::new())
    );
    assert_eq!(
        invoke(&["--current", "v1.4.2", "--format", "json"], b"docs: update guide"),
        (
            1,
            "{\"release\":false,\"current\":\"v1.4.2\",\"next\":null,\"bump\":null,\"commits\":1,\"breaking\":0,\"features\":0,\"fixes\":0,\"performance\":0}\n".to_owned(),
            String::new()
        )
    );
}

#[test]
fn rejects_invalid_prefixed_versions_verbatim() {
    for value in [
        "V1.4.2",
        "vv1.4.2",
        "v",
        "v1.4",
        "v 1.4.2",
        "v01.4.2",
        "v1.4.2 ",
        "v-1.4.2",
        "v1000000.1.1",
        "v1.1000000.1",
        "v1.1.1000000",
    ] {
        assert_eq!(
            invoke(&["--current", value], b""),
            (
                2,
                String::new(),
                format!("commitbump: invalid version \"{value}\"\n")
            ),
            "value: {value:?}"
        );
    }

    assert_eq!(
        invoke(&["--current", "v", "--format", "invalid"], &[0xff]),
        (
            2,
            String::new(),
            "commitbump: invalid version \"v\"\n".to_owned()
        )
    );
}

#[test]
fn uses_the_last_current_value_with_or_without_prefix() {
    let input = b"feat: add capability";

    assert_eq!(
        invoke(&["--current", "v1.4.2", "--current", "1.4.2"], input),
        (0, "1.5.0\n".to_owned(), String::new())
    );
    assert_eq!(
        invoke(&["--current", "1.4.2", "--current", "v1.4.2"], input),
        (0, "v1.5.0\n".to_owned(), String::new())
    );
}

#[test]
fn preserves_prefix_in_scoped_examples() {
    let input = concat!(
        "feat(api)!: drop the v1 endpoints\n",
        "----\n",
        "fix(db): close the pool\n",
        "----\n",
        "feat: add --json flag",
    );

    assert_eq!(
        invoke(&["--current", "v1.4.2", "--scope", "db"], input.as_bytes()),
        (0, "v1.4.3\n".to_owned(), String::new())
    );
    assert_eq!(
        invoke(
            &[
                "--current",
                "v1.4.2",
                "--scope",
                "ui",
                "--format",
                "json",
            ],
            input.as_bytes()
        ),
        (
            1,
            "{\"release\":false,\"current\":\"v1.4.2\",\"next\":null,\"bump\":null,\"commits\":0,\"breaking\":0,\"features\":0,\"fixes\":0,\"performance\":0}\n".to_owned(),
            String::new()
        )
    );
}

#[test]
fn accepts_zero_and_maximum_numeric_components_with_prefix() {
    assert_eq!(
        invoke(&["--current", "v0.0.0"], b"feat: first feature"),
        (0, "v0.0.1\n".to_owned(), String::new())
    );
    assert_eq!(
        invoke(
            &["--current", "v999999.999999.999999"],
            b"feat: maximum input",
        ),
        (0, "v999999.1000000.0\n".to_owned(), String::new())
    );
}

#[test]
fn documents_the_prefix_in_help_and_readme() {
    let usage = "Usage: commitbump --current [v]X.Y.Z [--format version|changelog|json|why] [--date YYYY-MM-DD] [--scope NAME] [--issues] [--strict]";
    assert_eq!(
        invoke(&["--help"], b""),
        (0, format!("{usage}\n"), String::new())
    );

    let readme = fs::read_to_string("README.md").expect("README.md should be readable");
    let tag_prefix_section = concat!(
        "## Tag prefix\n\n",
        "Git tags are often written `v1.4.2`. `--current` accepts a lowercase `v` in front of the three numbers, and the prefix is kept in everything the tool prints, so the output can be used as the next tag. With the commit stream of the scope filter section and `--current v1.4.2`, the default format prints `v2.0.0`, the changelog heading is `## v2.0.0 - 2026-03-01` and the JSON fields `current` and `next` read `v1.4.2` and `v2.0.0`. Without the `v` nothing changes. Only a single lowercase `v` is accepted; `V1.4.2`, `vv1.4.2`, `v 1.4.2` and a lone `v` are invalid versions (exit code 2). `no release` is printed as before.\n"
    );

    assert!(readme.contains(usage));
    assert!(readme.contains(
        "| `--current [v]X.Y.Z` | Required. The current version, with an optional leading `v`; see [Tag prefix](#tag-prefix). |"
    ));
    assert!(readme.contains(tag_prefix_section));
}
