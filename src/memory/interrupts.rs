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
//
// List of Interrupts
//
// 0x0        Division by 0                  [Fault]
// 0x1        Debug                          [Fault/Trap]
// 0x2        Non-maskable Interrupt         [Interrupt]
// 0x3        Breakpoint                     [Trap]
// 0x4        Overflow                       [Trap]
// 0x5        Bound Range Exceeded           [Fault]
// 0x6        Invalid Opcode                 [Fault]
// 0x7        Device Not Available           [Fault]
// 0x8        Double Fault                   [Abort] [Error Code Given]
// 0xA        Invalid TSS                    [Fault] [Error Code Given]
// 0xB        Segment Not Present            [Fault] [Error Code Given]
// 0xC        Stack-Segment Fault            [Fault] [Error Code Given]
// 0xD        General Protection Fault       [Fault] [Error Code Given]
// 0xE        Page Fault                     [Fault] [Error Code Given]
// 0XF        Reserved
// 0x10       x87 Floating-Point Exception   [Fault]
// 0x11       Alignment Check                [Fault] [Error Code Given]
// 0x12       Machine Check                  [Abort]
// 0x13       SIMD Floating-Point Exception  [Fault]
// 0x14       Virtualization Exception       [Fault]
// 0x15       Control Protection Exception   [Fault] [Error Code Given]
// 0x16-0x1B  Reserved
// 0x1C       Hypervisor Injection Exception [Fault]
// 0x1D       VMM Communication Exception    [Fault] [Error Code Given]
// 0x1E       Security Exception             [Fault] [Error Code Given]
// 0x1F       Reserved
// -          Triple Fault
//
// 64 bit TSS format
//
// Field                    Type
// Reserved                 u32
// Privilege Stack Table    [u64; 3]
// Reserved                 u64
// Interrupt Stack Table    [u64; 7]
// Reserved                 u64
// Reserved                 u16
// I/O Map Base Address     u16
//
// Breakpoints
//
// When the user sets a breakpoint, the debugger overwrites the corresponding instruction with
// the int3 instruction so that the CPU throws the breakpoint exception when it reaches that
// line. When the user wants to continue the program, the debugger replaces the in3 instruction
// with the original instruction again and continues the program.

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin;
use crate::{println, print, hlt_loop, memory::gdt};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
            idt.page_fault.set_handler_fn(page_fault_handler);
        }
        idt[InterruptIndex::Timer.as_usize()]
            .set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()]
            .set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

// Hardware Interrupt setup

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;
pub static PICS: spin::Mutex<ChainedPics> = spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}
impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

// Exception Handler
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}
extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    print!(".");
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use pc_keyboard::{layouts, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1,
                HandleControl::Ignore)
            );
    }

    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    crate::task::keyboard::add_scancode(scancode);
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}
extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();
}

// Tests
#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3(); // invoke a breakpoint exception
}
