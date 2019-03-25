//! bare3.rs
//!
//! String types in Rust
//!
//! What it covers:
//! - Types, str, arrays ([u8;uszie]), slices (&[u8])
//! - Iteration, copy
//! - Semihosting (tracing)

#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m_rt::entry;
use cortex_m_semihosting::{hprint, hprintln};

#[entry]
fn main() -> ! {
    hprintln!("bare3").unwrap();
    let s: &str = "ABCD";
    let bs: &[u8] = s.as_bytes();
    let _c: &[u8];
    let _i: i32;

    hprintln!("s = {}", s).unwrap();
    hprintln!("bs = {:?}", bs).unwrap();

    hprintln!("iterate over slice").unwrap();
    for _c in bs {
        hprint!("{},", _c).unwrap();
    }

    let a: [u8; 4] = [65u8; 4];
    let mut a = [0u8; 4];


    hprintln!("iterate iterate using (raw) indexing").unwrap();
    for _i in 0..s.len() {
        hprintln!("{},", bs[_i]).unwrap();
        //a[_i] = bs[_i];
    }

    hprintln!("").unwrap();

    a.clone_from_slice(&bs[..]);

    hprintln!("").unwrap();
    hprintln!("a = {}", core::str::from_utf8(&a).unwrap()).unwrap();

    loop {}
}

// 0. Build and run the application (debug build).
// 
//    > cargo build --example bare3
//    (or use the vscode build task)
//
// 1. What is the output in the `openocd` (Adapter Output) console?
//
//    bare3
// s = ABCD
// bs = [65, 66, 67, 68]
// iterate over slice
// 65,66,67,68,iterate iterate using (raw) indexing
// 65,
// 66,
// 67,
// 68,


// a = AAAA

//
//    What is the type of `s`?
//
//    &str
//
//    What is the type of `bs`?
//
//    it is a byte array &[u8]
//
//    What is the type of `c`?
//
//    it takes the type of the element in the array &[u8]
//
//    What is the type of `a`?
//
//    it's and array with 4 elements of type u8
//
//    What is the type of `i`?
//
//    i32
//
//    Commit your answers (bare3_1)
//
// 2. Make types of `s`, `bs`, `c`, `a`, `i` explicit.
//
//    Commit your answers (bare3_2)
//
// 3. Uncomment line `let mut a = [0u8; 4];
//`
//    Run the program, what happens and why?
//
//    the 65u8 part in the a decleration points to the letter A in the string. If you uncomment the new a decleration
//    it will overwrite a with the value of the 0u8 pointer. Which in this case is null.
//
//    Commit your answers (bare3_3)
//
// 4. Alter the program so that the data from `bs` is copied byte by byte into `a`.
//
//    Test that it works as intended.
//
//    Commit your answers (bare3_4)
//
// 5. Look for a way to make this copy done without a loop.
//    https://doc.rust-lang.org/std/primitive.slice.html
//
//    Implement and test your solution.
//
//    Commit your answers (bare3_5)
