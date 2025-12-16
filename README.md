# rust-c64

A Commodore 64 emulator written in Rust.

Requires binary files for the contents of the ROM chips. You can download them here: [http://www.zimmers.net/anonftp/pub/cbm/firmware/computers/c64/](http://www.zimmers.net/anonftp/pub/cbm/firmware/computers/c64/).

## What works?
* CPU emulation (it's almost cycle-accurate!)

## What doesn't?
* Everything else

# Installation

You'll need rust and libsdl2 to compile this project. See the [https://github.com/Rust-SDL2/rust-sdl2?tab=readme-ov-file#sdl20-development-libraries](SDL2 README) for instructions on how to install its prerequisites.

Then you should be able to build the project with `cargo build`.

# C64 Documentation
## MOS 6510 CPU
* [MCS 6500 Microcomputer Family Programming Manual](http://archive.6502.org/books/mcs6500_family_programming_manual.pdf)
* [6502 Instruction Set](http://e-tradition.net/bytes/6502/6502_instruction_set.html)
* [All About Your 64](http://unusedino.de/ec64/technical/aay/c64/)
* [Commodore 64 Memory Map](http://sta.c64.org/cbm64mem.html)

## VIC-II
* [The MOS 6567/6569 video controller (VIC-II) and its application in the Commodore 64](http://vice-emu.sourceforge.net/plain/VIC-Article.txt)
* [VIC-II for beginners](http://dustlayer.com/vic-ii/2013/4/22/when-visibility-matters)
