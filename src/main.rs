use std::{
    cmp::Ordering,
    fmt::{Debug, Display, Formatter},
};

fn main() {
    let mut tree = BTree::new(4);

    for i in 1..20 {
        tree.insert(Item::new(i));
    }

    tree.remove(8);
    println!("{}", tree);
}

#[derive(Debug)]
struct BTree<K: Ord + Debug, V: Debug> {
    root: Box<Node<K, V>>,
    max_size: usize,
}

#[derive(Debug)]
struct Node<K: Ord + Debug, V: Debug> {
    items: Vec<Box<Item<K, V>>>,
    children: Vec<Box<Node<K, V>>>,
    max_size: usize,
}

#[derive(Debug)]
struct Item<K: Ord + Debug, V: Debug> {
    key: K,
    value: V,
}

impl<K: Ord + Debug, V: Debug> BTree<K, V> {
    fn new(capacity: usize) -> Self {
        assert!(capacity > 1);

        Self {
            root: Box::new(Node::new(capacity)),
            max_size: capacity,
        }
    }

    fn insert(&mut self, item: Item<K, V>) {
        if self.root.is_full() {
            let mut new_root = Node::new(self.max_size);

            new_root.children.push(Box::new(std::mem::replace(
                &mut self.root,
                Node::new(self.max_size),
            )));

            new_root.split_child(0);
            self.root = Box::new(new_root);
        }

        self.root.insert(item);
    }

    fn remove(&mut self, key: K) {
        self.root.remove(&key);

        if self.root.items.is_empty() {
            self.root = self.root.children.remove(0);
        }
    }
}

impl<K: Ord + Debug, V: Debug> Node<K, V> {
    fn new(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
            children: Vec::with_capacity(capacity + 1),
            max_size: capacity,
        }
    }

    fn is_full(&self) -> bool {
        self.items.len() == self.max_size
    }

    fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    fn remove(&mut self, key: &K) {
        match self
            .items
            .binary_search_by(|element| element.as_ref().cmp_key(key))
        {
            Ok(index) => {
                self.items.remove(index);
            }
            Err(index) => {
                let child = self.children[index].as_mut();

                child.remove(key);
                self.rebalance_after_removal(index);
            }
        };
    }

    fn rebalance_after_removal(&mut self, index: usize) {
        if self.children[index].items.len() >= self.max_size / 2 {
            return;
        }

        let min_size = self.max_size / 2;

        if index != 0 {
            if self.is_leaf() {
                return;
            }

            if self.children[index - 1].items.len() > min_size {
                self.take_item_from(index, index - 1);
                return;
            }

            self.merge_child(index - 1);
            return;
        }

        if self.children[index + 1].items.len() > min_size {
            self.take_item_from(index, index + 1);
            return;
        }

        self.merge_child(index);
    }

    fn take_item_from(&mut self, dest: usize, src: usize) {
        let current_divider = if src < dest {
            self.items[src].as_mut()
        } else {
            self.items[dest].as_mut()
        };

        let taken_item = if src < dest {
            let index = self.children[src].items.len() - 1;

            self.children[src].items.remove(index)
        } else {
            self.children[src].items.remove(0)
        };

        let new_item = Box::new(std::mem::replace(current_divider, *taken_item));

        if src < dest {
            self.children[dest].items.insert(0, new_item);
        } else {
            self.children[dest].items.push(new_item);
        }
    }

    fn merge_child(&mut self, left_index: usize) {
        let mut left_node = self.children.remove(left_index);

        let taken_item = self.items.remove(left_index);
        left_node.items.push(taken_item);

        let right_node = self.children[left_index].as_mut();

        left_node.items.append(&mut right_node.items);

        if !right_node.is_leaf() {
            left_node.children.append(&mut right_node.children);
        }

        self.children[left_index] = left_node;
    }

    fn insert(&mut self, item: Item<K, V>) {
        let mut index = self.items.len();

        while index > 0 && item < *self.items[index - 1] {
            index -= 1;
        }

        if self.is_leaf() {
            self.items.insert(index, Box::new(item));
            return;
        }

        if self.children[index].as_ref().is_full() {
            self.split_child(index);

            if item > *self.items[index] {
                index += 1;
            }
        }

        self.children[index].insert(item);
    }

    fn split_child(&mut self, index: usize) {
        let middle_position = self.max_size / 2;
        let mut new_node = Node::new(self.max_size);

        let child = self.children[index].as_mut();

        new_node
            .items
            .extend(child.items.split_off(middle_position + 1));

        self.items
            .insert(index, child.items.remove(middle_position));

        if !child.is_leaf() {
            new_node
                .children
                .extend(child.children.split_off(middle_position + 1));
        }

        self.children.insert(index + 1, Box::new(new_node));
    }
}

impl<K: Ord + Debug, V: Debug> Item<K, V> {
    fn new(key: K, value: V) -> Self {
        Self { key, value }
    }

    fn cmp_key(&self, key: &K) -> Ordering {
        self.key.cmp(key)
    }
}

impl<K: Ord + Debug, V: Debug> PartialEq for Item<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<K: Ord + Debug, V: Debug> Eq for Item<K, V> {}

impl<K: Ord + Debug, V: Debug> PartialOrd for Item<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<K: Ord + Debug, V: Debug> Ord for Item<K, V> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key.cmp(&other.key)
    }
}

impl<K: Ord + Debug + Display, V: Debug> Display for BTree<K, V> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "BTree: \n{}", self.root)
    }
}

impl<K: Ord + Debug + Display, V: Debug> Display for Node<K, V> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let mut result = String::new();
        self.display(&mut result, 0);
        write!(f, "{}", result)
    }
}

impl<K: Ord + Debug + Display, V: Debug> Node<K, V> {
    fn display(&self, result: &mut String, depth: usize) {
        result.push_str(&format!("{:indent$}Node: ", "", indent = depth * 2));
        for item in &self.items {
            result.push_str(&format!("{} ", item.key));
        }
        result.push_str("\n");

        for child in &self.children {
            child.display(result, depth + 1);
        }
    }
}
