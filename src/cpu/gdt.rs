use spin::once::Once;
use x86_64::VirtAddr;
use x86_64::instructions::segmentation::Segment;
use x86_64::instructions::tables::load_tss;
use x86_64::registers::segmentation::{CS, DS, ES, FS, GS, SS, SegmentSelector};
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable};
use x86_64::structures::tss::TaskStateSegment;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

static GDT: Once<(GlobalDescriptorTable, Selectors)> = Once::new();
static TSS: Once<TaskStateSegment> = Once::new();

struct Selectors {
    kernel_code_selector: SegmentSelector,
    kernel_data_selector: SegmentSelector,
    user_data_selector: SegmentSelector,
    user_code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init() {
    let tss = TSS.call_once(|| {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(&raw const STACK);
            stack_start + STACK_SIZE as u64
        };
        tss
    });

    let gdt = GDT.call_once(|| {
        let mut gdt = GlobalDescriptorTable::new();
        let kernel_code_selector = gdt.append(Descriptor::kernel_code_segment()); // 0x08
        let kernel_data_selector = gdt.append(Descriptor::kernel_data_segment()); // 0x10
        let user_data_selector = gdt.append(Descriptor::user_data_segment()); // 0x18
        let user_code_selector = gdt.append(Descriptor::user_code_segment()); // 0x20
        let tss_selector = gdt.append(Descriptor::tss_segment(tss)); // 0x28 && 0x30
        (
            gdt,
            Selectors {
                kernel_code_selector,
                kernel_data_selector,
                user_data_selector,
                user_code_selector,
                tss_selector,
            },
        )
    });
    gdt.0.load();
    unsafe {
        CS::set_reg(gdt.1.kernel_code_selector);
        SS::set_reg(gdt.1.kernel_data_selector);
        ES::set_reg(gdt.1.kernel_data_selector);
        DS::set_reg(gdt.1.kernel_data_selector);
        GS::set_reg(gdt.1.kernel_data_selector);
        FS::set_reg(gdt.1.kernel_data_selector);
        load_tss(gdt.1.tss_selector);
    }
}
