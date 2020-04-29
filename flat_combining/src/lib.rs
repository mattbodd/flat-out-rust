pub mod seq_list;
mod fcqueue;
use crossbeam_utils::thread;
use fcqueue::FCQueue;
use std::sync::Arc;






pub fn fc_test(){
	let queue = FCQueue::new();

    let ten_millis = std::time::Duration::from_millis(100);

    thread::scope(|s| {
        let shared_queue = Arc::new(&queue);
        for i in 0..4 {
            let cloned_shared_queue = Arc::clone(&shared_queue);
            s.spawn(move |_| {
            	cloned_shared_queue.enqueue(i);
                
                    //println!("dequeued: {}", cloned_shared_queue.dequeue());
                
            });
        }
    })
    .unwrap();

}



mod tests{
	use super::*;

	#[test]
	fn enqueue(){
		let queue = FCQueue::new();
		queue.enqueue(1);
		queue.enqueue(2);
		queue.enqueue(3);
		queue.enqueue(4);


	}

	#[test]
	fn dequeue(){
		let queue = FCQueue::new();
		queue.enqueue(1);
		queue.enqueue(2);
		queue.enqueue(3);
		queue.enqueue(4);
		println!("dequeued: {}", queue.dequeue());
		println!("dequeued: {}", queue.dequeue());
		println!("dequeued: {}", queue.dequeue());
		println!("dequeued: {}", queue.dequeue());
		
	}




}