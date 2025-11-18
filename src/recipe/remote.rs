// Remote recipe parsing and fetching
//
// Handles `@org/repo/path/script.js@ref` syntax for remote recipes

use anyhow::{anyhow, Result};
use std::path::PathBuf;

/// Parsed remote recipe reference
#[derive(Debug, Clone, PartialEq)]
pub struct RemoteRecipe {
    /// Git host (e.g., "github.com", "gitlab.com")
    pub host: String,

    /// Organization/user name
    pub org: String,

    /// Repository name
    pub repo: String,

    /// Path to script within repository
    pub path: String,

    /// Optional git ref (branch, tag, or commit SHA)
    pub git_ref: Option<String>,
}

impl RemoteRecipe {
    /// Parse a remote recipe reference from `@` prefix syntax
    ///
    /// Examples:
    /// - `@tuist/recipes/build.js` → github.com/tuist/recipes, path: build.js, ref: main
    /// - `@tuist/recipes/build.js@v1.0.0` → github.com/tuist/recipes, path: build.js, ref: v1.0.0
    /// - `@gitlab.com/org/repo/script.js` → gitlab.com/org/repo, path: script.js, ref: main
    pub fn parse(input: &str) -> Result<Self> {
        // Strip @ prefix
        let input = input
            .strip_prefix('@')
            .ok_or_else(|| anyhow!("Remote recipe must start with @"))?;

        // Split by @ for git ref
        let (path_part, git_ref) = if let Some(idx) = input.rfind('@') {
            let (path, ref_str) = input.split_at(idx);
            (path, Some(ref_str[1..].to_string()))
        } else {
            (input, None)
        };

        // Split path by /
        let parts: Vec<&str> = path_part.split('/').collect();

        if parts.len() < 3 {
            return Err(anyhow!(
                "Remote recipe must have at least org/repo/path format"
            ));
        }

        // Check if first part is a host (contains a dot)
        let (host, org_idx) = if parts[0].contains('.') {
            (parts[0].to_string(), 1)
        } else {
            ("github.com".to_string(), 0)
        };

        let org = parts[org_idx].to_string();
        let repo = parts[org_idx + 1].to_string();
        let path = parts[org_idx + 2..].join("/");

        if path.is_empty() {
            return Err(anyhow!("Remote recipe must specify a script path"));
        }

        Ok(RemoteRecipe {
            host,
            org,
            repo,
            path,
            git_ref,
        })
    }

    /// Get the Git repository URL
    pub fn git_url(&self) -> String {
        format!("https://{}/{}/{}.git", self.host, self.org, self.repo)
    }

    /// Get the cache directory path for this remote recipe
    ///
    /// Uses XDG cache directory: ~/.cache/fabrik/recipes/{host}/{org}/{repo}/{ref}/
    pub fn cache_dir(&self) -> Result<PathBuf> {
        let base =
            dirs::cache_dir().ok_or_else(|| anyhow!("Could not determine cache directory"))?;

        let git_ref = self.git_ref.as_deref().unwrap_or("main");

        Ok(base
            .join("fabrik")
            .join("recipes")
            .join(&self.host)
            .join(&self.org)
            .join(&self.repo)
            .join(git_ref))
    }

    /// Get the full path to the script file in the cache
    pub fn script_path(&self) -> Result<PathBuf> {
        Ok(self.cache_dir()?.join(&self.path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_github() {
        let recipe = RemoteRecipe::parse("@tuist/recipes/build.js").unwrap();
        assert_eq!(recipe.host, "github.com");
        assert_eq!(recipe.org, "tuist");
        assert_eq!(recipe.repo, "recipes");
        assert_eq!(recipe.path, "build.js");
        assert_eq!(recipe.git_ref, None);
    }

    #[test]
    fn test_parse_with_version() {
        let recipe = RemoteRecipe::parse("@tuist/recipes/build.js@v1.0.0").unwrap();
        assert_eq!(recipe.host, "github.com");
        assert_eq!(recipe.org, "tuist");
        assert_eq!(recipe.repo, "recipes");
        assert_eq!(recipe.path, "build.js");
        assert_eq!(recipe.git_ref, Some("v1.0.0".to_string()));
    }

    #[test]
    fn test_parse_nested_path() {
        let recipe = RemoteRecipe::parse("@tuist/recipes/scripts/deploy/prod.js").unwrap();
        assert_eq!(recipe.path, "scripts/deploy/prod.js");
    }

    #[test]
    fn test_parse_explicit_gitlab() {
        let recipe = RemoteRecipe::parse("@gitlab.com/myorg/myrepo/script.js").unwrap();
        assert_eq!(recipe.host, "gitlab.com");
        assert_eq!(recipe.org, "myorg");
        assert_eq!(recipe.repo, "myrepo");
        assert_eq!(recipe.path, "script.js");
    }

    #[test]
    fn test_parse_self_hosted() {
        let recipe = RemoteRecipe::parse("@git.company.com/team/project/build.js@main").unwrap();
        assert_eq!(recipe.host, "git.company.com");
        assert_eq!(recipe.org, "team");
        assert_eq!(recipe.repo, "project");
        assert_eq!(recipe.path, "build.js");
        assert_eq!(recipe.git_ref, Some("main".to_string()));
    }

    #[test]
    fn test_parse_missing_prefix() {
        let result = RemoteRecipe::parse("tuist/recipes/build.js");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_too_short() {
        let result = RemoteRecipe::parse("@tuist/recipes");
        assert!(result.is_err());
    }

    #[test]
    fn test_git_url() {
        let recipe = RemoteRecipe::parse("@tuist/recipes/build.js").unwrap();
        assert_eq!(recipe.git_url(), "https://github.com/tuist/recipes.git");
    }

    #[test]
    fn test_cache_dir_structure() {
        let recipe = RemoteRecipe::parse("@tuist/recipes/build.js@v1.0.0").unwrap();
        let cache_dir = recipe.cache_dir().unwrap();

        // Should end with: fabrik/recipes/github.com/tuist/recipes/v1.0.0
        assert!(cache_dir
            .to_string_lossy()
            .contains("fabrik/recipes/github.com/tuist/recipes/v1.0.0"));
    }
}
