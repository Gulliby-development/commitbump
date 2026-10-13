<p align="center"><img src="docs/logo.svg" width="96" alt="" /></p>

# commitbump

A small Rust command-line tool that picks the next semantic version from Conventional Commits and can write a release changelog.

[![CI](https://github.com/Gulliby-development/commitbump/actions/workflows/ci.yml/badge.svg)](https://github.com/Gulliby-development/commitbump/actions/workflows/ci.yml) [![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## Why

You maintain a project that uses Conventional Commits and want a repeatable answer to "what is the next version, and what goes in the release notes?" without a heavyweight release tool. commitbump reads commit messages from standard input, uses only the Rust standard library and runs on Windows, macOS and Linux.

## Install

Requires a Rust toolchain (edition 2024, Rust 1.85 or newer). Install from a local checkout with:

```sh
cargo install --path .
```

## Usage

```text
Usage: commitbump --current X.Y.Z [--format version|changelog] [--date YYYY-MM-DD] [--strict]
```

The tool reads commit messages from standard input. The default output format is `version`; use `--format changelog` with a date to render release notes. Use `--strict` to stop at the first non-conventional commit instead of ignoring it.

Examples:

```sh
git log v1.4.2..HEAD --format='%B%n----' | commitbump --current 1.4.2
git log v1.4.2..HEAD --format='%B%n----' | commitbump --current 1.4.2 --format changelog --date 2026-03-01
commitbump --help
```

### Options

| Option | Description |
| --- | --- |
| `--current X.Y.Z` | Required. The current version. |
| `--format version\|changelog` | Output format, `version` by default. |
| `--date YYYY-MM-DD` | Release date; required with, and only allowed with, `--format changelog`. |
| `--strict` | Fail with exit code 2 at the first non-conventional commit instead of ignoring it. |
| `--help` | Print the usage line. |

## Header grammar

A conventional header has this shape:

```text
type(scope)!: description
```

The type must be one of `feat`, `fix`, `perf`, `docs`, `refactor`, `test`, `build`, `ci`, `chore`, `style`, or `revert`. The scope is optional and, when present, contains one or more lowercase ASCII letters, digits, or hyphens. The exclamation mark is optional. A colon and one space separate the header from a nonempty description. Descriptions are trimmed of spaces and tabs.

Commits are separated by a line containing exactly `----`. Lines beginning with `BREAKING CHANGE: ` or `BREAKING-CHANGE: ` in a commit body add breaking notes. If there are no nonempty notes, a header ending in `!` uses its description as the breaking note.

## Bump rules

The highest applicable change across all conventional commits determines the bump:

| Current major | Breaking commit | `feat` | `fix` or `perf` | Other types |
| --- | --- | --- | --- | --- |
| `0` | minor | patch | patch | no release |
| `1` or higher | major | minor | patch | no release |

A breaking `feat` contributes to both the breaking-changes and features sections of a changelog. Version components must be three decimal numbers; a next version can exceed the input limit of 999999.

## Output

The default `version` format prints only the next version. If no commit warrants a release, either format prints `no release`. Changelogs include only nonempty groups, in this order: Breaking changes, Features, Fixes, and Performance. Scoped entries use bullets such as `- **api:** update endpoint`.

## Exit codes

| Code | Meaning |
| --- | --- |
| `0` | A version or changelog was produced, or `--help` was requested. |
| `1` | No release is warranted. |
| `2` | Arguments, input, or (in strict mode) a commit header are invalid. |

## Example

For the following commit stream:

```text
feat(api)!: drop the v1 endpoints

BREAKING CHANGE: v1 endpoints are removed
----
fix: handle empty input
----
Merge branch 'main'
----
feat: add --json flag
----
docs: update readme
```

With `--current 1.4.2 --format changelog --date 2026-03-01`, the output is:

```text
## 2.0.0 - 2026-03-01

### Breaking changes

- **api:** v1 endpoints are removed

### Features

- **api:** drop the v1 endpoints
- add --json flag

### Fixes

- handle empty input
```

## Development

Build and check the project with:

```sh
cargo build
cargo test
cargo fmt --check
cargo clippy -- -D warnings
```

## License

MIT, see [LICENSE](LICENSE).

---

Built with BAKED, an open-source coding agent (bench loop round 9, BAKED a5870fe8).
