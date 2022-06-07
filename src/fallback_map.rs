pub trait MapProvider<K,V> {
    fn contains(&self, key: &K) -> bool;
    fn resolve(&self, key: &K) -> Option<&V>;
    fn is_active(&self, key: &K) -> bool;
}

#[derive(Default)]
pub struct FallbackMap<'a, K,V> {
    maps: Vec<&'a dyn MapProvider<K,V>>,
}

impl <'a, K,V> FallbackMap<'a, K,V> {    
    pub fn new() -> Self {
        FallbackMap { maps: Vec::new() }
    }

    pub fn from(provider: &'a dyn MapProvider<K,V>) -> Self {
        FallbackMap { maps: vec![provider] }
    }

    pub fn add(&mut self, provider: &'a dyn MapProvider<K,V>) -> &Self {
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

    pub fn is_active(&self, key: &K) -> bool {
        let mut res = false;
        for map in &self.maps {
            if map.is_active(key) { res = true; break; }
        }

        res
    }
    
}
