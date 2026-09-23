use core::sync::atomic::AtomicUsize;

use limine::memmap::{MEMMAP_ACPI_RECLAIMABLE, MEMMAP_USABLE};
use x86_64::{
    PhysAddr, VirtAddr,
    registers::control::Cr3,
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame,
        Size4KiB, page::PageRangeInclusive,
    },
};

use crate::mem::{HHDM_REQUEST, MMAP_REQUEST, PAGING};

pub struct Paging {
    mapper: OffsetPageTable<'static>,
    next: AtomicUsize,
}

impl Default for Paging {
    fn default() -> Self {
        let hhdm_offset = VirtAddr::new(HHDM_REQUEST.response().unwrap().offset);
        let ptl4 = Self::active_ptl4(hhdm_offset);
        let mapper: OffsetPageTable<'static> = unsafe { OffsetPageTable::new(ptl4, hhdm_offset) };
        Self {
            mapper,
            next: AtomicUsize::new(0),
        }
    }
}

impl Paging {
    pub fn map_memory_global(virt: VirtAddr, phys: PhysAddr, flags: PageTableFlags) {
        unsafe {
            PAGING
                .lock()
                .get_mut_unchecked()
                .map_memory(virt, phys, flags)
        };
    }

    pub fn map_memory_global_range(
        range: PageRangeInclusive,
        phys: PhysAddr,
        flags: PageTableFlags,
    ) {
        unsafe {
            let mut paging_lock = PAGING.lock();
            let paging = paging_lock.get_mut_unchecked();
            for page in range {
                paging.map_memory(page.start_address(), phys, flags);
            }
        };
    }

    pub fn allocate_frame_global() -> PhysFrame {
        unsafe { PAGING.lock().get_mut_unchecked().allocate_frame() }
    }

    pub fn map_memory(&mut self, virt: VirtAddr, phys: PhysAddr, flags: PageTableFlags) {
        let frame: PhysFrame<Size4KiB> = PhysFrame::containing_address(phys);
        let page = Page::containing_address(virt);
        unsafe {
            self.mapper
                .map_to_with_table_flags(page, frame, flags, flags, &mut FrameCursor(&self.next))
                .expect("Couldnt find available frame to allocate. >:C")
                .flush();
        };
    }

    pub fn allocate_frame(&self) -> PhysFrame {
        FrameCursor(&self.next).allocate_frame().unwrap()
    }

    pub fn active_ptl4(hhdm_offset: VirtAddr) -> &'static mut PageTable {
        let (plt4_frame, _) = Cr3::read();
        let phys = plt4_frame.start_address();
        let virt = hhdm_offset + phys.as_u64();
        unsafe { &mut *virt.as_mut_ptr() }
    }
}

pub struct FrameCursor<'a>(&'a AtomicUsize);

unsafe impl FrameAllocator<Size4KiB> for FrameCursor<'_> {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let next = self.0.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        MMAP_REQUEST
            .response()
            .unwrap()
            .entries()
            .iter()
            .filter(|e| matches!(e.type_, MEMMAP_ACPI_RECLAIMABLE | MEMMAP_USABLE))
            .map(|e| e.base..e.base + e.length)
            .flat_map(|e| e.step_by(4096))
            .map(|addr| PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(addr)))
            .nth(next)
    }
}
