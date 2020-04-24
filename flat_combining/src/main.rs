mod datastructures;
use datastructures::{queue, stack};

mod fcqueue;
use fcqueue;

fn main() {
    println!("Hey from `main.rs`! Let me introduce you to a couple `datastructures`:");
    stack::hey();
    queue::hey();
}
