use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, ValueEnum, parser::ValueSource};
use serde::Deserialize;

// Args struct - CLI interface
#[derive(Parser, Debug)]
#[command(version, about, author)]
pub struct Args {
    /// Path to config file (YAML)
    pub config: Option<PathBuf>,

    /// Output file
    #[arg(short, long = "output", value_name = "FILE")]
    pub output_file: Option<PathBuf>,

    /// Output format
    #[arg(
        short = 'f',
        long = "format",
        value_name = "FORMAT",
        default_value = "pdf"
    )]
    pub output_format: OutputFormat,

    /// Markdown files to process (overrides config)
    #[arg(short, long, value_name = "FILE", num_args = 1..)]
    pub pages: Vec<PathBuf>,

    /// Stylesheet to use (overrides config)
    #[arg(short, long, value_name = "FILE")]
    pub stylesheet: Option<PathBuf>,
}

#[derive(ValueEnum, Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Pdf,
    Html,
}

// Config struct - File-based configuration
#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub pages: Vec<PathBuf>,
    pub stylesheet: Option<PathBuf>,
    pub output_file: Option<PathBuf>,
    pub output_format: Option<OutputFormat>,
}

// Options struct - Final resolved configuration
#[derive(Debug)]
pub struct Options {
    pub pages: Vec<PathBuf>,
    pub stylesheet: Option<PathBuf>,
    pub output_file: PathBuf,
    pub output_format: OutputFormat,
}

impl Options {
    pub fn from_args_and_config(args: Args, config: Config) -> Result<Self> {
        // Determine config directory (for relative path resolution)
        let config_dir = args
            .config
            .as_ref()
            .and_then(|p| p.parent())
            .unwrap_or_else(|| Path::new("."));

        // Resolve pages (CLI overrides config)
        let pages = if !args.pages.is_empty() {
            // CLI paths are relative to CWD
            args.pages
        } else {
            // Config paths are relative to config file
            config
                .pages
                .into_iter()
                .map(|p| config_dir.join(p))
                .collect()
        };

        if pages.is_empty() {
            return Err(anyhow::anyhow!(
                "No pages specified. Use --pages or provide a config file."
            ));
        }

        // Resolve stylesheet (CLI overrides config)
        let stylesheet = args
            .stylesheet
            .or_else(|| config.stylesheet.map(|s| config_dir.join(s)));

        // Resolve output format (CLI overrides config only if explicitly set)
        let format_source = Args::command().get_matches().value_source("output_format");
        let output_format = if format_source != Some(ValueSource::DefaultValue) {
            args.output_format
        } else {
            config.output_format.unwrap_or(args.output_format)
        };

        // Resolve output file (CLI overrides config, then smart default)
        let output_file = args
            .output_file
            .or_else(|| config.output_file.map(|p| config_dir.join(p)))
            .unwrap_or_else(|| Self::derive_output_file(&pages, &output_format));

        Ok(Options {
            pages,
            stylesheet,
            output_file,
            output_format,
        })
    }

    fn derive_output_file(pages: &[PathBuf], format: &OutputFormat) -> PathBuf {
        // Try to use the first page's stem as the base name
        if let Some(first_page) = pages.first() {
            if let Some(stem) = first_page.file_stem() {
                let ext = match format {
                    OutputFormat::Pdf => "pdf",
                    OutputFormat::Html => "html",
                };
                return PathBuf::from(format!("{}.{}", stem.to_string_lossy(), ext));
            }
        }

        // Fallback
        PathBuf::from(match format {
            OutputFormat::Pdf => "output.pdf",
            OutputFormat::Html => "output.html",
        })
    }

    // Helper method to load config from file
    pub fn load_config(path: Option<&Path>) -> Result<Config> {
        match path {
            Some(config_path) => {
                let config_file = std::fs::File::open(config_path).context(format!(
                    "Failed to open config file: {}",
                    config_path.display()
                ))?;
                let config: Config =
                    serde_yaml::from_reader(config_file).context("Failed to parse config file")?;
                Ok(config)
            }
            None => Ok(Config::default()),
        }
    }
}
