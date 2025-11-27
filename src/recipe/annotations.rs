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
    // If runtime is empty, try all known prefixes
    let prefixes = if runtime.is_empty() {
        vec![
            "#FABRIK",
            "//FABRIK",
            ";FABRIK",
            "--FABRIK",
            "%FABRIK",
            "REM FABRIK",
            "'FABRIK",
        ]
    } else {
        get_comment_prefixes_for_runtime(&runtime)?
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

/// Get comment prefixes for a given runtime
///
/// Returns the appropriate comment prefix(es) for FABRIK annotations based on the runtime.
/// Most languages use a single comment style, but we support the common ones.
fn get_comment_prefixes_for_runtime(runtime: &str) -> Result<Vec<&'static str>> {
    match runtime {
        // Hash-style comments (#)
        // Shell languages
        "bash" | "sh" | "zsh" | "fish" | "ksh" | "csh" | "tcsh" | "dash" |
        // Python
        "python" | "python3" | "python2" | "pypy" | "pypy3" |
        // Ruby
        "ruby" | "jruby" |
        // Perl
        "perl" | "perl5" | "perl6" | "raku" |
        // R
        "r" | "rscript" |
        // PowerShell (also supports # comments)
        "pwsh" | "powershell" |
        // Other scripting languages with # comments
        "awk" | "gawk" | "sed" | "make" | "cmake" |
        // Configuration/data languages
        "yaml" | "toml" => Ok(vec!["#FABRIK"]),

        // Double-slash comments (//)
        // JavaScript/TypeScript runtimes
        "node" | "nodejs" | "deno" | "bun" | "ts-node" | "tsx" | "npx" |
        // C-family (for generated scripts or code generation)
        "swift" | "kotlin" | "kotlinc" | "scala" | "groovy" | "groovysh" |
        // Go (for go run scripts)
        "go" |
        // Rust (for rust-script)
        "rust-script" | "cargo-script" |
        // V lang
        "v" |
        // Zig
        "zig" => Ok(vec!["//FABRIK"]),

        // Semicolon comments (;)
        // Lisp family
        "lisp" | "sbcl" | "clisp" | "ecl" | "ccl" |
        // Clojure
        "clojure" | "clj" | "bb" | "babashka" |
        // Scheme
        "scheme" | "racket" | "guile" | "chicken" | "chez" |
        // Emacs Lisp
        "emacs" | "elisp" |
        // Assembly (some assemblers)
        "nasm" | "fasm" => Ok(vec![";FABRIK"]),

        // Double-dash comments (--)
        // SQL
        "sql" | "psql" | "mysql" | "sqlite" | "sqlite3" |
        // Lua
        "lua" | "luajit" |
        // Haskell
        "ghc" | "ghci" | "runghc" | "runhaskell" | "stack" |
        // Ada
        "ada" | "gnat" |
        // VHDL
        "vhdl" => Ok(vec!["--FABRIK"]),

        // REM comments (Windows batch)
        "cmd" | "bat" | "batch" => Ok(vec!["REM FABRIK"]),

        // Single-quote comments (')
        // VBScript, VBA
        "vbscript" | "cscript" | "wscript" => Ok(vec!["'FABRIK"]),

        // Languages with multiple comment styles - use most common
        // PHP supports # and //, prefer #
        "php" => Ok(vec!["#FABRIK", "//FABRIK"]),

        // Elixir supports # comments
        "elixir" | "iex" | "mix" => Ok(vec!["#FABRIK"]),

        // Erlang uses % for comments
        "erlang" | "escript" => Ok(vec!["%FABRIK"]),

        // OCaml/F# use (* *) but also support // in some contexts
        // For single-line annotations, we'll accept //
        "ocaml" | "fsharp" | "dotnet-fsi" => Ok(vec!["//FABRIK"]),

        // Julia uses # for comments
        "julia" => Ok(vec!["#FABRIK"]),

        // Nim uses # for comments
        "nim" | "nimble" => Ok(vec!["#FABRIK"]),

        // Crystal uses # for comments
        "crystal" => Ok(vec!["#FABRIK"]),

        // Tcl uses # for comments
        "tcl" | "tclsh" | "wish" => Ok(vec!["#FABRIK"]),

        _ => Err(anyhow!(
            "Unsupported runtime: '{}'. \n\
            Supported runtimes include:\n\
            - Shell: bash, sh, zsh, fish, ksh\n\
            - Python: python, python3, pypy\n\
            - Ruby: ruby, jruby\n\
            - JavaScript: node, deno, bun, ts-node\n\
            - Perl: perl, raku\n\
            - Lisp: sbcl, clisp, clojure, racket, guile\n\
            - Lua: lua, luajit\n\
            - SQL: psql, mysql, sqlite3\n\
            - Haskell: ghc, runghc, stack\n\
            - And many more...",
            runtime
        )),
    }
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

    // =========================================================================
    // Comment prefix tests for different runtimes
    // =========================================================================

    #[test]
    fn test_comment_prefix_hash_style() {
        // Shell languages
        assert_eq!(
            get_comment_prefixes_for_runtime("bash").unwrap(),
            vec!["#FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("sh").unwrap(),
            vec!["#FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("zsh").unwrap(),
            vec!["#FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("fish").unwrap(),
            vec!["#FABRIK"]
        );

        // Python
        assert_eq!(
            get_comment_prefixes_for_runtime("python").unwrap(),
            vec!["#FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("python3").unwrap(),
            vec!["#FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("pypy").unwrap(),
            vec!["#FABRIK"]
        );

        // Ruby
        assert_eq!(
            get_comment_prefixes_for_runtime("ruby").unwrap(),
            vec!["#FABRIK"]
        );

        // Perl
        assert_eq!(
            get_comment_prefixes_for_runtime("perl").unwrap(),
            vec!["#FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("raku").unwrap(),
            vec!["#FABRIK"]
        );

        // R
        assert_eq!(
            get_comment_prefixes_for_runtime("r").unwrap(),
            vec!["#FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("rscript").unwrap(),
            vec!["#FABRIK"]
        );

        // PowerShell
        assert_eq!(
            get_comment_prefixes_for_runtime("pwsh").unwrap(),
            vec!["#FABRIK"]
        );

        // Other hash-comment languages
        assert_eq!(
            get_comment_prefixes_for_runtime("awk").unwrap(),
            vec!["#FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("julia").unwrap(),
            vec!["#FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("nim").unwrap(),
            vec!["#FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("crystal").unwrap(),
            vec!["#FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("elixir").unwrap(),
            vec!["#FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("tcl").unwrap(),
            vec!["#FABRIK"]
        );
    }

    #[test]
    fn test_comment_prefix_double_slash_style() {
        // JavaScript/TypeScript
        assert_eq!(
            get_comment_prefixes_for_runtime("node").unwrap(),
            vec!["//FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("deno").unwrap(),
            vec!["//FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("bun").unwrap(),
            vec!["//FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("ts-node").unwrap(),
            vec!["//FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("tsx").unwrap(),
            vec!["//FABRIK"]
        );

        // C-family scripting
        assert_eq!(
            get_comment_prefixes_for_runtime("swift").unwrap(),
            vec!["//FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("kotlin").unwrap(),
            vec!["//FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("scala").unwrap(),
            vec!["//FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("groovy").unwrap(),
            vec!["//FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("go").unwrap(),
            vec!["//FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("rust-script").unwrap(),
            vec!["//FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("zig").unwrap(),
            vec!["//FABRIK"]
        );
    }

    #[test]
    fn test_comment_prefix_semicolon_style() {
        // Lisp family
        assert_eq!(
            get_comment_prefixes_for_runtime("lisp").unwrap(),
            vec![";FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("sbcl").unwrap(),
            vec![";FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("clisp").unwrap(),
            vec![";FABRIK"]
        );

        // Clojure
        assert_eq!(
            get_comment_prefixes_for_runtime("clojure").unwrap(),
            vec![";FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("clj").unwrap(),
            vec![";FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("babashka").unwrap(),
            vec![";FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("bb").unwrap(),
            vec![";FABRIK"]
        );

        // Scheme
        assert_eq!(
            get_comment_prefixes_for_runtime("scheme").unwrap(),
            vec![";FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("racket").unwrap(),
            vec![";FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("guile").unwrap(),
            vec![";FABRIK"]
        );

        // Emacs Lisp
        assert_eq!(
            get_comment_prefixes_for_runtime("elisp").unwrap(),
            vec![";FABRIK"]
        );
    }

    #[test]
    fn test_comment_prefix_double_dash_style() {
        // SQL
        assert_eq!(
            get_comment_prefixes_for_runtime("sql").unwrap(),
            vec!["--FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("psql").unwrap(),
            vec!["--FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("mysql").unwrap(),
            vec!["--FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("sqlite3").unwrap(),
            vec!["--FABRIK"]
        );

        // Lua
        assert_eq!(
            get_comment_prefixes_for_runtime("lua").unwrap(),
            vec!["--FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("luajit").unwrap(),
            vec!["--FABRIK"]
        );

        // Haskell
        assert_eq!(
            get_comment_prefixes_for_runtime("ghc").unwrap(),
            vec!["--FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("runghc").unwrap(),
            vec!["--FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("stack").unwrap(),
            vec!["--FABRIK"]
        );
    }

    #[test]
    fn test_comment_prefix_other_styles() {
        // Windows batch (REM)
        assert_eq!(
            get_comment_prefixes_for_runtime("cmd").unwrap(),
            vec!["REM FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("bat").unwrap(),
            vec!["REM FABRIK"]
        );

        // VBScript (')
        assert_eq!(
            get_comment_prefixes_for_runtime("vbscript").unwrap(),
            vec!["'FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("cscript").unwrap(),
            vec!["'FABRIK"]
        );

        // Erlang (%)
        assert_eq!(
            get_comment_prefixes_for_runtime("erlang").unwrap(),
            vec!["%FABRIK"]
        );
        assert_eq!(
            get_comment_prefixes_for_runtime("escript").unwrap(),
            vec!["%FABRIK"]
        );

        // PHP (supports both # and //)
        assert_eq!(
            get_comment_prefixes_for_runtime("php").unwrap(),
            vec!["#FABRIK", "//FABRIK"]
        );
    }

    #[test]
    fn test_unsupported_runtime_error() {
        let result = get_comment_prefixes_for_runtime("unknown_runtime");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unsupported runtime"));
    }

    // =========================================================================
    // Integration tests for parsing scripts with different comment styles
    // =========================================================================

    #[test]
    fn test_parse_bash_script() {
        let temp_dir = tempfile::tempdir().unwrap();
        let script_path = temp_dir.path().join("test.sh");
        std::fs::write(
            &script_path,
            r#"#!/usr/bin/env -S fabrik run bash
#FABRIK input "src/**/*.sh"
#FABRIK output "dist/"
#FABRIK env "PATH"

echo "Hello from bash"
"#,
        )
        .unwrap();

        let annotations = parse_annotations(&script_path).unwrap();
        assert_eq!(annotations.runtime, "bash");
        assert_eq!(annotations.inputs.len(), 1);
        assert_eq!(annotations.inputs[0].path, "src/**/*.sh");
        assert_eq!(annotations.outputs.len(), 1);
        assert_eq!(annotations.outputs[0].path, "dist/");
        assert_eq!(annotations.env_vars, vec!["PATH"]);
    }

    #[test]
    fn test_parse_python_script() {
        let temp_dir = tempfile::tempdir().unwrap();
        let script_path = temp_dir.path().join("test.py");
        std::fs::write(
            &script_path,
            r#"#!/usr/bin/env -S fabrik run python3
#FABRIK input "src/**/*.py"
#FABRIK output "dist/"
#FABRIK cache ttl="1h"

print("Hello from Python")
"#,
        )
        .unwrap();

        let annotations = parse_annotations(&script_path).unwrap();
        assert_eq!(annotations.runtime, "python3");
        assert_eq!(annotations.inputs[0].path, "src/**/*.py");
        assert_eq!(annotations.cache_ttl, Some(Duration::from_secs(3600)));
    }

    #[test]
    fn test_parse_node_script() {
        let temp_dir = tempfile::tempdir().unwrap();
        let script_path = temp_dir.path().join("test.js");
        std::fs::write(
            &script_path,
            r#"#!/usr/bin/env -S fabrik run node
//FABRIK input "src/**/*.js"
//FABRIK output "dist/"
//FABRIK env "NODE_ENV"

console.log("Hello from Node.js");
"#,
        )
        .unwrap();

        let annotations = parse_annotations(&script_path).unwrap();
        assert_eq!(annotations.runtime, "node");
        assert_eq!(annotations.inputs[0].path, "src/**/*.js");
        assert_eq!(annotations.env_vars, vec!["NODE_ENV"]);
    }

    #[test]
    fn test_parse_clojure_script() {
        let temp_dir = tempfile::tempdir().unwrap();
        let script_path = temp_dir.path().join("test.clj");
        std::fs::write(
            &script_path,
            r#"#!/usr/bin/env -S fabrik run bb
;FABRIK input "src/**/*.clj"
;FABRIK output "target/"
;FABRIK env "JAVA_HOME"

(println "Hello from Clojure/Babashka")
"#,
        )
        .unwrap();

        let annotations = parse_annotations(&script_path).unwrap();
        assert_eq!(annotations.runtime, "bb");
        assert_eq!(annotations.inputs[0].path, "src/**/*.clj");
        assert_eq!(annotations.outputs[0].path, "target/");
        assert_eq!(annotations.env_vars, vec!["JAVA_HOME"]);
    }

    #[test]
    fn test_parse_lua_script() {
        let temp_dir = tempfile::tempdir().unwrap();
        let script_path = temp_dir.path().join("test.lua");
        std::fs::write(
            &script_path,
            r#"#!/usr/bin/env -S fabrik run lua
--FABRIK input "src/**/*.lua"
--FABRIK output "dist/"

print("Hello from Lua")
"#,
        )
        .unwrap();

        let annotations = parse_annotations(&script_path).unwrap();
        assert_eq!(annotations.runtime, "lua");
        assert_eq!(annotations.inputs[0].path, "src/**/*.lua");
    }

    #[test]
    fn test_parse_sql_script() {
        let temp_dir = tempfile::tempdir().unwrap();
        let script_path = temp_dir.path().join("test.sql");
        std::fs::write(
            &script_path,
            r#"#!/usr/bin/env -S fabrik run psql
--FABRIK input "migrations/*.sql"
--FABRIK output "schema.dump"
--FABRIK env "DATABASE_URL"

SELECT 1;
"#,
        )
        .unwrap();

        let annotations = parse_annotations(&script_path).unwrap();
        assert_eq!(annotations.runtime, "psql");
        assert_eq!(annotations.inputs[0].path, "migrations/*.sql");
        assert_eq!(annotations.env_vars, vec!["DATABASE_URL"]);
    }

    #[test]
    fn test_parse_haskell_script() {
        let temp_dir = tempfile::tempdir().unwrap();
        let script_path = temp_dir.path().join("test.hs");
        std::fs::write(
            &script_path,
            r#"#!/usr/bin/env -S fabrik run runghc
--FABRIK input "src/**/*.hs"
--FABRIK output "dist/"

main = putStrLn "Hello from Haskell"
"#,
        )
        .unwrap();

        let annotations = parse_annotations(&script_path).unwrap();
        assert_eq!(annotations.runtime, "runghc");
        assert_eq!(annotations.inputs[0].path, "src/**/*.hs");
    }

    #[test]
    fn test_parse_erlang_script() {
        let temp_dir = tempfile::tempdir().unwrap();
        let script_path = temp_dir.path().join("test.erl");
        std::fs::write(
            &script_path,
            r#"#!/usr/bin/env -S fabrik run escript
%FABRIK input "src/**/*.erl"
%FABRIK output "_build/"

main(_) -> io:format("Hello from Erlang~n").
"#,
        )
        .unwrap();

        let annotations = parse_annotations(&script_path).unwrap();
        assert_eq!(annotations.runtime, "escript");
        assert_eq!(annotations.inputs[0].path, "src/**/*.erl");
    }

    #[test]
    fn test_parse_ruby_script() {
        let temp_dir = tempfile::tempdir().unwrap();
        let script_path = temp_dir.path().join("test.rb");
        std::fs::write(
            &script_path,
            r#"#!/usr/bin/env -S fabrik run ruby
#FABRIK input "lib/**/*.rb"
#FABRIK input "Gemfile"
#FABRIK output "pkg/"
#FABRIK env "BUNDLE_PATH"

puts "Hello from Ruby"
"#,
        )
        .unwrap();

        let annotations = parse_annotations(&script_path).unwrap();
        assert_eq!(annotations.runtime, "ruby");
        assert_eq!(annotations.inputs.len(), 2);
        assert_eq!(annotations.inputs[0].path, "lib/**/*.rb");
        assert_eq!(annotations.inputs[1].path, "Gemfile");
    }

    #[test]
    fn test_parse_perl_script() {
        let temp_dir = tempfile::tempdir().unwrap();
        let script_path = temp_dir.path().join("test.pl");
        std::fs::write(
            &script_path,
            r#"#!/usr/bin/env -S fabrik run perl
#FABRIK input "lib/**/*.pm"
#FABRIK output "blib/"

print "Hello from Perl\n";
"#,
        )
        .unwrap();

        let annotations = parse_annotations(&script_path).unwrap();
        assert_eq!(annotations.runtime, "perl");
        assert_eq!(annotations.inputs[0].path, "lib/**/*.pm");
    }

    #[test]
    fn test_parse_r_script() {
        let temp_dir = tempfile::tempdir().unwrap();
        let script_path = temp_dir.path().join("test.R");
        std::fs::write(
            &script_path,
            r#"#!/usr/bin/env -S fabrik run rscript
#FABRIK input "data/*.csv"
#FABRIK output "results/"

print("Hello from R")
"#,
        )
        .unwrap();

        let annotations = parse_annotations(&script_path).unwrap();
        assert_eq!(annotations.runtime, "rscript");
        assert_eq!(annotations.inputs[0].path, "data/*.csv");
    }

    #[test]
    fn test_parse_php_script_with_hash() {
        let temp_dir = tempfile::tempdir().unwrap();
        let script_path = temp_dir.path().join("test.php");
        std::fs::write(
            &script_path,
            r#"#!/usr/bin/env -S fabrik run php
#FABRIK input "src/**/*.php"
#FABRIK output "vendor/"

<?php echo "Hello from PHP"; ?>
"#,
        )
        .unwrap();

        let annotations = parse_annotations(&script_path).unwrap();
        assert_eq!(annotations.runtime, "php");
        assert_eq!(annotations.inputs[0].path, "src/**/*.php");
    }

    #[test]
    fn test_parse_php_script_with_double_slash() {
        let temp_dir = tempfile::tempdir().unwrap();
        let script_path = temp_dir.path().join("test.php");
        std::fs::write(
            &script_path,
            r#"#!/usr/bin/env -S fabrik run php
//FABRIK input "src/**/*.php"
//FABRIK output "vendor/"

<?php echo "Hello from PHP"; ?>
"#,
        )
        .unwrap();

        let annotations = parse_annotations(&script_path).unwrap();
        assert_eq!(annotations.runtime, "php");
        assert_eq!(annotations.inputs[0].path, "src/**/*.php");
    }
}
