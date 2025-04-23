use std::collections::HashMap;

use crate::fallback_map::MapProvider;

#[derive(Debug)]
enum ContextValue {
    String(String),
    VecOfString(Vec<String>),
}

#[derive(Debug)]
pub enum Modifier {
    INV,
    NONE,
}

#[derive(Debug, Default)]
pub struct Context {
    context: HashMap<String, ContextValue>,
}

/// Provides the methods to access the values present in the context struct
impl Context {
    pub fn new() -> Self {
        Context {
            context: HashMap::new(),
        }
    }

    pub fn insert<S: AsRef<str>>(&mut self, key: S, value: S) {
        self.context.insert(
            String::from(key.as_ref()),
            ContextValue::String(value.as_ref().to_owned()),
        );
    }

    /// Checks whether the context provides the given key
    pub fn contains<S: AsRef<str>>(&self, name: S) -> bool {
        self.context.contains_key(name.as_ref())
    }

    /// Retrieves a single value from the context for the given key (if available).
    /// In case of multiple values, the first value in the list will be returned.
    ///
    /// # Arguments
    /// * `name` - The name of the key to retrieve the value for.
    ///
    /// # Returns
    /// Returns an option containing a reference to the string value associated with the given key.
    pub fn get<S: AsRef<str>>(&self, name: S) -> Option<&String> {
        match self.context.get(name.as_ref()) {
            Some(ContextValue::String(s)) => Some(s),
            Some(ContextValue::VecOfString(v)) => v.first(),
            None => None,
        }
    }

    /// Retrieves all values for a given key (if multiple), or empty otherwise
    ///
    /// # Arguments
    /// * `name` - The name of the key to retrieve values for.
    ///
    /// # Returns
    /// Returns a vector of strings containing all values associated with the given key.
    pub fn get_all<S: AsRef<str>>(&self, name: S) -> Option<Vec<String>> {
        match self.context.get(name.as_ref()) {
            Some(ContextValue::String(_)) => None,
            Some(ContextValue::VecOfString(v)) => Some(v.iter().map(|x| x.to_owned()).collect()),
            None => None,
        }
    }

    /// Retrieves the boolean value of a given key in the context (if available).
    ///
    /// # Arguments
    /// * `name` - The name of the key to check.
    ///
    /// # Returns
    /// Returns `true` if the key exists and its value is "true" or "1", `false` otherwise.
    pub fn get_flag<S: AsRef<str>>(&self, name: S) -> Option<bool> {
        match self.context.get(name.as_ref()) {
            Some(ContextValue::String(s)) => {
                Some(matches!(s.trim().to_lowercase().as_str(), "true" | "1"))
            }
            _ => None,
        }
    }

    /// Inserts a list of values for the given key
    pub fn insert_many<S: AsRef<str>>(&mut self, key: S, values: Vec<String>) {
        self.context.insert(
            String::from(key.as_ref()),
            ContextValue::VecOfString(values),
        );
    }

    /// Inserts multiple values incrementally for an initial key i.e. : the key name
    /// will be incremented accordingly and will be inserted with each value in the given list.
    pub fn insert_multi<S: AsRef<str>>(&mut self, key: S, values: Vec<String>) {
        let mut count = 1;
        for v in values {
            let k = if count > 1 {
                format!("{}{}", key.as_ref(), count)
            } else {
                String::from(key.as_ref())
            };
            self.context.insert(k, ContextValue::String(v.to_string()));
            count += 1;
        }
    }

    pub fn print(&self) {
        for (k, v) in &self.context {
            println!("{}={:?}", k, v);
        }
    }
}

impl MapProvider<str, String> for Context {
    fn contains(&self, key: &str) -> bool {
        self.context.contains_key(key)
    }

    fn resolve(&self, key: &str) -> Option<&String> {
        self.get(key)
    }

    fn is_active(&self, key: &str) -> bool {
        match self.get(key) {
            Some(v) => v == "true",
            None => false,
        }
    }

    fn get_name(&self) -> Option<String> {
        Some("Context".to_string())
    }

    fn debug_entries(&self) -> Option<Vec<(String, String)>> {
        Some(
            self.context
                .iter()
                .filter_map(|(k, v)| match v {
                    ContextValue::String(s) => Some((k.clone(), s.clone())),
                    ContextValue::VecOfString(v) => v.first().map(|s| (k.clone(), s.clone())),
                })
                .collect(),
        )
    }
}
