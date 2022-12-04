use std::fmt::{self, Display};
use std::option::Option;

pub struct LinkedList<T> {
    head: Option<Box<Node<T>>>,
    size: usize,
}

struct Node<T> {
    value: T,
    next: Option<Box<Node<T>>>,
}

impl<T> Node<T> {   
    pub fn new(value: T, next: Option<Box<Node<T>>>) -> Node<T> {
        Node {value: value, next: next}
    }
}

impl<T: Clone> Clone for Node<T> {
    fn clone(&self) -> Self {
        Node { value: self.value.clone(), next: self.next.clone() }
    }
}

impl<T: PartialEq> PartialEq for Node<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.value == other.value && self.next == other.next {
            return true;
        }

        return false;
    }
}

impl<T> LinkedList<T>{
    pub fn new() -> LinkedList<T> {
        LinkedList {head: None, size: 0}
    }
    
    pub fn get_size(&self) -> usize {
        self.size
    }
    
    pub fn is_empty(&self) -> bool {
        self.get_size() == 0
    }
    
    pub fn push_front(&mut self, value: T) {
        let new_node: Box<Node<T>> = Box::new(Node::new(value, self.head.take()));
        self.head = Some(new_node);
        self.size += 1;
    }
    
    pub fn pop_front(&mut self) -> Option<T> {
        let node: Box<Node<T>> = self.head.take()?;
        self.head = node.next;
        self.size -= 1;
        Some(node.value)
    }
    
}

impl<T: Clone> Clone for LinkedList<T> {
    fn clone(&self) -> LinkedList<T> {
        // 注意 clone 与 copy 的区别，clone 只是获得了头结点的多个引用， 而copy 重新复制了一份
        LinkedList { head: self.head.clone(), size: self.size }
    }
}


impl<T: PartialEq> PartialEq for LinkedList<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.size == other.size && self.head == other.head {
            return true;
        }

        return false;
    }
}

impl<T> fmt::Display for LinkedList<T> 
    where T: Display {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut current: &Option<Box<Node<T>>> = &self.head;
        let mut result = String::new();
        loop {
            match current {
                Some(node) => {
                    result = format!("{} {}", result, node.value);
                    current = &node.next;
                },
                None => break,
            }
        }
        write!(f, "{}", result)
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        let mut current = self.head.take();
        while let Some(mut node) = current {
            current = node.next.take();
        }
    }
}

pub struct LinkedListIter<'a, T> {
    current: &'a Option<Box<Node<T>>>,
}

impl<T> Iterator for LinkedListIter<'_, T>  where T: Clone{
    type Item = T;
    fn next(&mut self) -> Option<T> {
        match self.current {
            Some(node) => {
                self.current = &node.next;
                Some(node.value.clone())
            },
            None => None
        }
    }
}

impl<'a, T> IntoIterator for &'a LinkedList<T> where T: Clone{
    type Item = T;
    type IntoIter = LinkedListIter<'a, T>;
    fn into_iter(self) -> LinkedListIter<'a, T> {
        LinkedListIter {current: &self.head}
    }
}

pub trait ComputeNorm {
    fn computenorm(&self) -> f64{
        0.0
    }
}
impl LinkedList<f64> {
    pub fn computenorm(&self) -> f64 {
        let mut result = 0.0;
        
        for i in self.into_iter() {
            result += i;
        }
        result
    }

}


