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
pub struct Context {
    context: HashMap<String, ContextValue>,   
}

impl Context {
    pub fn new() -> Self {
        Context { context: HashMap::new() }
    }

    pub fn insert<S: AsRef<str>>(&mut self, key: S, value: S) {
        self.context.insert(String::from(key.as_ref()), ContextValue::String(String::from(value.as_ref())));
    }


    pub fn get<S: AsRef<str>>(&self, name: S) -> Option<&String> {
        match self.context.get(name.as_ref()) {
            Some(ContextValue::String(s)) => Some(s),
            Some(ContextValue::VecOfString(v)) => v.first(),
            None => None,
        }           
    }

    pub fn insert_many<S: AsRef<str>>(&mut self, key: S, values: Vec<String>) {        
        self.context.insert(String::from(key.as_ref()), ContextValue::VecOfString(values));
    }

    pub fn insert_multi<S: AsRef<str>>(&mut self, key: S, values: Vec<String>) {
        let mut count = 1;
        for v in values {         
            let k = if count > 1 { format!("{}{}", key.as_ref(), count) } else { String::from(key.as_ref()) };       
            self.context.insert(k, ContextValue::String(v.to_string()));  
            count+=1;
        }
    }

    pub fn get_all<S: AsRef<str>>(&self, name: S) -> Option<&Vec<String>> {
        match self.context.get(name.as_ref()) {
            Some(ContextValue::String(_)) => None,
            Some(ContextValue::VecOfString(v)) => Some(v),
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
    fn resolve(&self, key: &String) -> Option<&String> {
        self.get(key)
    }
}