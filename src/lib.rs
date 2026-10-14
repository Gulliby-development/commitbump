pub mod parse;
pub mod render;
pub mod version;

use std::io::{Read, Write};

use parse::parse_commits;
use render::{changelog, summary_json};
use version::{Version, bump_level, next_version};

const USAGE: &str = "Usage: commitbump --current [v]X.Y.Z [--format version|changelog|json] [--date YYYY-MM-DD] [--scope NAME] [--strict]";

#[derive(Default)]
struct Options {
    current: Option<String>,
    format: Option<String>,
    date: Option<String>,
    scope: Option<String>,
    strict: bool,
}

/// Runs commitbump with injectable arguments and standard streams.
///
/// Returns the process exit code specified by the CLI contract.
pub fn run(
    args: &[String],
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    if args.len() == 1 && args[0] == "--help" {
        let _ = writeln!(stdout, "{USAGE}");
        return 0;
    }

    let options = match parse_args(args) {
        Ok(options) => options,
        Err(message) => return report_error(stderr, &message),
    };

    let Some(current_value) = options.current.as_deref() else {
        return report_error(stderr, "--current is required");
    };
    let prefixed = current_value.starts_with('v');
    let numeric_current_value = current_value.strip_prefix('v').unwrap_or(current_value);
    let Some(current) = Version::parse(numeric_current_value) else {
        return report_error(stderr, &format!("invalid version \"{current_value}\""));
    };

    let format = options.format.as_deref().unwrap_or("version");
    if format != "version" && format != "changelog" && format != "json" {
        return report_error(stderr, &format!("invalid format \"{format}\""));
    }
    if let Some(scope) = options.scope.as_deref()
        && !valid_scope(scope)
    {
        return report_error(stderr, &format!("invalid scope \"{scope}\""));
    }

    if options.date.is_some() && format != "changelog" {
        return report_error(stderr, "--date requires --format changelog");
    }
    if format == "changelog" && options.date.is_none() {
        return report_error(stderr, "--format changelog requires --date");
    }
    if let Some(date) = options.date.as_deref()
        && !valid_date(date)
    {
        return report_error(stderr, &format!("invalid date \"{date}\""));
    }

    let mut input = String::new();
    if stdin.read_to_string(&mut input).is_err() {
        return report_error(stderr, "input is not valid UTF-8");
    }

    let mut commits = match parse_commits(&input, options.strict) {
        Ok(commits) => commits,
        Err(error) => return report_error(stderr, &error.to_string()),
    };
    if let Some(scope) = options.scope.as_deref() {
        commits.retain(|commit| commit.scope.as_deref() == Some(scope));
    }
    let Some(bump) = bump_level(current, &commits) else {
        if format == "json" {
            let _ = writeln!(
                stdout,
                "{}",
                summary_json(current, prefixed, None, &commits)
            );
        } else {
            let _ = writeln!(stdout, "no release");
        }
        return 1;
    };
    let next = next_version(current, bump);

    if format == "version" {
        let prefix = if prefixed { "v" } else { "" };
        let _ = writeln!(stdout, "{prefix}{next}");
    } else if format == "json" {
        let _ = writeln!(
            stdout,
            "{}",
            summary_json(current, prefixed, Some((next, bump)), &commits)
        );
    } else if let Some(date) = options.date.as_deref() {
        let _ = writeln!(stdout, "{}", changelog(next, prefixed, date, &commits));
    }

    0
}

fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut options = Options::default();
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--current" | "--format" | "--date" | "--scope" => {
                let option = args[index].as_str();
                let Some(value) = args.get(index + 1) else {
                    return Err(format!("missing value for {option}"));
                };
                match option {
                    "--current" => options.current = Some(value.clone()),
                    "--format" => options.format = Some(value.clone()),
                    "--date" => options.date = Some(value.clone()),
                    "--scope" => options.scope = Some(value.clone()),
                    _ => unreachable!("matched a value-taking option"),
                }
                index += 2;
            }
            "--strict" => {
                options.strict = true;
                index += 1;
            }
            argument => return Err(format!("unknown argument \"{argument}\"")),
        }
    }

    Ok(options)
}

fn valid_scope(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn valid_date(value: &str) -> bool {
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

    let month = (bytes[5] - b'0') * 10 + bytes[6] - b'0';
    let day = (bytes[8] - b'0') * 10 + bytes[9] - b'0';
    (1..=12).contains(&month) && (1..=31).contains(&day)
}

fn report_error(stderr: &mut dyn Write, message: &str) -> i32 {
    let _ = writeln!(stderr, "commitbump: {message}");
    2
}
