use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[allow(clippy::all)]
struct LRUCache {
    capacity: i32,
    head: Option<Rc<RefCell<Node>>>,
    tail: Option<Rc<RefCell<Node>>>,
    dict: HashMap<i32, Rc<RefCell<Node>>>,
}

struct Node {
    key: i32,
    value: i32,
    prev: Option<Rc<RefCell<Node>>>,
    next: Option<Rc<RefCell<Node>>>,
}

impl Node {
    fn new(key: i32, value: i32) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            key,
            value,
            prev: None,
            next: None,
        }))
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.value == other.value
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        self.prev = None;
        self.next = None;
        println!("Dropping Node: key={}, value={}", self.key, self.value);
    }
}

/// `&self` means the method takes an immutable reference.
/// If you need a mutable reference, change it to `&mut self` instead.
#[allow(clippy::all)]
impl LRUCache {
    fn new(capacity: i32) -> Self {
        let dict = HashMap::with_capacity(capacity as usize);
        Self {
            capacity,
            head: None,
            dict,
            tail: None,
        }
    }

    fn move_to_head(&mut self, node: Rc<RefCell<Node>>) {
        match self.head.as_ref() {
            Some(head) => {
                if head == &node {
                    return;
                }
                if self.tail.as_ref() == Some(&node) {
                    self.tail = node.borrow().prev.clone();
                }
                let next = node.borrow().next.clone();
                let pre = node.borrow().prev.clone();
                if let Some(ref pre) = pre {
                    pre.borrow_mut().next = next.clone();
                }
                if let Some(ref next) = next {
                    next.as_ref().borrow_mut().prev = pre.clone();
                }

                let head = head.clone();
                node.borrow_mut().next = Some(head.clone());
                head.borrow_mut().prev = Some(node.clone());
                self.head = Some(node);
            }
            None => {
                self.head = Some(node.clone());
                self.tail = Some(node);
            }
        }
    }

    fn get(&mut self, key: i32) -> i32 {
        match self.dict.get(&key) {
            Some(node) => {
                let node = node.clone();
                let val = node.borrow().value;
                self.move_to_head(node);
                val
            }
            None => -1,
        }
    }

    fn put(&mut self, key: i32, value: i32) {
        if let Some(node) = self.dict.get(&key) {
            let node = node.clone();
            node.borrow_mut().value = value;
            self.move_to_head(node);
        } else {
            let node = Node::new(key, value);
            self.push_front(node.clone());
            self.dict.insert(key, node);
            if self.dict.len() > self.capacity as usize {
                self.remove_last();
            }
        }
    }

    fn remove_last(&mut self) {
        match self.tail {
            Some(ref tail) => {
                let tail = tail.clone();
                let pre = tail.borrow().prev.clone().unwrap();
                pre.borrow_mut().next = None;
                self.tail = Some(pre);
                self.dict.remove(&tail.borrow().key);
            }
            None => {}
        }
    }

    fn push_front(&mut self, node: Rc<RefCell<Node>>) {
        match self.head {
            Some(ref head) => {
                node.borrow_mut().next = Some(head.clone());
                head.borrow_mut().prev = Some(node.clone());
                self.head = Some(node.clone());
                if self.tail.is_none() {
                    self.tail = Some(node);
                }
            }
            None => {
                self.head = Some(node.clone());
                self.tail = Some(node);
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
