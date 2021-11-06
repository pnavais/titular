
pub trait MapProvider<K,V> {
    fn contains(&self, key: &K) -> bool;
    fn resolve(&self, key: &K) -> Option<&V>;
}

pub struct FallbackMap<'a, K,V> {
    maps: Vec<Box<&'a dyn MapProvider<K,V>>>,
}

impl <'a, K,V> FallbackMap<'a, K,V> {    
    pub fn new() -> Self {
        FallbackMap { maps: Vec::new() }
    }

    pub fn from(provider: Box<&'a dyn MapProvider<K,V>>) -> Self {
        FallbackMap { maps: vec![provider] }
    }

    pub fn add(&mut self, provider: Box<&'a dyn MapProvider<K,V>>) -> &Self {
        self.maps.push(provider);
        self
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let mut value = None;
        for map in &self.maps {
            value = map.resolve(key);
            if value.is_some() { break; }
        }
        value
    }

    pub fn contains(&self, key: &K) -> bool {
        let mut res = false;
        for map in &self.maps {
            if map.contains(key) { res = true; break; }
        }

        res
    }
    
}