mod datastructures;
use datastructures::{queue, stack};

fn main() {
    println!("Hey from `main.rs`! Let me introduce you to a couple `datastructures`:");
    stack::hey();
    queue::hey();
}
