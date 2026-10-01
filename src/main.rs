mod lint;
mod output;

use lint::Severity;
use std::env;
use std::fs;
use std::process::ExitCode;

const USAGE: &str = "usage: decklint [--json] <deck-file>";

fn main() -> ExitCode {
    let mut json = false;
    let mut path: Option<String> = None;

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--json" => json = true,
            "-h" | "--help" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            flag if flag.starts_with("--") => {
                eprintln!("unknown option '{flag}'\n{USAGE}");
                return ExitCode::FAILURE;
            }
            _ => {
                if path.is_some() {
                    eprintln!("only one deck file can be given\n{USAGE}");
                    return ExitCode::FAILURE;
                }
                path = Some(arg);
            }
        }
    }

    let path = match path {
        Some(p) => p,
        None => {
            eprintln!("{USAGE}");
            return ExitCode::FAILURE;
        }
    };

    let source = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("could not read {path}: {e}");
            return ExitCode::FAILURE;
        }
    };

    let findings = lint::lint_source(&source);
    let had_error = findings.iter().any(|f| f.severity == Severity::Error);

    if json {
        print!("{}", output::format_json(&path, &findings));
    } else {
        print!("{}", output::format_text(&path, &findings));
    }

    if had_error {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
