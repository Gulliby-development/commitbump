use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommitKind {
    Feat,
    Fix,
    Perf,
    Docs,
    Refactor,
    Test,
    Build,
    Ci,
    Chore,
    Style,
    Revert,
}

impl CommitKind {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "feat" => Some(Self::Feat),
            "fix" => Some(Self::Fix),
            "perf" => Some(Self::Perf),
            "docs" => Some(Self::Docs),
            "refactor" => Some(Self::Refactor),
            "test" => Some(Self::Test),
            "build" => Some(Self::Build),
            "ci" => Some(Self::Ci),
            "chore" => Some(Self::Chore),
            "style" => Some(Self::Style),
            "revert" => Some(Self::Revert),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Commit {
    pub header: String,
    pub kind: CommitKind,
    pub scope: Option<String>,
    pub description: String,
    pub breaking: bool,
    pub notes: Vec<String>,
    pub issues: Vec<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseError {
    pub commit_number: usize,
    pub header: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "commit {}: not a conventional commit: \"{}\"",
            self.commit_number, self.header
        )
    }
}

impl std::error::Error for ParseError {}

/// Parses commits from UTF-8 input, optionally rejecting the first nonconventional commit.
pub fn parse_commits(input: &str, strict: bool) -> Result<Vec<Commit>, ParseError> {
    let input = input.strip_prefix('\u{feff}').unwrap_or(input);
    let mut commits = Vec::new();
    let mut current_lines = Vec::new();
    let mut commit_number = 0;

    for raw_line in input.split_inclusive('\n') {
        let line = match raw_line.strip_suffix('\n') {
            Some(line) => line.strip_suffix('\r').unwrap_or(line),
            None => raw_line,
        };

        if line == "----" {
            process_commit(&current_lines, strict, &mut commit_number, &mut commits)?;
            current_lines.clear();
        } else {
            current_lines.push(line);
        }
    }

    process_commit(&current_lines, strict, &mut commit_number, &mut commits)?;
    Ok(commits)
}

fn process_commit(
    lines: &[&str],
    strict: bool,
    commit_number: &mut usize,
    commits: &mut Vec<Commit>,
) -> Result<(), ParseError> {
    let Some(header_index) = lines.iter().position(|line| !is_blank(line)) else {
        return Ok(());
    };

    *commit_number += 1;
    let header = lines[header_index];
    let body = &lines[header_index + 1..];

    if let Some(commit) = parse_conventional_commit(header, body) {
        commits.push(commit);
    } else if strict {
        return Err(ParseError {
            commit_number: *commit_number,
            header: header.trim_end_matches([' ', '\t']).to_owned(),
        });
    }

    Ok(())
}

fn parse_conventional_commit(header: &str, body: &[&str]) -> Option<Commit> {
    let bytes = header.as_bytes();
    let type_end = bytes
        .iter()
        .position(|byte| matches!(byte, b'(' | b'!' | b':'))
        .unwrap_or(bytes.len());
    let kind = CommitKind::parse(&header[..type_end])?;
    let mut cursor = type_end;
    let scope = if bytes.get(cursor) == Some(&b'(') {
        let scope_start = cursor + 1;
        let scope_end = bytes[scope_start..].iter().position(|byte| *byte == b')')? + scope_start;
        let scope_bytes = &bytes[scope_start..scope_end];
        if scope_bytes.is_empty()
            || !scope_bytes
                .iter()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
        {
            return None;
        }
        cursor = scope_end + 1;
        Some(header[scope_start..scope_end].to_owned())
    } else {
        None
    };

    let breaking_marker = bytes.get(cursor) == Some(&b'!');
    if breaking_marker {
        cursor += 1;
    }

    if !bytes[cursor..].starts_with(b": ") {
        return None;
    }
    let description = trim_ascii_horizontal(&header[cursor + 2..]);
    if description.is_empty() {
        return None;
    }

    let mut notes: Vec<String> = body
        .iter()
        .filter_map(|line| {
            let note = line
                .strip_prefix("BREAKING CHANGE: ")
                .or_else(|| line.strip_prefix("BREAKING-CHANGE: "))?;
            let note = trim_ascii_horizontal(note);
            (!note.is_empty()).then(|| note.to_owned())
        })
        .collect();
    if notes.is_empty() && breaking_marker {
        notes.push(description.to_owned());
    }

    let mut issues: Vec<u32> = Vec::new();
    for line in body {
        if let Some(number) = issue_number(line)
            && !issues.contains(&number)
        {
            issues.push(number);
        }
    }

    Some(Commit {
        header: header.to_owned(),
        kind,
        scope,
        description: description.to_owned(),
        breaking: breaking_marker || !notes.is_empty(),
        notes,
        issues,
    })
}

/// Reads `Closes #N`, `Fixes #N` or `Resolves #N` with N from 1 to 999999 and no leading zero.
fn issue_number(line: &str) -> Option<u32> {
    let rest = ["Closes #", "Fixes #", "Resolves #"]
        .iter()
        .find_map(|prefix| line.strip_prefix(prefix))?;
    let digits = rest.trim_end_matches([' ', '\t']);
    if digits.is_empty()
        || digits.len() > 6
        || digits.starts_with('0')
        || !digits.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    digits.parse().ok()
}

fn is_blank(line: &str) -> bool {
    trim_ascii_horizontal(line).is_empty()
}

fn trim_ascii_horizontal(value: &str) -> &str {
    value.trim_matches(|character| character == ' ' || character == '\t')
}

/// Returns whether a date has the required YYYY-MM-DD shape and numeric ranges.
pub fn is_valid_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || !bytes[..4].iter().all(u8::is_ascii_digit)
        || !bytes[5..7].iter().all(u8::is_ascii_digit)
        || !bytes[8..10].iter().all(u8::is_ascii_digit)
    {
        return false;
    }

    let month = u16::from(bytes[5] - b'0') * 10 + u16::from(bytes[6] - b'0');
    let day = u16::from(bytes[8] - b'0') * 10 + u16::from(bytes[9] - b'0');
    (1..=12).contains(&month) && (1..=31).contains(&day)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_header(header: &str) -> Option<Commit> {
        parse_conventional_commit(header, &[])
    }

    #[test]
    fn accepts_every_supported_type_and_valid_optional_syntax() {
        for kind in [
            "feat", "fix", "perf", "docs", "refactor", "test", "build", "ci", "chore", "style",
            "revert",
        ] {
            assert!(
                parse_header(&format!("{kind}: description")).is_some(),
                "{kind}"
            );
        }

        let scoped = parse_header("feat(db-2)!:  add rows: safely \t").unwrap();
        assert_eq!(scoped.kind, CommitKind::Feat);
        assert_eq!(scoped.scope.as_deref(), Some("db-2"));
        assert_eq!(scoped.description, "add rows: safely");
        assert!(scoped.breaking);

        for header in ["fix(a): x", "fix(a--9): x", "perf(9-a)!: faster"] {
            assert!(parse_header(header).is_some(), "{header}");
        }
    }

    #[test]
    fn rejects_headers_outside_the_closed_grammar() {
        for header in [
            "Feat: x",
            "feat:x",
            "feat(): x",
            "feat(a b): x",
            " feat: x",
            "Merge branch 'main'",
            "unknown: x",
            "feat : x",
            "feat(scope):",
            "feat(scope): \t ",
            "feat(scope)!!: x",
            "feat(UP): x",
            "feat(a_b): x",
            "feat(a):x",
            "feat(scope: x",
        ] {
            assert_eq!(parse_header(header), None, "{header:?}");
        }
    }

    #[test]
    fn parses_footer_notes_trims_them_and_ignores_empty_notes() {
        let commit = parse_conventional_commit(
            "fix: repair",
            &[
                "BREAKING CHANGE:  remove old behavior \t",
                "BREAKING-CHANGE:  rename field ",
                "BREAKING CHANGE:   ",
            ],
        )
        .unwrap();

        assert!(commit.breaking);
        assert_eq!(commit.notes, ["remove old behavior", "rename field"]);
    }

    #[test]
    fn uses_the_header_description_as_fallback_for_a_bang() {
        let commit = parse_conventional_commit("feat!:  replace endpoint  ", &[]).unwrap();
        assert!(commit.breaking);
        assert_eq!(commit.description, "replace endpoint");
        assert_eq!(commit.notes, ["replace endpoint"]);

        let footer_commit =
            parse_conventional_commit("docs: note in body", &["BREAKING CHANGE: changed"]).unwrap();
        assert!(footer_commit.breaking);
        assert_eq!(footer_commit.notes, ["changed"]);
    }

    #[test]
    fn empty_footer_notes_do_not_break_a_commit_without_a_bang() {
        let commit = parse_conventional_commit(
            "fix: harmless",
            &["BREAKING CHANGE: ", "BREAKING-CHANGE: \t "],
        )
        .unwrap();
        assert!(!commit.breaking);
        assert!(commit.notes.is_empty());
    }

    #[test]
    fn parses_bom_crlf_separators_and_missing_final_separator() {
        let commits =
            parse_commits("\u{feff}\r\nfeat: first\r\n----\r\nfix: second", false).unwrap();
        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].description, "first");
        assert_eq!(commits[1].description, "second");
    }

    #[test]
    fn skips_blank_commits_and_counts_only_nonblank_commits_for_strict_errors() {
        let error = parse_commits(
            "\n \t\nfeat: accepted\n----\n \t\n----\nMerge branch 'main' \t\n",
            true,
        )
        .unwrap_err();
        assert_eq!(error.commit_number, 2);
        assert_eq!(error.header, "Merge branch 'main'");
    }

    #[test]
    fn treats_lone_carriage_return_as_header_data_and_delimiters_as_exact_lines() {
        let commits = parse_commits("feat: one\r----\nfeat: two", false).unwrap();
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].description, "one\r----");

        let commits = parse_commits("feat: one\n ----\nfeat: two", false).unwrap();
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].description, "one");
    }

    #[test]
    fn validates_date_shape_and_ranges_without_calendar_checks() {
        for value in ["0000-01-01", "2026-03-01", "2026-02-31", "2026-12-31"] {
            assert!(is_valid_date(value), "{value}");
        }

        for value in [
            "",
            "2026-3-01",
            "2026-03-1",
            "2026/03/01",
            "2026-00-01",
            "2026-13-01",
            "2026-01-00",
            "2026-01-32",
            "２０２６-03-01",
            "2026-03-01x",
        ] {
            assert!(!is_valid_date(value), "{value}");
        }
    }
}
