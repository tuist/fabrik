/// Dependency resolution for script caching
///
/// Handles executing dependencies recursively with cycle detection.
use anyhow::{Context, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use super::annotations::{parse_annotations, InputSpec, ScriptAnnotations};

/// Dependency resolution context
pub struct DependencyResolver {
    visited: HashSet<PathBuf>,
}

impl Default for DependencyResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyResolver {
    pub fn new() -> Self {
        Self {
            visited: HashSet::new(),
        }
    }

    /// Resolve dependencies for a script
    ///
    /// Returns the list of dependencies in execution order (depth-first).
    /// Detects cycles and returns an error if found.
    pub fn resolve(&mut self, script_path: &Path) -> Result<Vec<ResolvedDependency>> {
        let mut dependencies = Vec::new();
        self.resolve_recursive(script_path, &mut dependencies)?;
        Ok(dependencies)
    }

    fn resolve_recursive(
        &mut self,
        script_path: &Path,
        dependencies: &mut Vec<ResolvedDependency>,
    ) -> Result<()> {
        let abs_path = script_path
            .canonicalize()
            .with_context(|| format!("Failed to canonicalize path: {}", script_path.display()))?;

        // Cycle detection
        if self.visited.contains(&abs_path) {
            return Err(anyhow::anyhow!(
                "Cyclic dependency detected: {}",
                script_path.display()
            ));
        }

        self.visited.insert(abs_path.clone());

        // Parse annotations
        let annotations = parse_annotations(&abs_path)
            .with_context(|| format!("Failed to parse annotations: {}", script_path.display()))?;

        // Recursively resolve dependencies
        for dep in &annotations.depends_on {
            let dep_path = if dep.script.is_absolute() {
                dep.script.clone()
            } else {
                script_path
                    .parent()
                    .ok_or_else(|| anyhow::anyhow!("Script has no parent directory"))?
                    .join(&dep.script)
            };

            self.resolve_recursive(&dep_path, dependencies)?;
        }

        // Add this dependency
        dependencies.push(ResolvedDependency {
            script_path: abs_path,
            annotations,
        });

        Ok(())
    }

    /// Augment annotations with dependency outputs
    ///
    /// If a dependency has `use-outputs=true`, add those outputs as inputs to the script.
    pub fn augment_with_dependency_outputs(
        script_path: &Path,
        annotations: &mut ScriptAnnotations,
        dependencies: &[ResolvedDependency],
    ) {
        for resolved_dep in dependencies {
            // Find matching dependency spec in annotations
            let dep_spec = annotations.depends_on.iter().find(|spec| {
                let spec_path = if spec.script.is_absolute() {
                    spec.script.clone()
                } else {
                    script_path
                        .parent()
                        .unwrap_or_else(|| Path::new("."))
                        .join(&spec.script)
                };
                spec_path.canonicalize().ok().as_ref() == Some(&resolved_dep.script_path)
            });

            if let Some(spec) = dep_spec {
                if spec.use_outputs {
                    // Add dependency outputs as inputs
                    for output in &resolved_dep.annotations.outputs {
                        annotations.inputs.push(InputSpec {
                            path: output.path.clone(),
                            hash: super::annotations::HashMethod::Content,
                        });
                    }
                }
            }
        }
    }
}

/// Resolved dependency with annotations
#[derive(Debug, Clone)]
pub struct ResolvedDependency {
    pub script_path: PathBuf,
    pub annotations: ScriptAnnotations,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_resolve_single_dependency() {
        let temp = TempDir::new().unwrap();

        // Create dependency script
        let dep_script = temp.path().join("dep.sh");
        fs::write(
            &dep_script,
            r#"#!/usr/bin/env -S fabrik run bash
#FABRIK output "dep-output.txt"
echo "dependency"
"#,
        )
        .unwrap();

        // Create main script
        let main_script = temp.path().join("main.sh");
        fs::write(
            &main_script,
            r#"#!/usr/bin/env -S fabrik run bash
#FABRIK depends "./dep.sh"
echo "main"
"#,
        )
        .unwrap();

        let mut resolver = DependencyResolver::new();
        let deps = resolver.resolve(&main_script).unwrap();

        // Should have 2 dependencies (dep + main)
        assert_eq!(deps.len(), 2);

        // dep.sh should be first
        assert!(deps[0].script_path.ends_with("dep.sh"));
        assert!(deps[1].script_path.ends_with("main.sh"));
    }

    #[test]
    fn test_detect_cycle() {
        let temp = TempDir::new().unwrap();

        // Create circular dependencies
        let script_a = temp.path().join("a.sh");
        fs::write(
            &script_a,
            r#"#!/usr/bin/env -S fabrik run bash
#FABRIK depends "./b.sh"
echo "a"
"#,
        )
        .unwrap();

        let script_b = temp.path().join("b.sh");
        fs::write(
            &script_b,
            r#"#!/usr/bin/env -S fabrik run bash
#FABRIK depends "./a.sh"
echo "b"
"#,
        )
        .unwrap();

        let mut resolver = DependencyResolver::new();
        let result = resolver.resolve(&script_a);

        // Should detect cycle
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Cyclic dependency"));
    }

    #[test]
    fn test_augment_with_dependency_outputs() {
        let temp = TempDir::new().unwrap();

        // Create dependency
        let dep_script = temp.path().join("dep.sh");
        fs::write(
            &dep_script,
            r#"#!/usr/bin/env -S fabrik run bash
#FABRIK output "dist/"
echo "build"
"#,
        )
        .unwrap();

        // Create main script
        let main_script = temp.path().join("main.sh");
        fs::write(
            &main_script,
            r#"#!/usr/bin/env -S fabrik run bash
#FABRIK depends "./dep.sh" use-outputs=#true
echo "deploy"
"#,
        )
        .unwrap();

        let mut resolver = DependencyResolver::new();
        let deps = resolver.resolve(&main_script).unwrap();

        // Parse main script annotations
        let mut main_annotations = parse_annotations(&main_script).unwrap();

        // Augment with dependency outputs
        DependencyResolver::augment_with_dependency_outputs(
            &main_script,
            &mut main_annotations,
            &deps,
        );

        // Should have dep's output as input
        assert!(main_annotations.inputs.iter().any(|i| i.path == "dist/"));
    }
}
