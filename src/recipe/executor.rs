// Recipe executor - Runs portable recipes in QuickJS runtime
//
// This module handles recipe execution with caching and metadata validation.

use anyhow::Result;
use rquickjs::async_with;
use std::path::PathBuf;

use super::metadata::RecipeMetadata;
use super::runtime::create_fabrik_runtime;

/// Executes portable recipes (JavaScript files with Fabrik APIs)
pub struct RecipeExecutor {
    recipe_path: PathBuf,
}

impl RecipeExecutor {
    /// Create a new recipe executor
    pub fn new(recipe_path: PathBuf) -> Self {
        Self { recipe_path }
    }

    /// Execute a recipe
    ///
    /// If target is None, executes the entire script at root level.
    /// If target is Some(name), calls the exported function with that name.
    pub async fn execute(&self, target: Option<&str>) -> Result<()> {
        if let Some(t) = target {
            tracing::info!(
                "[fabrik] Executing recipe: {:?} target: {}",
                self.recipe_path,
                t
            );
        } else {
            tracing::info!("[fabrik] Executing recipe: {:?}", self.recipe_path);
        }

        // Read recipe file
        let recipe_code = tokio::fs::read_to_string(&self.recipe_path).await?;

        // Create QuickJS runtime with Fabrik APIs
        let (_runtime, context) = create_fabrik_runtime().await?;

        // Execute recipe
        async_with!(context => |ctx| {
            if let Some(target_name) = target {
                // Function-based: evaluate the file, then call the target function
                ctx.eval::<(), _>(recipe_code.as_bytes())?;

                let globals = ctx.globals();
                let target_fn: rquickjs::Function = globals.get(target_name)?;
                let promise: rquickjs::Promise = target_fn.call(())?;

                // Wait for promise to complete
                promise.into_future::<()>().await?;
            } else {
                // Root-level: wrap in IIFE and execute
                let wrapped_code = format!("(async () => {{ {} }})();", recipe_code);
                let promise: rquickjs::Promise = ctx.eval(wrapped_code.as_bytes())?;

                // Wait for promise to complete
                promise.into_future::<()>().await?;
            }

            Ok::<_, rquickjs::Error>(())
        })
        .await?;

        tracing::info!("[fabrik] Recipe completed successfully");

        Ok(())
    }

    /// Parse recipe metadata (if present)
    pub async fn metadata(&self) -> Result<Option<RecipeMetadata>> {
        RecipeMetadata::from_file(&self.recipe_path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_execute_simple_recipe() {
        // Create a temporary recipe file
        let temp_dir = tempfile::tempdir().unwrap();
        let recipe_path = temp_dir.path().join("test.recipe.js");

        let recipe_code = r#"
            async function test() {
                const result = await Fabrik.exec("echo", ["hello from recipe"]);
                if (result.exit_code !== 0) {
                    throw new Error("Command failed");
                }
            }
        "#;

        tokio::fs::write(&recipe_path, recipe_code).await.unwrap();

        // Execute the recipe (function-based)
        let executor = RecipeExecutor::new(recipe_path);
        let result = executor.execute(Some("test")).await;

        assert!(result.is_ok(), "Recipe execution should succeed");
    }

    #[tokio::test]
    async fn test_execute_file_operations() {
        let temp_dir = tempfile::tempdir().unwrap();
        let recipe_path = temp_dir.path().join("file_test.recipe.js");
        let test_file = temp_dir.path().join("test_output.txt");

        let recipe_code = format!(
            r#"
            async function build() {{
                const encoder = new TextEncoder();
                const data = encoder.encode("Recipe output");
                await Fabrik.writeFile("{}", data);

                const exists = await Fabrik.exists("{}");
                if (!exists) {{
                    throw new Error("File should exist");
                }}
            }}
        "#,
            test_file.display(),
            test_file.display()
        );

        tokio::fs::write(&recipe_path, recipe_code).await.unwrap();

        // Execute recipe (function-based)
        let executor = RecipeExecutor::new(recipe_path);
        executor.execute(Some("build")).await.unwrap();

        // Verify file was created
        assert!(test_file.exists(), "Recipe should create output file");

        let content = tokio::fs::read_to_string(&test_file).await.unwrap();
        assert_eq!(content, "Recipe output");
    }

    #[tokio::test]
    async fn test_execute_root_level() {
        let temp_dir = tempfile::tempdir().unwrap();
        let recipe_path = temp_dir.path().join("root.recipe.js");
        let test_file = temp_dir.path().join("root_output.txt");

        // Root-level recipe (no functions)
        let recipe_code = format!(
            r#"
            console.log("Executing at root level");

            const encoder = new TextEncoder();
            await Fabrik.writeFile("{}", encoder.encode("Root level output"));

            const files = await Fabrik.glob("*.toml");
            console.log("Found", files.length, "toml files");
        "#,
            test_file.display()
        );

        tokio::fs::write(&recipe_path, recipe_code).await.unwrap();

        // Execute recipe (root-level, no target)
        let executor = RecipeExecutor::new(recipe_path);
        executor.execute(None).await.unwrap();

        // Verify file was created
        assert!(test_file.exists(), "Recipe should create output file");

        let content = tokio::fs::read_to_string(&test_file).await.unwrap();
        assert_eq!(content, "Root level output");
    }
}
