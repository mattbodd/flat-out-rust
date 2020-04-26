use crossbeam_utils::atomic::AtomicCell;
use std::rc::Rc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::thread;

// Global namespace
const MAX_THREADS: usize = 512;
const COMBINING_NODE_TIMEOUT: u64 = 10000;
const COMBINING_NODE_TIMEOUT_CHECK_FREQUENCY: u64 = 100;
const MAX_COMBINING_ROUNDS: u64 = 32;
const NUM_ROUNDS_IS_LINKED_CHECK_FREQUENCY: u64 = 100;

struct CombiningNode {
    is_linked: AtomicCell<bool>,
    last_request_timestamp: AtomicCell<u64>,
    next: AtomicPtr<Option<Box<CombiningNode>>>>,
    is_request_valid: AtomicCell<bool>,
    is_consumer: AtomicCell<bool>,
    item: AtomicCell<Option<i32>>,
}

impl CombiningNode {
    fn new() -> CombiningNode {
        // TODO: How to initiailize additional fields
        CombiningNode {
            is_linked: AtomicCell::new(false),
            last_request_timestamp: AtomicCell::new(0),
            next: AtomicPtr::new(None),
            is_request_valid: AtomicCell::new(false),
            is_consumer: AtomicCell::new(false),
            item: AtomicCell::new(None),
        }
    }
}

struct QueueFatNode {
    items: Vec<i32>,
    items_left: usize,
    next: Option<Box<QueueFatNode>>,
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
    current_timestamp: u64,
    comb_list_head: Option<Box<CombiningNode>>,
    queue_head: Option<Box<QueueFatNode>>,
    queue_tail: Option<Box<QueueFatNode>>,
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
            current_timestamp: 0,
            comb_list_head: Some(Box::new(CombiningNode::new())),
            queue_head: Some(Box::new(QueueFatNode::new())),
            // Incorrect implementation - tried to make rustc happy
            queue_tail: None,
        }
    }

    fn doFlatCombining(&mut self, combiner_thread_node: &CombiningNode) {
        let mut combining_round: u64 = 0;
        let mut num_pushed_items: usize = 0;
        let mut curr_comb_node: Option<Box<CombiningNode>> = None;
        let mut last_combining_node: Option<Box<CombiningNode>> = None;
        self.current_timestamp += 1;
        let local_current_timestamp: u64 = self.current_timestamp;
        let mut local_queue_head: Option<Box<QueueFatNode>> = self.queue_head;

        let check_timestamps: bool =
            local_current_timestamp % COMBINING_NODE_TIMEOUT_CHECK_FREQUENCY == 0;

        let mut have_work: bool = false;

        loop {
            num_pushed_items = 0;
            curr_comb_node = self.comb_list_head;
            last_combining_node = curr_comb_node;
            have_work = false;

            // At this point, `some_curr_comb_node` is a *copied* version
            while let Some(some_curr_comb_node) = &curr_comb_node {
                if !some_curr_comb_node.is_request_valid.load() {
                    let next_node: CombiningNode = *some_curr_comb_node.next.take().unwrap();

                    // Definitely an illegal second comparison
                    if check_timestamps
                        && (!std::ptr::eq(&curr_comb_node.unwrap(), &self.comb_list_head.unwrap()))
                        && ((local_current_timestamp
                            - some_curr_comb_node.last_request_timestamp.load())
                            > COMBINING_NODE_TIMEOUT)
                    {
                        last_combining_node
                            .unwrap()
                            .next
                            .store(Some(Box::new(next_node)));
                        some_curr_comb_node.is_linked.store(false);
                    }

                    *some_curr_comb_node = Box::new(next_node);
                    continue;
                }

                have_work = true;

                some_curr_comb_node
                    .last_request_timestamp
                    .store(local_current_timestamp);

                if some_curr_comb_node.is_consumer.load() {
                    let consumer_satisfied: bool = false;

                    while let Some(some_local_queue_head) = &mut local_queue_head {
                        if consumer_satisfied {
                            break;
                        }
                        let head_next: QueueFatNode = *(some_local_queue_head.next.unwrap());

                        if head_next.items_left == 0 {
                            *some_local_queue_head = Box::new(head_next);
                        } else {
                            head_next.items_left -= 1;
                            some_curr_comb_node
                                .item
                                .store(Some(head_next.items[head_next.items_left]));
                            consumer_satisfied = true;
                        }
                    }

                    if !consumer_satisfied && (num_pushed_items > 0) {
                        num_pushed_items -= 1;
                        some_curr_comb_node
                            .item
                            .store(Some(self.combined_pushed_items[num_pushed_items]));
                        consumer_satisfied = true;
                    }

                    if !consumer_satisfied {
                        some_curr_comb_node.item.store(None);
                    }
                } else {
                    self.combined_pushed_items[num_pushed_items] =
                        some_curr_comb_node.item.load().unwrap();
                    num_pushed_items += 1;
                }

                some_curr_comb_node.is_request_valid.store(false);

                last_combining_node = curr_comb_node;
                // Should this be a take or load?
                curr_comb_node = curr_comb_node.unwrap().next.take();
            }

            if num_pushed_items > 0 {
                let mut new_node = QueueFatNode::new();
                new_node.items_left = num_pushed_items;
                new_node.items = Vec::with_capacity(num_pushed_items);
                for an_item in self.combined_pushed_items {
                    new_node.items.push(an_item)
                }
                new_node.next = None;

                if let Some(some_queue_tail) = &mut self.queue_tail {
                    (*some_queue_tail).next = Some(Box::new(new_node));
                    *some_queue_tail = Box::new(new_node);
                }
            }

            combining_round += 1;
            if !have_work || combining_round >= MAX_COMBINING_ROUNDS {
                self.queue_head = local_queue_head;

                return;
            }
        }
    }

    fn link_in_combining(&self, cn: *mut Box<CombiningNode>) {
        loop {
            let curr_head = self.comb_list_head.load(Ordering::Relaxed);
        
            unsafe {
                //let unboxed_cn: Box<Box<CombiningNode>> = Box::from_raw(cn);
                //let next: Option<Box<CombiningNode>> = unboxed_cn.next;
                    //let unboxed_curr_head: Box<CombiningNode> = *Box::from_raw(curr_head);
                if curr_head.is_null() {
                    Box::from_raw(cn).next = None;
                } else {
                    Box::from_raw(cn).next = Some(*Box::from_raw(curr_head));
                }
            }
            
            if self.comb_list_head.compare_exchange(curr_head, cn, Ordering::Relaxed, Ordering::Relaxed).is_ok() {
                break;
            }
        }
    }

    fn wait_until_fulfilled(&mut self, comb_node: &CombiningNode) {
        let mut rounds = 0;

        loop {
            if (rounds % NUM_ROUNDS_IS_LINKED_CHECK_FREQUENCY == 0) && !comb_node.is_linked.load() {
                comb_node.is_linked.store(true);
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

                if !comb_node.is_request_valid.load() {
                    return;
                }

                rounds += 1;
            }
        }
    }

    fn enqueue(&mut self, val: i32) -> bool {
        // Combining node should be a thread local variable
        let mut comb_node: Option<CombiningNode> = None;
        FCQueue::combining_node.with(|cn| {
            // Create new instance of combining node using old instance's values
            comb_node = Some(CombiningNode {
                is_linked: AtomicCell::new(cn.is_linked.take()),
                last_request_timestamp: AtomicCell::new(cn.last_request_timestamp.take()),
                next: AtomicCell::new(cn.next.take()),
                is_request_valid: AtomicCell::new(cn.is_request_valid.take()),
                is_consumer: AtomicCell::new(cn.is_consumer.take()),
                item: AtomicCell::new(cn.item.take()),
            });
        });

        let comb_node: CombiningNode = match comb_node {
            Some(cn) => cn,
            None => panic!("No combining node found in `enqueue`"),
        };

        comb_node.is_consumer.store(false);
        comb_node.item.store(Some(val));

        comb_node.is_request_valid.store(true);

        self.wait_until_fulfilled(&comb_node);

        true
    }

    fn dequeue(&mut self) -> i32 {
        // Combining node should be a thread local variable
        let mut comb_node: Option<CombiningNode> = None;
        FCQueue::combining_node.with(|cn| {
            // Create new instance of combining node using old instance's values
            comb_node = Some(CombiningNode {
                is_linked: AtomicCell::new(cn.is_linked.take()),
                last_request_timestamp: AtomicCell::new(cn.last_request_timestamp.take()),
                next: AtomicCell::new(cn.next.take()),
                is_request_valid: AtomicCell::new(cn.is_request_valid.take()),
                is_consumer: AtomicCell::new(cn.is_consumer.take()),
                item: AtomicCell::new(cn.item.take()),
            });
        });

        let comb_node: CombiningNode = match comb_node {
            Some(cn) => cn,
            None => panic!("No combining node found in `enqueue`"),
        };

        comb_node.is_consumer.store(true);

        comb_node.is_request_valid.store(true);

        self.wait_until_fulfilled(&comb_node);

        comb_node.item.load().unwrap()
    }
}

// Unsure how to implement:
/*   final private static AtomicReferenceFieldUpdater comb_list_head_updater =
AtomicReferenceFieldUpdater.newUpdater(FCQueue.class,CombiningNode.class, "comb_list_head");
 */
// The above snippet seems to say that `comb_list_head` can be updated atomically
// Is this just the same as using synchronization around the `comb_list_head`
// object itself?
