/// KDL annotation parser for script caching
///
/// Parses #FABRIK directives from scripts to extract cache configuration.
use anyhow::{anyhow, Context, Result};
use kdl::{KdlDocument, KdlNode};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Input specification for cache key generation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputSpec {
    pub path: String,
    pub hash: HashMethod,
}

/// How to hash input files
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HashMethod {
    /// Hash file contents (SHA256) - most accurate
    #[default]
    Content,
    /// Hash modification time only - fast for large files
    Mtime,
    /// Hash file size only - fastest
    Size,
}

/// Output specification for caching
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputSpec {
    pub path: String,
    pub required: bool,
}

/// Dependency specification
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencySpec {
    pub script: PathBuf,
    pub use_outputs: bool,
}

/// Complete script annotations parsed from KDL directives
#[derive(Debug, Clone, Default)]
pub struct ScriptAnnotations {
    pub runtime: String,
    pub runtime_args: Vec<String>,
    pub inputs: Vec<InputSpec>,
    pub outputs: Vec<OutputSpec>,
    pub env_vars: Vec<String>,
    pub cache_ttl: Option<Duration>,
    pub cache_key: Option<String>,
    pub cache_disabled: bool,
    pub runtime_version: bool,
    pub exec_cwd: Option<PathBuf>,
    pub exec_timeout: Option<Duration>,
    pub exec_shell: bool,
    pub depends_on: Vec<DependencySpec>,
}

/// Parse annotations from a script file
pub fn parse_annotations(script_path: &Path) -> Result<ScriptAnnotations> {
    let content = fs::read_to_string(script_path)
        .with_context(|| format!("Failed to read script: {}", script_path.display()))?;

    let mut lines = content.lines();

    // Parse shebang (first line)
    let shebang = lines.next().ok_or_else(|| anyhow!("Empty script file"))?;

    let (runtime, runtime_args) = parse_shebang(shebang)?;

    // Detect comment prefix based on runtime (if provided)
    // If runtime is empty, try both prefixes
    let prefixes = if runtime.is_empty() {
        vec!["#FABRIK", "//FABRIK"]
    } else {
        match runtime.as_str() {
            "bash" | "sh" | "zsh" | "python3" | "python" | "ruby" | "perl" => vec!["#FABRIK"],
            "node" | "deno" | "bun" | "ts-node" => vec!["//FABRIK"],
            _ => {
                return Err(anyhow!(
                "Unsupported runtime: {}. Supported: bash, sh, zsh, python3, ruby, node, deno, bun",
                runtime
            ))
            }
        }
    };

    // Extract KDL directives from comments
    let mut kdl_lines = Vec::new();
    for line in lines {
        let trimmed = line.trim();
        // Try all prefixes
        for prefix in &prefixes {
            if let Some(directive) = trimmed.strip_prefix(prefix) {
                kdl_lines.push(directive.trim());
                break; // Found a match, move to next line
            }
        }
    }

    // Parse as KDL document
    let kdl_text = kdl_lines.join("\n");
    let doc: KdlDocument = kdl_text
        .parse()
        .map_err(|e| anyhow!("Invalid KDL syntax: {}", e))?;

    let mut annotations = ScriptAnnotations {
        runtime,
        runtime_args,
        ..Default::default()
    };

    // Process KDL nodes
    for node in doc.nodes() {
        parse_kdl_node(&mut annotations, node)
            .with_context(|| format!("Failed to parse directive: {}", node.name()))?;
    }

    // Validate that runtime is set (from shebang, directive, or will be from CLI)
    if annotations.runtime.is_empty() {
        return Err(anyhow!(
            "Runtime not specified. Add runtime to shebang (#!/usr/bin/env -S fabrik run bash), \
             use #FABRIK runtime directive, or pass as CLI arg (fabrik run bash script.sh)"
        ));
    }

    Ok(annotations)
}

/// Parse shebang line to extract runtime and args
///
/// Example: #!/usr/bin/env -S fabrik run bash -x
/// Or: #!/usr/bin/env -S fabrik run  (runtime comes from #FABRIK runtime directive)
fn parse_shebang(line: &str) -> Result<(String, Vec<String>)> {
    if !line.starts_with("#!") {
        return Err(anyhow!("Missing shebang (must start with #!)"));
    }

    let parts: Vec<&str> = line.split_whitespace().collect();

    // Find "fabrik" and "run"
    let fabrik_idx = parts
        .iter()
        .position(|&p| p == "fabrik")
        .ok_or_else(|| anyhow!("Shebang must contain 'fabrik run'"))?;

    let run_idx = fabrik_idx + 1;
    if parts.get(run_idx) != Some(&"run") {
        return Err(anyhow!("Shebang must be 'fabrik run [runtime]'"));
    }

    let runtime_idx = run_idx + 1;

    // Runtime is optional - can be specified via #FABRIK runtime directive
    if let Some(runtime) = parts.get(runtime_idx) {
        let runtime_args = parts[(runtime_idx + 1)..]
            .iter()
            .map(|s| s.to_string())
            .collect();
        Ok((runtime.to_string(), runtime_args))
    } else {
        // No runtime in shebang - will come from directive or CLI arg
        Ok((String::new(), Vec::new()))
    }
}

/// Parse a single KDL node into annotations
fn parse_kdl_node(annotations: &mut ScriptAnnotations, node: &KdlNode) -> Result<()> {
    match node.name().value() {
        "input" => {
            let path = get_positional_string(node, 0)
                .ok_or_else(|| anyhow!("input requires path argument"))?;

            let hash = node
                .get("hash")
                .and_then(|e| e.as_string())
                .unwrap_or("content");

            let hash_method = match hash {
                "content" => HashMethod::Content,
                "mtime" => HashMethod::Mtime,
                "size" => HashMethod::Size,
                _ => {
                    return Err(anyhow!(
                        "Invalid hash method: {}. Use: content, mtime, size",
                        hash
                    ))
                }
            };

            annotations.inputs.push(InputSpec {
                path,
                hash: hash_method,
            });
        }

        "output" => {
            let path = get_positional_string(node, 0)
                .ok_or_else(|| anyhow!("output requires path argument"))?;

            let required = node
                .get("required")
                .and_then(|e| e.as_bool())
                .unwrap_or(true);

            annotations.outputs.push(OutputSpec { path, required });
        }

        "env" => {
            // Multiple positional args for environment variables
            for entry in node.entries() {
                if let Some(var) = entry.value().as_string() {
                    annotations.env_vars.push(var.to_string());
                }
            }
        }

        "cache" => {
            if let Some(ttl) = node.get("ttl").and_then(|e| e.as_string()) {
                annotations.cache_ttl = Some(parse_duration(ttl)?);
            }
            if let Some(key) = node.get("key").and_then(|e| e.as_string()) {
                annotations.cache_key = Some(key.to_string());
            }
            if let Some(disabled) = node.get("disabled").and_then(|e| e.as_bool()) {
                annotations.cache_disabled = disabled;
            }
        }

        "runtime" => {
            // Get runtime name if specified as positional argument
            if let Some(runtime_name) = get_positional_string(node, 0) {
                annotations.runtime = runtime_name;
            }

            if let Some(include) = node.get("include-version").and_then(|e| e.as_bool()) {
                annotations.runtime_version = include;
            }
        }

        "runtime-arg" => {
            // Add runtime argument
            if let Some(arg) = get_positional_string(node, 0) {
                annotations.runtime_args.push(arg);
            }
        }

        "runtime-version" => {
            // Shorthand for including runtime version in cache key
            annotations.runtime_version = true;
        }

        "exec" => {
            if let Some(cwd) = node.get("cwd").and_then(|e| e.as_string()) {
                annotations.exec_cwd = Some(PathBuf::from(cwd));
            }
            if let Some(timeout) = node.get("timeout").and_then(|e| e.as_string()) {
                annotations.exec_timeout = Some(parse_duration(timeout)?);
            }
            if let Some(shell) = node.get("shell").and_then(|e| e.as_bool()) {
                annotations.exec_shell = shell;
            }
        }

        "depends" => {
            let script_path = get_positional_string(node, 0)
                .ok_or_else(|| anyhow!("depends requires script path"))?;

            let use_outputs = node
                .get("use-outputs")
                .and_then(|e| e.as_bool())
                .unwrap_or(false);

            annotations.depends_on.push(DependencySpec {
                script: PathBuf::from(script_path),
                use_outputs,
            });
        }

        _ => {
            return Err(anyhow!("Unknown directive: {}", node.name()));
        }
    }

    Ok(())
}

/// Get positional string argument from KDL node
fn get_positional_string(node: &KdlNode, index: usize) -> Option<String> {
    node.entries()
        .iter()
        .filter(|e| e.name().is_none()) // Only positional args
        .nth(index)
        .and_then(|e| e.value().as_string())
        .map(|s| s.to_string())
}

/// Parse duration string (e.g., "1h", "7d", "30d", "10m")
fn parse_duration(s: &str) -> Result<Duration> {
    if s.is_empty() {
        return Err(anyhow!("Empty duration string"));
    }

    let (num_str, unit) = s.split_at(s.len() - 1);
    let num: u64 = num_str
        .parse()
        .map_err(|_| anyhow!("Invalid duration: {}", s))?;

    let seconds = match unit {
        "s" => num,
        "m" => num * 60,
        "h" => num * 3600,
        "d" => num * 86400,
        _ => return Err(anyhow!("Invalid duration unit: {}. Use: s, m, h, d", unit)),
    };

    Ok(Duration::from_secs(seconds))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_shebang() {
        let shebang = "#!/usr/bin/env -S fabrik run bash";
        let (runtime, args) = parse_shebang(shebang).unwrap();
        assert_eq!(runtime, "bash");
        assert_eq!(args.len(), 0);

        let shebang = "#!/usr/bin/env -S fabrik run node --experimental-modules";
        let (runtime, args) = parse_shebang(shebang).unwrap();
        assert_eq!(runtime, "node");
        assert_eq!(args, vec!["--experimental-modules"]);
    }

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("1s").unwrap(), Duration::from_secs(1));
        assert_eq!(parse_duration("5m").unwrap(), Duration::from_secs(300));
        assert_eq!(parse_duration("2h").unwrap(), Duration::from_secs(7200));
        assert_eq!(parse_duration("7d").unwrap(), Duration::from_secs(604800));
    }

    #[test]
    fn test_parse_kdl_input() {
        let kdl = r#"
            input "src/**/*.ts" hash="content"
            input "package.json"
        "#;
        let doc: KdlDocument = kdl.parse().unwrap();
        let mut annotations = ScriptAnnotations::default();

        for node in doc.nodes() {
            parse_kdl_node(&mut annotations, node).unwrap();
        }

        assert_eq!(annotations.inputs.len(), 2);
        assert_eq!(annotations.inputs[0].path, "src/**/*.ts");
        assert_eq!(annotations.inputs[0].hash, HashMethod::Content);
        assert_eq!(annotations.inputs[1].path, "package.json");
        assert_eq!(annotations.inputs[1].hash, HashMethod::Content);
    }

    #[test]
    fn test_parse_kdl_cache() {
        let kdl = r#"cache ttl="7d" key="v2""#;
        let doc: KdlDocument = kdl.parse().unwrap();
        let mut annotations = ScriptAnnotations::default();

        for node in doc.nodes() {
            parse_kdl_node(&mut annotations, node).unwrap();
        }

        assert_eq!(
            annotations.cache_ttl.unwrap(),
            Duration::from_secs(7 * 86400)
        );
        assert_eq!(annotations.cache_key.unwrap(), "v2");
    }
}
