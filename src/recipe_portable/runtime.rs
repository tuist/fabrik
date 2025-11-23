// QuickJS runtime for portable recipes
//
// This module creates a QuickJS JavaScript runtime with Fabrik APIs exposed.

use anyhow::Result;
use rquickjs::{
    async_with,
    function::Async,
    loader::{BuiltinLoader, BuiltinResolver, ModuleLoader},
    AsyncContext, AsyncRuntime, Function, Module,
};
use std::path::PathBuf;
use tokio::process::Command;

use super::cache::{self, CacheOptions};

/// Create a QuickJS runtime with Fabrik APIs
///
/// The recipe_dir parameter is used to discover fabrik.toml for configuration
pub async fn create_fabrik_runtime_with_dir(
    recipe_dir: PathBuf,
) -> Result<(AsyncRuntime, AsyncContext)> {
    // Create runtime with module loader for LLRT modules + Fabrik modules
    let resolver = BuiltinResolver::default()
        .with_module("fs")
        .with_module("fs/promises")
        .with_module("child_process")
        .with_module("path")
        .with_module("fabrik:cache")
        .with_module("fabrik:fs")
        .with_module("fabrik:kv");

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

    // Store recipe directory in context for use by cache APIs
    let recipe_dir_clone = recipe_dir.clone();

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

        // Register fabrik:cache module
        Module::declare_def::<js_js_module_fabrik_cache, _>(ctx.clone(), "fabrik:cache")?;

        // Register fabrik:fs module
        Module::declare_def::<js_js_module_fabrik_fs, _>(ctx.clone(), "fabrik:fs")?;

        // Register fabrik:kv module
        Module::declare_def::<js_js_module_fabrik_kv, _>(ctx.clone(), "fabrik:kv")?;

        // Store working directory for cache APIs
        ctx.globals().set("__FABRIK_RECIPE_DIR__", recipe_dir_clone.to_string_lossy().to_string())?;

        Ok::<_, rquickjs::Error>(())
    })
    .await?;

    Ok((runtime, context))
}

/// Backward-compatible function without recipe_dir
#[allow(dead_code)]
pub async fn create_fabrik_runtime() -> Result<(AsyncRuntime, AsyncContext)> {
    create_fabrik_runtime_with_dir(std::env::current_dir()?).await
}

// Module definitions for fabrik:* modules

#[rquickjs::module]
mod js_module_fabrik_cache {
    use super::*;
    use rquickjs::{Ctx, Exception, Result as JsResult};

    /// needsRun(options) - Check if action needs to run
    #[rquickjs::function]
    pub async fn needs_run(ctx: Ctx<'_>, options: rquickjs::Object<'_>) -> JsResult<bool> {
        // Extract fields from JS object
        let inputs: Vec<String> = options.get("inputs").unwrap_or_default();
        let outputs: Vec<String> = options.get("outputs").unwrap_or_default();
        let env: Vec<String> = options.get("env").unwrap_or_default();
        let cache_dir: Option<String> = options.get("cacheDir").ok();
        let hash_method: String = options
            .get("hashMethod")
            .unwrap_or_else(|_| "content".to_string());

        let cache_options = CacheOptions {
            inputs,
            outputs,
            env,
            cache_dir,
            upstream: None,
            ttl: None,
            hash_method,
        };

        // Get working directory
        let working_dir_str: String = ctx.globals().get("__FABRIK_RECIPE_DIR__")?;
        let working_dir = PathBuf::from(working_dir_str);

        // Call Rust implementation
        cache::needs_run(cache_options, &working_dir)
            .await
            .map_err(|e| Exception::throw_message(&ctx, &format!("needsRun failed: {}", e)))
    }

    /// runCached(action, options) - Run action with caching
    #[rquickjs::function]
    pub async fn run_cached<'js>(
        ctx: Ctx<'js>,
        action: rquickjs::Function<'js>,
        options: rquickjs::Object<'js>,
    ) -> JsResult<rquickjs::Object<'js>> {
        use std::time::Instant;

        // Extract fields from JS object
        let inputs: Vec<String> = options.get("inputs").unwrap_or_default();
        let outputs: Vec<String> = options.get("outputs").unwrap_or_default();
        let env: Vec<String> = options.get("env").unwrap_or_default();
        let cache_dir: Option<String> = options.get("cacheDir").ok();
        let hash_method: String = options
            .get("hashMethod")
            .unwrap_or_else(|_| "content".to_string());

        let cache_options = CacheOptions {
            inputs,
            outputs,
            env,
            cache_dir,
            upstream: None,
            ttl: None,
            hash_method,
        };

        // Get working directory
        let working_dir_str: String = ctx.globals().get("__FABRIK_RECIPE_DIR__")?;
        let working_dir = PathBuf::from(working_dir_str);

        // Compute cache key
        let cache_key = cache::compute_cache_key(&cache_options, &working_dir)
            .await
            .map_err(|e| {
                Exception::throw_message(&ctx, &format!("Failed to compute cache key: {}", e))
            })?;

        // Determine cache directory
        let cache_dir = if let Some(ref dir) = cache_options.cache_dir {
            PathBuf::from(dir)
        } else {
            working_dir.join(".fabrik/cache")
        };

        // Check if cached
        let kv = cache::KvStore::new(&cache_dir);
        let is_cached = kv
            .has(&cache_key)
            .await
            .map_err(|e| Exception::throw_message(&ctx, &format!("KV check failed: {}", e)))?;

        let result_obj = rquickjs::Object::new(ctx.clone())?;
        result_obj.set("cacheKey", cache_key.clone())?;

        if is_cached {
            // Cache hit - restore outputs
            tracing::info!("[fabrik] Cache HIT: {}", &cache_key[..8]);

            let restored = cache::restore_outputs(
                &cache_options.outputs,
                &cache_dir,
                &cache_key,
                &working_dir,
            )
            .await
            .map_err(|e| {
                Exception::throw_message(&ctx, &format!("Failed to restore outputs: {}", e))
            })?;

            let restored_array = rquickjs::Array::new(ctx.clone())?;
            for (i, file) in restored.iter().enumerate() {
                restored_array.set(i, file.clone())?;
            }

            result_obj.set("cached", true)?;
            result_obj.set("restoredFiles", restored_array)?;
        } else {
            // Cache miss - run action
            tracing::info!("[fabrik] Cache MISS: {}", &cache_key[..8]);

            let start = Instant::now();

            // Call the action function
            let promise: rquickjs::Promise = action.call(())?;
            promise
                .into_future::<()>()
                .await
                .map_err(|e| Exception::throw_message(&ctx, &format!("Action failed: {:?}", e)))?;

            let duration = start.elapsed();

            // Archive outputs
            let archived = cache::archive_outputs(
                &cache_options.outputs,
                &cache_dir,
                &cache_key,
                &working_dir,
            )
            .await
            .map_err(|e| {
                Exception::throw_message(&ctx, &format!("Failed to archive outputs: {}", e))
            })?;

            // Store in KV
            kv.set(
                &cache_key,
                serde_json::json!({"timestamp": chrono::Utc::now().timestamp()}),
            )
            .await
            .map_err(|e| Exception::throw_message(&ctx, &format!("Failed to update KV: {}", e)))?;

            tracing::info!(
                "[fabrik] Cached {} outputs for key {}",
                archived.len(),
                &cache_key[..8]
            );

            result_obj.set("cached", false)?;
            result_obj.set("durationMs", duration.as_millis() as u64)?;
        }

        Ok(result_obj)
    }
}

#[rquickjs::module]
mod js_module_fabrik_fs {
    use rquickjs::{Ctx, Exception, Result as JsResult};

    /// glob(pattern) - Find files matching pattern
    #[rquickjs::function]
    pub async fn glob(_ctx: Ctx<'_>, pattern: String) -> JsResult<Vec<String>> {
        match glob::glob(&pattern) {
            Ok(iter) => {
                let paths = iter
                    .filter_map(|entry| entry.ok())
                    .map(|path| path.to_string_lossy().to_string())
                    .collect::<Vec<String>>();
                Ok(paths)
            }
            Err(_) => Err(Exception::throw_message(&_ctx, "Invalid glob pattern")),
        }
    }

    /// hashFile(path) - Compute SHA256 hash of file
    #[rquickjs::function]
    pub async fn hash_file(ctx: Ctx<'_>, path: String) -> JsResult<String> {
        use sha2::{Digest, Sha256};

        let data = tokio::fs::read(&path)
            .await
            .map_err(|e| Exception::throw_message(&ctx, &format!("Failed to read file: {}", e)))?;

        let hash = Sha256::digest(&data);
        Ok(hex::encode(hash))
    }
}

#[rquickjs::module]
mod js_module_fabrik_kv {
    use super::*;
    use rquickjs::{Ctx, Exception, Result as JsResult, Value};

    /// has(key) - Check if key exists in KV store
    #[rquickjs::function]
    pub async fn has(ctx: Ctx<'_>, key: String) -> JsResult<bool> {
        let working_dir_str: String = ctx.globals().get("__FABRIK_RECIPE_DIR__")?;
        let working_dir = PathBuf::from(working_dir_str);
        let cache_dir = working_dir.join(".fabrik/cache");

        let kv = cache::KvStore::new(&cache_dir);
        kv.has(&key)
            .await
            .map_err(|e| Exception::throw_message(&ctx, &format!("KV has failed: {}", e)))
    }

    /// get(key) - Get value from KV store
    #[rquickjs::function]
    pub async fn get<'js>(ctx: Ctx<'js>, key: String) -> JsResult<Value<'js>> {
        let working_dir_str: String = ctx.globals().get("__FABRIK_RECIPE_DIR__")?;
        let working_dir = PathBuf::from(working_dir_str);
        let cache_dir = working_dir.join(".fabrik/cache");

        let kv = cache::KvStore::new(&cache_dir);
        let value = kv
            .get(&key)
            .await
            .map_err(|e| Exception::throw_message(&ctx, &format!("KV get failed: {}", e)))?;

        match value {
            Some(_v) => {
                // For now, just return a placeholder object
                // Full JSON parsing would require additional dependencies
                let obj = rquickjs::Object::new(ctx.clone())?;
                obj.set("exists", true)?;
                Ok(obj.into())
            }
            None => Ok(Value::new_null(ctx)),
        }
    }

    /// set(key, value) - Set value in KV store
    #[rquickjs::function]
    pub async fn set<'js>(ctx: Ctx<'js>, key: String, _value: Value<'js>) -> JsResult<()> {
        let working_dir_str: String = ctx.globals().get("__FABRIK_RECIPE_DIR__")?;
        let working_dir = PathBuf::from(working_dir_str);
        let cache_dir = working_dir.join(".fabrik/cache");

        // For now, just store a simple marker
        let json_value = serde_json::json!({"stored": true});

        let kv = cache::KvStore::new(&cache_dir);
        kv.set(&key, json_value)
            .await
            .map_err(|e| Exception::throw_message(&ctx, &format!("KV set failed: {}", e)))
    }
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
        // Escape backslashes for Windows paths in JavaScript strings
        let temp_file_str = temp_file.to_str().unwrap().replace('\\', "\\\\");

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
