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
    pub fn load(root: &Path) -> Result<Self> {
        let config_path = root.join(".beans.yml");
        let content = std::fs::read_to_string(&config_path)
            .with_context(|| format!("failed to read {}", config_path.display()))?;
        let config: BeansConfig = serde_yml::from_str(&content)
            .with_context(|| format!("failed to parse {}", config_path.display()))?;
        Ok(config)
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
