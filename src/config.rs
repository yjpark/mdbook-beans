use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BeansConfig {
    pub project: ProjectConfig,
    pub beans: BeansPathConfig,
}

#[derive(Debug, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct BeansPathConfig {
    pub path: PathBuf,
    pub prefix: String,
}

impl BeansConfig {
    /// Load `.beans.yml` by searching from `root` upward through parent directories.
    /// This supports books nested inside a project (e.g., `project/docs/` where
    /// `.beans.yml` is at `project/`).
    pub fn load(root: &Path) -> Result<Self> {
        let config_path = Self::find_config(root)
            .with_context(|| format!("no .beans.yml found at or above {}", root.display()))?;
        let content = std::fs::read_to_string(&config_path)
            .with_context(|| format!("failed to read {}", config_path.display()))?;
        let config: BeansConfig = serde_yml::from_str(&content)
            .with_context(|| format!("failed to parse {}", config_path.display()))?;
        Ok(config)
    }

    /// Find `.beans.yml` by walking up from `start` through parent directories.
    fn find_config(start: &Path) -> Option<PathBuf> {
        let mut dir = start.to_path_buf();
        loop {
            let candidate = dir.join(".beans.yml");
            if candidate.exists() {
                return Some(candidate);
            }
            if !dir.pop() {
                return None;
            }
        }
    }

    /// Return the directory containing `.beans.yml` (the project root).
    pub fn project_root(root: &Path) -> Result<PathBuf> {
        let config_path = Self::find_config(root)
            .with_context(|| format!("no .beans.yml found at or above {}", root.display()))?;
        Ok(config_path.parent().unwrap().to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_beans_yml() {
        let yaml = r#"
project:
    name: test-project
beans:
    path: .beans
    prefix: test-
    id_length: 4
    default_status: todo
    default_type: task
worktree:
    base_ref: main
    setup: ""
    run: ""
    integrate: local
agent:
    enabled: false
    default_mode: act
server:
    port: 8881
"#;
        let config: BeansConfig = serde_yml::from_str(yaml).unwrap();
        assert_eq!(config.project.name, "test-project");
        assert_eq!(config.beans.path, PathBuf::from(".beans"));
        assert_eq!(config.beans.prefix, "test-");
    }
}
