use std::fs;
use std::path::Path;
use std::sync::LazyLock;

use anyhow::{Context, Result};
use comrak::{Options, markdown_to_html};

// Comrak options can be static since they're configuration
static COMRAK_OPTIONS: LazyLock<Options> = LazyLock::new(|| {
    let mut options = Options::default();

    // Enable common extensions
    options.extension.table = true;
    options.extension.strikethrough = true;
    options.extension.tasklist = true;
    options.extension.superscript = true;
    options.extension.footnotes = true;
    options.extension.description_lists = true;

    options.render.unsafe_ = true;
    options.render.hardbreaks = false;
    options.render.github_pre_lang = true;

    options
});

pub fn generate_html(
    page_path: &Path,
    stylesheet_path: Option<&Path>,
) -> Result<String, anyhow::Error> {
    // Read markdown content
    let markdown_content = fs::read_to_string(page_path).context(format!(
        "Failed to read markdown file: {}",
        page_path.display()
    ))?;

    // Convert markdown to HTML
    let html_content = markdown_to_html(&markdown_content, &COMRAK_OPTIONS);

    // Create full HTML document with optional CSS
    let css_content = if let Some(stylesheet_path) = stylesheet_path {
        if stylesheet_path.exists() {
            fs::read_to_string(stylesheet_path).context(format!(
                "Failed to read stylesheet: {}",
                stylesheet_path.display()
            ))?
        } else {
            eprintln!(
                "Warning: Stylesheet {} not found, proceeding without styles",
                stylesheet_path.display()
            );
            String::new()
        }
    } else {
        String::new()
    };

    let full_html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>{}</style>
</head>
<body>
{}
</body>
</html>"#,
        css_content, html_content
    );

    Ok(full_html)
}
