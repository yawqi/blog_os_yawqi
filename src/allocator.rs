pub mod bump;
pub mod linked_list;

use alloc::alloc::{GlobalAlloc, Layout};
use x86_64::{
    structures::paging::{
        mapper::{MapToError, PageTableFrameMapping},
        page::Page,
        FrameAllocator, Mapper, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024;

pub struct DummyAllocator;
#[allow(dead_code)]
static DUMMY_ALLOCATOR: DummyAllocator = DummyAllocator;

// #[global_allocator]
// static ALLOCATOR: LockedHeap = LockedHeap::empty();

/*
use bump::BumpAllocator;
#[global_allocator]
static ALLOCATOR: Locked<BumpAllocator> = Locked::new(BumpAllocator::new());
*/

use linked_list::LinkedListAllocator;
#[global_allocator]
static ALLOCATOR: Locked<LinkedListAllocator> = Locked::new(LinkedListAllocator::new());

unsafe impl GlobalAlloc for DummyAllocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        core::ptr::null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("dealloc should never be called")
    }
}

pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

pub fn align_up(addr: usize, align: usize) -> usize {
    match addr {
        addr if addr % align == 0 => addr,
        _ => addr - addr % align + align,
    }
}

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let vaddrs = (HEAP_START..HEAP_START + HEAP_SIZE).step_by(4096);
    let pages =
        vaddrs.map(|vaddr| Page::<Size4KiB>::containing_address(VirtAddr::new(vaddr as u64)));

    for page in pages {
        let flags = PageTableFlags::WRITABLE | PageTableFlags::PRESENT;
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush();
        }
    }
    /*
        let page_range = {
            let heap_start = VirtAddr::new(HEAP_START as u64);
            let heap_end = heap_start + HEAP_SIZE - 1u64;
            let heap_start_page = Page::containing_address(heap_start);
            let heap_end_page = Page::containing_address(heap_end);
            Page::range_inclusive(heap_start_page, heap_end_page)
        };

        for page in page_range {
            let frame = frame_allocator
                .allocate_frame()
                .ok_or(MapToError::FrameAllocationFailed)?;
            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
            unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
        }
    */
    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}
