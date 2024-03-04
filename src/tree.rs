use std::fmt;

#[derive(Debug, Clone)]
pub struct Node<K: Ord, D> {
  left: Option<Box<Node<K, D>>>,
  right: Option<Box<Node<K, D>>>,
  key: K,
  data: D,
  height: usize,
}
impl<K: Ord, D> Node<K, D> {
  pub fn new(key: K, data: D) -> Self {
    Self {
      left: None,
      right: None,
      key,
      data,
      height: 1,
    }
  }
  pub fn height(node: &Option<Box<Self>>) -> usize {
    match node {
      Some(ref n) => n.height,
      None => 0,
    }
  }
  pub fn balance(&self) -> isize {
    Self::height(&self.right) as isize - Self::height(&self.left) as isize
  }
  pub fn rotate_left(mut n: Box<Node<K, D>>) -> Box<Node<K, D>> {
    let mut r = n.right.take().expect("broken tree");
    n.right = r.left.take();
    n.height = Self::height(&n.left).max(Self::height(&n.right)) + 1;
    r.left = Some(n);
    r.height = Self::height(&r.left).max(Self::height(&r.right)) + 1;
    r
  }
  pub fn rotate_right(mut n: Box<Node<K, D>>) -> Box<Node<K, D>> {
    let mut l = n.left.take().expect("broken tree");
    n.left = l.right.take();
    n.height = Self::height(&n.left).max(Self::height(&n.right)) + 1;
    l.right = Some(n);
    l.height = Self::height(&l.left).max(Self::height(&l.right)) + 1;
    l
  }
  pub fn rotate_right_left(mut n: Box<Node<K, D>>) -> Box<Node<K, D>> {
    n.right = Some(Self::rotate_right(n.right.unwrap()));
    let mut root = Self::rotate_left(n);
    root.height = Self::height(&root.left).max(Self::height(&root.right)) + 1;
    root
  }
  pub fn rotate_left_right(mut n: Box<Node<K, D>>) -> Box<Node<K, D>> {
    n.left = Some(Self::rotate_left(n.left.unwrap()));
    let mut root = Self::rotate_right(n);
    root.height = Self::height(&root.left).max(Self::height(&root.right)) + 1;
    root
  }
  pub fn rebalance(n: Box<Node<K, D>>) -> Box<Node<K, D>> {
    if n.balance() < -1 && n.left.as_ref().is_some_and(|x| (*x).balance() == -1) {
      Self::rotate_right(n)
    } else if n.balance() > 1 && n.right.as_ref().is_some_and(|x| (*x).balance() == 1) {
      Self::rotate_left(n)
    } else if n.balance() < -1 && n.left.as_ref().is_some_and(|x| (*x).balance() == 1) {
      Self::rotate_left_right(n)
    } else if n.balance() > 1 && n.right.as_ref().is_some_and(|x| (*x).balance() == -1) {
      Self::rotate_right_left(n)
    } else {
      n
    }
  }
  pub fn insert(mut n: Box<Node<K, D>>, key: K, data: D) -> Box<Node<K, D>> {
    match key.cmp(&n.key) {
      std::cmp::Ordering::Less => match n.left {
        None => n.left = Some(Box::new(Node::new(key, data))),
        Some(node) => {
          n.left = Some(Self::insert(node, key, data));
        }
      },
      std::cmp::Ordering::Greater => match n.right {
        None => n.right = Some(Box::new(Node::new(key, data))),
        Some(node) => {
          n.right = Some(Self::insert(node, key, data));
        }
      },
      std::cmp::Ordering::Equal => {
        panic!("unimplemented")
      }
    };
    n.height = Self::height(&n.left).max(Self::height(&n.right)) + 1;
    Self::rebalance(n)
  }
  pub fn get<'a>(n: &'a Box<Node<K, D>>, key: &'a K) -> Option<&'a D> {
    match key.cmp(&n.key) {
      std::cmp::Ordering::Less => {
        let Some(l) = n.left.as_ref() else {
          return None;
        };
        Self::get(l, key)
      }
      std::cmp::Ordering::Greater => {
        let Some(r) = n.right.as_ref() else {
          return None;
        };
        Self::get(r, key)
      }
      std::cmp::Ordering::Equal => Some(&n.data),
    }
  }
}
#[allow(unused)]
fn print_node<K: Ord + fmt::Display, D: fmt::Display>(node: &Node<K, D>, level: usize) -> String {
  let mut result = String::new();
  if let Some(right) = &node.right {
    result.push_str(&print_node(&right, level + 1));
  }
  let s = format!("{}{}: {}\n", "\t".repeat(level), node.key, node.data);
  result.push_str(&s);
  if let Some(left) = &node.left {
    result.push_str(&print_node(&left, level + 1));
  }
  result
}
pub struct Tree<K: Ord, D> {
  root: Option<Box<Node<K, D>>>,
}
impl<K: Ord, D> Tree<K, D> {
  pub fn new() -> Self {
    Self { root: None }
  }
  pub fn insert(&mut self, key: K, data: D) {
    self.root = Some(Node::rebalance(match self.root.take() {
      Some(root) => Node::insert(root, key, data),
      None => Box::new(Node::new(key, data)),
    }));
  }
  pub fn get<'a>(&'a self, key: &'a K) -> Option<&D> {
    let Some(root) = self.root.as_ref() else {
      return None;
    };
    Node::get(root, key)
  }
  pub fn height(&self) -> usize {
    Node::height(&self.root)
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn tree_insert() {
    let mut tree = Tree::new();
    tree.insert("username", "John Smith");
    tree.insert("email", "johnsmith@email.com");
    tree.insert("password", "password123");
    assert!(tree.root.is_some());
    assert_eq!(tree.height(), 2);
    assert_eq!(tree.get(&"email"), Some(&"johnsmith@email.com"));
  }
}
