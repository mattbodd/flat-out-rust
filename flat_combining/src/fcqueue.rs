use crossbeam_queue::{ArrayQueue, PushError};
use crossbeam_utils::{atomic::AtomicCell, Backoff};
use std::collections::VecDeque;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::{Arc, Mutex};

// Global namespace
const MAX_THREADS: usize = 512;
const COMBINING_NODE_TIMEOUT: u64 = 10000;
const COMBINING_NODE_TIMEOUT_CHECK_FREQUENCY: u64 = 100;
const MAX_COMBINING_ROUNDS: u64 = 32;
const NUM_ROUNDS_IS_LINKED_CHECK_FREQUENCY: u64 = 100;

/* DEBUGGING START */
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(PartialEq)]
enum ProfilerOutput {
    stdout,
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

struct CombiningNode {
    is_linked: AtomicBool,
    last_request_timestamp: AtomicU64,
    is_request_valid: AtomicBool,
    is_consumer: AtomicBool,
    item: AtomicCell<Option<i32>>,
}

impl CombiningNode {
    fn new() -> CombiningNode {
        // TODO: How to initiailize additional fields
        CombiningNode {
            is_linked: AtomicBool::new(false),
            last_request_timestamp: AtomicU64::new(0),
            is_request_valid: AtomicBool::new(false),
            is_consumer: AtomicBool::new(false),
            item: AtomicCell::new(None),
        }
    }
}

#[derive(Clone)]
pub struct QueueFatNode {
    items: Vec<i32>,
    pub items_left: usize,
}

impl QueueFatNode {
    fn new() -> QueueFatNode {
        QueueFatNode {
            // TOOD: Determine what size to initialize by default
            items: Vec::new(),
            items_left: 0,
        }
    }
    pub fn get(&self) {
        for val in &self.items {
            println!("{}", val);
        }
    }
}
pub fn get_fat_queue(queue: &VecDeque<QueueFatNode>) {
    for val in queue {
        val.get();
    }
}

pub struct FCQueue {
    fc_lock: AtomicUsize,
    combined_pushed_items: ArrayQueue<i32>,
    current_timestamp: AtomicU64,
    curr_comb_held: AtomicBool,
    comb_list_head: Mutex<VecDeque<Arc<CombiningNode>>>,
    queue: Mutex<VecDeque<QueueFatNode>>,
}

impl FCQueue {
    pub fn new() -> FCQueue {
        FCQueue {
            fc_lock: AtomicUsize::new(0),
            combined_pushed_items: ArrayQueue::<i32>::new(MAX_THREADS),
            current_timestamp: AtomicU64::new(0),
            curr_comb_held: AtomicBool::new(false),
            comb_list_head: Mutex::new(VecDeque::new()),
            queue: Mutex::new(VecDeque::new()),
        }
    }

    #[allow(non_snake_case)]
    fn doFlatCombining(&self, tid: i32) {
        /* Debugging
        let mut do_flat_profiler: Profiler =
            Profiler::new(None, ProfilerOutput::stdout, "doFlatCombining".to_string());
        do_flat_profiler.start(tid);
        */

        let mut combining_round: u64 = 0;
        let mut num_pushed_items: usize;
        let mut curr_comb_node: VecDeque<Arc<CombiningNode>>;
        /*
        {
            curr_comb_node = VecDeque::new(); //self.comb_list_head.lock().unwrap().clone();
        }
         */

        self.current_timestamp.fetch_add(1, Ordering::Relaxed);
        let local_current_timestamp: u64 = self.current_timestamp.load(Ordering::Relaxed);

        let check_timestamps: bool =
            local_current_timestamp % COMBINING_NODE_TIMEOUT_CHECK_FREQUENCY == 0;

        let mut have_work: bool;

        loop {
            num_pushed_items = 0;

            /* Debugging
            let mut orig_comb_profiler: Profiler = Profiler::new(
                None,
                ProfilerOutput::stdout,
                "orig_comb_list_head".to_string(),
            );
            orig_comb_profiler.start(tid);
            */

            //orig_comb_profiler.end(tid);

            /* Debugging
            let mut curr_comb_profiler: Profiler =
                Profiler::new(None, ProfilerOutput::stdout, "curr_comb_node".to_string());
            curr_comb_profiler.start(tid);
            */
            {
                //let mut unlocked = self.comb_list_head.lock().unwrap();
                //curr_comb_node = unlocked.clone();
                //unlocked.clear();
                //self.comb_list_head.lock().unwrap().clear();
                curr_comb_node = self.comb_list_head.lock().unwrap().drain(..).collect();
            }

            let mut orig_comb_list_head: Option<Arc<CombiningNode>> = None;

            if !curr_comb_node.is_empty()
                && !curr_comb_node
                    .front()
                    .unwrap()
                    .is_request_valid
                    .load(Ordering::Relaxed)
            {
                orig_comb_list_head = match curr_comb_node.front() {
                    Some(head) => Some(Arc::clone(head)),
                    None => None,
                };
            }

            //curr_comb_profiler.end(tid);

            have_work = false;

            while !curr_comb_node.is_empty() {
                if !curr_comb_node
                    .front()
                    .unwrap()
                    .is_request_valid
                    .load(Ordering::Relaxed)
                {
                    // Unsure if `as_ref` gives us a reference that can actually
                    // be compared with `curr_comb_node.front().unwrap()`
                    if check_timestamps
                        && (!Arc::ptr_eq(
                            &curr_comb_node.front().unwrap(),
                            &orig_comb_list_head.as_ref().unwrap(),
                        ))
                        && ((local_current_timestamp
                            - curr_comb_node
                                .front()
                                .unwrap()
                                .last_request_timestamp
                                .load(Ordering::Relaxed))
                            > COMBINING_NODE_TIMEOUT)
                    {
                        println!("Reaching uncertain condition");
                        curr_comb_node
                            .front()
                            .unwrap()
                            .is_linked
                            .store(false, Ordering::Relaxed);
                    }

                    curr_comb_node.pop_front();
                    continue;
                }

                have_work = true;

                curr_comb_node
                    .front()
                    .unwrap()
                    .last_request_timestamp
                    .store(local_current_timestamp, Ordering::Relaxed);

                if curr_comb_node
                    .front()
                    .unwrap()
                    .is_consumer
                    .load(Ordering::Relaxed)
                {
                    let mut consumer_satisfied: bool = false;

                    while !self.queue.lock().unwrap().is_empty() && !consumer_satisfied {
                        if self.queue.lock().unwrap().front().unwrap().items_left == 0 {
                            self.queue.lock().unwrap().pop_front();
                        } else {
                            /* Debugging
                            let mut queue_profiler: Profiler =
                                Profiler::new(None, ProfilerOutput::stdout, "queue".to_string());
                            queue_profiler.start(tid);
                            */

                            self.queue.lock().unwrap().front_mut().unwrap().items_left -= 1;

                            //queue_profiler.end(tid);

                            curr_comb_node
                                .front()
                                .unwrap()
                                .item
                                .store(self.queue.lock().unwrap().front_mut().unwrap().items.pop());
                            consumer_satisfied = true;
                        }
                    }

                    if !consumer_satisfied && (num_pushed_items > 0) {
                        num_pushed_items -= 1;

                        /* Debugging
                        let mut curr_comb_add_profiler: Profiler = Profiler::new(
                            None,
                            ProfilerOutput::stdout,
                            "orig_comb_list_head".to_string(),
                        );
                        curr_comb_add_profiler.start(tid);
                        */

                        curr_comb_node
                            .front()
                            .unwrap()
                            .item
                            .store(Some(self.combined_pushed_items.pop().unwrap()));

                        //curr_comb_add_profiler.end(tid);

                        consumer_satisfied = true;
                    }

                    if !consumer_satisfied {
                        curr_comb_node.front().unwrap().item.store(None);
                    }
                } else {
                    /* Debugging
                    let mut combined_pushed_add_profiler: Profiler = Profiler::new(
                        None,
                        ProfilerOutput::stdout,
                        "combined_pushed_items_add".to_string(),
                    );
                    combined_pushed_add_profiler.start(tid);
                     */

                    // Old
                    self.combined_pushed_items
                        .push(curr_comb_node.front().unwrap().item.load().unwrap());

                    //combined_pushed_add_profiler.end(tid);

                    /*
                    self.combined_pushed_items
                        .lock()
                        .unwrap()
                        .push(curr_comb_node.front().unwrap().item.load().unwrap());
                     */

                    num_pushed_items += 1;
                }

                curr_comb_node
                    .front()
                    .unwrap()
                    .is_request_valid
                    .store(false, Ordering::Relaxed);

                curr_comb_node.pop_front();
            }

            if num_pushed_items > 0 {
                let mut new_node: QueueFatNode = QueueFatNode::new();

                // No more than MAX_THREADS items can be in combined_pushed_items
                // at a time
                assert!(num_pushed_items < MAX_THREADS);

                new_node.items_left = num_pushed_items;

                for item in 0..num_pushed_items {
                    new_node
                        .items
                        .push(self.combined_pushed_items.pop().unwrap());
                }

                self.queue.lock().unwrap().push_back(new_node);
            }

            combining_round += 1;
            if !have_work || combining_round >= MAX_COMBINING_ROUNDS {
                // Debugging
                //do_flat_profiler.end(tid);
                return;
            }
        }
    }

    fn link_in_combining(&self, cn: Arc<CombiningNode>, tid: i32) {
        /* Debugging
        let mut profiler: Profiler = Profiler::new(
            None,
            ProfilerOutput::stdout,
            "link_in_combining_lock".to_string(),
        );
         */

        // Block until we have access to the global `comb_list_head` at which point
        // we merge our thread local queue
        //profiler.start(tid);
        let backoff = Backoff::new();
        while self
            .curr_comb_held
            .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .is_err()
        {
            backoff.snooze();
        }

        let mut curr_comb_queue = self.comb_list_head.lock().unwrap();
        //profiler.end(tid);
        curr_comb_queue.push_front(cn);
        //  Mutex is unlocked at end of scope

        self.curr_comb_held.store(false, Ordering::Relaxed);
    }

    fn wait_until_fulfilled(&self, shared_comb_node: Arc<CombiningNode>, tid: i32) {
        let mut rounds = 0;

        //let shared_comb_node: Arc<CombiningNode> = Arc::new(comb_node);

        loop {
            if (rounds % NUM_ROUNDS_IS_LINKED_CHECK_FREQUENCY == 0)
                && !shared_comb_node.is_linked.load(Ordering::Relaxed)
            {
                shared_comb_node.is_linked.store(true, Ordering::Relaxed);
                self.link_in_combining(Arc::clone(&shared_comb_node), tid);
            }

            if self.fc_lock.load(Ordering::Relaxed) == 0 {
                let cae: Result<usize, usize> =
                    self.fc_lock
                        .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed);
                if cae.is_ok() {
                    self.doFlatCombining(tid);
                    self.fc_lock.store(0, Ordering::Relaxed);
                }

                if !shared_comb_node.is_request_valid.load(Ordering::Relaxed) {
                    break;
                }

                rounds += 1;
            }
        }
    }

    pub fn enqueue(&self, val: i32, tid: i32) -> bool {
        let combining_node: CombiningNode = CombiningNode::new();

        combining_node.is_consumer.store(false, Ordering::Relaxed);
        combining_node.item.store(Some(val));

        combining_node
            .is_request_valid
            .store(true, Ordering::Relaxed);

        let shared_comb_node: Arc<CombiningNode> = Arc::new(combining_node);
        self.wait_until_fulfilled(Arc::clone(&shared_comb_node), tid);

        true
    }

    pub fn dequeue(&self, tid: i32) -> i32 {
        let combining_node: CombiningNode = CombiningNode::new();

        combining_node.is_consumer.store(true, Ordering::Relaxed);

        combining_node
            .is_request_valid
            .store(true, Ordering::Relaxed);

        let shared_comb_node: Arc<CombiningNode> = Arc::new(combining_node);
        self.wait_until_fulfilled(Arc::clone(&shared_comb_node), tid);

        return shared_comb_node.item.load().unwrap();
    }
}
