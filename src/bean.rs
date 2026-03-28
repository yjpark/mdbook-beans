use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::config::BeansConfig;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BeanStatus {
    Draft,
    Todo,
    InProgress,
    Done,
    Completed,
    Archived,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BeanType {
    Epic,
    Feature,
    Task,
    Bug,
    Spike,
    Chore,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BeanFrontmatter {
    pub title: String,
    pub status: BeanStatus,
    #[serde(rename = "type")]
    pub bean_type: BeanType,
    #[serde(default = "default_priority")]
    pub priority: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub parent: Option<String>,
    #[serde(default)]
    pub blocked_by: Vec<String>,
}

fn default_priority() -> String {
    "normal".to_string()
}

#[derive(Debug, Clone)]
pub struct Bean {
    pub id: String,
    pub frontmatter: BeanFrontmatter,
    pub body: String,
}

/// Extract bean ID from filename.
/// Pattern: `{prefix}{id}--{slug}.md` → `{prefix}{id}`
/// Example: `litmus-0uoe--update-litmus-cli.md` → `litmus-0uoe`
fn bean_id_from_filename(filename: &str) -> Option<String> {
    let name = filename.strip_suffix(".md")?;
    let id_end = name.find("--")?;
    Some(name[..id_end].to_string())
}

/// Parse a bean markdown file into frontmatter + body.
/// Strips YAML comment lines (starting with #) from frontmatter before parsing.
fn parse_bean(content: &str, filename: &str) -> Result<Bean> {
    let id = bean_id_from_filename(filename)
        .with_context(|| format!("cannot extract bean ID from filename: {filename}"))?;

    let trimmed = content.trim_start();
    let rest = trimmed
        .strip_prefix("---")
        .with_context(|| format!("missing opening --- in {filename}"))?;

    let end = rest
        .find("\n---")
        .with_context(|| format!("missing closing --- in {filename}"))?;

    let yaml_block = &rest[..end];
    let body = rest[end + 4..].trim().to_string();

    // Strip YAML comment lines (like `# bean-id`)
    let cleaned_yaml: String = yaml_block
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");

    let frontmatter: BeanFrontmatter = serde_yml::from_str(&cleaned_yaml)
        .with_context(|| format!("failed to parse frontmatter in {filename}"))?;

    Ok(Bean {
        id,
        frontmatter,
        body,
    })
}

/// Load all bean files from the beans directory, skipping archive/ and dotfiles.
pub fn load_beans(root: &Path, config: &BeansConfig) -> Result<Vec<Bean>> {
    let beans_dir = root.join(&config.beans.path);

    if !beans_dir.exists() {
        return Ok(Vec::new());
    }

    let mut beans = Vec::new();

    for entry in std::fs::read_dir(&beans_dir)
        .with_context(|| format!("failed to read {}", beans_dir.display()))?
    {
        let entry = entry?;
        let filename = entry.file_name();
        let filename_str = filename.to_string_lossy();

        // Skip directories (archive/, .conversations/) and non-markdown files
        if entry.file_type()?.is_dir() {
            continue;
        }
        if !filename_str.ends_with(".md") {
            continue;
        }
        if filename_str.starts_with('.') {
            continue;
        }

        let content = std::fs::read_to_string(entry.path())?;
        match parse_bean(&content, &filename_str) {
            Ok(bean) => beans.push(bean),
            Err(e) => eprintln!("warning: skipping {filename_str}: {e}"),
        }
    }

    // Sort by ID for stable output
    beans.sort_by(|a, b| a.id.cmp(&b.id));

    Ok(beans)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_bean_id_from_filename() {
        assert_eq!(
            bean_id_from_filename("litmus-0uoe--update-litmus-cli.md"),
            Some("litmus-0uoe".to_string())
        );
        assert_eq!(
            bean_id_from_filename("beans-a1b2--some-task.md"),
            Some("beans-a1b2".to_string())
        );
        assert_eq!(bean_id_from_filename("no-separator.md"), None);
        assert_eq!(bean_id_from_filename("not-markdown.txt"), None);
    }

    #[test]
    fn parse_bean_with_comment_in_frontmatter() {
        let content = r#"---
# beans-test
title: Test task
status: todo
type: task
priority: normal
tags: []
created_at: 2025-01-01T00:00:00Z
updated_at: 2025-01-01T00:00:00Z
---

This is the body content.
"#;
        let bean = parse_bean(content, "beans-test--test-task.md").unwrap();
        assert_eq!(bean.id, "beans-test");
        assert_eq!(bean.frontmatter.title, "Test task");
        assert_eq!(bean.frontmatter.status, BeanStatus::Todo);
        assert_eq!(bean.frontmatter.bean_type, BeanType::Task);
        assert_eq!(bean.body, "This is the body content.");
    }

    #[test]
    fn parse_bean_with_parent_and_blocked_by() {
        let content = r#"---
title: Subtask
status: in-progress
type: feature
priority: high
parent: beans-epic
blocked_by:
    - beans-dep1
    - beans-dep2
created_at: 2025-01-01T00:00:00Z
updated_at: 2025-01-01T00:00:00Z
---

Subtask body.
"#;
        let bean = parse_bean(content, "beans-sub1--subtask.md").unwrap();
        assert_eq!(bean.frontmatter.parent, Some("beans-epic".to_string()));
        assert_eq!(bean.frontmatter.blocked_by, vec!["beans-dep1", "beans-dep2"]);
        assert_eq!(bean.frontmatter.status, BeanStatus::InProgress);
    }

    #[test]
    fn parse_epic_bean() {
        let content = r#"---
title: Big epic
status: in-progress
type: epic
priority: critical
tags:
    - milestone
created_at: 2025-01-01T00:00:00Z
updated_at: 2025-01-01T00:00:00Z
---

Epic description.
"#;
        let bean = parse_bean(content, "beans-ep01--big-epic.md").unwrap();
        assert_eq!(bean.frontmatter.bean_type, BeanType::Epic);
        assert_eq!(bean.frontmatter.priority, "critical");
        assert_eq!(bean.frontmatter.tags, vec!["milestone"]);
    }
}
