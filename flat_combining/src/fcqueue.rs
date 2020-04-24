// Time-stamp: <2020-04-24 12:17:34 (mbodd)>

use std::sync::atomic::AtomicUsize;
use std::cell::RefCell;
use std::thread;

// Global namespace
const MAX_THREADS: usize = 512;
const COMBINING_NODE_TIMEOUT: i64 = 10000;
const COMBINING_NODE_TIMEOUT_CHECK_FREQUENCY: i64 = 100;
const MAX_COMBINING_ROUNDS: i64 = 32;

struct CombiningNode<T> {
    is_linked: bool,
    last_request_timestamp: i64,,
    next: Option<CombiningNode>,
    is_request_valid: bool,
    is_consume: bool,
    item: T,
}

impl CombiningNode {
    fn new() -> CombiningNode {
        // TODO: How to initiailize additional fields
        CombiningNode {
            is_linked: false,
            // last_request_timestamp: ?,
            next: None,
            is_request_valid = false,
            // is_comsume: ?,
            // item: ?,
        }
    }
}

struct QueueFatNode<T> {
    items: Vec<T>,
    items_left: i64,
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

struct FCQueue<T> {
    fc_lock: AtomicUsize,
    combined_pushed_items: Vec<T>,
    current_timestamp: i64,
    combining_node: RefCell<CombiningNode>,
    comb_list_head: Option<RefCell<CombiningNode>>,
    queue_head: Option<QueueFatNode>,
    queue_tail: Option<QueueFatNode>,
    
}

impl FCQueue {
    fn new() -> FCQueue {
        FCQueue {
            fc_lock: AtomicUsize::new(0),
            combined_pushed_items: Vec::with_capacity(MAX_THREADS),
            // current_timestamp: ?,
            thread_local! {
                combining_node: RefCell::new(CombiningNode::new());
            }
            comb_list_head: Some(RefCell::new(CombiningNode::new()));
            queue_head: QueueFatNode::new(),
            // Definitely illegal
            queue_tail: queue_head,
        }
    }
}

// Unsure how to implement:
/*   final private static AtomicReferenceFieldUpdater comb_list_head_updater =
AtomicReferenceFieldUpdater.newUpdater(FCQueue.class,CombiningNode.class, "comb_list_head");
 */
// The above snippet seems to say that `comb_list_head` can be updated atomically
// Is this just the same as using synchronization around the `comb_list_head`
// object itself?
