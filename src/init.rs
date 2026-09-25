use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::Path;

use crate::cmd::Colors;

pub const SUPPORTED_THEMES: &[&str] = &["textbook", "default", "sepia", "nord", "solarized"];

#[derive(Debug, Clone)]
pub struct ProjectConfig {
    pub title: String,
    pub author: String,
    pub theme: String,
    pub description: String,
    pub sample_content: bool,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            title: "My DocUP Project".to_string(),
            author: default_author_name(),
            theme: "textbook".to_string(),
            description: "A document built with DocUP".to_string(),
            sample_content: true,
        }
    }
}

/// Scaffolds a new DocUP project into a newly created folder.
pub fn scaffold_new(
    target_dir: &Path,
    cli_config: Option<ProjectConfig>,
    yes: bool,
    colors: Colors,
) -> Result<(), String> {
    if target_dir.exists() {
        if target_dir.is_file() {
            return Err(format!(
                "target path \"{}\" already exists as a file",
                target_dir.display()
            ));
        }
        let entries = fs::read_dir(target_dir)
            .map_err(|e| format!("cannot inspect target directory: {e}"))?
            .count();
        if entries > 0 {
            return Err(format!(
                "directory \"{}\" is not empty; use `docup init` to initialize in an existing directory",
                target_dir.display()
            ));
        }
    } else {
        fs::create_dir_all(target_dir).map_err(|e| {
            format!(
                "failed to create directory \"{}\": {e}",
                target_dir.display()
            )
        })?;
    }

    let default_title = dir_name_to_title(target_dir);
    let config = resolve_config(default_title, cli_config, yes, colors)?;

    write_project_files(target_dir, &config, colors)?;

    println!(
        "\n{}{}✓ Created new DocUP project in{} {}{}{}",
        colors.bold,
        colors.green,
        colors.reset,
        colors.cyan,
        target_dir.display(),
        colors.reset
    );
    println!("  Run:");
    println!(
        "    {}cd {}{}",
        colors.dim,
        target_dir.display(),
        colors.reset
    );
    println!("    {}docup watch index.du{}\n", colors.bold, colors.reset);

    Ok(())
}

/// Scaffolds DocUP into the current working directory without creating a subfolder.
pub fn scaffold_init(
    target_dir: &Path,
    cli_config: Option<ProjectConfig>,
    yes: bool,
    colors: Colors,
) -> Result<(), String> {
    let index_file = target_dir.join("index.du");
    if index_file.exists() {
        return Err(format!(
            "index.du already exists in \"{}\"; aborting to avoid overwrite",
            target_dir.display()
        ));
    }

    let default_title = dir_name_to_title(target_dir);
    let config = resolve_config(default_title, cli_config, yes, colors)?;

    write_project_files(target_dir, &config, colors)?;

    println!(
        "\n{}{}✓ Initialized DocUP in{} {}{}{}",
        colors.bold,
        colors.green,
        colors.reset,
        colors.cyan,
        target_dir.display(),
        colors.reset
    );
    println!("  Run:");
    println!("    {}docup watch index.du{}\n", colors.bold, colors.reset);

    Ok(())
}

fn resolve_config(
    default_title: String,
    cli_config: Option<ProjectConfig>,
    yes: bool,
    colors: Colors,
) -> Result<ProjectConfig, String> {
    let base = cli_config.unwrap_or_default();
    let is_interactive = io::stdin().is_terminal() && io::stdout().is_terminal() && !yes;

    if !is_interactive {
        let mut final_cfg = base;
        if final_cfg.title.is_empty() || final_cfg.title == "My DocUP Project" {
            final_cfg.title = default_title;
        }
        if !SUPPORTED_THEMES.contains(&final_cfg.theme.as_str()) {
            final_cfg.theme = "textbook".to_string();
        }
        return Ok(final_cfg);
    }

    println!(
        "{}{}DocUP Project Setup Wizard{}\n",
        colors.bold, colors.cyan, colors.reset
    );

    let title_prompt = format!("Project title [{}]: ", default_title);
    let title_input = prompt_input(&title_prompt)?;
    let title = if title_input.trim().is_empty() {
        default_title
    } else {
        title_input.trim().to_string()
    };

    let author_default = if base.author.is_empty() {
        default_author_name()
    } else {
        base.author.clone()
    };
    let author_prompt = format!("Author name [{}]: ", author_default);
    let author_input = prompt_input(&author_prompt)?;
    let author = if author_input.trim().is_empty() {
        author_default
    } else {
        author_input.trim().to_string()
    };

    println!("\nAvailable themes:");
    for (i, t) in SUPPORTED_THEMES.iter().enumerate() {
        let note = if *t == "textbook" {
            " (LaTeX style with Literata & Cascadia Code - recommended)"
        } else {
            ""
        };
        println!("  {}. {}{}", i + 1, t, note);
    }
    let theme_prompt = "Select theme [1 - textbook]: ";
    let theme_input = prompt_input(theme_prompt)?;
    let theme = match theme_input.trim() {
        "1" | "textbook" | "" => "textbook".to_string(),
        "2" | "default" => "default".to_string(),
        "3" | "sepia" => "sepia".to_string(),
        "4" | "nord" => "nord".to_string(),
        "5" | "solarized" => "solarized".to_string(),
        other => {
            if SUPPORTED_THEMES.contains(&other) {
                other.to_string()
            } else {
                println!(
                    "{}Unknown theme \"{}\", defaulting to textbook.{}",
                    colors.yellow, other, colors.reset
                );
                "textbook".to_string()
            }
        }
    };

    let desc_prompt = "Project description (optional): ";
    let desc_input = prompt_input(desc_prompt)?;
    let description = desc_input.trim().to_string();

    let sample_prompt = "Include sample content and showcase sections? [Y/n]: ";
    let sample_input = prompt_input(sample_prompt)?;
    let sample_content = !sample_input.trim().eq_ignore_ascii_case("n");

    Ok(ProjectConfig {
        title,
        author,
        theme,
        description,
        sample_content,
    })
}

fn prompt_input(label: &str) -> Result<String, String> {
    print!("{label}");
    io::stdout()
        .flush()
        .map_err(|e| format!("failed to flush stdout: {e}"))?;

    let mut buf = String::new();
    io::stdin()
        .read_line(&mut buf)
        .map_err(|e| format!("failed to read line: {e}"))?;
    Ok(buf)
}

fn write_project_files(
    target_dir: &Path,
    config: &ProjectConfig,
    colors: Colors,
) -> Result<(), String> {
    let index_path = target_dir.join("index.du");
    let style_path = target_dir.join("style.css");
    let gitignore_path = target_dir.join(".gitignore");
    let readme_path = target_dir.join("README.md");

    let index_du_content = if config.sample_content {
        sample_index_du(config)
    } else {
        minimal_index_du(config)
    };

    fs::write(&index_path, index_du_content.as_bytes())
        .map_err(|e| format!("failed to write index.du: {e}"))?;
    println!(
        "  {}created{} {}",
        colors.green,
        colors.reset,
        index_path.display()
    );

    if !style_path.exists() {
        let style_content = r#"/* Custom CSS overrides for DocUP */
/* This file is loaded via: docup build index.du -s style.css */

/* Example: customize maximum reading container width */
/*
body {
  max-width: 820px;
}
*/
"#;
        fs::write(&style_path, style_content.as_bytes())
            .map_err(|e| format!("failed to write style.css: {e}"))?;
        println!(
            "  {}created{} {}",
            colors.green,
            colors.reset,
            style_path.display()
        );
    }

    if !gitignore_path.exists() {
        let gitignore_content = "# DocUP generated outputs\n*.html\ndist/\n";
        fs::write(&gitignore_path, gitignore_content.as_bytes())
            .map_err(|e| format!("failed to write .gitignore: {e}"))?;
        println!(
            "  {}created{} {}",
            colors.green,
            colors.reset,
            gitignore_path.display()
        );
    }

    if !readme_path.exists() {
        let readme_content = format!(
            "# {}\n\n{}\n\n## Quick Start\n\n```bash\n# Live development server\ndocup watch index.du\n\n# Compile to standalone HTML5\ndocup build index.du\n\n# Format documents\ndocup fmt\n```\n",
            config.title,
            if config.description.is_empty() {
                "A standalone document generated with DocUP."
            } else {
                &config.description
            }
        );
        fs::write(&readme_path, readme_content.as_bytes())
            .map_err(|e| format!("failed to write README.md: {e}"))?;
        println!(
            "  {}created{} {}",
            colors.green,
            colors.reset,
            readme_path.display()
        );
    }

    Ok(())
}

fn dir_name_to_title(path: &Path) -> String {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let name = canonical
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("document");

    let mut words = Vec::new();
    for part in name.split(['-', '_']) {
        if !part.is_empty() {
            let mut c = part.chars();
            let first = c.next().unwrap().to_ascii_uppercase();
            let rest: String = c.collect();
            words.push(format!("{first}{rest}"));
        }
    }
    if words.is_empty() {
        "My Document".to_string()
    } else {
        words.join(" ")
    }
}

fn default_author_name() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "Author".to_string())
}

fn minimal_index_du(cfg: &ProjectConfig) -> String {
    let mut out = String::new();
    out.push_str("meta {\n");
    out.push_str(&format!("    title: \"{}\",\n", cfg.title));
    out.push_str(&format!("    author: \"{}\",\n", cfg.author));
    out.push_str("    version: \"0.1.0\",\n");
    out.push_str(&format!("    theme: \"{}\"", cfg.theme));
    if !cfg.description.is_empty() {
        out.push_str(",\n");
        out.push_str(&format!("    description: \"{}\"\n", cfg.description));
    } else {
        out.push('\n');
    }
    out.push_str("}\n\n");
    out.push_str(&format!("h(1) {{ {} }}\n\n", cfg.title));
    out.push_str("p {\n    Welcome to your new DocUP document.\n}\n");
    out
}

fn sample_index_du(cfg: &ProjectConfig) -> String {
    let mut out = String::new();
    out.push_str("meta {\n");
    out.push_str(&format!("    title: \"{}\",\n", cfg.title));
    out.push_str(&format!("    author: \"{}\",\n", cfg.author));
    out.push_str("    version: \"0.1.0\",\n");
    out.push_str(&format!("    theme: \"{}\"", cfg.theme));
    if !cfg.description.is_empty() {
        out.push_str(",\n");
        out.push_str(&format!("    description: \"{}\"\n", cfg.description));
    } else {
        out.push('\n');
    }
    out.push_str("}\n\n");

    out.push_str(&format!("h(1) {{ {} }}\n\n", cfg.title));
    out.push_str("toc {}\n\n");

    out.push_str("h(2, id: \"introduction\") { 1. Introduction }\n\n");
    out.push_str("p {\n");
    out.push_str("    This document was generated using code{docup} with the ");
    out.push_str(&format!("b{{{}}} theme. ", cfg.theme));
    out.push_str(
        "DocUP compiles unambiguous markup into a standalone HTML5 file with zero dependencies.\n",
    );
    out.push_str("}\n\n");

    out.push_str("callout(type: \"tip\") {\n");
    out.push_str("    Start the live preview dev server with code{docup watch index.du} and edit this file in your favorite text editor.\n");
    out.push_str("}\n\n");

    out.push_str("h(2, id: \"features\") { 2. Document Features }\n\n");
    out.push_str("p {\n");
    out.push_str("    DocUP supports inline elements like b{bold}, i{italic}, inline code{let x = 42;}, and mathematical expressions such as m{e^{i\\pi} + 1 = 0}.\n");
    out.push_str("}\n\n");

    out.push_str("codeblock(lang: \"rust\", file: \"src/main.rs\", line_numbers: true) {! \n");
    out.push_str("fn main() {\n");
    out.push_str("    println!(\"Hello from DocUP!\");\n");
    out.push_str("}\n");
    out.push_str("!}\n\n");

    out.push_str("h(3, id: \"math\") { 2.1 Mathematics }\n\n");
    out.push_str("math {! \n");
    out.push_str("\\int_{-\\infty}^{\\infty} e^{-x^2} dx = \\sqrt{\\pi}\n");
    out.push_str("!}\n\n");

    out.push_str("h(2, id: \"checklist\") { 3. Next Steps }\n\n");
    out.push_str("list {\n");
    out.push_str("    task(done: true) { Initialized project with DocUP }\n");
    out.push_str("    task(done: false) { Customize index.du content }\n");
    out.push_str("    task(done: false) { Run code{docup build index.du} to compile }\n");
    out.push_str("}\n");

    out
}
