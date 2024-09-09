use std::collections::HashMap;
use std::hash::Hash;

pub enum Operation {
    Add,
    Clear,
    Remove,
    Set,
}

pub struct SyncDictionary<K, V> {
    pub objects: HashMap<K, V>,
    pub on_add: Option<Box<dyn Fn(&K)>>,
    pub on_set: Option<Box<dyn Fn(&K, &V)>>,
    pub on_remove: Option<Box<dyn Fn(&K, &V)>>,
    pub on_change: Option<Box<dyn Fn(Operation, &K, &V)>>,
    pub on_clear: Option<Box<dyn Fn()>>,
}

impl<K, V> SyncDictionary<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn new() -> Self {
        SyncDictionary {
            objects: HashMap::new(),
            on_add: None,
            on_set: None,
            on_remove: None,
            on_change: None,
            on_clear: None,
        }
    }

    pub fn add(&mut self, key: K, value: V) {
        self.objects.insert(key.clone(), value.clone());
        if let Some(ref callback) = self.on_add {
            callback(&key);
        }
        if let Some(ref callback) = self.on_change {
            callback(Operation::Add, &key, &value);
        }
    }

    pub fn set(&mut self, key: K, value: V) {
        if let Some(old_value) = self.objects.get(&key).cloned() {
            self.objects.insert(key.clone(), value.clone());
            if let Some(ref callback) = self.on_set {
                callback(&key, &old_value);
            }
            if let Some(ref callback) = self.on_change {
                callback(Operation::Set, &key, &old_value);
            }
        }
    }

    pub fn remove(&mut self, key: K) {
        if let Some(old_value) = self.objects.remove(&key) {
            if let Some(ref callback) = self.on_remove {
                callback(&key, &old_value);
            }
            if let Some(ref callback) = self.on_change {
                callback(Operation::Remove, &key, &old_value);
            }
        }
    }

    pub fn clear(&mut self) {
        self.objects.clear();
        if let Some(ref callback) = self.on_clear {
            callback();
        }
        if let Some(ref callback) = self.on_change {
            callback(Operation::Clear, &K::default(), &V::default());  // Assumes K and V implement Default
        }
    }
}

impl<K: Eq + Hash + Clone, V: Clone> SyncDictionary<K, V> {
    pub fn with_capacity(capacity: usize) -> Self {
        SyncDictionary {
            objects: HashMap::with_capacity(capacity),
            on_add: None,
            on_set: None,
            on_remove: None,
            on_change: None,
            on_clear: None,
        }
    }
}

impl<K, V> Default for SyncDictionary<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}
