pub trait MapProvider<K: ?Sized, V> {
    fn contains(&self, key: &K) -> bool;
    fn resolve(&self, key: &K) -> Option<&V>;
    fn is_active(&self, key: &K) -> bool;
    fn get_name(&self) -> Option<String> {
        None
    }

    /// Returns a vector of (key, value) pairs for debugging purposes.
    /// This is optional and can return None if not implemented.
    ///
    /// # Returns
    /// A vector of (key, value) pairs for debugging purposes.
    fn debug_entries(&self) -> Option<Vec<(String, String)>> {
        None
    }
}

#[derive(Default)]
pub struct FallbackMap<'a, K, V> {
    maps: Vec<&'a dyn MapProvider<K, V>>,
}

impl<'a, K, V> FallbackMap<'a, K, V> {
    pub fn new() -> Self {
        FallbackMap { maps: Vec::new() }
    }

    pub fn from(provider: &'a dyn MapProvider<K, V>) -> Self {
        FallbackMap {
            maps: vec![provider],
        }
    }

    pub fn add(&mut self, provider: &'a dyn MapProvider<K, V>) -> &Self {
        self.maps.push(provider);
        self
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let mut value = None;
        for map in &self.maps {
            value = map.resolve(key);
            if value.is_some() {
                break;
            }
        }
        value
    }

    pub fn contains(&self, key: &K) -> bool {
        let mut res = false;
        for map in &self.maps {
            if map.contains(key) {
                res = true;
                break;
            }
        }

        res
    }

    pub fn is_active(&self, key: &K) -> bool {
        let mut res = false;
        for map in &self.maps {
            if map.is_active(key) {
                res = true;
                break;
            }
        }

        res
    }

    /// Creates a string containing all keys and values from all providers.
    /// This is intended for debugging purposes.
    ///
    /// # Returns
    /// A string containing all keys and values from all providers.
    pub fn debug_dump(&self) -> String
    where
        K: std::fmt::Display,
        V: std::fmt::Display,
    {
        let mut result = String::new();
        for (i, provider) in self.maps.iter().enumerate() {
            result.push_str(&format!(
                "Provider [{}/{}]{}:\n",
                i + 1,
                self.maps.len(),
                provider
                    .get_name()
                    .map_or("".to_string(), |name| format!(" ({})", name))
            ));
            if let Some(entries) = provider.debug_entries() {
                for (key, value) in entries {
                    result.push_str(&format!("  {}: \"{}\"\n", key, value));
                }
            }
        }
        result
    }
}

/// Implementations for String keys
///
/// These implementations are useful for the context map, which is a map of strings to values.
///
/// The context map is a map of strings to values, and it is used to store the context of the application.
///
/// The context map is used to store the context of the application.
///
impl<'a, V> FallbackMap<'a, String, V> {
    /// Get a value using a string-like key (String or &str)
    pub fn get_str<S: AsRef<str>>(&self, key: S) -> Option<&V> {
        self.get(&key.as_ref().to_string())
    }

    /// Check if a key exists using a string-like key (String or &str)
    pub fn contains_str<S: AsRef<str>>(&self, key: S) -> bool {
        self.contains(&key.as_ref().to_string())
    }

    /// Check if a key is active using a string-like key (String or &str)
    pub fn is_active_str<S: AsRef<str>>(&self, key: S) -> bool {
        self.is_active(&key.as_ref().to_string())
    }

    /// Gets a value by key, returning a string slice.
    /// This is more efficient than get_str when you need a &str.
    pub fn get_str_ref<S: AsRef<str>>(&self, key: S) -> Option<&str>
    where
        V: AsRef<str>,
    {
        self.get(&key.as_ref().to_string()).map(|s| s.as_ref())
    }

    /// Gets a value by key, returning a string slice or a default value.
    /// This is more efficient than get_str when you need a &str.
    pub fn get_str_ref_or<'b, S: AsRef<str>>(&'b self, key: S, default: &'b str) -> &'b str
    where
        V: AsRef<str>,
    {
        let key_str = key.as_ref().to_string();
        self.get(&key_str).map(|s| s.as_ref()).unwrap_or(default)
    }
}
