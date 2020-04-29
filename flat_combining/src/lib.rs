use crossbeam_utils::thread;
use std::sync::Arc;
mod fcqueue;
use fcqueue::FCQueue;

static SOME_ELEMS: i32 = 10;
static MANY_ELEMS: i32 = 100_000;
static NUM_THREADS: i32 = 4;
static SOME_ELEMS_PER_THREAD: i32 = SOME_ELEMS / NUM_THREADS;
static MANY_ELEMS_PER_THREAD: i32 = MANY_ELEMS / NUM_THREADS;

/* DEBUGGING START */
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(PartialEq)]
enum ProfilerOutput {
    stdout,
    fileout,
    all,
}

struct Profiler {
    start_time: u128,
    end_time: u128,
    elapsed_time: u128,
    target_thread: Option<i32>,
    output_type: ProfilerOutput,
    func_name: String,
}

// A profiler keeps track of which threads it is monitoring as well as
impl Profiler {
    fn new(target_thread: Option<i32>, output_type: ProfilerOutput, func_name: String) -> Profiler {
        Profiler {
            start_time: 0,
            end_time: 0,
            elapsed_time: 0,
            target_thread,
            output_type,
            func_name,
        }
    }

    fn log(&self, state: &str, arriving_thread: i32) {
        match self.target_thread {
            Some(tid) => {
                if arriving_thread == tid {
                    if self.output_type == ProfilerOutput::stdout {
                        println!(
                            "Thread {} {} {}: {}",
                            arriving_thread,
                            state,
                            self.func_name,
                            SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_nanos()
                        );
                    }
                }
            }
            None => {
                if self.output_type == ProfilerOutput::stdout {
                    println!(
                        "Thread {} {} {}: {}",
                        arriving_thread,
                        state,
                        self.func_name,
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_nanos()
                    );
                }
            }
        }
    }

    pub fn start(&mut self, arriving_thread: i32) {
        //self.log("starting", arriving_thread);
        self.start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
    }

    pub fn end(&mut self, arriving_thread: i32) {
        self.end_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        self.elapsed_time = self.end_time - self.start_time;
        println!(
            "{},{}:{}",
            arriving_thread, self.func_name, self.elapsed_time
        );
        //self.log("ending", arriving_thread);
    }
}
/* DEBUGGING END */

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

        let mut profiler = Profiler::new(None, ProfilerOutput::stdout, "total".to_string());
        profiler.start(0);

        // Enqueue `num_elems` elements
        for elem in 0..MANY_ELEMS {
            queue.enqueue(elem, 0);
        }

        profiler.end(0);
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

        let mut item_count = 0;
        let mut expected = 0;
        for elem in 0..MANY_ELEMS {
            if elem > 0 && elem % 3 == 0 {
                assert_eq!(expected, queue.dequeue(0));
                expected += 1;
            } else {
                queue.enqueue(elem, 0);
            }
        }

        while expected < MANY_ELEMS {
            assert_eq!(expected, queue.dequeue(0));
            expected += 1;
        }
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
                    let mut profiler =
                        Profiler::new(None, ProfilerOutput::stdout, "total".to_string());
                    profiler.start(i);

                    for elem in (i * MANY_ELEMS_PER_THREAD)..((i + 1) * MANY_ELEMS_PER_THREAD) {
                        cloned_shared_queue.enqueue(elem, i);
                    }

                    profiler.end(i);
                });
            }
        })
        .unwrap();
    }

    #[test]
    fn stress_flush() {
        let queue = FCQueue::new();

        thread::scope(|s| {
            let shared_queue = Arc::new(&queue);
            for i in 0..NUM_THREADS {
                let cloned_shared_queue = Arc::clone(&shared_queue);
                s.spawn(move |_| {
                    for elem in (i * MANY_ELEMS_PER_THREAD)..((i + 1) * MANY_ELEMS_PER_THREAD) {
                        cloned_shared_queue.enqueue(elem, i);
                        cloned_shared_queue.dequeue(i);
                    }
                });
            }
        })
        .unwrap();
    }
}
