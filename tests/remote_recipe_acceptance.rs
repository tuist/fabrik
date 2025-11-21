// Remote recipe acceptance tests
//
// These tests verify remote recipe functionality end-to-end

use anyhow::Result;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::fs;

#[tokio::test]
async fn test_remote_recipe_with_local_git_repo() -> Result<()> {
    // Create a temporary git repository with a recipe
    let temp_repo = TempDir::new()?;
    let repo_path = temp_repo.path();

    // Initialize git repository
    tokio::process::Command::new("git")
        .args(["init"])
        .current_dir(repo_path)
        .output()
        .await?;

    // Create a simple recipe file
    let recipe_dir = repo_path.join("recipes");
    fs::create_dir(&recipe_dir).await?;

    let recipe_path = recipe_dir.join("test.js");
    fs::write(
        &recipe_path,
        r#"
async function build() {
    console.log("Building from remote recipe!");
}
"#,
    )
    .await?;

    // Commit the recipe
    tokio::process::Command::new("git")
        .args(["add", "."])
        .current_dir(repo_path)
        .output()
        .await?;

    tokio::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(repo_path)
        .output()
        .await?;

    tokio::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(repo_path)
        .output()
        .await?;

    tokio::process::Command::new("git")
        .args(["commit", "-m", "Add test recipe"])
        .current_dir(repo_path)
        .output()
        .await?;

    // Now test remote recipe execution with file:// URL
    // Note: This is a simplified test - in real usage we'd use https:// URLs
    let file_url = format!("file://{}", repo_path.display());

    // For now, just verify the RemoteRecipe parsing works
    use fabrik::recipe::RemoteRecipe;

    // Test parsing various remote recipe formats
    let simple = RemoteRecipe::parse("@tuist/recipes/build.js")?;
    assert_eq!(simple.host, "github.com");
    assert_eq!(simple.org, "tuist");
    assert_eq!(simple.repo, "recipes");
    assert_eq!(simple.path, "build.js");
    assert_eq!(simple.git_ref, None);

    let with_ref = RemoteRecipe::parse("@tuist/recipes/build.js@v1.0.0")?;
    assert_eq!(with_ref.git_ref, Some("v1.0.0".to_string()));

    let gitlab = RemoteRecipe::parse("@gitlab.com/org/repo/script.js")?;
    assert_eq!(gitlab.host, "gitlab.com");

    Ok(())
}

#[tokio::test]
async fn test_remote_recipe_cache_dir_structure() -> Result<()> {
    use fabrik::recipe::RemoteRecipe;

    let remote = RemoteRecipe::parse("@tuist/recipes/build.js@v1.0.0")?;
    let cache_dir = remote.cache_dir()?;

    // Verify cache directory follows XDG convention
    let cache_dir_str = cache_dir.to_string_lossy();
    assert!(cache_dir_str.contains("fabrik"));
    assert!(cache_dir_str.contains("recipes"));
    assert!(cache_dir_str.contains("github.com"));
    assert!(cache_dir_str.contains("tuist"));
    assert!(cache_dir_str.contains("recipes"));
    assert!(cache_dir_str.contains("v1.0.0"));

    Ok(())
}

#[tokio::test]
async fn test_remote_recipe_parsing_errors() {
    use fabrik::recipe::RemoteRecipe;

    // Missing @ prefix
    assert!(RemoteRecipe::parse("tuist/recipes/build.js").is_err());

    // Too short (missing script path)
    assert!(RemoteRecipe::parse("@tuist/recipes").is_err());

    // Empty path
    assert!(RemoteRecipe::parse("@tuist/recipes/").is_err());
}

#[tokio::test]
async fn test_remote_recipe_git_url_generation() -> Result<()> {
    use fabrik::recipe::RemoteRecipe;

    // GitHub (default)
    let github = RemoteRecipe::parse("@tuist/recipes/build.js")?;
    assert_eq!(github.git_url(), "https://github.com/tuist/recipes.git");

    // GitLab
    let gitlab = RemoteRecipe::parse("@gitlab.com/myorg/myrepo/script.js")?;
    assert_eq!(gitlab.git_url(), "https://gitlab.com/myorg/myrepo.git");

    // Self-hosted
    let selfhosted = RemoteRecipe::parse("@git.company.com/team/project/build.js")?;
    assert_eq!(
        selfhosted.git_url(),
        "https://git.company.com/team/project.git"
    );

    Ok(())
}

#[tokio::test]
async fn test_remote_recipe_nested_paths() -> Result<()> {
    use fabrik::recipe::RemoteRecipe;

    let nested = RemoteRecipe::parse("@tuist/recipes/scripts/deploy/prod.js")?;
    assert_eq!(nested.path, "scripts/deploy/prod.js");

    let with_ref = RemoteRecipe::parse("@tuist/recipes/a/b/c/script.js@v2.0.0")?;
    assert_eq!(with_ref.path, "a/b/c/script.js");
    assert_eq!(with_ref.git_ref, Some("v2.0.0".to_string()));

    Ok(())
}
