use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    hash::Hash,
};

#[derive(Debug, Default)]
pub struct DB<K, V> {
    messages: HashMap<K, V>,
}

impl<K: Clone + Eq + PartialEq + Hash + Display, V: Clone> DB<K, V> {
    pub fn new() -> DB<K, V> {
        DB {
            messages: HashMap::new(),
        }
    }

    pub fn add_message(&mut self, id: K, message: V) {
        self.messages.insert(id, message);
    }

    pub fn get_messages_as_value(&self) -> Vec<V> {
        self.messages.values().cloned().collect::<Vec<_>>()
    }

    pub fn get_message(&mut self, id: &K) -> Option<V> {
        self.messages.get(id).cloned()
    }
    pub fn delete_message(&mut self, id: &K) {
        self.messages.remove(id);
    }
}
