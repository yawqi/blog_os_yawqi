use bootloader::bootinfo::MemoryMap;
use bootloader::bootinfo::MemoryRegionType;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::page::Page;
use x86_64::structures::paging::page_table::FrameError;
use x86_64::structures::paging::FrameAllocator;
use x86_64::structures::paging::Mapper;
use x86_64::structures::paging::OffsetPageTable;
use x86_64::structures::paging::PageTable;
use x86_64::structures::paging::PageTableFlags;
use x86_64::structures::paging::PhysFrame;
use x86_64::structures::paging::Size4KiB;
use x86_64::{PhysAddr, VirtAddr};

pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        None
    }
}

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    pub unsafe fn new(memory_map: &'static MemoryMap) -> Self {
        Self {
            memory_map,
            next: 0,
        }
    }

    fn usable_frame(&self) -> impl Iterator<Item = PhysFrame> {
        let regions_iter = self
            .memory_map
            .iter()
            .filter(|r| r.region_type == MemoryRegionType::Usable);

        regions_iter
            .flat_map(|r| {
                let range = r.range.start_addr()..r.range.end_addr();
                range.step_by(4096)
            })
            .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usable_frame().nth(self.next);
        self.next += 1;
        frame
    }
}

unsafe fn active_level_4_page_table(offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();

    let level_4_table_phys_addr = level_4_table_frame.start_address();
    let level_4_table_virt_addr = offset + level_4_table_phys_addr.as_u64();

    let pt: *mut PageTable = level_4_table_virt_addr.as_mut_ptr();
    &mut *pt
}

pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_page_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

pub unsafe fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    page_frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));

    let res = unsafe { mapper.map_to(page, frame, flags, page_frame_allocator) };
    res.expect("map_to failed").flush();
}

pub unsafe fn translate_virtaddr(
    virt_addr: VirtAddr,
    physical_memory_offset: VirtAddr,
) -> Option<PhysAddr> {
    translate_virtaddr_inner(virt_addr, physical_memory_offset)
}

fn translate_virtaddr_inner(
    virt_addr: VirtAddr,
    physical_memory_offset: VirtAddr,
) -> Option<PhysAddr> {
    let page_table_indexs = [
        virt_addr.p4_index(),
        virt_addr.p3_index(),
        virt_addr.p2_index(),
        virt_addr.p1_index(),
    ];
    let (mut page_table_frame, _) = Cr3::read();
    for idx in page_table_indexs {
        let virt_addr = physical_memory_offset + page_table_frame.start_address().as_u64();
        let page_table = unsafe { &*virt_addr.as_ptr() as &'static PageTable };
        let page_table_entry = &page_table[idx];

        page_table_frame = match page_table_entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("Huge pages not supported!"),
        };
    }

    Some(page_table_frame.start_address() + u64::from(virt_addr.page_offset()))
}
