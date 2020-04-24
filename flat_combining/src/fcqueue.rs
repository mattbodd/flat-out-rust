use crossbeam_utils::atomic::AtomicCell;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::rc::Rc;
use std::thread;

// Global namespace
const MAX_THREADS: usize = 512;
const COMBINING_NODE_TIMEOUT: u64 = 10000;
const COMBINING_NODE_TIMEOUT_CHECK_FREQUENCY: u64 = 100;
const MAX_COMBINING_ROUNDS: u64 = 32;
const NUM_ROUNDS_IS_LINKED_CHECK_FREQUENCY: i64 = 100;

struct CombiningNode {
    is_linked: bool,
    last_request_timestamp: u64,
    next: Option<CombiningNode>,
    is_request_valid: bool,
    is_consumer: bool,
    item: Option<i32>,
}

impl CombiningNode {
    fn new() -> CombiningNode {
        // TODO: How to initiailize additional fields
        CombiningNode {
            is_linked: false,
            last_request_timestamp: (-1),
            next: None,
            is_request_valid: false,
            is_consumer: false,
            item: None,
        }
    }
}

struct QueueFatNode {
    items: Vec<i32>,
    items_left: usize,
    next: Option<QueueFatNode>,
}

impl QueueFatNode {
    fn new() -> QueueFatNode {
        QueueFatNode {
            // TOOD: Determine what size to initialize by default
            items: Vec::new(),
            items_left: 0,
            next: None,
        }
    }
}

struct FCQueue {
    fc_lock: AtomicUsize,
    combined_pushed_items: Vec<i32>,
    current_timestamp: i64,
    combining_node: CombiningNode,
    comb_list_head: Option<CombiningNode>,
    queue_head: Option<QueueFatNode>,
    queue_tail: Option<QueueFatNode>,
}

impl FCQueue {
    //maybe this works??
    thread_local! {
        static combining_node: CombiningNode = CombiningNode::new();
    }

    fn new() -> FCQueue {
        FCQueue {
            fc_lock: AtomicUsize::new(0),
            combined_pushed_items: Vec::with_capacity(MAX_THREADS),
            // current_timestamp: ?,
            // thread_local! {
            //     combining_node: CombiningNode::new(),
            // }
            comb_list_head: Some(CombiningNode::new()),
            queue_head: Some(QueueFatNode::new()),
            queue_tail: &queue_head,
        }
    }

    fn doFlatCombining(&mut self, combiner_thread_node: CombiningNode) {
        let combining_round: i64 = 0;
        let num_pushed_items: usize = 0;
        let curr_comb_node: Option<CombiningNode> = None;
        let last_combining_node: Option<CombiningNode> = None;
        self.current_timestamp += 1;
        let local_current_timestamp: i64 = self.current_timestamp;
        let local_queue_head: Option<QueueFatNode> = &self.queue_head;

        let check_timestamps: bool =
            (local_current_timestamp % COMBINING_NODE_TIMEOUT_CHECK_FREQUENCY == 0);

        let have_work: bool = false;

        loop {
            num_pushed_items = 0;
            curr_comb_node = &self.comb_list_head;
            last_combining_node = curr_comb_node;
            have_work = false;

            // At this point, `some_curr_comb_node` is a *copied* version
            while let Some(some_curr_comb_node) = &mut curr_comb_node {
                if !some_curr_comb_node.is_request_valid {
                    let next_node: CombiningNode = some_curr_comb_node.next.unwrap();

                    // Definitely an illegal second comparison
                    if check_timestamps
                        && (!std::ptr::eq(&curr_comb_node.unwrap(), &self.comb_list_head.unwrap()))
                        && ((local_current_timestamp - some_curr_comb_node.last_request_timestamp)
                            > COMBINING_NODE_TIMEOUT)
                    {
                        last_combining_node.unwrap().next = Some(next_node);
                        some_curr_comb_node.is_linked = false;
                    }

                    *some_curr_comb_node = next_node;
                    continue;
                }

                have_work = true;

                some_curr_comb_node.last_request_timestamp = local_current_timestamp;

                if some_curr_comb_node.is_consumer {
                    let consumer_satisfied: bool = false;

                    while let Some(some_local_queue_head) = &mut local_queue_head {
                        if (consumer_satisfied) {
                            break;
                        }
                        let head_next: QueueFatNode = some_local_queue_head.next.unwrap();

                        if (head_next.items_left == 0) {
                            *some_local_queue_head = head_next;
                        } else {
                            head_next.items_left -= 1;
                            some_curr_comb_node.item = Some(head_next.items[head_next.items_left]);
                            consumer_satisfied = true;
                        }
                    }

                    if !consumer_satisfied && (num_pushed_items > 0) {
                        num_pushed_items -= 1;
                        some_curr_comb_node.item =
                            Some(self.combined_pushed_items[num_pushed_items]);
                        consumer_satisfied = true;
                    }

                    if !consumer_satisfied {
                        some_curr_comb_node.item = None;
                    }
                } else {
                    self.combined_pushed_items[num_pushed_items] =
                        some_curr_comb_node.item.unwrap();
                    num_pushed_items += 1;
                }

                some_curr_comb_node.is_request_valid = false;

                last_combining_node = curr_comb_node;
                curr_comb_node = curr_comb_node.unwrap().next;
            }

            if num_pushed_items > 0 {
                let new_node = QueueFatNode::new();
                new_node.items_left = num_pushed_items;
                new_node.items = Vec::with_capacity(num_pushed_items);
                for an_item in self.combined_pushed_items {
                    new_node.items.push(an_item)
                }
                new_node.next = None;

                if let Some(some_queue_tail) = &mut self.queue_tail {
                    (*some_queue_tail).next = Some(new_node);
                    *some_queue_tail = new_node;
                }
            }

            combining_round += 1;
            if !have_work || combining_round >= MAX_COMBINING_ROUNDS {
                self.queue_head = local_queue_head;

                return;
            }
        }
    }

    fn link_in_combining(&self, cn: CombiningNode) {
        loop {
            let curr_head: Option<CombiningNode> = self.comb_list_head;
            cn.next = curr_head;

            // Unsure about this
            if std::ptr::eq(&self.comb_list_head.unwrap(), &curr_head.unwrap()) {
                // CAS and conditionally return
            }
        }
    }

    fn wait_until_fulfilled(&self, comb_node: CombiningNode) {
        let mut rounds = 0;

        loop {
            if (rounds % NUM_ROUNDS_IS_LINKED_CHECK_FREQUENCY == 0) && !comb_node.is_linked {
                comb_node.is_linked = true;
                self.link_in_combining(comb_node);
            }

            if self.fc_lock.load(Ordering::Relaxed) == 0 {
                let cae: Result<usize, usize> =
                    self.fc_lock
                        .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed);
                if cae.is_ok() {
                    self.doFlatCombining(comb_node);
                    self.fc_lock.store(0, Ordering::Relaxed);
                }

                if !comb_node.is_request_valid {
                    return;
                }

                rounds += 1;
            }
        }
    }

    fn enqueue(&self, val: i32) -> bool {
        // Combining node should be a thread local variable
        let comb_node: CombiningNode = self.combining_node;
        comb_node.is_consumer = false;
        comb_node.item = Some(val);

        comb_node.is_request_valid = true;

        self.wait_until_fulfilled(comb_node);

        true
    }

    fn dequeue(&self) -> i32 {
        // Combining node should be a thread local variable
        let comb_node = self.combining_node;
        comb_node.is_consumer = true;

        comb_node.is_request_valid = true;

        self.wait_until_fulfilled(comb_node);

        comb_node.item.unwrap()
    }
}

// Unsure how to implement:
/*   final private static AtomicReferenceFieldUpdater comb_list_head_updater =
AtomicReferenceFieldUpdater.newUpdater(FCQueue.class,CombiningNode.class, "comb_list_head");
 */
// The above snippet seems to say that `comb_list_head` can be updated atomically
// Is this just the same as using synchronization around the `comb_list_head`
// object itself?
