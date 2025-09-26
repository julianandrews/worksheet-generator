use std::fs;
use std::path::Path;
use std::sync::LazyLock;

use anyhow::{Context, Result};
use comrak::{Options, markdown_to_html};
use lol_html::html_content::Element;
use lol_html::{RewriteStrSettings, element, rewrite_str, text};
use slug::slugify;

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

pub fn generate_html(page_path: &Path, stylesheet_path: Option<&Path>) -> Result<String> {
    // Read markdown content
    let markdown_content = fs::read_to_string(page_path).context(format!(
        "Failed to read markdown file: {}",
        page_path.display()
    ))?;

    // Convert markdown to HTML
    let generated_html = markdown_to_html(&markdown_content, &COMRAK_OPTIONS);
    let final_html = add_section_wrappers_to_html(&generated_html)?;

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
        css_content, final_html
    );

    Ok(full_html)
}

pub fn add_section_wrappers_to_html(html: &str) -> Result<String> {
    let headings = std::rc::Rc::new(std::cell::RefCell::new(vec![]));
    let buffer = std::rc::Rc::new(std::cell::RefCell::new(String::new()));

    rewrite_str(
        html,
        RewriteStrSettings {
            element_content_handlers: vec![
                element!("h1, h2, h3, h4, h5, h6", |el: &mut Element| {
                    // Truncate string for each new heading.
                    buffer.borrow_mut().clear();
                    let buffer = buffer.clone();
                    let headings = headings.clone();
                    let location = el.source_location().bytes().start;
                    let level = el.tag_name().chars().nth(1).unwrap().to_digit(10).unwrap() as u8;

                    if let Some(handlers) = el.end_tag_handlers() {
                        handlers.push(Box::new(move |_| {
                            let slug = slugify(&*buffer.borrow());
                            headings.borrow_mut().push((location, level, slug));
                            Ok(())
                        }));
                    }
                    Ok(())
                }),
                text!("h1, h2, h3, h4, h5, h6", |t| {
                    // Save the text contents for the end tag handler
                    buffer.borrow_mut().push_str(t.as_str());
                    Ok(())
                }),
            ],
            ..RewriteStrSettings::new()
        },
    )?;

    let headings = headings.borrow();

    // Process headings in order of appearance
    let mut result = String::new();
    let mut header_stack: Vec<(u8, String)> = Vec::new(); // (level, slug)
    let mut last_pos = 0;

    for &(location, level, ref slug) in headings.iter() {
        // Add content before this heading
        result.push_str(&html[last_pos..location]);

        // Close previous sections at same or higher level
        while let Some(&(last_level, _)) = header_stack.last() {
            if last_level >= level {
                header_stack.pop();
                result.push_str("</div>");
            } else {
                break;
            }
        }

        // Open new section before this heading
        result.push_str(&format!("\n<div class=\"{}\">\n", slug));

        // Update stack
        header_stack.push((level, slug.clone()));
        last_pos = location;
    }

    // Add remaining content after last heading
    result.push_str(&html[last_pos..]);

    // Close any remaining open sections
    if !header_stack.is_empty() {
        result.push_str(&"</div>\n".repeat(header_stack.len() - 1));
        result.push_str("</div>"); // Last one without newline
    }

    Ok(result)
}
