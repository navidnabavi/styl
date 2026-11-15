use clap::Parser;
use std::process;

use styl::{cli, diagnostic, formatter, linter, span, style, validator};

use cli::{Cli, Command, OutputFormat, Spec};
use diagnostic::{render_github, render_html, render_human, render_json};
use linter::config::{discover_config, load_config, Config};
use style::{parse_style, Style};

fn main() {
    let cli = Cli::parse();
    let exit_code = run(&cli);
    process::exit(exit_code);
}

fn run(cli: &Cli) -> i32 {
    // The language server owns stdin/stdout and takes no input file, so it has
    // to short-circuit ahead of `read_input`.
    if matches!(cli.command, Command::Lsp { .. }) {
        return serve_lsp();
    }

    // Load config
    let config = load_effective_config(cli);

    // An explicit `--spec` outranks `.stylrc`, which outranks the default. The
    // language server resolves this the same way, so the two never disagree.
    let spec = cli
        .spec
        .clone()
        .or_else(|| config.resolved_spec())
        .unwrap_or(Spec::Both);

    // Read input
    let (content, filename) = match read_input(cli) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: {}", e);
            return 2;
        }
    };

    // Parse JSON
    let mut value: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: invalid JSON: {}", e);
            return 2;
        }
    };

    // Text the diagnostics refer to. `lint --fix` re-lints the rewritten
    // document, so spans must be resolved against that, not the original.
    let mut span_source = content.clone();

    let mut diagnostics: Vec<diagnostic::Diagnostic> = match &cli.command {
        Command::Check { .. } => {
            let style = match load_style(&content, &filename) {
                Ok(s) => s,
                Err(code) => return code,
            };
            let mut diags = validator::run_all(&style, &spec);
            diags.extend(linter::run_all(&style, &spec));
            diags
        }
        Command::Validate { .. } => {
            let style = match load_style(&content, &filename) {
                Ok(s) => s,
                Err(code) => return code,
            };
            validator::run_all(&style, &spec)
        }
        Command::Lint { fix, .. } => {
            let style = match load_style(&content, &filename) {
                Ok(s) => s,
                Err(code) => return code,
            };
            let diags = linter::run_all(&style, &spec);
            if *fix {
                match apply_fixes(&mut value, cli, &config, &spec) {
                    Ok(formatted) => span_source = formatted,
                    Err(code) => return code,
                }
                // Re-lint after fixes to get remaining diagnostics for exit code
                let fixed_style = match load_style(&span_source, &filename) {
                    Ok(s) => s,
                    Err(code) => return code,
                };
                linter::run_all(&fixed_style, &spec)
            } else {
                diags
            }
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
        // Handled at the top of `run`, before any input is read.
        Command::Lsp { .. } => unreachable!("lsp short-circuits earlier"),
    };

    config.apply_severity(&mut diagnostics);
    span::resolve_ranges(&mut diagnostics, &span::SourceMap::parse(&span_source));

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

/// Deserialize a style, reporting the offending field and its source location
/// rather than a bare serde message with no position.
fn load_style(text: &str, filename: &str) -> Result<Style, i32> {
    parse_style(text).map_err(|error| {
        eprintln!("error: style parse failed {}", error);
        let location = span::SourceMap::parse(text)
            .range_for_path(&error.path)
            .map(|range| {
                let (line, column) = range.start.one_based();
                format!("{}:{}:{}", filename, line, column)
            })
            .unwrap_or_else(|| filename.to_string());
        eprintln!("  --> {}", location);
        2i32
    })
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
        Command::Lint { file, .. } => file.as_ref(),
        Command::Validate { file } => file.as_ref(),
        Command::Lsp { .. } => None,
    }
}

#[cfg(feature = "lsp")]
fn serve_lsp() -> i32 {
    match styl::lsp::serve() {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("error: language server failed: {}", e);
            2
        }
    }
}

#[cfg(not(feature = "lsp"))]
fn serve_lsp() -> i32 {
    eprintln!("error: this build of styl was compiled without the \"lsp\" feature");
    2
}

fn apply_fixes(
    value: &mut serde_json::Value,
    cli: &Cli,
    config: &Config,
    spec: &Spec,
) -> Result<String, i32> {
    // `run_fixes` now reports only the rules that actually changed something, so
    // there is nothing left to filter against the detected diagnostics.
    let fixed = linter::run_fixes(value, spec);
    if !fixed.is_empty() && !cli.quiet {
        eprintln!("fixed {} issue(s) ({})", fixed.len(), fixed.join(", "));
    }
    let formatted = formatter::format_style(value, config.format.indent);
    if let Some(path) = get_file_path(cli) {
        std::fs::write(path, &formatted).map_err(|e| {
            eprintln!("error: {}", e);
            2i32
        })?;
    } else {
        print!("{}", formatted);
    }
    Ok(formatted)
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
