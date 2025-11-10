/// Script executor
///
/// Handles spawning the runtime, capturing output, monitoring timeout, and handling exit codes.
use anyhow::{Context, Result};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use super::annotations::ScriptAnnotations;

/// Result of script execution
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub exit_code: i32,
    pub duration: Duration,
    #[allow(dead_code)] // May be used in future for output caching
    pub stdout: Vec<u8>,
    #[allow(dead_code)] // May be used in future for output caching
    pub stderr: Vec<u8>,
}

/// Script executor
pub struct ScriptExecutor {
    verbose: bool,
}

impl ScriptExecutor {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    /// Execute script with the specified runtime
    pub fn execute(
        &self,
        script_path: &Path,
        annotations: &ScriptAnnotations,
        args: &[String],
    ) -> Result<ExecutionResult> {
        let start = Instant::now();

        if self.verbose {
            eprintln!(
                "[fabrik] Executing: {} {} {}",
                annotations.runtime,
                script_path.display(),
                args.join(" ")
            );
        }

        // Prepare command - resolve runtime from PATH
        let runtime_path = which::which(&annotations.runtime).unwrap_or_else(|e| {
            if self.verbose {
                eprintln!(
                    "[fabrik] Warning: Could not find '{}' in PATH: {}. Trying as-is.",
                    annotations.runtime, e
                );
            }
            // Fallback to the original runtime name if not found in PATH
            std::path::PathBuf::from(&annotations.runtime)
        });

        if self.verbose {
            eprintln!("[fabrik] Using runtime: {}", runtime_path.display());
        }

        let mut cmd = Command::new(&runtime_path);

        // Add runtime args
        cmd.args(&annotations.runtime_args);

        // Add script path
        cmd.arg(script_path);

        // Add script arguments
        cmd.args(args);

        // Set working directory
        if let Some(cwd) = &annotations.exec_cwd {
            let abs_cwd = if cwd.is_absolute() {
                cwd.clone()
            } else {
                script_path
                    .parent()
                    .ok_or_else(|| anyhow::anyhow!("Script has no parent directory"))?
                    .join(cwd)
            };
            cmd.current_dir(abs_cwd);
        } else {
            // Default: script's directory (if it has one and it's not empty)
            if let Some(parent) = script_path.parent() {
                if parent != std::path::Path::new("") {
                    cmd.current_dir(parent);
                }
            }
        }

        // Capture output
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        if self.verbose {
            eprintln!("[fabrik] Command: {:?}", cmd);
        }

        // Spawn process
        let mut child = cmd
            .spawn()
            .with_context(|| format!("Failed to spawn runtime: {}", annotations.runtime))?;

        // Handle timeout if specified
        let result = if let Some(timeout) = annotations.exec_timeout {
            self.wait_with_timeout(&mut child, timeout)
        } else {
            // Wait indefinitely
            child
                .wait()
                .map(|status| (status, false))
                .context("Failed to wait for child process")
        };

        let (status, timed_out) = result?;

        let duration = start.elapsed();

        if timed_out {
            return Err(anyhow::anyhow!(
                "Script execution timed out after {}s",
                annotations.exec_timeout.unwrap().as_secs()
            ));
        }

        // Capture output
        let output = child
            .wait_with_output()
            .context("Failed to capture output")?;

        let exit_code = status.code().unwrap_or(-1);

        if self.verbose {
            eprintln!(
                "[fabrik] Completed in {:.2}s with exit code {}",
                duration.as_secs_f64(),
                exit_code
            );
        }

        Ok(ExecutionResult {
            exit_code,
            duration,
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }

    /// Wait for child process with timeout
    #[cfg(unix)]
    fn wait_with_timeout(
        &self,
        child: &mut std::process::Child,
        timeout: Duration,
    ) -> Result<(std::process::ExitStatus, bool)> {
        use std::os::unix::process::ExitStatusExt;
        use std::thread;

        let start = Instant::now();

        loop {
            match child.try_wait()? {
                Some(status) => return Ok((status, false)),
                None => {
                    if start.elapsed() >= timeout {
                        // Kill the process
                        child.kill()?;
                        child.wait()?; // Reap zombie
                        return Ok((
                            std::process::ExitStatus::from_raw(128 + 9), // SIGKILL
                            true,
                        ));
                    }
                    thread::sleep(Duration::from_millis(100));
                }
            }
        }
    }

    /// Wait for child process with timeout (Windows version)
    #[cfg(windows)]
    fn wait_with_timeout(
        &self,
        child: &mut std::process::Child,
        timeout: Duration,
    ) -> Result<(std::process::ExitStatus, bool)> {
        use std::os::windows::io::AsRawHandle;
        use std::os::windows::process::ExitStatusExt;
        use std::thread;
        use winapi::um::processthreadsapi::TerminateProcess;
        use winapi::um::winnt::HANDLE;

        let start = Instant::now();

        loop {
            match child.try_wait()? {
                Some(status) => return Ok((status, false)),
                None => {
                    if start.elapsed() >= timeout {
                        // Terminate the process
                        unsafe {
                            let handle = child.as_raw_handle() as HANDLE;
                            TerminateProcess(handle, 1);
                        }
                        child.wait()?;
                        return Ok((std::process::ExitStatus::from_raw(1), true));
                    }
                    thread::sleep(Duration::from_millis(100));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::annotations::ScriptAnnotations;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_execute_simple_script() {
        let temp = TempDir::new().unwrap();
        let script = temp.path().join("test.sh");

        let content = r#"#!/usr/bin/env -S fabrik run bash
#FABRIK output "output.txt"

echo "hello world" > output.txt
"#;

        fs::write(&script, content).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&script).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&script, perms).unwrap();
        }

        let annotations = ScriptAnnotations {
            runtime: "bash".to_string(),
            runtime_args: vec![],
            inputs: vec![],
            outputs: vec![],
            env_vars: vec![],
            cache_ttl: None,
            cache_key: None,
            cache_disabled: false,
            runtime_version: false,
            exec_cwd: None,
            exec_timeout: None,
            exec_shell: false,
            depends_on: vec![],
        };

        let executor = ScriptExecutor::new(false);
        let result = executor.execute(&script, &annotations, &[]).unwrap();

        assert_eq!(result.exit_code, 0);

        // Check output file was created
        assert!(temp.path().join("output.txt").exists());
    }

    #[test]
    fn test_execute_with_timeout() {
        let temp = TempDir::new().unwrap();
        let script = temp.path().join("test.sh");

        let content = r#"#!/usr/bin/env -S fabrik run bash
sleep 10
"#;

        fs::write(&script, content).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&script).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&script, perms).unwrap();
        }

        let annotations = ScriptAnnotations {
            runtime: "bash".to_string(),
            runtime_args: vec![],
            inputs: vec![],
            outputs: vec![],
            env_vars: vec![],
            cache_ttl: None,
            cache_key: None,
            cache_disabled: false,
            runtime_version: false,
            exec_cwd: None,
            exec_timeout: Some(Duration::from_secs(1)),
            exec_shell: false,
            depends_on: vec![],
        };

        let executor = ScriptExecutor::new(false);
        let result = executor.execute(&script, &annotations, &[]);

        // Should timeout
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("timed out"));
    }
}
