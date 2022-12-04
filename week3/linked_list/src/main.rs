use linked_list::LinkedList;
pub mod linked_list;

fn main() {
    let mut list: LinkedList<f64> = LinkedList::new();
    assert!(list.is_empty());
    assert_eq!(list.get_size(), 0);
    let mut k = 1.0 ;
    for i in 1..12 {
        k = k + 1.0;
        list.push_front(k);
    }
    println!("{}", list);
    println!("list size: {}", list.get_size());
    println!("top element: {}", list.pop_front().unwrap());
    println!("{}", list);
    println!("size: {}", list.get_size());
    println!("{}", list.to_string()); // ToString impl for anything impl Display

    // If you implement iterator trait:
    for val in &list {
       println!("{}", val);
    }
    println!("computenorm = {}", list.computenorm());
}
