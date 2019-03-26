# E7020E-Project

# E-Toaster
Design and build a PCB to upgrade a toaster to an e-Toaster !

# Prerequisites 

* Archlinux
* Visual studio

# Implementation

For the software part we will use RUST to program the MCU.
Build with :
* Cargo
* OpenOCD

# Components

## General
1 Nucleo board to program our own board
1 USB Mini-B connector
1 bluetooth sensor RN 42 Series Class 2
1 STM32F411RET6 MCU
1 LM1117IMP-ADJ voltage converter
3 6pins header
1 temp sensor LMT84
1 LCD screen EA DOGS102W-6
2 red LEDs
1 green LED
2 pushbuttons
1 transistor

## Capacitors
2 4.7u
12 100n
5 1u
2 18p
1 0.01u
2 10u polarized

## Resistors
2 4.7k
3 10k
5 100
1 5k
2 470
4 3.3k
1 1k
1 820
1 160


# Authors 

* **SCHRUB Valentin** - Hardware design and realization.

* **ROMARY Lucas** - Hardware design and realization.

* **FRISENSTAM Carl** - Software implementation.

* **RIMSBY Oscar** - Software implementation.

# Grades 

**Grade 3**:
Implement LED, push button, temperature sensor and LCD screen.

**Grade 4**:
Implement Bluetooth functionality. Write software to enable remote start of toaster. Enable LCD screen to display relevant information.

**Grade 5**:
Extend functionality of LCD screen. Additional information to be displayed. Improve remote functionality? Mobile application.
