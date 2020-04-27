mod fcqueue;
use fcqueue::FCQueue;

fn main() {
    let mut new_queue: FCQueue = FCQueue::new();

    new_queue.enqueue(3);
    new_queue.enqueue(4);
    new_queue.enqueue(5);
    println!("---");

    //println!("{}",new_queue.dequeue());



   

    
    //new_queue.queue[1].get();
    // println!("---");
    // new_queue.queue[2].get();
    // new_queue.queue[3].get();
    // new_queue.queue[4].get();
    // new_queue.queue[5].get();
}
