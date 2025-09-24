# Worksheet Generator

A CLI tool for generating printable PDF worksheets from Markdown files. Perfect
for weekly planning, fitness routines, self-care checklists, or any other
structured documentation you want in a printable format.

## Features

- Markdown to PDF conversion using Pandoc and WeasyPrint
- Automatic printing with configurable options  
- Customizable CSS styling
- Flexible configuration via YAML files or command-line arguments

## Installation

### Prerequisites

- Rust toolchain (install via [rustup](https://rustup.rs/))
- System dependencies:
  - `pandoc`
  - `weasyprint` (PDF engine for pandoc)
  - `pdfunite` (usually comes with poppler-utils)
  - `lp` (printing, usually pre-installed on Linux)

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

### Generate without Printing

```bash
worksheet-generator --generate-only config.yaml
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
output_dir: /tmp
lp_options: "-o media=A4 -o fit-to-page"
```

## Custom Styling

Note: While the generator works without a custom stylesheet (using Pandoc's
default template), you may see some harmless warnings from WeasyPrint about
unsupported CSS properties. For the cleanest experience, we recommend providing
your own stylesheet. If the default template warnings bother you and you'd like
a cleaner built-in solution, please open an issue on GitHub! I'm far too lazy
to fix this without at least *some* evidence that someone cares.

Create a `style.css` file to customize the PDF appearance:

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
```
