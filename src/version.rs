use std::fmt;

use crate::parse::{Commit, CommitKind};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

impl Version {
    pub fn parse(value: &str) -> Option<Self> {
        let mut components = value.split('.');
        let major = parse_component(components.next()?)?;
        let minor = parse_component(components.next()?)?;
        let patch = parse_component(components.next()?)?;
        if components.next().is_some() {
            return None;
        }

        Some(Self {
            major,
            minor,
            patch,
        })
    }
}

impl fmt::Display for Version {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

fn parse_component(value: &str) -> Option<u64> {
    if value.is_empty()
        || !value.bytes().all(|byte| byte.is_ascii_digit())
        || (value.len() > 1 && value.starts_with('0'))
    {
        return None;
    }

    let component = value.parse::<u64>().ok()?;
    (component <= 999_999).then_some(component)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse_commits;

    fn commit(header: &str) -> Commit {
        parse_commits(header, false).unwrap().pop().unwrap()
    }

    #[test]
    fn parses_only_three_bounded_decimal_components() {
        for value in ["0.0.0", "1.2.3", "999999.999999.999999"] {
            assert!(Version::parse(value).is_some(), "{value}");
        }

        for value in [
            "",
            "1.2",
            "1.2.3.4",
            "01.2.3",
            "1.02.3",
            "1.2.03",
            "1.2.3-beta",
            "v1.2.3",
            "1000000.0.0",
            "1.1000000.0",
            "1.2.1000000",
            "١.2.3",
        ] {
            assert_eq!(Version::parse(value), None, "{value}");
        }
    }

    #[test]
    fn displays_versions_with_decimal_components() {
        assert_eq!(
            Version {
                major: 1,
                minor: 4,
                patch: 2,
            }
            .to_string(),
            "1.4.2"
        );
    }

    #[test]
    fn applies_the_major_one_bump_matrix() {
        let current = Version {
            major: 1,
            minor: 4,
            patch: 2,
        };

        assert_eq!(
            bump_level(current, &[commit("chore!: breaking")]),
            Some(Bump::Major)
        );
        assert_eq!(
            bump_level(current, &[commit("feat: feature")]),
            Some(Bump::Minor)
        );
        assert_eq!(
            bump_level(current, &[commit("fix: fix")]),
            Some(Bump::Patch)
        );
        assert_eq!(
            bump_level(current, &[commit("perf: faster")]),
            Some(Bump::Patch)
        );

        for header in [
            "docs: documentation",
            "refactor: cleanup",
            "test: coverage",
            "build: packaging",
            "ci: pipeline",
            "chore: maintenance",
            "style: formatting",
            "revert: undo",
        ] {
            assert_eq!(bump_level(current, &[commit(header)]), None, "{header}");
        }
    }

    #[test]
    fn applies_the_zero_major_bump_matrix() {
        let current = Version {
            major: 0,
            minor: 9,
            patch: 1,
        };

        assert_eq!(
            bump_level(current, &[commit("chore!: breaking")]),
            Some(Bump::Minor)
        );
        assert_eq!(
            bump_level(current, &[commit("feat: feature")]),
            Some(Bump::Patch)
        );
        assert_eq!(
            bump_level(current, &[commit("fix: fix")]),
            Some(Bump::Patch)
        );
        assert_eq!(
            bump_level(current, &[commit("perf: faster")]),
            Some(Bump::Patch)
        );
        assert_eq!(bump_level(current, &[commit("docs: documentation")]), None);
    }

    #[test]
    fn selects_the_highest_level_and_allows_the_next_major_to_exceed_the_limit() {
        let current = Version {
            major: 1,
            minor: 4,
            patch: 2,
        };
        let commits = [
            commit("fix: fix"),
            commit("feat: feature"),
            commit("chore!: breaking"),
        ];
        assert_eq!(bump_level(current, &commits), Some(Bump::Major));
        assert_eq!(
            next_version(
                Version {
                    major: 999_999,
                    minor: 0,
                    patch: 0,
                },
                Bump::Major
            ),
            Version {
                major: 1_000_000,
                minor: 0,
                patch: 0,
            }
        );
    }

    #[test]
    fn increments_each_version_component_for_its_bump_level() {
        let current = Version {
            major: 1,
            minor: 4,
            patch: 2,
        };
        assert_eq!(
            next_version(current, Bump::Major),
            Version {
                major: 2,
                minor: 0,
                patch: 0,
            }
        );
        assert_eq!(
            next_version(current, Bump::Minor),
            Version {
                major: 1,
                minor: 5,
                patch: 0,
            }
        );
        assert_eq!(
            next_version(current, Bump::Patch),
            Version {
                major: 1,
                minor: 4,
                patch: 3,
            }
        );
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Bump {
    Patch,
    Minor,
    Major,
}

/// Chooses the highest bump required by the supplied conventional commits.
pub fn bump_level(current: Version, commits: &[Commit]) -> Option<Bump> {
    commits
        .iter()
        .filter_map(|commit| bump_for_commit(current, commit))
        .max()
}

fn bump_for_commit(current: Version, commit: &Commit) -> Option<Bump> {
    if commit.breaking {
        return Some(if current.major == 0 {
            Bump::Minor
        } else {
            Bump::Major
        });
    }

    match commit.kind {
        CommitKind::Feat => Some(if current.major == 0 {
            Bump::Patch
        } else {
            Bump::Minor
        }),
        CommitKind::Fix | CommitKind::Perf => Some(Bump::Patch),
        _ => None,
    }
}

/// Applies a bump level. A resulting component may exceed the input validation limit.
pub fn next_version(current: Version, bump: Bump) -> Version {
    match bump {
        Bump::Major => Version {
            major: current.major + 1,
            minor: 0,
            patch: 0,
        },
        Bump::Minor => Version {
            major: current.major,
            minor: current.minor + 1,
            patch: 0,
        },
        Bump::Patch => Version {
            major: current.major,
            minor: current.minor,
            patch: current.patch + 1,
        },
    }
}
