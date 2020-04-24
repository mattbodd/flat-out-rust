// use std::ptr;

// //Trying to make this a sort of extensible "base class" that both seq_queue and seq_stack could use to implement themselves
// pub struct List<T> {
//     head: Link<T>,
//     tail: Option<Node<T>>,
//     length: usize,
// }

// type Link<T> = Option<Box<Node<T>>>;

// struct Node<T> {
//     data: T,
//     next: Link<T>,
// }





// impl<T> List<T> {
//     pub fn new() -> Self {
//         List { head: None, tail: ptr::null_mut(), length: 0 }
//     }
//     pub fn first(&self) -> Option<&T> 
//     {
//     	self.head.as_ref().map(|node| &node.data)
//     }
//     pub fn last(&self) -> Option<&T>
//     {
//     	if !self.tail.is_null()
//     	{
//     		unsafe{
//     			self.tail.as_ref().map(|node| &node.data)
//     		}
//     	}
//     	else {
//     		self.head.as_ref().map(|node| &node.data)
//     	}
//     }

//     fn push_front_ptr_mut(&mut self, data: T) {


//         let mut new_node = Box::new(Node {
//             data: data,
//             next: self.head.take(),
//         });

//         let raw_tail: *mut _ = &mut *new_node;

        
//         if self.tail.is_null() {
//         	self.tail=Some
//         	self.tail = raw_tail;
//         	unsafe {
//                 (*self.tail).next = None;
//             } 
//         } 

	
//     	self.head = Some(new_node);
//         self.length += 1;
//     }

//     pub fn len(&self) -> usize
//     {
//     	self.length
//     }
//     pub fn is_empty(&self) -> bool
//     {
//     	self.length == 0
//     }

//     #[must_use]
//     pub fn push_front(&self, data: T) -> List<T>
//     {
//         let mut new_list = self.clone();

//         new_list.push_front_mut(data);

//     }
//     pub fn push_front_mut(&mut self, data: T) {
//         self.push_front_ptr_mut(data)
//     }


// }


// #[cfg(test)]
// mod test {
// 	use super::List;
// 	#[test]
// 	fn test_new() {
// 	    let empty_list: List<i32> = List::new();
	
// 	    assert!(empty_list.head.is_none());
	
// 	    assert_eq!(empty_list.len(), 0);
// 	    assert!(empty_list.is_empty());
// 	}

// 	#[test]
// 	fn test_first() {
// 	    let empty_list: List<i32> = List::new();

// 	    let mut singleton_list: List<&str> = List::new();
// 	    singleton_list.push_front_ptr_mut("hello");

// 	    let mut list: List<i32> = List::new();
// 	    list.push_front(0);
// 	    list.push_front(1);
// 	    list.push_front(2);
// 	    list.push_front(3);
// 	    //let list = list![0, 1, 2, 3];
	
// 	    assert_eq!(empty_list.first(), None);
// 	    assert_eq!(singleton_list.first(), Some(&"hello"));
// 	    assert_eq!(list.first(), Some(&3));
// 	}
// 	#[test]
// 	fn test_last() {
// 	    let empty_list: List<i32> = List::new();

// 	    let mut singleton_list: List<&str> = List::new();
// 	    singleton_list.push_front_ptr_mut("hello");

// 	    let mut list: List<i32> = List::new();
// 	    list.push_front_ptr_mut(0);
// 	    list.push_front_ptr_mut(1);
// 	    list.push_front_ptr_mut(2);
// 	    list.push_front_ptr_mut(3);
// 	    //let list = list![0, 1, 2, 3];
	
// 	    assert_eq!(empty_list.last(), None);
// 	    assert_eq!(singleton_list.last(), Some(&"hello"));
// 	    assert_eq!(list.last(), Some(&0));
// 	}


// }
