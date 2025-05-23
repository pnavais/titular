use crate::error::{Error, Result};
use serde::Serialize;
use serde_json::value::Value;
use std::any::Any;
use std::collections::{HashMap, HashSet};
use tera::Context as TeraContext;

#[derive(Debug)]
pub struct MissingVar {
    pub key: &'static str,
    pub var: String,
}

/// Template context for variable substitution
#[derive(Debug, Default)]
struct TemplateContext {
    data: TeraContext,
    keys: HashSet<&'static str>,
}

/// Registry for storing and retrieving arbitrary components
#[derive(Debug, Default)]
struct Registry {
    items: HashMap<&'static str, Box<dyn Any + Send + Sync>>,
}

#[derive(Debug, Default)]
pub struct Context {
    template: TemplateContext,
    registry: Registry,
}

/// Provides the methods to access the values present in the context struct
impl Context {
    pub fn new() -> Self {
        Context {
            template: TemplateContext::default(),
            registry: Registry::default(),
        }
    }

    /// Stores a component in the registry
    ///
    /// # Arguments
    /// * `key` - The key to store the component under
    /// * `value` - The component to store
    ///
    /// # Returns
    /// The key used to store the component
    pub fn store_object<T: 'static + Send + Sync>(&mut self, key: &str, value: T) -> &'static str {
        let key_str = key.to_string();
        let key_ref: &'static str = Box::leak(key_str.into_boxed_str());
        self.registry.items.insert(key_ref, Box::new(value));
        key_ref
    }

    /// Retrieves a component from the registry
    ///
    /// # Arguments
    /// * `key` - The key to retrieve the component for
    ///
    /// # Returns
    /// An option containing a reference to the component if found
    pub fn get_object<T: 'static + Send + Sync>(&self, key: &str) -> Option<&T> {
        self.registry
            .items
            .get(key)
            .and_then(|obj| obj.downcast_ref::<T>())
    }

    /// Removes a component from the registry
    ///
    /// # Arguments
    /// * `key` - The key to remove the component for
    ///
    /// # Returns
    /// The removed component if found
    pub fn remove_object<T: 'static + Send + Sync>(&mut self, key: &str) -> Option<T> {
        self.registry
            .items
            .remove(key)
            .and_then(|obj| obj.downcast::<T>().ok().map(|boxed| *boxed))
    }

    /// Resolves a variable reference in the format $var or ${var:default_value}
    fn resolve_variable(&self, value: &str, visited: &mut HashSet<String>) -> Result<String> {
        // Check if the value is a variable reference
        if !value.starts_with('$') {
            return Ok(value.to_string());
        }

        // Extract variable name and default value if present
        let (var_name, default_value) = if value.starts_with("${") && value.ends_with('}') {
            let content = &value[2..value.len() - 1];
            match content.split_once(':') {
                Some((name, default)) => (name, Some(default)),
                None => (content, None),
            }
        } else if value.starts_with('$') {
            (&value[1..], None)
        } else {
            return Ok(value.to_string());
        };

        // Check for cycles
        if visited.contains(var_name) {
            return Err(Error::ContextCyclicReference(var_name.to_string()));
        }

        // Mark as visited
        visited.insert(var_name.to_string());

        // Get the value from context
        let resolved = match self.get(var_name) {
            Some(value) => value.to_string(),
            None => {
                // If no value found and we have a default, try to resolve it
                if let Some(default) = default_value {
                    // Recursively resolve the default value
                    self.resolve_variable(&format!("${}", default), visited)?
                } else {
                    return Err(Error::ContextVariableNotFound(var_name.to_string()));
                }
            }
        };

        // Recursively resolve any nested variables
        let final_value = self.resolve_variable(&resolved, visited)?;

        // Remove from visited set
        visited.remove(var_name);

        Ok(final_value)
    }

    /// Returns a reference to the underlying tera context
    pub fn get_data(&self) -> &TeraContext {
        &self.template.data
    }

    /// Attempts to resolve a list of previously failed variables
    ///
    /// # Arguments
    /// * `missing_vars` - Vector of missing variables with their associated keys
    fn resolve_missing_vars(&mut self, _missing_vars: Vec<MissingVar>) {
        // Intentionally left empty for now
        for missing in _missing_vars {
            let value = match self.resolve_variable(&missing.var, &mut HashSet::new()) {
                Ok(resolved) => Value::String(resolved),
                Err(_) => Value::String(if missing.var.starts_with('$') {
                    String::new()
                } else {
                    missing.var
                }),
            };
            self.template.data.insert(missing.key, &value);
            self.template.keys.insert(missing.key);
        }
    }

    pub fn from<I, K, V>(context: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Serialize + std::fmt::Display,
    {
        let mut context_map = Context::new();
        let mut missing_vars = Vec::new();
        for (key, value) in context {
            let key_str = key.into();
            if let Some(missing) = context_map.insert(key_str, &value) {
                missing_vars.push(missing);
            }
        }
        context_map.resolve_missing_vars(missing_vars);
        context_map
    }

    /// Extends the context with the given context replacing existing keys.
    ///
    /// # Arguments
    /// * `context` - The context to extend the current context with.
    pub fn extend<I, K, V>(&mut self, context: I)
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Serialize + std::fmt::Display,
    {
        let mut missing_vars = Vec::new();
        for (key, value) in context {
            if let Some(missing) = self.insert(key, &value) {
                missing_vars.push(missing);
            }
        }
        self.resolve_missing_vars(missing_vars);
    }

    /// Extends the context with the given context replacing existing keys.
    ///
    /// # Arguments
    /// * `context` - The context to extend the current context with.
    pub fn append<I, K, V>(&mut self, context: I)
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String> + AsRef<str>,
        V: Serialize + std::fmt::Display,
    {
        let mut missing_vars = Vec::new();
        for (key, value) in context {
            if !self.contains(key.as_ref()) {
                if let Some(missing) = self.insert(key, &value) {
                    missing_vars.push(missing);
                }
            }
        }
        self.resolve_missing_vars(missing_vars);
    }

    /// Extends the context with the given context replacing existing keys.
    ///
    /// # Arguments
    /// * `context` - The context to extend the current context with.
    pub fn append_from(&mut self, context: &Context) {
        let mut missing_vars = Vec::new();
        for key in &context.template.keys {
            if let Some(value) = context.get_raw(key) {
                if let Some(missing) = self.insert(*key, value) {
                    missing_vars.push(missing);
                }
            }
        }
        self.resolve_missing_vars(missing_vars);
    }

    /// Inserts a value into the context
    ///
    /// # Arguments
    /// * `key` - The key to insert the value into.
    /// * `val` - The value to insert into the context.
    ///
    /// # Returns
    /// Returns `Some(MissingVar)` if a variable resolution failed, or `None` if all resolutions succeeded
    pub fn insert<T: Serialize + std::fmt::Display + ?Sized, S: Into<String>>(
        &mut self,
        key: S,
        val: &T,
    ) -> Option<MissingVar> {
        let mut failed_value: Option<String> = None;
        let key_str = key.into();
        let value = match serde_json::to_value(val) {
            Ok(Value::String(s)) => {
                // Try to resolve variable references
                match self.resolve_variable(&s, &mut HashSet::new()) {
                    Ok(resolved) => Value::String(resolved),
                    Err(_) => {
                        let value = Value::String(if s.starts_with('$') {
                            String::new()
                        } else {
                            s.clone()
                        });
                        failed_value = Some(s);
                        value
                    }
                }
            }
            Ok(v) => v,
            Err(_) => Value::String(val.to_string()),
        };

        let key_ref: &'static str = Box::leak(key_str.into_boxed_str());
        self.template.data.insert(key_ref, &value);
        self.template.keys.insert(key_ref);
        failed_value.map(|var| MissingVar { key: key_ref, var })
    }

    /// Checks whether the context provides the given key
    pub fn contains<S: AsRef<str>>(&self, name: S) -> bool {
        self.template.data.contains_key(name.as_ref())
    }

    /// Retrieves the raw value from the context for the given key (if available).
    ///
    /// # Arguments
    /// * `name` - The name of the key to retrieve the value for.
    ///
    /// # Returns
    /// Returns an option containing a reference to the value associated with the given key.
    pub fn get_raw(&self, key: &str) -> Option<&Value> {
        self.template.data.get(key)
    }

    /// Retrieves a single value from the context for the given key (if available)
    /// as a string
    /// In case of multiple values, the first value in the list will be returned.
    ///
    /// # Arguments
    /// * `name` - The name of the key to retrieve the value for.
    ///
    /// # Returns
    /// Returns an option containing a reference to the value associated with the given key.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.get_raw(key).and_then(|v| match v {
            Value::Array(arr) if !arr.is_empty() => arr[0].as_str(),
            _ => v.as_str(),
        })
    }

    /// Retrieves all values for a given key (if multiple), or empty otherwise
    ///
    /// # Arguments
    /// * `name` - The name of the key to retrieve values for.
    ///
    /// # Returns
    /// Returns a vector of strings containing all values associated with the given key.
    pub fn get_all(&self, key: &str) -> Option<Vec<&str>> {
        self.get_raw(key).and_then(|v| match v {
            Value::Array(arr) => {
                let strings: Vec<&str> = arr.iter().filter_map(|v| v.as_str()).collect();
                if strings.is_empty() {
                    None
                } else {
                    Some(strings)
                }
            }
            _ => v.as_str().map(|s| vec![s]),
        })
    }

    /// Retrieves the boolean value of a given key in the context (if available).
    ///
    /// # Arguments
    /// * `name` - The name of the key to check.
    ///
    /// # Returns
    /// Returns `true` if the key exists and its value is "true" or "1", `false` otherwise.
    pub fn get_flag(&self, key: &str) -> Option<bool> {
        match self.template.data.get(key) {
            Some(v) => Some(matches!(
                v.as_str()?.trim().to_lowercase().as_str(),
                "true" | "1"
            )),
            _ => None,
        }
    }

    /// Checks whether the context provides the given key
    /// and if it is set to "true"
    ///
    /// # Arguments
    /// * `key` - The key to check.
    ///
    /// # Returns
    /// Returns `true` if the key exists and its value is "true" or "1", `false` otherwise.
    pub fn is_active(&self, key: &str) -> bool {
        match self.get(key) {
            Some(v) => matches!(v.trim().to_lowercase().as_str(), "true" | "1"),
            None => false,
        }
    }

    /// Inserts a list of values for the given key
    ///
    /// # Arguments
    /// * `key` - The key to insert the values into.
    /// * `values` - The values to insert into the context.
    pub fn insert_many(&mut self, key: &str, values: Vec<&str>) {
        let resolved_values: Vec<Value> = values
            .into_iter()
            .map(|v| match self.resolve_variable(v, &mut HashSet::new()) {
                Ok(resolved) => Value::String(resolved),
                Err(_) => Value::String(v.to_string()),
            })
            .collect();

        let json_values = Value::Array(resolved_values);

        let key_ref: &'static str = Box::leak(key.to_string().into_boxed_str());
        self.template.data.insert(key_ref, &json_values);
        self.template.keys.insert(key_ref);
    }

    /// Inserts multiple values incrementally for an initial key i.e. : the key name
    /// will be incremented accordingly and will be inserted with each value in the given list.
    ///
    /// # Arguments
    /// * `key` - The key to insert the values into.
    /// * `values` - The values to insert into the context.
    pub fn insert_multi(&mut self, key: &str, values: Vec<&str>) {
        let mut count = 1;
        for v in values {
            let k = if count > 1 {
                let mut k = String::new();
                k.push_str(key);
                k.push_str(&count.to_string());
                k
            } else {
                key.to_string()
            };
            self.insert(k, &v);
            count += 1;
        }
    }
}
