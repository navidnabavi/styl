use clap::Parser;
use std::process;

use styl::{cli, diagnostic, formatter, linter, style, validator};

use cli::{Cli, Command, OutputFormat};
use diagnostic::{render_github, render_html, render_human, render_json};
use linter::config::{discover_config, load_config, Config};
use style::Style;

fn main() {
    let cli = Cli::parse();
    let exit_code = run(&cli);
    process::exit(exit_code);
}

fn run(cli: &Cli) -> i32 {
    // Load config
    let config = load_effective_config(cli);

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
        Command::Check { .. } => {
            let style: Style = match serde_json::from_value(value.clone()) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error: style parse failed: {}", e);
                    return 2;
                }
            };
            let mut diags = validator::run_all(&style, &crate::cli::Spec::Both);
            diags.extend(linter::run_all(&style));
            diags
        }
        Command::Validate { .. } => {
            let style: Style = match serde_json::from_value(value.clone()) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error: style parse failed: {}", e);
                    return 2;
                }
            };
            validator::run_all(&style, &crate::cli::Spec::Both)
        }
        Command::Lint { .. } => {
            let style: Style = match serde_json::from_value(value.clone()) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error: style parse failed: {}", e);
                    return 2;
                }
            };
            linter::run_all(&style)
        }
        Command::Fmt { check, .. } => {
            let formatted = formatter::format_style(&value, config.format.indent);
            if *check {
                if formatted != content {
                    if !cli.quiet {
                        eprintln!("{} would be reformatted", filename);
                    }
                    return 1;
                }
            } else if let Some(path) = get_file_path(cli) {
                std::fs::write(path, &formatted)
                    .map_err(|e| {
                        eprintln!("error: {}", e);
                    })
                    .ok();
            } else {
                print!("{}", formatted);
            }
            return 0;
        }
    };

    if !cli.quiet {
        let output = match cli.format {
            OutputFormat::Human => render_human(&diagnostics, &filename),
            OutputFormat::Json => render_json(&diagnostics),
            OutputFormat::Github => render_github(&diagnostics, &filename),
            OutputFormat::Html => render_html(&diagnostics, &filename),
        };
        print!("{}", output);
    }

    if diagnostics.iter().any(|d| {
        matches!(
            d.severity,
            diagnostic::Severity::Error | diagnostic::Severity::Warning
        )
    }) {
        1
    } else {
        0
    }
}

fn read_input(cli: &Cli) -> Result<(String, String), String> {
    if cli.stdin {
        use std::io::Read;
        let mut content = String::new();
        std::io::stdin()
            .read_to_string(&mut content)
            .map_err(|e| e.to_string())?;
        return Ok((content, "<stdin>".to_string()));
    }
    let path = get_file_path(cli).ok_or("no input file specified")?;
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("{}: {}", path.display(), e))?;
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

fn load_effective_config(cli: &Cli) -> Config {
    // Explicit --config path takes priority
    if let Some(config_path) = &cli.config {
        return load_config(config_path).unwrap_or_else(|e| {
            eprintln!("warning: {}", e);
            Config::default()
        });
    }
    // Auto-discover from the file's directory or cwd
    let start = get_file_path(cli)
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    discover_config(&start)
        .and_then(|p| load_config(&p).ok())
        .unwrap_or_default()
}
