# Worksheet Generator

A CLI tool for generating printable PDF worksheets from Markdown files. Perfect
for weekly planning, fitness routines, self-care checklists, or any other
structured documentation you want in a printable format.

## Features

- Markdown to PDF conversion using WeasyPrint
- Customizable CSS styling
- Flexible configuration via YAML files or command-line arguments
- Page breaks between multiple input files

## Installation

### Prerequisites

- Rust toolchain (install via [rustup](https://rustup.rs/))
- `weasyprint` (if outputting to PDF)

### Building from Source

```bash
git clone https://github.com/yourusername/worksheet-generator
cd worksheet-generator
cargo build --release
cp target/release/worksheet-generator ~/.local/bin/
```

## Usage

### Command-Line Only

```bash
worksheet-generator --pages workout.md --pages self-care.md --stylesheet custom.css
```

### With Config File

```bash
worksheet-generator config.yaml
```

### Generate HTML instead of a PDF

```bash
worksheet-generator --format html config.yaml
```

### Path Resolution

- CLI paths are relative to the current working directory
- Config file paths are relative to the config file's directory
- Absolute paths are supported in both contexts

### Git-Based Workflow

This tool works perfectly with version-controlled Markdown repositories. Just
store your config and stylesheet with the markdown files:

```
fitness-repo/
├── config.yaml
├── styles.css
├── weekly-workout.md
├── progress-tracker.md
```

## Config File

### Minimal

```yaml
pages:
    - weekly-workout.md
```

### Full

```yaml
pages:
    - weekly-workout.md
    - progress-tracker.md
stylesheet: styles.css
output_file: workout.pdf
output_format: pdf
```

## Custom Styling

Create a `style.css` file to customize the PDF appearance. The generator
automatically wraps each section in `div` elements with CSS classes based on
your heading hierarchy, making it easy to target specific parts of your
document. For example, content under a `## Monday` heading gets the `monday`
class. Use `--format html` to generate an HTML preview for testing your styles
before creating the final PDF.

```css
@page {
  size: Letter;
  margin: 0.5in 0.5in;
}

body {
  font-family: 'Inter', sans-serif;
  font-size: 10pt;
}

h1 {
  font-size: 1.8em;
}

table {
  border-collapse: collapse;
  width: 100%;
}

th, td {
  border: 1px solid #666;
  padding: 5px;
}

/* Style Monday headings with extra energy! */
.monday h2 {
  color: #ff6b6b;
}

.monday table {
  border-color: #ff6b6b;  /* Coordinated tables */
}
```
