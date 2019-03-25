//! bare0.rs
//!
//! Simple bare metal application
//! What it covers:
//! - constants
//! - global (static) variables
//! - checked vs. wrapping arithmetics
//! - safe and unsafe code
//! - making a safe API

// build without the Rust standard library
#![no_std]
// no standard main, we declare main using [entry]
#![no_main]

extern crate panic_halt;

// Minimal runtime / startup for Cortex-M microcontrollers
use cortex_m_rt::entry;

// a constant (cannot be changed at run-time)
const X_INIT: u32 = u32::max_value();

// global mutabale variables (changed using unsafe code)
static mut X: u32 = X_INIT;
static mut Y: u32 = 0;

#[entry]
fn main() -> ! {
    // local mutabale variable (changed in safe code)
    let t = read_x();
    let mut x = unsafe { X };


    loop {
        x = x.wrapping_add(1); // <- place breakpoint here (3)
        write_x(1);
        write_y(read_x());
    }
}

fn read_x() -> u32{
    unsafe {
        X
    }
}

fn write_x(i: u32) {
    unsafe{
        X = X.wrapping_add(i);
    }
}

fn read_y() -> u32{
    unsafe {
        Y
    }
}

fn write_y(i: u32) {
    unsafe{
        Y = i;
    }
}

// 0. Compile/build the example in debug (dev) mode.
//
//    > cargo build --example bare0
//    (or use the vscode build task)
//
// 1. Run the program in the debugger, let the program run for a while and
//    then press pause. Look in the (Local -vscode) Variables view what do you find.
//
//    64584532
//
//    In the Expressions (WATCH -vscode) view add X and Y
//    what do you find
//
//    Both X and Y is 1 less than the previous; 64584531 because they have yet to be updated
//
//    Step through one complete iteration of the loop
//    and see how the (Local) Variables are updated
//    can you foresee what will eventually happen?
//
// 	  Eventually the local variable x will reach an integer overflow and wrap. This will cause an integer overflow, panic ensues.
//
//    Commit your answers (bare0_1)
//
// 2. Alter the constant X_INIT so that `x += 1` directly causes `x` to wrap
// 	  what happens when `x` wraps
//
//    core panic due to int overflow
//
//    Commit your answers (bare0_2)
//
// 3. Place a breakpoint at `x += 1`
//
//    Change (both) += opertions to use wrapping_add
//    load and run the progam, what happens
//    When the overflow should occur the varriable instead resets back to 0
//
//    Now continue exectution, what happens
//    it goes back to 0 then keeps adding 1 like before
//
//    Commit your answers (bare0_3)
//
//    (If the program did not succeed back to the breakpoint
//    you have some fault in the program and go back to 3.)
//
// 4. Change the asserion to `assert!(x == X && X == Y + 1)`, what happens?
//
//    Core panic when the assert fails.
//
//    Commit your answers (bare0_4)
//
// 5. Remove the assertion and implement "safe" functions for
//    reading and writing X and Y
//    e.g. read_x, read_y, write_x, write_y
//
//    Rewrite the program to use ONLY "safe" code besides the
//    read/write functions (which are internally "unsafe")
//
//    Commit your solution (bare0_5)
//
// 6. *Optional
//    Implement a read_u32/write_u32, taking a reference to a
//    "static" variable
//
//    Rewrite the program to use this abstraction instead of "read_x", etc.
//
//    Commit your solution (bare0_6)
//
