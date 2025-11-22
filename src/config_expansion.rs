use anyhow::Result;
use std::env;

/// Expands environment variables in configuration content
///
/// Supports:
/// - `${VAR}` - Required variable (error if not set)
/// - `${VAR:-default}` - Optional variable with default value
/// - `$$` - Literal dollar sign (escaped)
///
/// # Examples
///
/// ```
/// # std::env::set_var("SECRET", "my-secret");
/// # std::env::set_var("PORT", "8080");
/// let input = r#"
/// secret = "${SECRET}"
/// port = ${PORT}
/// fallback = "${MISSING:-default-value}"
/// literal = "$$100"
/// "#;
/// let result = fabrik::config_expansion::expand_env_vars(input).unwrap();
/// assert!(result.contains(r#"secret = "my-secret""#));
/// assert!(result.contains("port = 8080"));
/// assert!(result.contains(r#"fallback = "default-value""#));
/// assert!(result.contains(r#"literal = "$100""#));
/// ```
pub fn expand_env_vars(content: &str) -> Result<String> {
    let mut result = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' {
            // Check for escaped dollar sign: $$
            if chars.peek() == Some(&'$') {
                chars.next(); // consume second $
                result.push('$');
                continue;
            }

            // Check for variable expansion: ${VAR} or ${VAR:-default}
            if chars.peek() == Some(&'{') {
                chars.next(); // consume {

                // Read variable name and potential default
                let mut var_expr = String::new();
                let mut depth = 1;
                for ch in chars.by_ref() {
                    if ch == '{' {
                        depth += 1;
                        var_expr.push(ch);
                    } else if ch == '}' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                        var_expr.push(ch);
                    } else {
                        var_expr.push(ch);
                    }
                }

                // Parse variable name and default value
                let (var_name, default_value) = if let Some(idx) = var_expr.find(":-") {
                    (&var_expr[..idx], Some(&var_expr[idx + 2..]))
                } else {
                    (var_expr.as_str(), None)
                };

                // Expand variable
                match env::var(var_name) {
                    Ok(value) => result.push_str(&value),
                    Err(_) => {
                        if let Some(default) = default_value {
                            result.push_str(default);
                        } else {
                            anyhow::bail!(
                                "Environment variable '{}' is required but not set. \
                                 Use ${{{}:-default}} to provide a default value.",
                                var_name,
                                var_name
                            );
                        }
                    }
                }
            } else {
                // Just a literal $, not followed by {
                result.push('$');
            }
        } else {
            result.push(c);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_required_var() {
        env::set_var("TEST_VAR", "test-value");
        let input = r#"key = "${TEST_VAR}""#;
        let output = expand_env_vars(input).unwrap();
        assert_eq!(output, r#"key = "test-value""#);
        env::remove_var("TEST_VAR");
    }

    #[test]
    fn test_expand_var_with_default() {
        env::remove_var("MISSING_VAR");
        let input = r#"key = "${MISSING_VAR:-default-value}""#;
        let output = expand_env_vars(input).unwrap();
        assert_eq!(output, r#"key = "default-value""#);
    }

    #[test]
    fn test_expand_var_with_default_override() {
        env::set_var("PRESENT_VAR", "actual-value");
        let input = r#"key = "${PRESENT_VAR:-default-value}""#;
        let output = expand_env_vars(input).unwrap();
        assert_eq!(output, r#"key = "actual-value""#);
        env::remove_var("PRESENT_VAR");
    }

    #[test]
    fn test_escaped_dollar_sign() {
        let input = r#"price = "$$100""#;
        let output = expand_env_vars(input).unwrap();
        assert_eq!(output, r#"price = "$100""#);
    }

    #[test]
    fn test_missing_required_var() {
        env::remove_var("REQUIRED_VAR");
        let input = r#"key = "${REQUIRED_VAR}""#;
        let result = expand_env_vars(input);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Environment variable 'REQUIRED_VAR' is required"));
    }

    #[test]
    fn test_multiple_vars() {
        env::set_var("VAR1", "value1");
        env::set_var("VAR2", "value2");
        let input = r#"
key1 = "${VAR1}"
key2 = "${VAR2}"
key3 = "${VAR3:-default3}"
"#;
        let output = expand_env_vars(input).unwrap();
        assert!(output.contains(r#"key1 = "value1""#));
        assert!(output.contains(r#"key2 = "value2""#));
        assert!(output.contains(r#"key3 = "default3""#));
        env::remove_var("VAR1");
        env::remove_var("VAR2");
    }

    #[test]
    fn test_no_expansion_needed() {
        let input = r#"key = "literal-value""#;
        let output = expand_env_vars(input).unwrap();
        assert_eq!(output, r#"key = "literal-value""#);
    }

    #[test]
    fn test_numeric_expansion() {
        env::set_var("PORT", "8080");
        let input = "port = ${PORT}"; // No quotes
        let output = expand_env_vars(input).unwrap();
        assert_eq!(output, "port = 8080");
        env::remove_var("PORT");
    }

    #[test]
    fn test_nested_braces_in_default() {
        env::remove_var("MISSING");
        let input = r#"key = "${MISSING:-{"nested": "value"}}""#;
        let output = expand_env_vars(input).unwrap();
        assert_eq!(output, r#"key = "{"nested": "value"}""#);
    }
}
