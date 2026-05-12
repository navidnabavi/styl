use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "mapbox-style-tool",
    about = "Lint, validate, and format Mapbox/MapLibre GL Style JSON"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Spec version to validate against
    #[arg(long, global = true, default_value = "maplibre")]
    pub spec: Spec,

    /// Output format
    #[arg(long, global = true, default_value = "human")]
    pub format: OutputFormat,

    /// Path to .mapboxlintrc config file
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
    Lint { file: Option<PathBuf> },
    /// Run only spec validation
    Validate { file: Option<PathBuf> },
}

#[derive(Clone, ValueEnum, Debug)]
pub enum Spec {
    Maplibre,
    Mapbox,
}

#[derive(Clone, ValueEnum, Debug)]
pub enum OutputFormat {
    Human,
    Json,
    Github,
}
