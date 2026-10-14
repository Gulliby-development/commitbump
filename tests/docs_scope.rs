use std::fs;

const SCOPE_FILTER_SECTION: &str = concat!(
    "## Scope filter\n",
    "\n",
    "In a repository that holds several packages, the scope of a header names the package, as in `feat(api): add paging` or `fix(db): close the pool`. With `--scope NAME` commitbump uses only the conventional commits whose scope is exactly `NAME` (lowercase ASCII letters, digits and hyphens, as in the header grammar). Commits without a scope and commits with another scope are left out of the bump, of the changelog and of every count of the JSON output. Only the form `--scope NAME` is accepted, and when the option is repeated the last value is used. `--strict` still checks every commit of the input, whatever its scope.\n",
    "\n",
    "For this commit stream and `--current 1.4.2`:\n",
    "\n",
    "```text\n",
    "feat(api)!: drop the v1 endpoints\n",
    "----\n",
    "fix(db): close the pool\n",
    "----\n",
    "feat: add --json flag\n",
    "```\n",
    "\n",
    "`--scope db` prints `1.4.3`, `--scope api` prints `2.0.0`, and `--scope ui` prints `no release` and exits with code 1. Without `--scope` the result is `2.0.0`.\n",
);

#[test]
fn readme_documents_the_scope_filter_contract() {
    let readme = fs::read_to_string("README.md").expect("README.md should be readable");
    let normalized_readme = readme.replace("\r\n", "\n");
    let section_start = normalized_readme
        .find("## Scope filter\n")
        .expect("README.md should contain the Scope filter section");
    let exit_codes_start = normalized_readme
        .find("\n## Exit codes\n")
        .expect("README.md should contain the Exit codes section");

    assert!(
        section_start < exit_codes_start,
        "the Scope filter section should appear before Exit codes"
    );
    assert_eq!(
        &normalized_readme[section_start..exit_codes_start],
        SCOPE_FILTER_SECTION,
        "README.md should preserve the specified Scope filter section"
    );
}
