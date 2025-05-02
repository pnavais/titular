use serde::Serialize;
use serde_json::value::Value;
use std::collections::HashSet;
use tera::Context as TeraContext;

#[derive(Debug)]
pub enum Modifier {
    INV,
    NONE,
}

#[derive(Debug, Default)]
pub struct Context {
    data: TeraContext,
    keys: HashSet<&'static str>,
}

/// Provides the methods to access the values present in the context struct
impl Context {
    pub fn new() -> Self {
        Context {
            data: TeraContext::new(),
            keys: HashSet::new(),
        }
    }

    /// Returns a reference to the underlying tera context
    pub fn get_data(&self) -> &TeraContext {
        &self.data
    }

    pub fn from<I, K, V>(context: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Serialize,
    {
        let mut context_map = Context::new();
        for (key, value) in context {
            let key_str = key.into();
            context_map.insert(key_str, &value);
        }
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
        V: Serialize,
    {
        for (key, value) in context {
            self.insert(key, &value);
        }
    }

    /// Extends the context with the given context replacing existing keys.
    ///
    /// # Arguments
    /// * `context` - The context to extend the current context with.
    pub fn extend_from(&mut self, context: &Context) {
        for key in &context.keys {
            if let Some(value) = context.get_raw(key) {
                self.insert(*key, value);
            }
        }
    }

    /// Inserts a value into the context
    ///
    /// # Arguments
    /// * `key` - The key to insert the value into.
    /// * `val` - The value to insert into the context.
    pub fn insert<T: Serialize + ?Sized, S: Into<String>>(&mut self, key: S, val: &T) {
        let key_ref: &'static str = Box::leak(key.into().into_boxed_str());
        self.data.insert(key_ref, val);
        self.keys.insert(key_ref);
    }

    /// Checks whether the context provides the given key
    pub fn contains<S: AsRef<str>>(&self, name: S) -> bool {
        self.data.contains_key(name.as_ref())
    }

    /// Retrieves the raw value from the context for the given key (if available).
    ///
    /// # Arguments
    /// * `name` - The name of the key to retrieve the value for.
    ///
    /// # Returns
    /// Returns an option containing a reference to the value associated with the given key.
    pub fn get_raw(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
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
        match self.data.get(key) {
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
        let json_values = Value::Array(
            values
                .into_iter()
                .map(|v| Value::String(v.to_string()))
                .collect(),
        );

        let key_ref: &'static str = Box::leak(key.to_string().into_boxed_str());
        self.data.insert(key_ref, &json_values);
        self.keys.insert(key_ref);
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
