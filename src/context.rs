use std::collections::HashMap;

use crate:: {
    fallback_map::MapProvider,
};

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


#[derive(Debug)]
pub struct Context {
    context: HashMap<String, ContextValue>,   
}

/// Provides the methods to access the values present in the context struct
impl Context {
    pub fn new() -> Self {
        Context { context: HashMap::new() }
    }

    pub fn insert<S: AsRef<str>>(&mut self, key: S, value: S) {
        self.context.insert(String::from(key.as_ref()), ContextValue::String(value.as_ref().to_owned()));
    }
    
    /// Checks whether the context provides the given key
    pub fn contains<S: AsRef<str>>(&self, name: S) -> bool {
        self.context.contains_key(name.as_ref())
    }

    /// Retrieves a single value from the context for the given key (if available).
    /// In case of multiple values, the first value in the list will be returned.
    pub fn get<S: AsRef<str>>(&self, name: S) -> Option<&String> {
        match self.context.get(name.as_ref()) {
            Some(ContextValue::String(s)) => Some(s),
            Some(ContextValue::VecOfString(v)) => v.first(),
            None => None,
        }           
    }

    /// Inserts a list of values for the given key
    pub fn insert_many<S: AsRef<str>>(&mut self, key: S, values: Vec<String>) {
        self.context.insert(String::from(key.as_ref()), ContextValue::VecOfString(values));
    }

    /// Inserts multiple values incrementally for an initial key i.e. : the key name
    /// will be incremented accordingly and will be inserted with each value in the given list.
    pub fn insert_multi<S: AsRef<str>>(&mut self, key: S, values: Vec<String>) {
        let mut count = 1;
        for v in values {         
            let k = if count > 1 { format!("{}{}", key.as_ref(), count) } else { String::from(key.as_ref()) };       
            self.context.insert(k, ContextValue::String(v.to_string()));  
            count+=1;
        }
    }

    /// Retrieves all values for a given key (if multiple), or empty otherwise
    pub fn get_all<S: AsRef<str>>(&self, name: S) -> Option<Vec<String>> {
        match self.context.get(name.as_ref()) {
            Some(ContextValue::String(_)) => None,
            Some(ContextValue::VecOfString(v)) => Some(v.iter().map(|x| x.to_owned()).collect()),
            None => None,
        }   
    }

    pub fn print(&self) {
        for (k,v) in &self.context {
            println!("{}={:?}",k,v);
        }
    }
}

impl MapProvider<String, String> for Context {
    fn contains(&self, key: &String) -> bool {
        self.context.contains_key(key)
    }
    
    fn resolve(&self, key: &String) -> Option<&String> {
        self.get(key)
    }
}