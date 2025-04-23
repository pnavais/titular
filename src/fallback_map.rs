//! A module providing a fallback map implementation for key-value lookups.
//!
//! This module implements a fallback mechanism for key-value lookups where values
//! can be provided by multiple sources in a chain. If a key is not found in one
//! provider, it will try the next one in the chain.
//!
//! # Examples
//!
//! ```rust
//! use titular::fallback_map::{FallbackMap, MapProvider};
//! use std::collections::HashMap;
//!
//! struct MyProvider {
//!     vars: HashMap<String, String>,
//! }
//!
//! impl MapProvider<str, String> for MyProvider {
//!     fn contains(&self, key: &str) -> bool {
//!         self.vars.contains_key(key)
//!     }
//!
//!     fn resolve(&self, key: &str) -> Option<&String> {
//!         self.vars.get(key)
//!     }
//!
//!     fn is_active(&self, _key: &str) -> bool {
//!         true
//!     }
//! }
//!
//! let mut provider1 = MyProvider { vars: HashMap::new() };
//! provider1.vars.insert("key1".to_string(), "value1".to_string());
//!
//! let mut provider2 = MyProvider { vars: HashMap::new() };
//! provider2.vars.insert("key2".to_string(), "value2".to_string());
//!
//! let mut map = FallbackMap::from(&provider1);
//! map.add(&provider2);
//!
//! assert_eq!(map.get("key1"), Some(&"value1".to_string()));
//! assert_eq!(map.get("key2"), Some(&"value2".to_string()));
//! assert_eq!(map.get("key3"), None);
//! ```
//!
//! # Thread Safety
//!
//! The `MapProvider` trait requires `Sync` to ensure thread safety. This means
//! that any implementation of `MapProvider` must be safe to share between threads.
//!
//! # Performance Considerations
//!
//! - The lookup is performed in order of providers, so the most frequently accessed
//!   values should be in the first provider.
//! - The implementation uses references to avoid cloning data, making it memory efficient.
use std::fmt::Debug;

/// A trait for providers that can supply key-value pairs.
///
/// This trait defines the interface for any type that can provide key-value pairs.
/// It is designed to be implemented by different types of data sources, such as
/// configuration files, environment variables, or in-memory maps.
///
/// # Type Parameters
/// * `K` - The key type, which can be unsized (e.g., str)
/// * `V` - The value type
///
/// # Examples
///
/// ```rust
/// use titular::fallback_map::MapProvider;
/// use std::collections::HashMap;
///
/// struct ConfigProvider {
///     vars: HashMap<String, String>,
/// }
///
/// impl MapProvider<str, String> for ConfigProvider {
///     fn contains(&self, key: &str) -> bool {
///         self.vars.contains_key(key)
///     }
///
///     fn resolve(&self, key: &str) -> Option<&String> {
///         self.vars.get(key)
///     }
///
///     fn is_active(&self, _key: &str) -> bool {
///         true
///     }
/// }
/// ```
pub trait MapProvider<K: ?Sized, V>: Sync {
    /// Checks if the provider contains the given key.
    ///
    /// # Arguments
    /// * `key` - The key to check for
    ///
    /// # Returns
    /// `true` if the provider contains the key, `false` otherwise.
    fn contains(&self, key: &K) -> bool;

    /// Resolves a key to its value.
    ///
    /// # Arguments
    /// * `key` - The key to resolve
    ///
    /// # Returns
    /// `Some(value)` if the key is found, `None` otherwise.
    fn resolve(&self, key: &K) -> Option<&V>;

    /// Checks if the key is active in this provider.
    ///
    /// This method can be used to implement conditional logic, such as
    /// feature flags or environment-specific settings.
    ///
    /// # Arguments
    /// * `key` - The key to check
    ///
    /// # Returns
    /// `true` if the key is active, `false` otherwise.
    fn is_active(&self, key: &K) -> bool;

    /// Returns an optional name for this provider.
    ///
    /// This is useful for debugging and logging purposes.
    ///
    /// # Returns
    /// An optional string containing the provider's name.
    fn get_name(&self) -> Option<String> {
        None
    }

    /// Returns a vector of (key, value) pairs for debugging purposes.
    ///
    /// This is optional and can return None if not implemented.
    ///
    /// # Returns
    /// A vector of (key, value) pairs for debugging purposes.
    fn debug_entries(&self) -> Option<Vec<(String, String)>> {
        None
    }
}

/// A map that falls back to other providers if a key is not found.
///
/// This is a pure reference-based structure that doesn't own any data.
/// It maintains a chain of providers and tries each one in sequence
/// until a value is found.
///
/// # Type Parameters
/// * `K` - The key type, which can be unsized (e.g., str)
/// * `V` - The value type
///
/// # Examples
///
/// ```rust
/// use titular::fallback_map::{FallbackMap, MapProvider};
/// use std::collections::HashMap;
///
/// struct MyProvider {
///     vars: HashMap<String, String>,
/// }
///
/// impl MapProvider<str, String> for MyProvider {
///     fn contains(&self, key: &str) -> bool {
///         self.vars.contains_key(key)
///     }
///
///     fn resolve(&self, key: &str) -> Option<&String> {
///         self.vars.get(key)
///     }
///
///     fn is_active(&self, _key: &str) -> bool {
///         true
///     }
/// }
///
/// let mut provider1 = MyProvider { vars: HashMap::new() };
/// provider1.vars.insert("key1".to_string(), "value1".to_string());
///
/// let mut provider2 = MyProvider { vars: HashMap::new() };
/// provider2.vars.insert("key2".to_string(), "value2".to_string());
///
/// let mut map = FallbackMap::from(&provider1);
/// map.add(&provider2);
///
/// assert_eq!(map.get("key1"), Some(&"value1".to_string()));
/// assert_eq!(map.get("key2"), Some(&"value2".to_string()));
/// ```
pub struct FallbackMap<'a, K, V>
where
    K: ?Sized,
{
    maps: Vec<&'a dyn MapProvider<K, V>>,
}

impl<'a, K, V> FallbackMap<'a, K, V>
where
    K: ?Sized,
{
    /// Creates a new empty FallbackMap.
    ///
    /// # Returns
    /// A new empty FallbackMap.
    pub fn new() -> Self {
        FallbackMap { maps: Vec::new() }
    }

    /// Creates a new FallbackMap with a single provider.
    ///
    /// # Arguments
    /// * `provider` - The initial provider to use
    ///
    /// # Returns
    /// A new FallbackMap with the given provider.
    pub fn from(provider: &'a dyn MapProvider<K, V>) -> Self {
        FallbackMap {
            maps: vec![provider],
        }
    }

    /// Adds a new provider to the fallback chain.
    ///
    /// # Arguments
    /// * `provider` - The provider to add
    ///
    /// # Returns
    /// `self` for method chaining.
    pub fn add(&mut self, provider: &'a dyn MapProvider<K, V>) -> &Self {
        self.maps.push(provider);
        self
    }

    /// Gets a value from the first provider that contains the key.
    ///
    /// # Arguments
    /// * `key` - The key to look up
    ///
    /// # Returns
    /// `Some(value)` if the key is found in any provider, `None` otherwise.
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

    /// Checks if any provider contains the key.
    ///
    /// # Arguments
    /// * `key` - The key to check
    ///
    /// # Returns
    /// `true` if any provider contains the key, `false` otherwise.
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

    /// Checks if any provider has the key active.
    ///
    /// # Arguments
    /// * `key` - The key to check
    ///
    /// # Returns
    /// `true` if any provider has the key active, `false` otherwise.
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

    /// Returns the number of providers in the fallback chain.
    ///
    /// # Returns
    /// The number of providers.
    pub fn provider_count(&self) -> usize {
        self.maps.len()
    }

    /// Creates a string containing all keys and values from all providers.
    ///
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

impl<'a, K, V> Debug for FallbackMap<'a, K, V>
where
    K: ?Sized + Sync + Send,
    V: Sync + Send,
    dyn MapProvider<K, V>: Sync + Send,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FallbackMap")
            .field("provider_count", &self.provider_count())
            .finish()
    }
}
