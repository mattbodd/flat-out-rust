use crossbeam_utils::atomic::AtomicCell;
use std::collections::VecDeque;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

// Global namespace
const MAX_THREADS: usize = 512;
const COMBINING_NODE_TIMEOUT: u64 = 10000;
const COMBINING_NODE_TIMEOUT_CHECK_FREQUENCY: u64 = 100;
const MAX_COMBINING_ROUNDS: u64 = 32;
const NUM_ROUNDS_IS_LINKED_CHECK_FREQUENCY: u64 = 100;

struct CombiningNode {
    is_linked: AtomicCell<bool>,
    last_request_timestamp: AtomicCell<u64>,
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
            is_request_valid: AtomicCell::new(false),
            is_consumer: AtomicCell::new(false),
            item: AtomicCell::new(None),
        }
    }
}

#[derive(Clone)]
pub struct QueueFatNode {
    items: Vec<i32>,
    items_left: usize,
}

impl QueueFatNode {
    fn new() -> QueueFatNode {
        QueueFatNode {
            // TOOD: Determine what size to initialize by default
            items: Vec::new(),
            items_left: 0,
        }
    }
    pub fn get(&self)
    {
    	for val in &self.items{
    		println!("{}",val);
    	}
    }
}

pub struct FCQueue {
    fc_lock: AtomicUsize,
    combined_pushed_items: Vec<i32>,
    current_timestamp: AtomicCell<u64>,
    comb_list_head: Mutex<VecDeque<Arc<CombiningNode>>>,
    pub queue: VecDeque<QueueFatNode>,
}

impl FCQueue {
    pub fn new() -> FCQueue{
        FCQueue {
            fc_lock: AtomicUsize::new(0),
            combined_pushed_items: vec![0;MAX_THREADS],
            current_timestamp: AtomicCell::new(0),
            comb_list_head: Mutex::new(VecDeque::new()),
            queue: VecDeque::new(),
        }
    }


    fn doFlatCombining(&mut self) {
        let mut combining_round: u64 = 0;
        let mut num_pushed_items: usize = 0;
        let mut curr_comb_node: VecDeque<Arc<CombiningNode>>;
        {
            curr_comb_node = self.comb_list_head.lock().unwrap().clone();
        }

        let mut last_combining_node: Option<usize> = None;
        self.current_timestamp.fetch_add(1);
        let local_current_timestamp: u64 = self.current_timestamp.load();
        let mut local_queue_head: VecDeque<QueueFatNode> = self.queue.clone();

        let check_timestamps: bool =
            local_current_timestamp % COMBINING_NODE_TIMEOUT_CHECK_FREQUENCY == 0;

        let mut have_work: bool = false;

        loop {
            num_pushed_items = 0;
            {
                curr_comb_node = self.comb_list_head.lock().unwrap().drain(..).collect();
            }
            last_combining_node = Some(0);

            have_work = false;

            while !curr_comb_node.is_empty() {
                /*
                if !curr_comb_node.front().unwrap().is_request_valid.load() {
                    let next_node: &CombiningNode = curr_comb_node[1];

                    // Definitely an illegal second comparison
                    if check_timestamps
                        && (!std::ptr::eq(
                            &mut curr_comb_node.front().unwrap(),
                            &self.comb_list_head.front().unwrap(),
                        ))
                        && ((local_current_timestamp
                            - some_curr_comb_node.last_request_timestamp.load())
                            > COMBINING_NODE_TIMEOUT)
                    {
                        last_combining_node;
                        curr_comb_node
                    }

                    *some_curr_comb_node = Box::new(next_node);
                    continue;
                }
                */

                have_work = true;

                curr_comb_node
                    .front()
                    .unwrap()
                    .last_request_timestamp
                    .store(local_current_timestamp);

                if false { /*some_curr_comb_node.is_consumer.load() {
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
                     }*/
                } else {

                    self.combined_pushed_items[num_pushed_items] =
                        curr_comb_node.front().unwrap().item.load().unwrap();
                    num_pushed_items += 1;
                }

                curr_comb_node
                    .front()
                    .unwrap()
                    .is_request_valid
                    .store(false);

                // last_combining_node = curr_comb_node;
                curr_comb_node.pop_front();
            }

            if num_pushed_items > 0 {
                let mut new_node: QueueFatNode = QueueFatNode::new();
                new_node.items_left = num_pushed_items;
                new_node.items = Vec::with_capacity(num_pushed_items);
                for an_item in &self.combined_pushed_items {
                    new_node.items.push(*an_item);
                    
                }


                self.queue.push_back(new_node);
            }

            combining_round += 1;
            if !have_work || combining_round >= MAX_COMBINING_ROUNDS {
                //self.queue = local_queue_head;

                return;
            }
        }
    }

    fn link_in_combining(&self, cn:  Arc<CombiningNode>) {
        // Block until we have access to the global `comb_list_head` at which point
        // we merge our thread local queue
        let mut curr_comb_queue = self.comb_list_head.lock().unwrap();
        curr_comb_queue.push_front(cn);
        //  Mutex is unlocked at end of scope
    }

    fn wait_until_fulfilled(&mut self, comb_node: CombiningNode) {
        let mut rounds = 0;

        let shared_comb_node:Arc<CombiningNode>=Arc::new(comb_node);

        loop {
            if (rounds % NUM_ROUNDS_IS_LINKED_CHECK_FREQUENCY == 0) && !shared_comb_node.is_linked.load() {
                shared_comb_node.is_linked.store(true);
                self.link_in_combining(Arc::clone(&shared_comb_node));
            }

            if self.fc_lock.load(Ordering::Relaxed) == 0 {
                let cae: Result<usize, usize> =
                    self.fc_lock
                        .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed);
                if cae.is_ok() {
                    self.doFlatCombining();
                    self.fc_lock.store(0, Ordering::Relaxed);
                }

                if !shared_comb_node.is_request_valid.load() {
                    break;
                }

                rounds += 1;
            }
        }


    }

    pub fn enqueue(&mut self, val: i32) -> bool {
        let combining_node: CombiningNode = CombiningNode::new();

        combining_node.is_consumer.store(false);
        combining_node.item.store(Some(val));

        combining_node.is_request_valid.store(true);

        self.wait_until_fulfilled(combining_node);

        true
    }

    /*
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
     */
}
