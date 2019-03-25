//! bare1.rs
//!
//! Inspecting the generated assembly
//!
//! What it covers
//! - tracing over semihosting and ITM
//! - assembly calls and inline assembly
//! - more on arithmetics

#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m::{iprintln, Peripherals};
use cortex_m_rt::entry;


#[entry]
#[inline(never)]
fn main() -> ! {
    // Prepend by `x` by _ to avoid warning (never used).
    // The compiler is smart enough to figure out that
    // `x` is not used in any menaningful way.

    let mut _x = 0;
    loop {
        _x += 1;
        cortex_m::asm::nop();
        cortex_m::asm::bkpt();
        _x -= 1;
    }
}

// 0. Setup
//    For this example we will use the `nightly` compiler
//    to get inline assembly.
//    (Inline assembly is currently not stabelized.)
//
//    > rustup override set nightly
//
//    In the `Corgo.toml` file, uncomment
//    # features = ["inline-asm"] # <- currently requires nightly compiler
//
//    You may need/want to install addititonal components also,
//    to that end look at the install section in the README.md.
//    If you change toolchain, exit and re-start `vscode`.
//
// 1. Build and run the application
//
//    > cargo build --example bare1
//    (or use the vscode build task)
//
//    Look at the `hello.rs` and `itm.rs` examples to setup the tracing.
//
//    When debugging the application it should get stuck in the
//    loop, (press pause/suspend to verify this).
//    what is the output in the ITM console
//
//    no output in itm console
//
//    What is the output in the semihosting (openocd) console
//    it says semihosting is enabled but nothing out of the usual
//
//    Commit your answers (bare1_1)
//
// 2. Inspecting the generated assembly code
//    If in `vcsode` the gdb console in DEBUG CONSOLE
//
//    What is the output of:
//    (gdb) disassemble
//
// Dump of assembler code for function main:
//    0x08000400 <+0>:	sub	sp, #16
//    0x08000402 <+2>:	movs	r0, #0
//    0x08000404 <+4>:	str	r0, [sp, #12]
//    0x08000406 <+6>:	b.n	0x8000408 <main+8>
// => 0x08000408 <+8>:	ldr	r0, [sp, #12]
//    0x0800040a <+10>:	adds	r1, r0, #1
//    0x0800040c <+12>:	mov	r2, r1
//    0x0800040e <+14>:	cmp	r1, r0
//    0x08000410 <+16>:	str	r2, [sp, #8]
//    0x08000412 <+18>:	bvs.n	0x800042c <main+44>
//    0x08000414 <+20>:	b.n	0x8000416 <main+22>
//    0x08000416 <+22>:	ldr	r0, [sp, #8]
//    0x08000418 <+24>:	str	r0, [sp, #12]
//    0x0800041a <+26>:	ldr	r1, [sp, #12]
//    0x0800041c <+28>:	subs	r2, r1, #1
//    0x0800041e <+30>:	cmp	r1, #1
//    0x08000420 <+32>:	str	r2, [sp, #4]
//    0x08000422 <+34>:	bvs.n	0x800043a <main+58>
//    0x08000424 <+36>:	b.n	0x8000426 <main+38>
//    0x08000426 <+38>:	ldr	r0, [sp, #4]
//    0x08000428 <+40>:	str	r0, [sp, #12]
//    0x0800042a <+42>:	b.n	0x8000408 <main+8>
//    0x0800042c <+44>:	movw	r0, #2268	; 0x8dc
//    0x08000430 <+48>:	movt	r0, #2048	; 0x800
//    0x08000434 <+52>:	bl	0x800045c <panic>
//    0x08000438 <+56>:	udf	#254	; 0xfe
//    0x0800043a <+58>:	movw	r0, #2340	; 0x924
//    0x0800043e <+62>:	movt	r0, #2048	; 0x800
//    0x08000442 <+66>:	bl	0x800045c <panic>
//    0x08000446 <+70>:	udf	#254	; 0xfe
// End of assembler dump.
//    Commit your answers (bare1_2)
//
// 3. Now remove the comment for `cortex_m::asm::nop()`.
//    Rebuild and debug, pause the program.
//
//    What is the output of:
//    (gdb) disassemble
//
// Dump of assembler code for function main:
//    0x0800040a <+0>:	sub	sp, #16
//    0x0800040c <+2>:	movs	r0, #0
//    0x0800040e <+4>:	str	r0, [sp, #12]
//    0x08000410 <+6>:	b.n	0x8000412 <main+8>
//    0x08000412 <+8>:	ldr	r0, [sp, #12]
//    0x08000414 <+10>:	adds	r1, r0, #1
//    0x08000416 <+12>:	mov	r2, r1
//    0x08000418 <+14>:	cmp	r1, r0
//    0x0800041a <+16>:	str	r2, [sp, #8]
//    0x0800041c <+18>:	bvs.n	0x800043c <main+50>
//    0x0800041e <+20>:	b.n	0x8000420 <main+22>
//    0x08000420 <+22>:	ldr	r0, [sp, #8]
//    0x08000422 <+24>:	str	r0, [sp, #12]
// => 0x08000424 <+26>:	bl	0x8000400 <cortex_m::asm::nop>
//    0x08000428 <+30>:	b.n	0x800042a <main+32>
//    0x0800042a <+32>:	ldr	r0, [sp, #12]
//    0x0800042c <+34>:	subs	r1, r0, #1
//    0x0800042e <+36>:	cmp	r0, #1
//    0x08000430 <+38>:	str	r1, [sp, #4]
//    0x08000432 <+40>:	bvs.n	0x800044a <main+64>
//    0x08000434 <+42>:	b.n	0x8000436 <main+44>
//    0x08000436 <+44>:	ldr	r0, [sp, #4]
//    0x08000438 <+46>:	str	r0, [sp, #12]
//    0x0800043a <+48>:	b.n	0x8000412 <main+8>
//    0x0800043c <+50>:	movw	r0, #2284	; 0x8ec
//    0x08000440 <+54>:	movt	r0, #2048	; 0x800
//    0x08000444 <+58>:	bl	0x800046e <panic>
//    0x08000448 <+62>:	udf	#254	; 0xfe
//    0x0800044a <+64>:	movw	r0, #2356	; 0x934
//    0x0800044e <+68>:	movt	r0, #2048	; 0x800
//    0x08000452 <+72>:	bl	0x800046e <panic>
//    0x08000456 <+76>:	udf	#254	; 0xfe
// End of assembler dump.
//
//    Commit your answers (bare1_3)
//
// 4. Now remove the comment for `cortex_m::asm::bkpt()`
//    Rebuild and debug, let the program run until it halts.
//
//    What is the output of:
//    (gdb) disassemble
//
//    Dump of assembler code for function main:
//    0x0800040a <+0>:	sub	sp, #16
//    0x0800040c <+2>:	movs	r0, #0
//    0x0800040e <+4>:	str	r0, [sp, #12]
//    0x08000410 <+6>:	b.n	0x8000412 <main+8>
//    0x08000412 <+8>:	ldr	r0, [sp, #12]
//    0x08000414 <+10>:	adds	r1, r0, #1
//    0x08000416 <+12>:	mov	r2, r1
//    0x08000418 <+14>:	cmp	r1, r0
//    0x0800041a <+16>:	str	r2, [sp, #8]
//    0x0800041c <+18>:	bvs.n	0x8000442 <main+56>
//    0x0800041e <+20>:	b.n	0x8000420 <main+22>
//    0x08000420 <+22>:	ldr	r0, [sp, #8]
//    0x08000422 <+24>:	str	r0, [sp, #12]
//    0x08000424 <+26>:	bl	0x8000400 <cortex_m::asm::nop>
//    0x08000428 <+30>:	b.n	0x800042a <main+32>
//    0x0800042a <+32>:	bl	0x800045e <__bkpt>
// => 0x0800042e <+36>:	b.n	0x8000430 <main+38>
//    0x08000430 <+38>:	ldr	r0, [sp, #12]
//    0x08000432 <+40>:	subs	r1, r0, #1
//    0x08000434 <+42>:	cmp	r0, #1
//    0x08000436 <+44>:	str	r1, [sp, #4]
//    0x08000438 <+46>:	bvs.n	0x8000450 <main+70>
//    0x0800043a <+48>:	b.n	0x800043c <main+50>
//    0x0800043c <+50>:	ldr	r0, [sp, #4]
//    0x0800043e <+52>:	str	r0, [sp, #12]
//    0x08000440 <+54>:	b.n	0x8000412 <main+8>
//    0x08000442 <+56>:	movw	r0, #2284	; 0x8ec
//    0x08000446 <+60>:	movt	r0, #2048	; 0x800
//    0x0800044a <+64>:	bl	0x8000478 <panic>
//    0x0800044e <+68>:	udf	#254	; 0xfe
//    0x08000450 <+70>:	movw	r0, #2356	; 0x934
//    0x08000454 <+74>:	movt	r0, #2048	; 0x800
//    0x08000458 <+78>:	bl	0x8000478 <panic>
//    0x0800045c <+82>:	udf	#254	; 0xfe
// End of assembler dump.

//
//    Commit your answers (bare1_4)
//
// 5. Release mode (optimized builds).
//    Rebuild `bare1.rs` in release (optimized mode).
//  
//    > cargo build --example bare1 --release
//    (or using the vscode build task)
//
//    Compare the generated assembly for the loop
//    between the dev (unoptimized) and release (optimized) build.
//
//    Dump of assembler code for function main:
//    0x08000400 <+0>:	bl	0x800067a <__nop>
//    0x08000404 <+4>:	bl	0x8000676 <__bkpt>
// => 0x08000408 <+8>:	b.n	0x8000400 <main>
// End of assembler dump.
//
//    commit your answers (bare1_5)
//
//    Tips: The optimized build should have 3 instructions
//    while the debug (dev) build should have > 20 instructions
//    (both counting the inner loop only). The debug build
//    should have additional code that call panic if the additon
//    wraps (and in such case call panic).
//
//    Discussion:
//    In release (optimized) mode the addition is unchecked,
//    so there is a semantic difference here in between
//    the dev and release modes. This is motivited by:
//    1) efficiency, unchecked is faster
//    2) convenience, it would be inconvenient to explicitly use
//    wrapping arithmetics, and wrapping is what the programmer
//    typically would expect in any case. So the check
//    in dev/debug mode is just there for some extra safety
//    if your intention is NON-wrapping arithmetics.
//
// 6. *Optional
//    You can pass additional flags to the Rust `rustc` compiler.
//
//    `-Z force-overflow-checks=off`
//
//    Under this flag, code is never generated for oveflow checking.
//    You can enable this flag (uncomment the corresponding flag in
//    the `.cargo/config` file.)
//
//    What is now the disassembly of the loop (in debug mode):
//
//    ** your answer here **
//
//    commit your answers (bare1_6)
//
//    Now restore the `.cargo/config` to its original state.
//
// 7. *Optional
//    There is another way to conveniently use wrapping arithmetics
//    without passing flags to the compiler.
//
//    https://doc.rust-lang.org/std/num/struct.Wrapping.html
//
//    Rewrite the code using this approach.
//
//    What is now the disassembly of the code in dev mode?
//
//    ** your answer here **
//
//    What is now the disassembly of the code in release mode?
//
//    ** your answer here **
//
//    commit your answers (bare1_7)
//
//    Final discussion:
//
//    Embedded code typically is performance sensitve, hence
//    it is important to understand how code is generated
//    to achieve efficient implementations.
//
//    Moreover, arithmetics are key to processing of data,
//    so its important that we are in control over the
//    computations. E.g. comupting checksums, hashes, cryptos etc.
//    all require precise control over wrapping vs. overflow behaviour.
//
//    If you write a library depending on wrapping arithmetics
//    do NOT rely on a compiler flag. (The end user might compile
//    it without this flag enabled, and thus get erronous results.)
//
//    NOTICE:
//    ------
//    You are now on a `nightly` release of the compiler for good and bad.
//    You can chose to switch back to the stable channel. If so you must
//    restore the `Cargo.toml` (comment out the `features = ["inline-asm"]`)
//
//    Pros and cons of nightly:
//    + Acccess to new Rust features (such as inline assembly)
//    - No guarantee these features will work, they might change semantics,
//      or even be revoked.
//
//    The compiler itself is the same, the stable release is just a snapchot
//    of the nightly (released each 6 week). It is the latest nightly
//    that passed some additional regression test, not a different compiler.
//    And of course, the stable has the experimental features disabled.
//
//    So its up to you to decide if you want to use the stable or nightly.
