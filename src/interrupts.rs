// InterruptDescriptorTable (IDT)
// IDT is used to catch and handle exception
// We specify a handler function for each CPU exception
//
// Type  Name                      Description
// u16   Function Pointer [0:15]   The lower bits of the pointer to handler function
// u16   GDT selector              Selector of a code segment in the GDT
// u16   Options                   (below)
// u16   Function Pointer [16:31]  The middle bits of the pointer to the handler function
// u16   Function Pointer [32:63]  The remaining bits of the pointer to the handler function
// u32   Reserved
//
//
// Options Field
//
// Bits    Name                             Description
// 0-2     Interrupt Stack Table Index      0: Don't switch stacks, 1-7: Switch to the nth stack in the
                                         // Interrupt Stack Table when this handler is called
// 3-7     Reserved
// 8       0: Interrupt Gate, 1: Trap Gate  If this bit is 0, interrupts are disabled when this
                                         // handler is called
// 9-11    must be one
// 12      must be zero
// 13-14   Descriptor Privilege Level       The minimal privilege level required for calling this
                                         // handler
// 15      Present

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use lazy_static::lazy_static;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

use crate::println;

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

// Tests
#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}
