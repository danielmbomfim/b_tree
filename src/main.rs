use std::fmt::{Debug, Display, Formatter};

fn main() {
    let mut tree = BTree::new(4);
    tree.insert(Item::new(10));
    tree.insert(Item::new(20));
    tree.insert(Item::new(5));
    tree.insert(Item::new(4));
    tree.insert(Item::new(8));
    tree.insert(Item::new(9));
    tree.insert(Item::new(6));
    tree.insert(Item::new(12));
    tree.insert(Item::new(30));
    tree.insert(Item::new(25));
    tree.insert(Item::new(35));
    tree.insert(Item::new(36));
    tree.insert(Item::new(21));

    println!("{}", tree);

    tree.remove(Item::new(35));
    tree.remove(Item::new(36));

    println!("{}", tree);
}

#[derive(Debug)]
struct BTree<T: Ord + Debug> {
    root: Box<Node<T>>,
    max_size: usize,
}

#[derive(Debug)]
struct Node<T: Ord + Debug> {
    items: Vec<Box<Item<T>>>,
    children: Vec<Box<Node<T>>>,
    max_size: usize,
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
struct Item<T: Ord + Debug> {
    value: T,
}

impl<T: Ord + Debug> BTree<T> {
    fn new(capacity: usize) -> Self {
        assert!(capacity > 1);

        Self {
            root: Box::new(Node::new(capacity)),
            max_size: capacity,
        }
    }

    fn insert(&mut self, item: Item<T>) {
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

    fn remove(&mut self, item: Item<T>) {
        self.root.remove(item);
    }
}

impl<T: Ord + Debug> Node<T> {
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

    fn remove(&mut self, item: Item<T>) {
        let mut index = self.items.len() - 1;

        while index > 0 && item < *self.items[index] {
            index -= 1;
        }

        if *self.items[index] == item {
            self.items.remove(index);
            return;
        }

        let (child_index, child) = if *self.items[index] > item {
            (index, self.children[index].as_mut())
        } else {
            (index + 1, self.children[index + 1].as_mut())
        };

        child.remove(item);

        if child.is_leaf() {
            self.rebalance_leaf_after_removal(child_index);
        }
    }

    fn rebalance_leaf_after_removal(&mut self, index: usize) {
        if self.children[index].items.len() >= self.max_size / 2 {
            return;
        }

        let min_size = self.max_size / 2;

        if index != 0 {
            if self.children[index - 1].items.len() > min_size {
                self.take_item_from(index, index - 1);
                return;
            }

            self.merge_child(index, index - 1);
            return;
        }

        if self.children[index + 1].items.len() > min_size {
            self.take_item_from(index, index + 1);
            return;
        }

        self.merge_child(index, index + 1);
    }

    fn take_item_from(&mut self, dest: usize, src: usize) {
        let current_divider = if src < dest {
            self.items[src].as_mut()
        } else {
            self.items[dest].as_mut()
        };

        let item_taken = if src < dest {
            let index = self.children[src].items.len() - 1;

            self.children[src].items.remove(index)
        } else {
            self.children[src].items.remove(0)
        };

        let new_item = Box::new(std::mem::replace(current_divider, *item_taken));

        if src < dest {
            self.children[dest].items.insert(0, new_item);
        } else {
            self.children[dest].items.push(new_item);
        }
    }

    fn merge_child(&mut self, dest: usize, src: usize) {
        let mut temp_vec: Vec<Box<Item<T>>> = self.children[src].items.drain(..).collect();
        let item = if src < dest {
            self.items.remove(src)
        } else {
            self.items.remove(dest)
        };

        let node = self.children[dest].as_mut();

        temp_vec.push(item);
        node.items.splice(0..0, temp_vec);

        self.children.remove(src);
    }

    fn insert(&mut self, item: Item<T>) {
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

impl<T: Ord + Debug> Item<T> {
    fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T: Ord + Debug + Display> Display for BTree<T> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "BTree: \n{}", self.root)
    }
}

impl<T: Ord + Debug + Display> Display for Node<T> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let mut result = String::new();
        self.display(&mut result, 0);
        write!(f, "{}", result)
    }
}

impl<T: Ord + Debug + Display> Node<T> {
    fn display(&self, result: &mut String, depth: usize) {
        result.push_str(&format!("{:indent$}Node: ", "", indent = depth * 2));
        for item in &self.items {
            result.push_str(&format!("{} ", item.value));
        }
        result.push_str("\n");

        for child in &self.children {
            child.display(result, depth + 1);
        }
    }
}
