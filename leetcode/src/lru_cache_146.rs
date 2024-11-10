use std::{cell::RefCell, collections::HashMap, rc::Rc};

struct Node {
    key: i32,
    value: i32,
    pre: Option<Rc<RefCell<Node>>>,
    next: Option<Rc<RefCell<Node>>>,
}

impl Node {
    pub fn new(key: i32, value: i32) -> Self {
        Self {
            key,
            value,
            pre: None,
            next: None,
        }
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.value == other.value
    }
}

struct LRUCache {
    capacity: i32,
    head: Option<Rc<RefCell<Node>>>,
    tail: Option<Rc<RefCell<Node>>>,
    dict: HashMap<i32, Rc<RefCell<Node>>>,
}

/// `&self` means the method takes an immutable reference.
/// If you need a mutable reference, change it to `&mut self` instead.
impl LRUCache {
    fn new(capacity: i32) -> Self {
        let dict = HashMap::with_capacity(capacity as usize);
        Self {
            capacity,
            dict,
            tail: None,
            head: None,
        }
    }

    fn move_node_to_head(&mut self, node: Rc<RefCell<Node>>) {
        if self.head.as_ref() == Some(&node) {
            return;
        }
        if self.tail.as_ref() == Some(&node) {
            self.tail = node.borrow_mut().pre.clone();
        }

        let pre = node.borrow().pre.clone();
        let next = node.borrow().next.clone();
        if let Some(pre) = &pre {
            pre.borrow_mut().next = next.clone();
        }
        if let Some(next) = &next {
            next.borrow_mut().pre = pre.clone();
        }

        node.borrow_mut().next = self.head.clone();
        if let Some(head) = &self.head {
            head.borrow_mut().pre = Some(node.clone());
        }
        self.head = Some(node);
    }

    fn push_node_to_head(&mut self, node: Rc<RefCell<Node>>) {
        if self.tail.is_none() {
            self.tail = Some(node.clone());
        }
        if self.head.is_none() {
            self.head = Some(node);
            return;
        }
        node.borrow_mut().next = self.head.clone();
        if let Some(head) = &self.head {
            head.borrow_mut().pre = Some(node.clone());
        }

        self.head = Some(node);
    }

    fn delete_tail(&mut self) {
        if let Some(tail) = self.tail.as_ref() {
            self.dict.remove(&tail.borrow().key);
            let pre = tail.borrow().pre.clone().take();
            self.tail = pre;
        }
    }

    fn get(&mut self, key: i32) -> i32 {
        match self.dict.get(&key) {
            Some(node) => {
                let val = node.borrow().value;
                self.move_node_to_head(node.clone());
                val
            }
            None => -1,
        }
    }

    fn put(&mut self, key: i32, value: i32) {
        match self.dict.get(&key) {
            Some(node) => {
                node.borrow_mut().value = value;
                self.move_node_to_head(node.clone());
            }
            None => {
                let node = Rc::new(RefCell::new(Node::new(key, value)));
                self.dict.insert(key, node.clone());
                self.push_node_to_head(node);
                if self.dict.len() > self.capacity as usize {
                    self.delete_tail();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lru_cache_146::LRUCache;

    #[test]
    fn test_lru_cache() {
        let mut cache = LRUCache::new(2);
        cache.put(1, 1);
        cache.put(2, 2);
        assert_eq!(cache.get(1), 1, "cache:{:?}", cache.dict.keys());
        cache.put(3, 3);
        assert_eq!(cache.get(2), -1, "cache:{:?}", cache.dict.keys());
        cache.put(4, 4);
        assert_eq!(cache.get(1), -1);
        assert_eq!(cache.get(3), 3);
        assert_eq!(cache.get(4), 4);
    }

    #[test]
    fn test_lru_cache2() {
        let mut cache = LRUCache::new(1);
        cache.put(2, 1);
        assert_eq!(cache.get(2), 1);
    }

    #[test]
    fn test_lru_cache3() {
        let mut cache = LRUCache::new(1);
        cache.put(2, 1);
        assert_eq!(cache.get(2), 1);
        cache.put(3, 2);
        assert_eq!(cache.get(2), -1);
        assert_eq!(cache.get(3), 2);
    }
}
