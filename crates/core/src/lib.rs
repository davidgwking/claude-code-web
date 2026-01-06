/// Core library for the workspace
/// 
/// This crate provides foundational functionality for the workspace.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub version: String,
}

impl Config {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = Config::new("test", "1.0.0");
        assert_eq!(config.name, "test");
        assert_eq!(config.version, "1.0.0");
    }
}
