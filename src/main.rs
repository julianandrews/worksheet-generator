use anyhow::{Context, Result, anyhow};
use clap::Parser;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Deserialize, Default)]
struct Config {
    pages: Vec<String>,
    stylesheet: Option<String>,
    output_dir: Option<String>,
    lp_options: Option<String>,
}

#[derive(Parser)]
#[command(version, about, author)]
struct Args {
    /// Path to config file (YAML)
    config: Option<String>,
    /// Markdown files to process
    #[arg(short, long, value_name = "FILE", num_args = 1..)]
    pages: Vec<String>,
    /// Stylesheet to use
    #[arg(short, long, value_name = "FILE")]
    stylesheet: Option<String>,
    /// Output directory for PDFs
    #[arg(short, long, value_name = "DIR")]
    output_dir: Option<String>,
    /// Options to pass to lp command
    #[arg(long, value_name = "OPTIONS")]
    lp_options: Option<String>,
    /// Generate PDFs without printing or cleanup
    #[arg(long)]
    generate_only: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut config = if let Some(config_path) = &args.config {
        load_config(config_path)?
    } else {
        Config::default()
    };

    // Merge command-line arguments over config
    if !args.pages.is_empty() {
        config.pages = args.pages;
    }
    if args.stylesheet.is_some() {
        config.stylesheet = args.stylesheet;
    }
    if args.output_dir.is_some() {
        config.output_dir = args.output_dir;
    }
    if args.lp_options.is_some() {
        config.lp_options = args.lp_options;
    }

    if config.pages.is_empty() {
        return Err(anyhow!("Config must contain at least one page"));
    }

    // Validate required binaries
    let config_path = args.config.as_deref().unwrap_or(".");
    validate_binaries(&["pandoc", "pdfunite", "weasyprint", "lp"])?;

    // Generate PDFs
    let pdf_paths = generate_pdfs(&config, config_path)?;

    // Combine PDFs
    let combined_pdf = combine_pdfs(&pdf_paths, &config)?;

    if args.generate_only {
        println!("✓ PDFs generated at: {}", combined_pdf.display());
        println!(
            "* Individual PDFs retained in: {}",
            config.output_dir.as_deref().unwrap_or("/tmp")
        );
    } else {
        // Print and clean up
        print_pdf(&combined_pdf, &config)?;
        cleanup(&pdf_paths, &combined_pdf)?;
        println!("✓ Worksheets generated and printed successfully!");
    }

    Ok(())
}

fn load_config(path: &str) -> Result<Config> {
    let config_file =
        std::fs::File::open(path).context(format!("Failed to open config file: {}", path))?;
    Ok(serde_yaml::from_reader(config_file).context("Failed to parse config file")?)
}

fn validate_binaries(binaries: &[&str]) -> Result<()> {
    for bin in binaries {
        which::which(bin).context(format!("Required binary '{}' not found in PATH", bin))?;
    }
    Ok(())
}

fn generate_pdfs(config: &Config, config_path: &str) -> Result<Vec<PathBuf>> {
    let config_dir = Path::new(config_path)
        .parent()
        .unwrap_or_else(|| Path::new("."));

    let stylesheet = config.stylesheet.as_deref().unwrap_or("styles.css");
    let stylesheet_path = config_dir.join(stylesheet);

    let output_dir = config.output_dir.as_deref().unwrap_or("/tmp");
    let output_path = Path::new(output_dir);

    let mut pdf_paths = Vec::new();

    for page in &config.pages {
        let page_path = config_dir.join(page);
        let output_pdf = output_path.join(format!(
            "{}.pdf",
            page_path.file_stem().unwrap().to_string_lossy()
        ));

        let status = Command::new("pandoc")
            .arg(&page_path)
            .arg("-o")
            .arg(&output_pdf)
            .arg("--css")
            .arg(&stylesheet_path)
            .arg("--pdf-engine")
            .arg("weasyprint")
            .status()
            .context("Failed to execute pandoc")?;

        if !status.success() {
            return Err(anyhow!("pandoc failed for file: {}", page));
        }

        pdf_paths.push(output_pdf);
    }

    Ok(pdf_paths)
}

fn combine_pdfs(pdf_paths: &[PathBuf], config: &Config) -> Result<PathBuf> {
    let output_dir = config.output_dir.as_deref().unwrap_or("/tmp");
    let combined_pdf = Path::new(output_dir).join("combined_worksheets.pdf");

    let mut cmd = Command::new("pdfunite");
    for pdf in pdf_paths {
        cmd.arg(pdf);
    }
    cmd.arg(&combined_pdf);

    let status = cmd.status().context("Failed to execute pdfunite")?;

    if !status.success() {
        return Err(anyhow!("pdfunite failed to combine PDFs"));
    }

    Ok(combined_pdf)
}

fn print_pdf(pdf_path: &Path, config: &Config) -> Result<()> {
    let mut cmd = Command::new("lp");

    if let Some(options) = &config.lp_options {
        // Split options string into arguments
        for opt in options.split_whitespace() {
            cmd.arg(opt);
        }
    }

    cmd.arg(pdf_path);

    let status = cmd.status().context("Failed to execute lp")?;

    if !status.success() {
        return Err(anyhow!("lp failed to print PDF"));
    }

    Ok(())
}

fn cleanup(pdf_paths: &[PathBuf], combined_pdf: &Path) -> Result<()> {
    for pdf in pdf_paths {
        if let Err(e) = std::fs::remove_file(pdf) {
            eprintln!("Warning: Failed to delete {}: {}", pdf.display(), e);
        }
    }

    if let Err(e) = std::fs::remove_file(combined_pdf) {
        eprintln!(
            "Warning: Failed to delete {}: {}",
            combined_pdf.display(),
            e
        );
    }

    Ok(())
}
