mod datastructures;
use datastructures::{queue, stack};

mod fcqueue;
use fcqueue::{FCQueue};

fn main() {


    let mut new_queue:FCQueue =FCQueue::new();

    new_queue.enqueue(3);
    new_queue.queue[0].get();


}
