use crossbeam_utils::thread;
use std::sync::Arc;
mod fcqueue;
use fcqueue::FCQueue;

static SOME_ELEMS: i32 = 10;
static MANY_ELEMS: i32 = 100_000;
static NUM_THREADS: i32 = 4;
static SOME_ELEMS_PER_THREAD: i32 = SOME_ELEMS / NUM_THREADS;
static MANY_ELEMS_PER_THREAD: i32 = MANY_ELEMS / NUM_THREADS;

pub fn fc_test() {
    let queue = FCQueue::new();

    thread::scope(|s| {
        let shared_queue = Arc::new(&queue);
        for i in 0..NUM_THREADS {
            let cloned_shared_queue = Arc::clone(&shared_queue);
            s.spawn(move |_| {
                for elem in 0..MANY_ELEMS_PER_THREAD {
                    cloned_shared_queue.enqueue(elem, i);
                }
            });
        }
    })
    .unwrap();
}

pub mod seq {
    use super::*;

    #[test]
    fn enqueue() {
        let queue = FCQueue::new();

        // Enqueue `num_elems` elements
        for elem in 0..SOME_ELEMS {
            queue.enqueue(elem, 0);
        }
    }

    #[test]
    pub fn stress_enqueue() {
        let queue = FCQueue::new();

        // Enqueue `num_elems` elements
        for elem in 0..MANY_ELEMS {
            queue.enqueue(elem, 0);
        }
    }

    #[test]
    fn flush() {
        let queue = FCQueue::new();

        // Enqueue `num_elems` elements
        for elem in 0..SOME_ELEMS {
            queue.enqueue(elem, 0);
        }

        // Dequeue `num_elem` elements
        for elem in 0..SOME_ELEMS {
            queue.dequeue(0);
        }
    }

    #[test]
    fn stress_flush() {
        let queue = FCQueue::new();

        // Enqueue `num_elems` elements
        for elem in 0..MANY_ELEMS {
            queue.enqueue(elem, 0);
        }

        // Dequeue `num_elem` elements
        for elem in 0..MANY_ELEMS {
            queue.dequeue(0);
        }
    }

    #[test]
    fn checked_flush() {
        let queue = FCQueue::new();

        // Enqueue `num_elems` elements
        for elem in 0..SOME_ELEMS {
            queue.enqueue(elem, 0);
        }

        // Dequeue `num_elem` elements
        for elem in 0..SOME_ELEMS {
            assert_eq!(elem, queue.dequeue(0));
        }
    }

    #[test]
    fn stress_checked_flush() {
        let queue = FCQueue::new();

        // Enqueue `num_elems` elements
        for elem in 0..MANY_ELEMS {
            queue.enqueue(elem, 0);
        }

        // Dequeue `num_elem` elements
        for elem in 0..MANY_ELEMS {
            assert_eq!(elem, queue.dequeue(0));
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

        queue.dequeue(0);
    }

    #[test]
    #[should_panic]
    fn stress_over_read() {
        let queue = FCQueue::new();

        // Enqueue `num_elems` elements
        for elem in 0..MANY_ELEMS {
            queue.enqueue(elem, 0);
        }

        // Dequeue `num_elem` elements
        for elem in 0..MANY_ELEMS {
            assert_eq!(elem, queue.dequeue(0));
        }

        // Overreaching!
        queue.dequeue(0);
    }
}

mod par {
    use super::*;

    #[test]
    fn enqueue() {
        let queue = FCQueue::new();

        thread::scope(|s| {
            let shared_queue = Arc::new(&queue);
            for i in 0..NUM_THREADS {
                let cloned_shared_queue = Arc::clone(&shared_queue);
                s.spawn(move |_| {
                    for elem in (i * SOME_ELEMS_PER_THREAD)..((i + 1) * SOME_ELEMS_PER_THREAD) {
                        cloned_shared_queue.enqueue(elem, i);
                    }
                });
            }
        })
        .unwrap();
    }

    #[test]
    fn stress_enqueue() {
        let queue = FCQueue::new();

        thread::scope(|s| {
            let shared_queue = Arc::new(&queue);
            for i in 0..NUM_THREADS {
                let cloned_shared_queue = Arc::clone(&shared_queue);
                s.spawn(move |_| {
                    for elem in (i * MANY_ELEMS_PER_THREAD)..((i + 1) * MANY_ELEMS_PER_THREAD) {
                        cloned_shared_queue.enqueue(elem, i);
                    }
                });
            }
        })
        .unwrap();
    }

    #[test]
    fn stress_flush() {}
}
