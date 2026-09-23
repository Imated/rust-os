use crate::mem::paging::Paging;
use limine::request::{HhdmRequest, MemmapRequest};
use spin::{Mutex, Once};
use talc::{
    DefaultBinning, min_first_heap_size,
    source::{Claim, Manual},
    sync::TalcLock,
};
use x86_64::{
    VirtAddr,
    structures::paging::{Page, PageTableFlags, Size4KiB},
};
pub mod paging;

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024;

#[global_allocator]
static TALC: TalcLock<spinning_top::RawSpinlock, Claim, DefaultBinning> =
    TalcLock::new(Claim::standby());

#[unsafe(link_section = ".requests")]
pub static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();
#[unsafe(link_section = ".requests")]
pub static MMAP_REQUEST: MemmapRequest = MemmapRequest::new();

pub static PAGING: Mutex<Once<Paging>> = Mutex::new(Once::new());

pub fn init() {
    PAGING.lock().call_once(Paging::default);
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE as u64 - 1u64;
        let heap_start_page = Page::<Size4KiB>::containing_address(heap_start);
        let heap_end_page = Page::<Size4KiB>::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    Paging::map_memory_global_range(
        page_range,
        Paging::allocate_frame_global().start_address(),
        PageTableFlags::PRESENT
            | PageTableFlags::WRITABLE
            | PageTableFlags::USER_ACCESSIBLE
            | PageTableFlags::NO_EXECUTE,
    );

    unsafe {
        TALC.lock().claim(
            HEAP_SIZE as *mut u8,
            HEAP_SIZE + HEAP_SIZE + min_first_heap_size::<DefaultBinning>(),
        );
    }
}
