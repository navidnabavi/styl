use clap::Parser;
use std::process;

mod cli;
mod diagnostic;
mod formatter;
mod linter;
mod style;
mod validator;

use cli::{Cli, Command, OutputFormat};
use diagnostic::{render_github, render_human, render_json};
use style::Style;

fn main() {
    let cli = Cli::parse();
    let exit_code = run(&cli);
    process::exit(exit_code);
}

fn run(cli: &Cli) -> i32 {
    // Read input
    let (content, filename) = match read_input(cli) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: {}", e);
            return 2;
        }
    };

    // Parse JSON
    let value: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: invalid JSON: {}", e);
            return 2;
        }
    };

    let diagnostics: Vec<diagnostic::Diagnostic> = match &cli.command {
        Command::Check { .. } | Command::Lint { .. } | Command::Validate { .. } => {
            // Parse into Style for future validators/linters
            let _style: Result<Style, _> = serde_json::from_value(value.clone());
            // Validators and linters will be wired in subsequent tasks
            vec![]
        }
        Command::Fmt { check, .. } => {
            // Pretty-print round-trip for now
            let formatted = serde_json::to_string_pretty(&value).unwrap();
            if *check {
                if formatted != content {
                    eprintln!("error: {} would be reformatted", filename);
                    return 1;
                }
            } else {
                if let Some(path) = get_file_path(cli) {
                    std::fs::write(path, &formatted).unwrap_or_else(|e| {
                        eprintln!("error: {}", e);
                    });
                } else {
                    print!("{}", formatted);
                }
            }
            return 0;
        }
    };

    if !cli.quiet {
        let output = match cli.format {
            OutputFormat::Human => render_human(&diagnostics, &filename),
            OutputFormat::Json => render_json(&diagnostics),
            OutputFormat::Github => render_github(&diagnostics, &filename),
        };
        print!("{}", output);
    }

    if diagnostics.iter().any(|d| matches!(d.severity, diagnostic::Severity::Error | diagnostic::Severity::Warning)) {
        1
    } else {
        0
    }
}

fn read_input(cli: &Cli) -> Result<(String, String), String> {
    if cli.stdin {
        use std::io::Read;
        let mut content = String::new();
        std::io::stdin().read_to_string(&mut content).map_err(|e| e.to_string())?;
        return Ok((content, "<stdin>".to_string()));
    }
    let path = get_file_path(cli).ok_or("no input file specified")?;
    let content = std::fs::read_to_string(&path).map_err(|e| format!("{}: {}", path.display(), e))?;
    Ok((content, path.display().to_string()))
}

fn get_file_path(cli: &Cli) -> Option<&std::path::PathBuf> {
    match &cli.command {
        Command::Check { file } => file.as_ref(),
        Command::Fmt { file, .. } => file.as_ref(),
        Command::Lint { file } => file.as_ref(),
        Command::Validate { file } => file.as_ref(),
    }
}
