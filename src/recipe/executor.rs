// Recipe executor - Runs portable recipes in QuickJS runtime

use anyhow::Result;
use rquickjs::async_with;
use std::path::PathBuf;

use super::runtime::create_fabrik_runtime_with_dir;

/// Executes portable recipes (JavaScript files with Fabrik APIs)
pub struct RecipeExecutor {
    recipe_path: PathBuf,
}

impl RecipeExecutor {
    /// Create a new recipe executor
    pub fn new(recipe_path: PathBuf) -> Self {
        Self { recipe_path }
    }

    /// Execute a recipe at root level
    ///
    /// Recipes are plain JavaScript files that run from top to bottom.
    /// They should NOT export functions - all logic is at the root level.
    pub async fn execute(&self) -> Result<()> {
        tracing::info!("[fabrik] Executing recipe: {:?}", self.recipe_path);

        // Read recipe file
        let recipe_code = tokio::fs::read_to_string(&self.recipe_path).await?;

        // Get recipe directory for config discovery
        let recipe_dir = self
            .recipe_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .to_path_buf();

        // Create QuickJS runtime with Fabrik APIs
        let (_runtime, context) = create_fabrik_runtime_with_dir(recipe_dir).await?;

        // Execute recipe at root level (wrap in async IIFE)
        async_with!(context => |ctx| {
            let wrapped_code = format!("(async () => {{ {} }})();", recipe_code);
            let promise: rquickjs::Promise = ctx.eval(wrapped_code.as_bytes())?;

            // Wait for promise to complete
            promise.into_future::<()>().await?;

            Ok::<_, rquickjs::Error>(())
        })
        .await?;

        tracing::info!("[fabrik] Recipe completed successfully");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_simple_recipe() {
        // Create a temporary recipe file
        let temp_dir = tempfile::tempdir().unwrap();
        let recipe_path = temp_dir.path().join("test.recipe.js");

        let recipe_code = r#"
            console.log("Running simple recipe");
            const exitCode = await Fabrik.exec("echo", ["hello from recipe"]);
            if (exitCode !== 0) {
                throw new Error("Command failed");
            }
        "#;

        tokio::fs::write(&recipe_path, recipe_code).await.unwrap();

        // Execute the recipe at root level
        let executor = RecipeExecutor::new(recipe_path);
        let result = executor.execute().await;

        assert!(result.is_ok(), "Recipe execution should succeed");
    }

    #[tokio::test]
    async fn test_execute_file_operations() {
        let temp_dir = tempfile::tempdir().unwrap();
        let recipe_path = temp_dir.path().join("file_test.recipe.js");
        let test_file = temp_dir.path().join("test_output.txt");

        // Pre-create test file for the recipe to check
        tokio::fs::write(&test_file, b"test data").await.unwrap();

        // Escape backslashes for Windows paths in JavaScript strings
        let test_file_str = test_file.to_str().unwrap().replace('\\', "\\\\");
        let temp_dir_str = temp_dir.path().to_str().unwrap().replace('\\', "\\\\");

        let recipe_code = format!(
            r#"
            console.log("Checking file operations");
            const exists = await Fabrik.exists("{}");
            if (!exists) {{
                throw new Error("File should exist");
            }}

            const files = await Fabrik.glob("{}/*.txt");
            if (files.length === 0) {{
                throw new Error("Should find at least one .txt file");
            }}
        "#,
            test_file_str, temp_dir_str
        );

        tokio::fs::write(&recipe_path, recipe_code).await.unwrap();

        // Execute recipe at root level
        let executor = RecipeExecutor::new(recipe_path);
        executor.execute().await.unwrap();
    }

    #[tokio::test]
    async fn test_execute_root_level() {
        let temp_dir = tempfile::tempdir().unwrap();
        let recipe_path = temp_dir.path().join("root.recipe.js");

        // Root-level recipe - just tests basic execution
        let recipe_code = r#"
            console.log("Executing at root level");

            const files = await Fabrik.glob("*.toml");
            console.log("Found", files.length, "toml files");

            if (files.length === 0) {
                throw new Error("Expected to find some .toml files");
            }
        "#;

        tokio::fs::write(&recipe_path, recipe_code).await.unwrap();

        // Execute recipe at root level
        let executor = RecipeExecutor::new(recipe_path);
        executor.execute().await.unwrap();
    }
}
