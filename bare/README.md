# `app`

> Examples and exercises for the Nucleo STM32F401re/STM32F11re devkits.

---

## Dependencies


- Rust 1.32, or later.

- `rust-std` components (pre-compiled `core` crate) for the ARM Cortex-M
  targets. Run:

``` console
$ rustup target add thumbv7em-none-eabihf
```

- For programming (flashing) and debugging
  - `openocd` (install using your package manager)
  - `arm-none-eabi` toolchain (install using your package manager). In the following we refer the `arm-none-eabi-gdb` as just `gdb` for brevity.

- `st-flash` (for low level access to the MCU flash)
- `itmdump` (for ITM trace output)
- `vscode` and `cortex-debug` (optional for an integrated debugging experience)

* https://marketplace.visualstudio.com/items?itemName=marus25.cortex-debug

---

## Examples

---

### Hello World! Building and Debugging an Application

1. Connect your devkit using USB. To check that it is found you can run:

``` console
$ lsusb
...
Bus 001 Device 004: ID 0483:374b STMicroelectronics ST-LINK/V2.1
...
```

(Bus/Device/ID may vary.)

2. In a terminal in the `app` folder run:

``` console
$ cargo build --example hello
$ openocd -f openocd.cfg
...
Info : Listening on port 6666 for tcl connections
Info : Listening on port 4444 for telnet connections
Info : clock speed 2000 kHz
Info : STLINK V2J20M4 (API v2) VID:PID 0483:374B
Info : Target voltage: 3.254773
Info : stm32f4x.cpu: hardware has 6 breakpoints, 4 watchpoints
Info : Listening on port 3333 for gdb connections
```

`openocd` should connect to your target using the `stlink` programmer (onboard your Nucleo devkit). See the `Trouble Shooting` section if you run into trouble. 

3. In another terminal (in the same `app` folder) run:

``` console
$ arm-none-eabi-gdb target/thumbv7em-none-eabihf/debug/examples/hello -x openocd.gdb
```

This starts gdb with `file` being the `hello` (elf) binary, and runs the `openocd.gdb` script, which loads (flashes) the binary to the target (our devkit). The script connects to the `openocd` server, sets `breakpoint`s at `main` (as well as some exception handlers, more on those later), enables `semihosting`, loads the binary and finally runs the first intsruction (`stepi`).

4. You can now continue debugging of the program:

``` console
(gdb) c
Continuing.

Breakpoint 3, main () at examples/hello.rs:13
13          hprintln!("Hello, world!").unwrap();
```

The `cortex-m-rt` run-time initializes the system and your global variables (in this case there are none). After that it calls the `[entry]` function. Here you hit a breakpoint.

5. You can contine debugging:

``` console
(gdb) c
Continuing.
halted: PC: 0x08000608
```

At this point, the `openocd` terminal should read:

``` console
Info : halted: PC: 0x08000608
Hello, world!
```

Your program is now stuck in an infinite loop (doing nothing).

6. Press `CTRL-c` in the `gdb` terminal:

``` console
Program received signal SIGINT, Interrupt.
0x08000624 in main () at examples/hello.rs:14
14          loop {}
(gdb)
```

You have now compiled and debugged a minimal Rust `hello` example. `gdb` is a very useful tool so lookup some tutorials/docs (e.g., https://sourceware.org/gdb/onlinedocs/gdb/), a Cheat Sheet can be found at https://darkdust.net/files/GDB%20Cheat%20Sheet.pdf.

---

### ITM Tracing

The `hello.rs` example uses the `semihosting` interface to emit the trace information (appearing in the `openocd` terminal). The drawback is that `semihosting` is incredibly slow as it involves a lot of machinery to process each character. (Essentially, it writes a character to a given position in memory, runs a dedicated break instruction, `openocd` detecects the break, reads the character at the given postition in memory and emits the character to the console.)

A better approach is to use de bultin in ITM (Instrumentation Trace Macrocell), designed to more efficently implement tracing. The onboard `stlink` programmer can put up to 4 characters into an ITM package, and transmit that to the host (`openocd`). `openocd` can process the incoming data and send it to a file or FIFO queue. The ITM package stream needs to be decoded (header + data). To this end we use the `itmdump` tool.

In a separate terminal:

``` console
$ mkfifo /tmp/itm.log
$ itmdump -f /tmp/itm.log -F
```

Now you can compile and run the `itm.rs` application using the same steps as the `hello` program. In the `itmdump` console you should now have the trace output.

``` console
$ mkfifo /tmp/itm.log
$ itmdump -f /tmp/itm.log -F
Hello, world!
```

Under the hood there is much less overhead, the serial transfer rate is set to 2MBit in between the ITM (inside of the MCU) and `stlink` programmer (onboard the Nucleo devkit). So in theory we can transmit some 200kByte/s data over ITM. However, we are limited by the USB interonnection and `openocd` to recieve and forward packages.

The `stlink` programmer, buffers packages but has limited buffer space. Hence in practise, you should keep tracing to short messages, else the buffer will overflow. See trouble shooting section if you run into trouble.

---

### Rust `panic` Handling

The `rust` compiler statically analyses your code, but in cases some errors cannot be detected at compile time (e.g., array indexing out of bounds, division by zore etc.). The `rust` compiler generates code checking such faults at run-time, instead of just crashing (or even worse, continuing with faulty/undefined values like a `C` program would) . A fault in Rust will render a `panic`, whith an associated error message (useful to debugging the application). We can choose how such `panic`s should be treated, e.g., transmitting the error message using `semihosting`, `ITM`, some other channel (e.g. a serial port), or simply aborting the program.

The `panic` example demonstrates some possible use cases.

The `openocd.gdb` script sets a breakpoint at `rust_begin_unwind` (a function in the  `rust core` library, used to recover errors.)

When running the example (see above howto compile and run), the `gdb` terminal will show:

``` console
...
Breakpoint 2, main () at examples/panic.rs:27
27          panic!("Oops")
(gdb) c
Continuing.
halted: PC: 0x08000404

Breakpoint 1, rust_begin_unwind (_info=0x20017fb4)
    at /home/pln/.cargo/registry/src/github.com-1ecc6299db9ec823/panic-halt-0.2.0/src/lib.rs:33
33              atomic::compiler_fence(Ordering::SeqCst);
(gdb) p *_info
$1 = core::panic::PanicInfo {payload: core::any::&Any {pointer: 0x8000760 <.Lanon.21a036e607595cc96ffa1870690e4414.142> "\017\004\000", vtable: 0x8000760 <.Lanon.21a036e607595cc96ffa1870690e4414.142>}, message: core::option::Option<&core::fmt::Arguments>::Some(0x20017fd0), location: core::panic::Location {file: <error reading variable>, line: 27, col: 5}}
```

Here `p *_info` prints the arument to `rust_begin_unwind`, at the far end you will find `line: 27, col 5`, which correstponds to the source code calling `panic("Ooops")`. (`gdb` is not (yet) Rust aware enough to figure out how the `file` field should be interpreted, but at least we get some useful information).

Alternatively we can trace the panic message over `semihosting` (comment out `extern crate panic_halt` and uncomment `extern crate panic_semihosting`).

The `openocd` console should now show:

``` console
Info : halted: PC: 0x080011a0
panicked at 'Oops', examples/panic.rs:27:5
```

Under the hood, this approach involves *formatting* of the panic message, which implementation occupies a bit of flash memory (in our case we have 512kB so plenty enough, but for the smallest of MCUs this may be a problem). Another drawback is that it requires a debugger to be connected and active.

Another alternative is to use ITM (uncomment `extern crate panic_itm`), this is faster, but be aware, the message may overflow the `ITM` buffer, so it may be unreliable. Also it assumes, that the ITM stream is actively monitored.

A third alternative would be to store the panic message in some non-volatile memory (flash, eeprom, etc.). This allows for true post-mortem debugging of a unit put in production. This approach is used e.g. in automotive applications where the workshop can read-out error codes of your vehicle.

---

### Exception Handling and Core Peripheral Access

The ARM Cortex-M processors features a set of *core* peripherpherals and *exception* handlers. These offer basic functionality independent of vendor (NXP, STM, ...). The `SysTick` perihperal is a 24-bit countdown timer, that raises a `SysTick` exception when hitting 0 and reloads the set value. Seen as a real-time system, we can dispatch the `SysTick` task in a periodic fashion (without accumulated drift under some additional constraints).

In the `exception.rs` example  a `.` is emitted by the `SysTick` handler using `semihosting`. Running the example should give you a periodic updated of the `openocd` console.

The `exception_itm.rs` and `exception_itm_raw.rs` uses the ITM instead. The difference is the way
they gain access to the `ITM` periphereal. In the first case we *steal* the whole set of core peripherals (stealing works only in `--release` mode), while the in the second case we use *raw* pointer access to the `ITM`. In both cases, the code is *unsafe*, as there is no guarantee that other tasks may access the peripheral simultaneously (causing a conflict/race). Later we will see how the concurrency problem is solved in RTFM to offer safe access to peripherals.

---

### Crash - Analysing the Exception Frame

In case the execution of an intstruction fails, a `HardFault` exception is raised by the hardware, and the `HardFault` handler is executed. We can define our own handler as in example `crash.rs`. In `main` we attempt to read an illegal address, causing a `HardFault`, and we hit a breakpoint (`openocd.gdb` script sets a breakbpoint at the `HardFualt` handler). From there you can print the exception frame, reflecting the state of the MCU when the error occured. You can use `gdb` to give a `back trace` of the call-stack leading up to the error. See the example for detailed information.

---

### Device Crates and System View Descriptions (SVDs)

Besides the ARM provided *core* peripherals the STM32F401re/STM32F411re MCUs has numerous vendor specific peripherals (GPIOs, Timers, USARTs etc.). The vendor provides a System View Description (SVD) specifying the register block layouts (fields, enumerated values, etc.). Using the `svd2rust` tool we can derive a `Peripheral Access Crate` (PAC) providing an API for the device that allow us to access each register according to the vendors specification. The `device.rs` example showcase how a PAC for the  STM32F401re/STM32F411re MCUs can be added. (These MCUs have the same set of peripherals, only the the maximum clock rating differs.) Here we use the STM32F413 as that covers the STM32F401/411/413 devices.

The example output a `.` each second over `semihosting` and `ITM`.

Looking at the `Corgo.toml` file we find:

``` toml
[package]
authors = ["Per Lindgren <per.lindgren@ltu.se>"]
edition = "2018"
readme = "README.md"
name = "app"
version = "0.1.0"

[dependencies]
cortex-m-rt = "0.6.7"
cortex-m-semihosting = "0.3.2"

# panic-abort = "0.3.1" # requires nightly toolchain
panic-halt = "0.2.0"
panic-semihosting = "0.5.1"
panic-itm = "0.4.0"

bare-metal = "0.2.4"
nb = "0.1.1"
heapless = "0.4.1"

[dependencies.cortex-m-rtfm]
version = "0.4.0"
optional = true

[dependencies.cortex-m]
version = "0.5.8"
features = ["inline-asm"] # <- currently requires nightly compiler

# Uncomment for the allocator example.
# alloc-cortex-m = "0.3.5"

[dependencies.stm32f4]
version = "0.5.0"
features = ["stm32f413", "rt"]
optional = true

[dependencies.stm32f4xx-hal]
git = "https://github.com/stm32-rs/stm32f4xx-hal.git"
version = "0.2.8"
features = ["stm32f413", "rt"]
optional = true

[features]
pac = ["stm32f4"]
hal = ["stm32f4xx-hal"]
rtfm = ["cortex-m-rtfm"]
rtfm-tq = ["cortex-m-rtfm/timer-queue"]

# this lets you use `cargo fix`!
[[bin]]
name = "app"
test = false
bench = false

[profile.release]
incremental = false # disable incremental build to allow lto on nightly
codegen-units = 1   # better optimizations
debug = true        # symbols are nice and they don't increase the size on Flash
lto = true          # better optimizations
```

We compile `stm32f4` (a generic library for all STMF4 MCUs) with `features = ["stm32f413", "rt"]`, which indicates the specific MCU with `rt` (run-time support) enabled. By having the PAC as an optional dependency, we did not need to compile it (unless we need it, and as you might have experienced already compiling the PAC takes a bit of time to compile initially). (An SVD file is typically > 50k lines, amounting to the same (or more) lines of Rust code.)

---

### Hardware Abstraction Layer

For convenience common functionality can be implemented for a specific MCU (or family of MCUs). The `stm32f4xx-hal` is a Work In Progress, implementing a Hardware Abstraction Layer (hal) for the `stm32f4` family. It implements the `https://crates.io/search?q=embedded-hal` serial trait (interface), to read and write single bytes over a serial port. However, setting up communication is out of scope for the `embedded-hal`.

The `serial.rs` example showcase a simple echo application,
repeating back incoming data over a serial port (byte by byte). You will also get trace output over the ITM.

Looking closer at the example, `rcc` is a *singleton* (`constrain` consumes the `RCC` and returns a singleton. The `freeze` consumes the  singleton (`rcc`) and sets the MCU clock tree according to the (default) `cfgr`. (Later in the exercises you will change this.)

This pattern ensures that the clock configuration will remain unchanged (the `freeze` function cannot be called again, as the `rcc` is consumed, also you cannot get a new `rcc` as the `RCC` was consumed by `contstrain`).

Why is this important you may ask? Well, this *pattern* allows the compiler to check and ensure that your code (or some library that you use) does not make changes to the system (in this case the clocking), wich reduces the risk of errors and improves robustness.

Similarly, we `split` the `GPIOA` into its parts (pins), and select the operating mode to `af7` for `tx` (the transmit pin `pa2`), and `rx` (the receive pin `pa3`). For details see, RM0368, figure 17 (page 151), and table 9 in STM32F401xD STM32F401xE. The GPIO pins `pa2` and `pa3` are (by default) connected to the `stlink` programmer, see section 6.8 of the Nucleo64 user manual `UM1724`. When the `stlink` programmer is connected to a linux host, the device `\dev\ttyACM0` appears as a *virtual com port* (connected to `pa2`/`pa3` by default).

Now we can call `Serial::usart2` to setup the serial communication, (according to table 9 in STM32F401xD STM32F401xE documentation it is USART2).

Following the *singleton* pattern it consumes the `USART2` peripheral (to ensure that only one configuration can be active at any time). The second parameter is the pair of pins `(tx, px)` that we setup earlier.
The third parameter is the USART configuration. By defualt its set to 8 bit data, and one stop bit. We set the baudrate to 115200. We also pass the `clocks` (holding information about the MCU clock setup).

At this point `tx` and `rx` is owned by `serial`. We can get access to them again by `serial.split()`.

In the loop we match on the result of `block!(rx.read())`. `block!` repeatedly calls `rx.read()` until ether a byte is received or an error returned. In case `rx.read()` succeeded, we trace the received byte over the ITM, and echo it by `tx.write(byte)`, ignoring the result (we just assume sending will always succeed). In case `rx.read` returned with an error, we trace the error message.

As the underlying hardware implementation buffers only a single byte, the input buffer may overflow (resulting in `tx.read` returning an error).

You can now compile and run the example. Start `moserial` (or some other serial communication tool), connect to `/dev/ttyACM0` with 115200 8N1). Now write a single character `a` in the `Outgoing` pane, followed by pressing <Enter>. You should receive back an `a` from the target. In the ITM trace output you should see:

``` console
[2019-01-08T10:31:36.867Z]   Ok 97. 
```

Depending if <Enter> was encoded as CR+LF, CR, LF, TAB, ... you will get additional bytes sent (and received). Try sending multiple characters at once, e.g. `abcd`, you will see that the you well get a buffer overflow.

This is an example of a bad programming pattern, typically leading to serious problems in (real-time) embedded programming (so it takes more than just Rust to get it right). Later in the exercises we will see how better patterns can be adopted.

---

### RTFM Blinky

No example suit is complete without the mandatory `blinky` demo, so here we go! The exampe `rtfm_blinky.rs` showcase the simplicity of using the RTFM framework.

An application using RTFM is defined using the `app` attribute, specifying the target PAC.

- The `static mut` section defines the shared resources. In this example we need to access the `GPIOA` in the `SysTick` exception handler. Initially all resources are delegated to the `init` task (running before the system goes live).

- In `init` we:
  - configure the `SysTick` exception/task to be triggered periodically (each second),- configure the `RCC` (enabling the `GPIOA` peripheral),
  - configure the `PA5` pin (connected to the Green LED on the Nucleo) as an output, and finally
  - delegate access to `GPIO` as a "late" resource.

- The `#[exception (resources = [GPIOA])]` attribute enables access to the `GPIOA` peripheral for the `SysTick` exception/task. (Resource access goes through `resources.<RESOURCE>`.)

- In `SysTick` we define a task local resource `TOGGLE` to hold the current state, and we set/clear the `PA5` pin correspondingly. (The `bs5` field sets the `PA5` high, while `br5` clears the corresponding bit controlling the led.) Finnally the `TOGGLE` state is inverted.

Notice here, under the hood the RTFM framework analyses the `app` and concludes that `TOGGLE` and `GPIOA` are accessible (in scope) for `SysTick` exclusively, so we safely (without running any risk of race conditions) access the resources directly.

Side note: Accessing a resource shared with a higher priority task, requires the user to lock the resource in order to guarantee race-free access. For further information, see the RTFM API documentation and book, cf. `cortex-m-rtfm`.

``` rust
use rtfm::app;

#[app(device = stm32f4::stm32f413)]
const APP: () = {
    // late resorce binding
    static mut GPIOA: GPIOA = ();

    // init runs in an interrupt free section
    #[init]
    fn init() {
        // configures the system timer to trigger a SysTick exception every second
        core.SYST.set_clock_source(SystClkSource::Core);
        core.SYST.set_reload(16_000_000); // period = 1s
        core.SYST.enable_counter();
        core.SYST.enable_interrupt();

        // power on GPIOA, RM0368 6.3.11
        device.RCC.ahb1enr.modify(|_, w| w.gpioaen().set_bit());
        // configure PA5 as output, RM0368 8.4.1
        device.GPIOA.moder.modify(|_, w| w.moder5().bits(1));

        // pass on late resources
        GPIOA = device.GPIOA;
    }

    #[exception (resources = [GPIOA])]
    fn SysTick() {
        static mut TOGGLE: bool = false;

        if *TOGGLE {
            resources.GPIOA.bsrr.write(|w| w.bs5().set_bit());
        } else {
            resources.GPIOA.bsrr.write(|w| w.br5().set_bit());
        }

        *TOGGLE = !*TOGGLE;
    }
};
```

---

## Trouble Shooting

Working with embedded targets involves a lot of tooling, and many things can go wrong.

---

### `openocd` fails to connect

If you end up with a program that puts the MCU in a bad state.

- Hold the `RESET` button (black), while starting `openocd`. If that does not work, disconnect the USB cable, hold the `RESET` button, re-connect the USB, start `openocd` then then let go of the button.

- However even a reset might not help you. In that case you can erase the flash memory. `st-flash` connects to the target directly (bypassing `gdb` and `openocd`) and hence more likely to get access to the target even if its in a bad state.

``` console
> st-flash erase
```

- Make sure that the `st-link` firmare is up to date, you can use the java application: https://my.st.com/content/my_st_com/en/products/development-tools/software-development-tools/stm32-software-development-tools/stm32-programmers/stsw-link007.license=1549034381973.product=STSW-LINK007.version=2.33.25.html, to check/update the firmware. (Current verison is 2.33.25)


---

### `gdb` fails to connect

`openocd` acts as a *gdb server*, while `gdb` is a *gdb client*. By default they connect over port `:3333` (: indicates that the port is on the *localhost*, not a remote connection). In cases you might have another `gdb` connection blocking the port.

``` console
$ ps -all
F S   UID   PID  PPID  C PRI  NI ADDR SZ WCHAN  TTY          TIME CMD
0 S  1000  5659  8712  0  80   0 -  6139 -      pts/1    00:00:28 openocd
0 S  1000  7549 16215  0  80   0 - 25930 se_sys pts/4    00:00:00 arm-none-eabi-g
...
```

In this case you can try killing `gdb` by:

``` console
$ kill -9 7549
```

or even

``` console
$ killall -9 arm-none-eabi-g
```

Notice, the process name is truncated for some reason...

If this did not help you can check if some other client has aquired the port, and kill the intruder accordingly. (In this case it was the gdb process so the above method would have worked, but in general it could be another process blocking the port.)

``` console
$ lsof -i :3333
COMMAND    PID USER   FD   TYPE DEVICE SIZE/OFF NODE NAME
openocd   5659  pln   12u  IPv4 387143      0t0  TCP localhost:dec-notes (LISTEN)
openocd   5659  pln   13u  IPv4 439988      0t0  TCP localhost:dec-notes->localhost:59560 (ESTABLISHED)
arm-none- 7825  pln   14u  IPv4 442734      0t0  TCP localhost:59560->localhost:dec-notes (ESTABLISHED)
$ kill -9 7825
```

---

### `itmdump` no tracing or faulty output

There can be a number of reasons ITM tracing fails.

- The `openocd.gdb` script enables ITM tracing assuming the `/tmp/itm.log` and `itmdump` has been correctly setup before `gdb` is launched (and the script run). So the first thing is to check that you follow the sequence suggested above.

- `openocd.gdb`sets enables ITM tracing by:

``` txt
# 16000000 must match the core clock frequency
monitor tpiu config internal /tmp/itm.log uart off 16000000
monitor itm port 0 on
```

The transfer speed (baud rate) is automatically negotiated, however you can set it explicitly (maximum 2000000).  

``` txt
monitor tpiu config internal /tmp/itm.log uart off 16000000 2000000
```

You may try a lower value.

- The `stm32f401re/stm32f411re` defaults to 16000000 (16MHz) as the core clock frequency, based on an internal oscillator. If your application sets another core clock frequency the `openocd.gdb` script (`tpiu` setting) must be changed accordingly.

- `openocd` implements a number of `events` which might be called by `gdb`, e.g.:

``` console
(gdb) monitor reset init
adapter speed: 2000 kHz
target halted due to debug-request, current mode: Thread
xPSR: 0x01000000 pc: 0x08001298 msp: 0x20018000, semihosting
adapter speed: 8000 kHz
```

This invokes the `init` event, which sets the core clock to 64MHz. If you intend to run the MCU at 64MHz (using this approach), ITM will not work unless the `tpiu` setting matches 64MHz.

If you on the other hand want to use `monitor reset init` but not having the core clock set to 64MHz, you can use a custom `stlink.cfg` (instead of the one shipped with `openocd`). The original looks like this:

``` txt
...
$_TARGETNAME configure -event reset-init {
	# Configure PLL to boost clock to HSI x 4 (64 MHz)
	mww 0x40023804 0x08012008   ;# RCC_PLLCFGR 16 Mhz /8 (M) * 128 (N) /4(P)
	mww 0x40023C00 0x00000102   ;# FLASH_ACR = PRFTBE | 2(Latency)
	mmw 0x40023800 0x01000000 0 ;# RCC_CR |= PLLON
	sleep 10                    ;# Wait for PLL to lock
	mmw 0x40023808 0x00001000 0 ;# RCC_CFGR |= RCC_CFGR_PPRE1_DIV2
	mmw 0x40023808 0x00000002 0 ;# RCC_CFGR |= RCC_CFGR_SW_PLL

	# Boost JTAG frequency
	adapter_khz 8000
}
```

The clock configuration can be commented out:

``` txt
...
$_TARGETNAME configure -event reset-init {
	# # Configure PLL to boost clock to HSI x 4 (64 MHz)
	# mww 0x40023804 0x08012008   ;# RCC_PLLCFGR 16 Mhz /8 (M) * 128 (N) /4(P)
	# mww 0x40023C00 0x00000102   ;# FLASH_ACR = PRFTBE | 2(Latency)
	# mmw 0x40023800 0x01000000 0 ;# RCC_CR |= PLLON
	# sleep 10                    ;# Wait for PLL to lock
	# mmw 0x40023808 0x00001000 0 ;# RCC_CFGR |= RCC_CFGR_PPRE1_DIV2
	# mmw 0x40023808 0x00000002 0 ;# RCC_CFGR |= RCC_CFGR_SW_PLL

	# Boost JTAG frequency
	adapter_khz 8000
}
```

You can start `openocd` to use these (local) settings by:

``` console
$ openocd -f stlink.cfg -f stm32f4x.cfg
```

A possible advantege of `monitor reset init` is that the `adapter speed` is set to 8MHz, which at least in theory gives better transfer rate between `openocd` and the `stlink` programmer (default is 2MBit). I'm not sure the improvement is noticable.

- ITM buffer overflow

In case the ITM buffer is saturated, ITM tracing stops working (and might be hard to recover). In such case:

  1. correct and recompile the program,

  2. erase the flash (using `st-flash`), 

  3. power cycle the Nucleo (disconnect-and-re-connect),

  4. remove/re-make fifo, and finally re-start `openocd`/`gdb`.
  
This ensures 1) the program will not yet again overflow the ITM buffer, 2) the faulty program is gone (and not restarted accidently on a `RESET`), 3) the programmer firmware is restarted and does not carry any persistent state, notice a `RESET` applies only to the target, not the programmer, so if the programmer crashes it needs to be power cycled), 4) the FIFO `/tmp/itm.log`, `openocd` and `gdb` will have fresh states.

- Check/udate the Nucleo `st-link` firmware (as mentioned above).

---

## Visual Studio Code

`vscode` is highly configurable, (keyboard shortcuts, keymaps, plugins etc.) There is Rust support through the `rls-vscode` plugin (https://github.com/rust-lang/rls-vscode).

It is possible to run `arm-none-eabi-gdb` from within the `vscode` using the `cortex-debug` plugin (https://marketplace.visualstudio.com/items?itemName=marus25.cortex-debug).

For general informaiton regarding debugging in `vscode`, see https://code.visualstudio.com/docs/editor/debugging.

Some useful (default) shortcuts:

- `CTRL+SHIFT+b` compilation tasks, (e.g., compile all examples `cargo build --examples`). Cargo is smart and just re-compiles what is changed.

- `CTRL+SHIFT+d` debug launch configurations, enter debug mode to choose a binary (e.g., `itm 64MHz (debug)`)

- `F5` to start. It will open the `cortex_m_rt/src/lib.rs` file, which contains the startup code. From there you can continue `F5` again.
- `F6` to break. The program will now be in the infinite loop (for this example). In general it will just break wherever the program counter happens to be.
- You can view the ITM trace in the `OUTPUT` tab, choose the dropdown `SWO: ITM [port 0, type console]`. It should now display:

``` txt
[2019-01-02T21:35:26.457Z]   Hello, world!
```

- `SHIFT-F5` shuts down the debugger.

You may step, view the current context `variables`, add `watches`, inspect the `call stack`, add `breakpoints`, inspect `peripherals` and `registers`. Read more in the documentation for the plugin.

### Caveats

Visual Studio Code is not an "IDE", its a text editor with plugin support, with an API somewhat limiting what can be done from within a plugin (in comparison to Eclipse, IntelliJ...) regarding panel layouts etc. E.g., as far as I know you cannot view the `adapter output` (`openocd`) at the same time as the ITM trace, they are both under the `OUTPUT` tab. Moreover, each time you re-start a debug session, you need to re-select the `SWO: Name [port 0, type console]` to view the ITM output. There are some `hax` around this:

- Never shut down the debug session. Instead use the `DEBUG CONSOLE` (`CTRL+SHIFT+Y`) to get to the `gdb` console. This is not the *full* `gdb` interactive shell with some limitations (no *tab* completion e.g.). Make sure the MCU is stopped (`F6`). The console should show something like:

``` txt
Program
 received signal SIGINT, Interrupt.
0x0800056a in main () at examples/itm.rs:31
31	    loop {}
```

- Now you can edit an re-compile your program, e.g. changing the text:

> iprintln!(stim, "Hello, again!");

- In the `DEBUG CONSOLE`, write `load` press `ENTER` write `monitor reset init` press `ENTER`.

``` txt
load
{"token":97,"outOfBandRecord":[{"isStream":false,"type":"status","asyncClass":"download","output":[]}]}
`/home/pln/rust/app/target/thumbv7em-none-eabihf/debug/examples/itm' has changed; re-reading symbols.
Loading section .vector_table, size 0x400 lma 0x8000000
Loading section .text, size 0x10c8 lma 0x8000400
Loading section .rodata, size 0x2a8 lma 0x80014d0
Start address 0x8001298, load size 6000
Transfer rate: 9 KB/sec, 2000 bytes/write.

mon reset init
{"token":147,"outOfBandRecord":[],"resultRecords":{"resultClass":"done","results":[]}}
adapter speed: 2000 kHz
target halted due to debug-request, current mode: Thread 
xPSR: 0x01000000 pc: 0x08001298 msp: 0x20018000
adapter speed: 8000 kHz
```

- The newly compiled binary is now loaded and you can continue (`F5`). Switching to the `OUTPUT` window now preserves the ITM view and displays both traces:

``` txt
[2019-01-02T21:43:27.988Z]   Hello, world!
[2019-01-02T22:07:29.090Z]   Hello, again!
```

- Using the `gdb` terminal (`DEBUG CONSOLE`) from within `vscode` is somewhat instable/experimental. E.g., `CTRL+c` does not `break` the target (use `F6`, or write `interrupt`). The `contiune` command, indeed continues execution (and the *control bar* changes mode, but you cannot `break` using neither `F6` nor `interrupt`). So it seems that the *state* of the `cortex-debug` plugin is not correctly updated. Moreover setting breakpoints from the `gdb` terminal indeed informs `gdb` about the breakpoint, but the state in `vscode` is not updated, so be aware.

---

### Vscode Launch Configurations

Some example launch configurations from the `.vscode/launch.json` file:

``` json
 {
            "type": "cortex-debug",
            "request": "launch",
            "servertype": "openocd",
            "name": "itm 64Mhz (debug)",
            "executable": "./target/thumbv7em-none-eabihf/debug/examples/itm",
            "configFiles": [
                "interface/stlink.cfg",
                "target/stm32f4x.cfg"
            ],
            "postLaunchCommands": [
                "monitor reset init"
            ],
            "swoConfig": {
                "enabled": true,
                "cpuFrequency": 64000000,
                "swoFrequency": 2000000,
                "source": "probe",
                "decoders": [
                    {
                        "type": "console",
                        "label": "ITM",
                        "port": 0
                    }
                ]
            },
            "cwd": "${workspaceRoot}"
        },
        {
            "type": "cortex-debug",
            "request": "launch",
            "servertype": "openocd",
            "name": "hello 16Mhz (debug)",
            "executable": "./target/thumbv7em-none-eabihf/debug/examples/hello",
            "configFiles": [
                "interface/stlink.cfg",
                "target/stm32f4x.cfg"
            ],
            "postLaunchCommands": [
                "monitor arm semihosting enable"
            ],
            "cwd": "${workspaceRoot}"
        },

      {
            "type": "cortex-debug",
            "request": "launch",
            "servertype": "openocd",
            "name": "itm 16Mhz (debug)",
            "executable": "./target/thumbv7em-none-eabihf/debug/examples/itm",
            // uses local config files
            "configFiles": [
                "./stlink.cfg",
                "./stm32f4x.cfg"
            ],
            "postLaunchCommands": [
                "monitor reset init"
            ],
            "swoConfig": {
                "enabled": true,
                "cpuFrequency": 16000000,
                "swoFrequency": 2000000,
                "source": "probe",
                "decoders": [
                    {
                        "type": "console",
                        "label": "ITM",
                        "port": 0
                    }
                ]
            },
            "cwd": "${workspaceRoot}"
        },
```

We see some similarities to the `openocd.gdb` file, we don't need to explicitly connect to the target (that is automatic). Also launching `openocd` is automatic (for good and bad, its re-started each time). `postLaunchCommands` allows arbitrary commands to be executed by `gdb` once the session is up. E.g. in the `hello` case we enable `semihosting`, while in the `itm` case we run `monitor reset init` to get the MCU in 64MHz (first example) or 16MHz (third example), before running the application (continue). Notice the first example uses the "stock" `openocd` configuration files, while the third example uses our local configuration files (that does not change the core frequency).

---

## GDB Advanced Usage

There are numerous ways to automate `gdb`. Scripts can be run by the `gdb` command `source` (`so` for short). Scripting common tasks like setting breakpoints, dumping some memory region etc. can be really helpful.
