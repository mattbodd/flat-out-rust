mod fcqueue;
pub mod seq_list;
use crossbeam_utils::thread;
use fcqueue::FCQueue;
use std::sync::Arc;

pub fn fc_test() {
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

mod seq {
    use super::*;

    static SOME_ELEMS: i32 = 10;
    static MANY_ELEMS: i32 = 1_000_000;

    #[test]
    fn enqueue() {
        let queue = FCQueue::new();

        // Enqueue `num_elems` elements
        for elem in 0..SOME_ELEMS {
            queue.enqueue(elem);
        }
    }

    #[test]
    fn stress_enqueue() {
        let queue = FCQueue::new();

        // Enqueue `num_elems` elements
        for elem in 0..MANY_ELEMS {
            queue.enqueue(elem);
        }
    }

    #[test]
    fn flush() {
        let queue = FCQueue::new();

        // Enqueue `num_elems` elements
        for elem in 0..SOME_ELEMS {
            queue.enqueue(elem);
        }

        // Dequeue `num_elem` elements
        for elem in 0..SOME_ELEMS {
            queue.dequeue();
        }
    }

    #[test]
    fn stress_flush() {
        let queue = FCQueue::new();

        // Enqueue `num_elems` elements
        for elem in 0..MANY_ELEMS {
            queue.enqueue(elem);
        }

        // Dequeue `num_elem` elements
        for elem in 0..MANY_ELEMS {
            queue.dequeue();
        }
    }

    #[test]
    fn checked_flush() {
        let queue = FCQueue::new();

        // Enqueue `num_elems` elements
        for elem in 0..SOME_ELEMS {
            queue.enqueue(elem);
        }

        // Dequeue `num_elem` elements
        for elem in 0..SOME_ELEMS {
            assert_eq!(elem, queue.dequeue());
        }
    }

    #[test]
    fn stress_checked_flush() {
        let queue = FCQueue::new();

        // Enqueue `num_elems` elements
        for elem in 0..MANY_ELEMS {
            queue.enqueue(elem);
        }

        // Dequeue `num_elem` elements
        for elem in 0..MANY_ELEMS {
            assert_eq!(elem, queue.dequeue());
        }
    }

    #[test]
    fn varried_flush() {
        let queue = FCQueue::new();
    }

    #[test]
    fn stress_varried_flush() {
        let queue = FCQueue::new();
    }

    #[test]
    #[should_panic]
    fn over_reach() {
        let queue = FCQueue::new();

        queue.dequeue();
    }

    #[test]
    #[should_panic]
    fn stress_over_read() {
        let queue = FCQueue::new();

        // Enqueue `num_elems` elements
        for elem in 0..MANY_ELEMS {
            queue.enqueue(elem);
        }

        // Dequeue `num_elem` elements
        for elem in 0..MANY_ELEMS {
            assert_eq!(elem, queue.dequeue());
        }

        // Overreaching!
        queue.dequeue();
    }
}

mod par {
    use super::*;

    #[test]
    fn enqueue() {}

    #[test]
    fn dequeue() {}
}
