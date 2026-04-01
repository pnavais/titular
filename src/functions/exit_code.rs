use std::collections::HashMap;
use tera::{Error as TeraError, Value};

/// Get the last exit code from the shell
///
/// # Arguments
/// * `_args` - A HashMap containing the function arguments (not used)
///
/// # Returns
/// A Tera Value containing the last exit code as a number
///
/// # Example
/// ```tera
/// {% if get_last_exit_code() == 0 %}
///     Command succeeded!
/// {% else %}
///     Command failed with code {{ get_last_exit_code() }}
/// {% endif %}
/// ```
pub fn get_last_exit_code(_args: &HashMap<String, Value>) -> Result<Value, TeraError> {
    // Get the last exit code from the environment
    let exit_code = std::env::var("LAST_EXIT_CODE")
        .or_else(|_| std::env::var("?"))
        .unwrap_or_else(|_| "0".to_string())
        .parse::<i64>()
        .map_err(|_| TeraError::msg("Failed to parse exit code as number"))?;

    Ok(Value::Number(exit_code.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_get_last_exit_code() {
        let args = HashMap::new();

        // Test with LAST_EXIT_CODE set
        env::set_var("LAST_EXIT_CODE", "42");
        let result = get_last_exit_code(&args).unwrap();
        assert_eq!(result.as_number().unwrap().as_i64().unwrap(), 42);

        // Test with ? set
        env::remove_var("LAST_EXIT_CODE");
        env::set_var("?", "1");
        let result = get_last_exit_code(&args).unwrap();
        assert_eq!(result.as_number().unwrap().as_i64().unwrap(), 1);

        // Test with neither set (should default to 0)
        env::remove_var("LAST_EXIT_CODE");
        env::remove_var("?");
        let result = get_last_exit_code(&args).unwrap();
        assert_eq!(result.as_number().unwrap().as_i64().unwrap(), 0);
    }
}
