use crate::TERMINAL;
use lazy_static::lazy_static;
use log::{error, info};
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::instructions::interrupts;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::cpu::gdt::DOUBLE_FAULT_IST_INDEX;


#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = 32, // start of PICs
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = unsafe {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(DOUBLE_FAULT_IST_INDEX);
        idt[InterruptIndex::Timer as u8].set_handler_fn(timer_interrupt_handler);
        idt
    };
}

pub static PICS: Mutex<ChainedPics> = Mutex::new(
    unsafe {
        ChainedPics::new(32, 32 + 8)
    }
); // each PIC holds 8 interrupt lines so second pic starts 8 indices later

pub fn init() {
    IDT.load();
    unsafe {
        let mut pics = PICS.lock();
        pics.initialize();
        pics.write_masks(0xFE, 0xFF); // unmask only IRQ0 (timer); everything else stays masked
    }
    interrupts::enable();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    TERMINAL.lock().clear();
    error!("Exception occurred: Breakpoint\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) -> ! {
    TERMINAL.lock().clear();
    panic!("Exception occurred: Double Fault!\nError Code: {:?}\n{:#?}", error_code, stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    info!(".");
    unsafe { PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer as u8); }
}
