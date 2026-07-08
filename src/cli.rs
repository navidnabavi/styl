use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "styl",
    about = "Linter, validator, and formatter for MapLibre GL / Mapbox GL style JSON"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Spec version to validate against
    #[arg(long, global = true, default_value = "both")]
    pub spec: Spec,

    /// Output format
    #[arg(long, global = true, default_value = "human")]
    pub format: OutputFormat,

    /// Path to .stylrc config file
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    /// Read from stdin
    #[arg(long, global = true)]
    pub stdin: bool,

    /// Suppress output, only exit code
    #[arg(short, long, global = true)]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Command {
    /// Validate + lint a style file
    Check { file: Option<PathBuf> },
    /// Format a style file in-place
    Fmt {
        file: Option<PathBuf>,
        /// Check only, don't write
        #[arg(long)]
        check: bool,
    },
    /// Run only lint rules
    Lint {
        file: Option<PathBuf>,
        /// Automatically fix safe, mechanical issues in-place
        #[arg(long)]
        fix: bool,
    },
    /// Run only spec validation
    Validate { file: Option<PathBuf> },
}

#[derive(Clone, ValueEnum, Debug, PartialEq)]
pub enum Spec {
    /// Validate compatibility with both MapLibre and Mapbox (default)
    Both,
    /// Validate against MapLibre spec only
    Maplibre,
    /// Validate against Mapbox spec only
    Mapbox,
}

#[derive(Clone, ValueEnum, Debug)]
pub enum OutputFormat {
    Human,
    Json,
    Github,
    Html,
}
