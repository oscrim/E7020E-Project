//! Debugging a crash (exception)
//!
//! Most crash conditions trigger a hard fault exception, whose handler is defined via
//! `exception!(HardFault, ..)`. The `HardFault` handler has access to the exception frame, a
//! snapshot of the CPU registers at the moment of the exception.
//!
//!
//! ``` text
//! (gdb) continue
//! Continuing.
//!
//! Program received signal SIGTRAP, Trace/breakpoint trap.
//! HardFault (ef=0x2000ffe0) at examples/crash.rs:82
//! 122         asm::bkpt();
//! (gdb) p/x *ef
//! $1 = cortex_m_rt::ExceptionFrame {
//!     r0: 0x2fffffff,
//!     r1: 0xf00000,
//!     r2: 0x20000000,
//!     r3: 0x0,
//!     r12: 0x0,
//!     lr: 0x8000699,
//!     pc: 0x800042c,
//!     xpsr: 0x61000000
//! }     
//! ```
//!
//! The program counter (pc) contains the address of the instruction that caused the exception. In GDB one can
//! disassemble the program around this address to observe the instruction that caused the
//! exception.
//!
//! ```
//! (gdb) disassemble ef.pc
//! Dump of assembler code for function main:
//!    0x08000428 <+0>:     mvn.w   r0, #3489660928 ; 0xd0000000
//!    0x0800042c <+4>:     ldr     r0, [r0, #0]
//!    0x0800042e <+6>:     b.n     0x800042e <main+6>
//! End of assembler dump.
//! ```
//!
//! `ldr r0, [r0, #0]` caused the exception. This instruction tried to load (read) a 32-bit word
//! from the address stored in the register `r0`. Looking again at the contents of `ExceptionFrame`
//! we find that `r0` contained the address `0x2FFF_FFFF` when this instruction was executed.
//!
//! We can further backtrace the calls leading up to the fault.
//! ``` text
//! (gdb) bt
//! #0  HardFault (ef=0x20017fe0) at examples/crash.rs:79
//! #1  <signal handler called>
//! #2  core::ptr::read_volatile (src=0x2fffffff) at libcore/ptr.rs:878
//! #3  main () at examples/crash.rs:68
//! ```

#![no_main]
#![no_std]

extern crate panic_halt;

use core::ptr;

use cortex_m_rt::{entry, exception};

#[entry]
#[inline(never)]
fn main() -> ! {
    unsafe {
        // read an address outside of the RAM region; this causes a HardFault exception
        ptr::read_volatile(0x2FFF_FFFF as *const u32);
    }

    loop {}
}

#[exception]
fn HardFault(ef: &cortex_m_rt::ExceptionFrame) -> ! {
    // (gdb) p/x *ef
    // prints the exception frame.
    // (gdb) backrace 
    // will give you the calls leading up to the error
    // (gdb) frame 3
    // will put you in the main frame
    // (gdb) disassemble
    // will show the assembly instruction causing the fault

    unsafe {
        ptr::read_volatile(ef);
    }

    panic!();
}
