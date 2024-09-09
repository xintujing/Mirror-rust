use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
pub enum Operation {
    Add,
    Set,
    Insert,
    RemoveAt,
    Clear,
}

type Callback<T> = Arc<dyn Fn(Operation, usize, &T) + Send + Sync>;

struct SyncList<T> {
    items: Vec<T>,
    on_add: Option<Callback<T>>,
    on_insert: Option<Callback<T>>,
    on_set: Option<Callback<T>>,
    on_remove: Option<Callback<T>>,
    on_clear: Option<Callback<T>>,
    changes: RefCell<VecDeque<Change<T>>>,
}

struct Change<T> {
    operation: Operation,
    index: usize,
    item: T,
}

impl<T: Clone> SyncList<T> {
    pub fn new() -> Self {
        SyncList {
            items: Vec::new(),
            on_add: None,
            on_insert: None,
            on_set: None,
            on_remove: None,
            on_clear: None,
            changes: RefCell::new(VecDeque::new()),
        }
    }

    pub fn add(&mut self, item: T) {
        let index = self.items.len();
        self.items.push(item.clone());
        if let Some(ref callback) = self.on_add {
            callback(Operation::Add, index, &item);
        }
        self.changes.borrow_mut().push_back(Change {
            operation: Operation::Add,
            index,
            item,
        });
    }

    pub fn insert(&mut self, index: usize, item: T) {
        self.items.insert(index, item.clone());
        if let Some(ref callback) = self.on_insert {
            callback(Operation::Insert, index, &item);
        }
        self.changes.borrow_mut().push_back(Change {
            operation: Operation::Insert,
            index,
            item,
        });
    }

    pub fn set(&mut self, index: usize, item: T) {
        let old_item = self.items[index].clone();
        self.items[index] = item.clone();
        if let Some(ref callback) = self.on_set {
            callback(Operation::Set, index, &old_item);
        }
        self.changes.borrow_mut().push_back(Change {
            operation: Operation::Set,
            index,
            item,
        });
    }

    pub fn remove_at(&mut self, index: usize) {
        let old_item = self.items.remove(index);
        if let Some(ref callback) = self.on_remove {
            callback(Operation::RemoveAt, index, &old_item);
        }
        self.changes.borrow_mut().push_back(Change {
            operation: Operation::RemoveAt,
            index,
            item: old_item,
        });
    }

    pub fn clear(&mut self) {
        if let Some(ref callback) = self.on_clear {
            callback(Operation::Clear, 0, &self.items[0]); // Simplified: assuming non-empty list for example
        }
        self.items.clear();
        self.changes.borrow_mut().push_back(Change {
            operation: Operation::Clear,
            index: 0,
            item: Default::default(), // Assuming T: Default
        });
    }
}

impl<T: Clone + Default> Default for SyncList<T> {
    fn default() -> Self {
        Self::new()
    }
}
