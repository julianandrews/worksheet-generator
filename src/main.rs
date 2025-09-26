mod config;
mod html_gen;

use anyhow::{Context, Result};
use clap::Parser;

use config::{Args, Options, OutputFormat};

fn main() -> Result<()> {
    let args = Args::parse();
    let config = Options::load_config(args.config.as_deref())?;
    let options = Options::from_args_and_config(args, config)?;

    match options.output_format {
        OutputFormat::Html => {
            let html = html_gen::generate_html(&options.pages, options.stylesheet.as_deref())?;
            std::fs::write(&options.output_file, html).context(format!(
                "Failed to write HTML to {}",
                options.output_file.display()
            ))?;
            println!("✓ HTML generated at {}", options.output_file.display());
        }
        OutputFormat::Pdf => {
            which::which("weasyprint").context("'weasyprint' not found in PATH".to_string())?;
            let html = html_gen::generate_html(&options.pages, options.stylesheet.as_deref())?;
            pdf_gen::generate_pdf(&html, &options.output_file)?;
            println!("✓ PDF generated at {}", options.output_file.display());
        }
    }

    Ok(())
}

mod pdf_gen {
    use std::io::Write;
    use std::path::Path;
    use std::process::{Command, Stdio};

    use anyhow::{Context, Result, anyhow};

    pub fn generate_pdf(html: &str, output: &Path) -> Result<()> {
        // Pipe HTML directly to weasyprint via stdin
        let mut weasyprint_cmd = Command::new("weasyprint");
        let mut weasyprint = weasyprint_cmd
            .arg("-") // Read from stdin
            .arg(output)
            .stdin(Stdio::piped())
            .spawn()
            .context("Failed to spawn weasyprint")?;
        if let Some(mut stdin) = weasyprint.stdin.take() {
            stdin.write_all(html.as_bytes())?;
        }

        let status = weasyprint.wait().context("Weasyprint failed")?;

        if !status.success() {
            return Err(anyhow!("weasyprint failed"));
        }
        Ok(())
    }
}
