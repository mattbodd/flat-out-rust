mod fcqueue;
use fcqueue::FCQueue;

fn main() {
    let mut new_queue: FCQueue = FCQueue::new();

    new_queue.enqueue(3);
    new_queue.enqueue(4);
    new_queue.enqueue(5);

    println!("---");
    new_queue.queue[0].get();
    println!("---");
    new_queue.queue[1].get();
    println!("---");
    new_queue.queue[2].get();

    println!("---");
    println!("{}", new_queue.dequeue());
}
