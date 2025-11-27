/// CLI utilities for consistent output formatting
use std::io::IsTerminal;

/// Get a colored prefix
///
/// Returns bright cyan if stderr is a TTY, plain text otherwise.
pub fn fabrik_prefix() -> &'static str {
    if std::io::stderr().is_terminal() {
        // Bright cyan for the entire prefix
        "\x1b[96m[fabrik]\x1b[0m"
    } else {
        "[fabrik]"
    }
}
