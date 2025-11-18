// Recipe metadata parsing and validation
//
// This module handles parsing the `export const recipe = {...}` metadata
// from portable recipe files.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Recipe metadata exported from a recipe file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeMetadata {
    /// Recipe name
    pub name: String,

    /// Recipe version (semver)
    pub version: String,

    /// Input globs for cache invalidation
    #[serde(default)]
    pub inputs: Vec<String>,

    /// Output paths to cache
    #[serde(default)]
    pub outputs: Vec<String>,

    /// Environment variables to track
    #[serde(default)]
    pub env: Vec<String>,

    /// Cache TTL (e.g., "7d", "2h")
    #[serde(default, rename = "cacheTtl")]
    pub cache_ttl: Option<String>,

    /// Description
    #[serde(default)]
    pub description: Option<String>,
}

impl Default for RecipeMetadata {
    fn default() -> Self {
        Self {
            name: "unnamed".to_string(),
            version: "0.0.0".to_string(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            env: Vec::new(),
            cache_ttl: None,
            description: None,
        }
    }
}

impl RecipeMetadata {
    /// Parse recipe metadata from a recipe file
    pub async fn from_file(_path: &PathBuf) -> Result<Option<Self>> {
        // TODO: Execute the recipe file and extract the metadata export
        // For now, return None (no metadata)
        Ok(None)
    }

    /// Validate recipe metadata
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(anyhow!("Recipe name cannot be empty"));
        }

        // Validate version is semver-ish
        if !self.version.contains('.') {
            return Err(anyhow!(
                "Recipe version must be in semver format (e.g., 1.0.0)"
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_metadata() {
        let metadata = RecipeMetadata::default();
        assert_eq!(metadata.name, "unnamed");
        assert_eq!(metadata.version, "0.0.0");
    }

    #[test]
    fn test_validate_empty_name() {
        let metadata = RecipeMetadata {
            name: "".to_string(),
            ..Default::default()
        };
        assert!(metadata.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_version() {
        let metadata = RecipeMetadata {
            version: "1".to_string(),
            ..Default::default()
        };
        assert!(metadata.validate().is_err());
    }

    #[test]
    fn test_validate_valid_metadata() {
        let metadata = RecipeMetadata {
            name: "test-recipe".to_string(),
            version: "1.0.0".to_string(),
            inputs: vec!["src/**/*.ts".to_string()],
            outputs: vec!["dist/".to_string()],
            env: vec!["NODE_ENV".to_string()],
            cache_ttl: Some("7d".to_string()),
            description: Some("Test recipe".to_string()),
        };

        assert!(metadata.validate().is_ok());
    }
}
