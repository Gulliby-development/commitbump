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
Usage: commitbump --current [v]X.Y.Z [--format version|changelog|json|why] [--date YYYY-MM-DD] [--scope NAME] [--strict]
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
| `--current [v]X.Y.Z` | Required. The current version, with an optional leading `v`; see [Tag prefix](#tag-prefix). |
| `--format version\|changelog\|json\|why` | Output format, `version` by default; see [Reasons](#reasons). |
| `--date YYYY-MM-DD` | Release date; required with, and only allowed with, `--format changelog`. |
| `--scope NAME` | Only use commits whose scope is exactly `NAME`; see [Scope filter](#scope-filter). |
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

## JSON output

With `--format json`, commitbump prints one compact object describing the release decision. For the worked commit stream above with `--current 1.4.2`, it prints:

```json
{"release":true,"current":"1.4.2","next":"2.0.0","bump":"major","commits":4,"breaking":1,"features":2,"fixes":1,"performance":0}
```

`release` indicates whether a release is warranted; `current` is the supplied version, and `next` is the computed version or `null` when there is no release. `bump` is the level applied (`major`, `minor`, or `patch`); for a breaking commit on a `0.y.z` version, the existing bump rules apply `minor`. `commits` counts parsed conventional commits, excluding ignored non-conventional headers. `breaking` counts breaking commits, while `features`, `fixes`, and `performance` count commits of types `feat`, `fix`, and `perf`; a breaking `feat` counts in both `breaking` and `features`. When no release is warranted, `release` is `false` and `next` and `bump` are `null`; the command exits with code 1.

## Scope filter

In a repository that holds several packages, the scope of a header names the package, as in `feat(api): add paging` or `fix(db): close the pool`. With `--scope NAME` commitbump uses only the conventional commits whose scope is exactly `NAME` (lowercase ASCII letters, digits and hyphens, as in the header grammar). Commits without a scope and commits with another scope are left out of the bump, of the changelog and of every count of the JSON output. Only the form `--scope NAME` is accepted, and when the option is repeated the last value is used. `--strict` still checks every commit of the input, whatever its scope.

For this commit stream and `--current 1.4.2`:

```text
feat(api)!: drop the v1 endpoints
----
fix(db): close the pool
----
feat: add --json flag
```

`--scope db` prints `1.4.3`, `--scope api` prints `2.0.0`, and `--scope ui` prints `no release` and exits with code 1. Without `--scope` the result is `2.0.0`.

## Exit codes

| Code | Meaning |
| --- | --- |
| `0` | A version or changelog was produced, or `--help` was requested. |
| `1` | No release is warranted. |
| `2` | Arguments, input, or (in strict mode) a commit header are invalid. |

## Tag prefix

Git tags are often written `v1.4.2`. `--current` accepts a lowercase `v` in front of the three numbers, and the prefix is kept in everything the tool prints, so the output can be used as the next tag. With the commit stream of the scope filter section and `--current v1.4.2`, the default format prints `v2.0.0`, the changelog heading is `## v2.0.0 - 2026-03-01` and the JSON fields `current` and `next` read `v1.4.2` and `v2.0.0`. Without the `v` nothing changes. Only a single lowercase `v` is accepted; `V1.4.2`, `vv1.4.2`, `v 1.4.2` and a lone `v` are invalid versions (exit code 2). `no release` is printed as before.

## Reasons

With `--format why`, commitbump says why the version changed. It prints the lines `next` (the next version, with the `v` prefix when `--current` has it) and `bump` (`major`, `minor` or `patch`), then one `because` line for every commit that causes the bump level, in input order: the conventional commits whose own level equals the bump level, shown as their header without trailing spaces and tabs. At most 5 `because` lines are printed; with more causes a last line `more` gives the number left out. The scope filter applies first. When no release is warranted it prints `no release` and exits with code 1, as the other formats do.

For the commit stream of the scope filter section and `--current 1.4.2`:

```text
next     2.0.0
bump     major
because  feat(api)!: drop the v1 endpoints
```

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
