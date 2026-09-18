use crate::cpu::gdt::DOUBLE_FAULT_IST_INDEX;
use crate::graphics::terminal::Terminal;
use core::fmt::Write;
use log::error;
use pc_keyboard::layouts::Us104Key;
use pc_keyboard::{DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use pic8259::ChainedPics;
use spin::{Mutex, Once};
use x86_64::instructions::interrupts;
use x86_64::instructions::port::Port;
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = 32, // start of PICs
    Keyboard,
}

static IDT: Once<InterruptDescriptorTable> = Once::new();

pub static PICS: Mutex<ChainedPics> = Mutex::new(unsafe { ChainedPics::new_contiguous(32) });

pub fn init() {
    IDT.call_once(|| unsafe {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(DOUBLE_FAULT_IST_INDEX);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt[InterruptIndex::Timer as u8].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard as u8].set_handler_fn(keyboard_interrupt_handler);
        idt
    })
    .load();
    unsafe {
        let mut pics = PICS.lock();
        pics.initialize();
        pics.write_masks(0x00, 0x00); // unmask EVERYTHING
    }
    interrupts::enable();
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    unsafe {
        Terminal::with_forced(|terminal| {
            terminal.clear();
        })
    };
    panic!(
        "Exception occurred: Page Fault!\nError Code: {:?}\nAddress Accessed: {}\n{:#?}",
        error_code,
        Cr2::read_raw(),
        stack_frame
    );
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    error!("Exception occurred: Breakpoint\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    unsafe {
        Terminal::with_forced(|terminal| {
            terminal.clear();
        })
    };
    panic!(
        "Exception occurred: Double Fault!\nError Code: {:?}\n{:#?}",
        error_code, stack_frame
    );
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer as u8);
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    static KEYBOARD: Mutex<Keyboard<Us104Key, ScancodeSet1>> = Mutex::new(Keyboard::new(
        ScancodeSet1::new(),
        Us104Key,
        HandleControl::Ignore,
    ));

    let mut keyboard = KEYBOARD.lock();
    let mut port: Port<u8> = Port::new(0x60);
    let scancode = unsafe { port.read() };
    Terminal::with(|terminal| {
        let Ok(Some(key_event)) = keyboard.add_byte(scancode) else {
            return;
        };
        let Some(key) = keyboard.process_keyevent(key_event) else {
            return;
        };

        match key {
            DecodedKey::RawKey(character) => write!(terminal, "{:?}", character).expect(""),
            DecodedKey::Unicode(key) => write!(terminal, "{}", key).expect(""),
        }
    });

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard as u8);
    }
}
