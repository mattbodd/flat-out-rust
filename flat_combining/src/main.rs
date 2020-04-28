mod fcqueue;
use fcqueue::FCQueue;
//use std::thread;
use crossbeam_utils::thread::scope;




fn main() {

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

    println!("---");



	const COUNT: usize  = 50;
    const THREADS: usize = 4;

    let q :FCQueue = FCQueue::new();
    q.enqueue(3);
    println!("{}", q.dequeue());
		

    
	

	//println!("{}", new_queue.dequeue());
	
}
