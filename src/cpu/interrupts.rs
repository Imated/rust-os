use crate::TERMINAL;
use lazy_static::lazy_static;
use log::error;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::cpu::gdt::DOUBLE_FAULT_IST_INDEX;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = unsafe {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(DOUBLE_FAULT_IST_INDEX);
        idt
    };
}
pub fn init() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    TERMINAL.lock().clear();
    error!("Exception occurred: Breakpoint\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) -> ! {
    TERMINAL.lock().clear();
    panic!("Exception occurred: Double Fault!\nError Code: {:?}\n{:#?}", error_code, stack_frame);
}
