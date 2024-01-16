use core::{
    alloc::{self, GlobalAlloc, Layout},
    ptr::{self, NonNull},
};

use super::Locked;

const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

struct ListNode {
    next: Option<&'static mut ListNode>,
}

impl ListNode {
    fn new(next: Option<&'static mut ListNode>) -> Self {
        Self { next }
    }
}

fn list_index(layout: &Layout) -> Option<usize> {
    let size = layout.size().max(layout.align());
    BLOCK_SIZES.iter().position(|sz| *sz >= size)
}

pub struct FixedSizeAllocator {
    lists_allocator: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
    fallback_allocator: linked_list_allocator::Heap,
}

const EMPTY: Option<&'static mut ListNode> = None;
impl FixedSizeAllocator {
    pub const fn new() -> Self {
        Self {
            lists_allocator: [EMPTY; BLOCK_SIZES.len()],
            fallback_allocator: linked_list_allocator::Heap::empty(),
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.fallback_allocator.init(heap_start, heap_size)
    }

    pub fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        match self.fallback_allocator.allocate_first_fit(layout) {
            Ok(mut ptr) => unsafe { ptr.as_mut() },
            _ => ptr::null_mut(),
        }
    }
}

unsafe impl GlobalAlloc for Locked<FixedSizeAllocator> {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let mut allocator = self.lock();
        match list_index(&layout) {
            Some(idx) => match allocator.lists_allocator[idx].take() {
                Some(head) => {
                    allocator.lists_allocator[idx] = head.next.take();
                    head as *mut ListNode as *mut u8
                }
                None => {
                    let block_size = BLOCK_SIZES[idx];
                    let layout = Layout::from_size_align(block_size, block_size).unwrap();
                    allocator.fallback_alloc(layout)
                }
            },
            None => allocator.fallback_alloc(layout),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let mut allocator = self.lock();
        match list_index(&layout) {
            Some(idx) => {
                let new_head = ListNode::new(allocator.lists_allocator[idx].take());
                let new_head_ptr = ptr as *mut ListNode;
                new_head_ptr.write(new_head);
                allocator.lists_allocator[idx] = Some(&mut *new_head_ptr);
            }
            None => allocator
                .fallback_allocator
                .deallocate(NonNull::new_unchecked(ptr), layout),
        };
    }
}
