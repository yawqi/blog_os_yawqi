use core::{
    alloc::{GlobalAlloc, Layout},
    ptr,
};

use super::{align_up, Locked};

struct ListNode {
    size: usize,
    next: Option<&'static mut ListNode>,
}

impl ListNode {
    const fn new(size: usize) -> Self {
        Self { size, next: None }
    }

    fn start_address(&self) -> usize {
        (self as *const Self) as usize
    }

    fn end_address(&self) -> usize {
        self.start_address() + self.size
    }
}

pub struct LinkedListAllocator {
    head: ListNode,
}

impl LinkedListAllocator {
    pub const fn new() -> Self {
        Self {
            head: ListNode::new(0),
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.add_free_region(heap_start, heap_size);
    }

    unsafe fn add_free_region(&mut self, start_addr: usize, size: usize) {
        assert_eq!(
            align_up(start_addr, core::mem::size_of::<ListNode>()),
            start_addr
        );
        assert!(size >= core::mem::size_of::<ListNode>());
        let mut node = ListNode::new(size);
        node.next = self.head.next.take();
        let node_ptr = start_addr as *mut ListNode;
        node_ptr.write(node);
        self.head.next = Some(&mut *node_ptr);
    }

    fn find_region(&mut self, size: usize, align: usize) -> Option<(&'static mut ListNode, usize)> {
        let mut curr = &mut self.head;

        while let Some(ref mut region) = curr.next {
            if let Ok(start_addr) = Self::alloc_from_region(&region, size, align) {
                let target_region = curr.next.take().unwrap();
                curr.next = target_region.next.take();
                let ret = (target_region, start_addr);
                return Some(ret);
            }
            curr = curr.next.as_mut().unwrap();
        }
        None
    }

    fn alloc_from_region(region: &ListNode, size: usize, align: usize) -> Result<usize, ()> {
        let start_addr = align_up(region.start_address(), align);
        let end_addr = start_addr
            .checked_add(size)
            .ok_or("end address overflow")
            .unwrap();

        if start_addr > region.end_address() {
            return Err(());
        }

        if align_up(end_addr, core::mem::size_of::<ListNode>()) > region.end_address() {
            return Err(());
        }

        Ok(start_addr)
    }

    fn size_align(layout: Layout) -> (usize, usize) {
        let layout = layout
            .align_to(core::mem::align_of::<ListNode>())
            .expect("align failed")
            .pad_to_align();
        let size = layout.size().max(core::mem::size_of::<ListNode>());
        (size, layout.align())
    }
}

unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let (size, align) = LinkedListAllocator::size_align(layout);
        let mut allocator = self.lock();
        if let Some((region, start_addr)) = allocator.find_region(size, align) {
            let end_addr = start_addr.checked_add(size).expect("addr overflow");
            let end_region = region.end_address();

            if end_region > end_addr {
                allocator.add_free_region(end_addr, end_region - end_addr);
            }

            start_addr as *mut u8
        } else {
            ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let (size, _) = LinkedListAllocator::size_align(layout);
        self.lock().add_free_region(ptr as usize, size)
    }
}
