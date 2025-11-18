// QuickJS runtime for portable recipes
//
// This module creates a QuickJS JavaScript runtime with Fabrik APIs exposed.

use anyhow::Result;
use rquickjs::{
    async_with,
    function::Async,
    loader::{BuiltinLoader, BuiltinResolver, ModuleLoader},
    AsyncContext, AsyncRuntime, Function,
};
use tokio::process::Command;

/// Create a QuickJS runtime with Fabrik APIs
pub async fn create_fabrik_runtime() -> Result<(AsyncRuntime, AsyncContext)> {
    // Create runtime with module loader for LLRT modules
    let resolver = BuiltinResolver::default()
        .with_module("fs")
        .with_module("fs/promises")
        .with_module("child_process")
        .with_module("path");

    let mut module_loader = ModuleLoader::default();
    module_loader
        .add_module("fs", llrt_fs::FsModule)
        .add_module("fs/promises", llrt_fs::FsPromisesModule)
        .add_module("child_process", llrt_child_process::ChildProcessModule)
        .add_module("path", llrt_path::PathModule);

    let loader = (BuiltinLoader::default(), module_loader);

    let runtime = AsyncRuntime::new()?;
    runtime.set_loader(resolver, loader).await;

    let context = AsyncContext::full(&runtime).await?;

    // Register Fabrik APIs
    async_with!(context => |ctx| {
        // Create Fabrik global object
        let fabrik = rquickjs::Object::new(ctx.clone())?;

        // File I/O functions - use closures for Async wrapper
        fabrik.set("readFile", Function::new(ctx.clone(), Async(|path: String| async move {
            let data = tokio::fs::read(&path).await?;
            Ok::<Vec<u8>, rquickjs::Error>(data)
        })))?;

        fabrik.set("writeFile", Function::new(ctx.clone(), Async(|path: String, data: Vec<u8>| async move {
            tokio::fs::write(&path, &data).await?;
            Ok::<(), rquickjs::Error>(())
        })))?;

        fabrik.set("exists", Function::new(ctx.clone(), Async(|path: String| async move {
            Ok::<bool, rquickjs::Error>(tokio::fs::metadata(&path).await.is_ok())
        })))?;

        fabrik.set("glob", Function::new(ctx.clone(), Async(|pattern: String| async move {
            match glob::glob(&pattern) {
                Ok(iter) => {
                    let paths = iter
                        .filter_map(|entry| entry.ok())
                        .map(|path| path.to_string_lossy().to_string())
                        .collect::<Vec<String>>();
                    Ok(paths)
                },
                Err(_) => Err(rquickjs::Error::Exception),
            }
        })))?;

        // Process execution - returns just exit code for now
        // TODO: Return stdout/stderr as well
        fabrik.set("exec", Function::new(ctx.clone(), Async(|command: String, args: Option<Vec<String>>| async move {
            let args = args.unwrap_or_default();

            tracing::debug!("[fabrik] Executing: {} {:?}", command, args);

            let output = match Command::new(&command).args(&args).output().await {
                Ok(o) => o,
                Err(_) => return Err(rquickjs::Error::Exception),
            };

            // Return just exit code for now
            Ok::<i32, rquickjs::Error>(output.status.code().unwrap_or(-1))
        })))?;

        // Hashing
        fabrik.set("hashFile", Function::new(ctx.clone(), Async(|path: String| async move {
            use sha2::{Digest, Sha256};

            let data = match tokio::fs::read(&path).await {
                Ok(d) => d,
                Err(_) => return Err(rquickjs::Error::Exception),
            };
            let hash = Sha256::digest(&data);
            Ok::<String, rquickjs::Error>(hex::encode(hash))
        })))?;

        // Cache operations (placeholders)
        let cache = rquickjs::Object::new(ctx.clone())?;

        cache.set("get", Function::new(ctx.clone(), Async(|hash: String| async move {
            tracing::debug!("[fabrik] Cache GET: {}", hash);
            // TODO: Integrate with actual cache storage
            Ok::<Option<Vec<u8>>, rquickjs::Error>(None)
        })))?;

        cache.set("put", Function::new(ctx.clone(), Async(|hash: String, data: Vec<u8>| async move {
            tracing::debug!("[fabrik] Cache PUT: {} ({} bytes)", hash, data.len());
            // TODO: Integrate with actual cache storage
            Ok::<(), rquickjs::Error>(())
        })))?;

        cache.set("has", Function::new(ctx.clone(), Async(|hash: String| async move {
            tracing::debug!("[fabrik] Cache HAS: {}", hash);
            // TODO: Integrate with actual cache storage
            Ok::<bool, rquickjs::Error>(false)
        })))?;

        fabrik.set("cache", cache)?;

        // Set global
        ctx.globals().set("Fabrik", fabrik)?;

        // Register LLRT Node.js-compatible modules
        // Note: Only llrt_buffer and llrt_console have direct init functions for global registration
        // fs, child_process, and path are ES modules that need module loader
        llrt_buffer::init(&ctx)?;
        llrt_console::init(&ctx)?;

        Ok::<_, rquickjs::Error>(())
    })
    .await?;

    Ok((runtime, context))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_runtime() {
        let result = create_fabrik_runtime().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fabrik_apis_available() {
        let (_runtime, context) = create_fabrik_runtime().await.unwrap();

        async_with!(context => |ctx| {
            // Test that Fabrik global is available
            let script = r#"
                typeof Fabrik !== 'undefined' &&
                typeof Fabrik.readFile === 'function' &&
                typeof Fabrik.writeFile === 'function' &&
                typeof Fabrik.exec === 'function' &&
                typeof Fabrik.glob === 'function' &&
                typeof Fabrik.hashFile === 'function' &&
                typeof Fabrik.cache.get === 'function'
            "#;

            let result: bool = ctx.eval(script.as_bytes())?;
            assert!(result, "Fabrik APIs should be available");

            Ok::<_, rquickjs::Error>(())
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_exec_command() {
        let (_runtime, context) = create_fabrik_runtime().await.unwrap();

        async_with!(context => |ctx| {
            let script = r#"
                (async () => {
                    const exitCode = await Fabrik.exec("echo", ["hello"]);
                    return exitCode;
                })()
            "#;

            let promise = ctx.eval::<rquickjs::Promise, _>(script.as_bytes())?;
            let result: i32 = promise.into_future().await?;
            assert_eq!(result, 0, "Command should succeed with exit code 0");

            Ok::<_, rquickjs::Error>(())
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_file_operations() {
        let (_runtime, context) = create_fabrik_runtime().await.unwrap();
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("fabrik_test_file.txt");
        let temp_file_str = temp_file.to_str().unwrap();

        // Write test data from Rust side first
        tokio::fs::write(&temp_file, b"test content").await.unwrap();

        async_with!(context => |ctx| {
            // Test that file exists
            let script = format!(r#"
                (async () => {{
                    return await Fabrik.exists("{}");
                }})()
            "#, temp_file_str);

            let promise = ctx.eval::<rquickjs::Promise, _>(script.as_bytes())?;
            let exists: bool = promise.into_future().await?;
            assert!(exists, "File should exist");

            // Test reading file
            let script = format!(r#"
                (async () => {{
                    const data = await Fabrik.readFile("{}");
                    return data.length;
                }})()
            "#, temp_file_str);

            let promise = ctx.eval::<rquickjs::Promise, _>(script.as_bytes())?;
            let length: usize = promise.into_future().await?;
            assert_eq!(length, 12, "Should read 12 bytes ('test content')");

            Ok::<_, rquickjs::Error>(())
        })
        .await
        .unwrap();

        // Cleanup
        let _ = tokio::fs::remove_file(&temp_file).await;
    }
}
