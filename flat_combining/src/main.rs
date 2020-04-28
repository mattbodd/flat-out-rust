mod fcqueue;
use crossbeam_utils::thread;
use fcqueue::FCQueue;
use std::sync::Arc;

fn main() {
    let queue = FCQueue::new();

    let ten_millis = std::time::Duration::from_millis(1000);

    thread::scope(|s| {
        let shared_queue = Arc::new(&queue);
        for i in 1..4 {
            let cloned_shared_queue = Arc::clone(&shared_queue);
            s.spawn(move |_| {
                if i == 1 {
                    std::thread::sleep(ten_millis);
                    std::thread::sleep(ten_millis);
                    std::thread::sleep(ten_millis);
                    cloned_shared_queue.enqueue(i);
                } else {
                    cloned_shared_queue.enqueue(i);
                }
                if i == 2 {
                    std::thread::sleep(ten_millis);
                    println!("dequeued: {}", cloned_shared_queue.dequeue());
                }
            });
        }
    })
    .unwrap();

    // new_queue.enqueue(3);
    // new_queue.enqueue(4);
    // new_queue.enqueue(5);

    // println!("---");
    // new_queue.queue[0].get();
    // println!("---");
    // new_queue.queue[1].get();
    // println!("---");
    // new_queue.queue[2].get();

    // println!("---");
    // println!("{}", new_queue.dequeue());
    // println!("{}", new_queue.dequeue());
    // println!("{}", new_queue.dequeue());
}
