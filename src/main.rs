mod lint;

use lint::Severity;
use std::env;
use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let path = match args.next() {
        Some(p) => p,
        None => {
            eprintln!("usage: decklint <deck-file>");
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

    if findings.is_empty() {
        println!("{path}: no issues found");
    } else {
        for finding in &findings {
            println!("{path}:{}: {}: {}", finding.line, finding.severity, finding.message);
        }
    }

    if had_error {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
