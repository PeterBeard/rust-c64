# rust-c64

A Commodore 64 emulator written in Rust.

Requires binary files for the contents of the ROM chips. You can download them here: [http://www.zimmers.net/anonftp/pub/cbm/firmware/computers/c64/](http://www.zimmers.net/anonftp/pub/cbm/firmware/computers/c64/).

## What works?
* CPU emulation (it's almost cycle-accurate!) at PAL or NTSC clock speed

## What doesn't?
* Everything else

# Installation

## Prerequisites

You'll need rust and libsdl2 to compile this project. See the [https://github.com/Rust-SDL2/rust-sdl2?tab=readme-ov-file#sdl20-development-libraries](SDL2 README) for instructions on how to install its prerequisites.

Then you should be able to build the project with `cargo build`.

## ROMs

This emulator doesn't include binaries for the various ROM chips in the Commodore 64. You can download them from [http://www.zimmers.net/anonftp/pub/cbm/firmware/computers/c64/](http://www.zimmers.net/anonftp/pub/cbm/firmware/computers/c64/).

Here are the ROMs required by the emulator with links to the specific ones I've tested:
* BASIC ROM: [https://www.zimmers.net/anonftp/pub/cbm/firmware/computers/c64/basic.901226-01.bin](basic.901226-01.bin) [8 kiB, binary]
* Character ROM: [https://www.zimmers.net/anonftp/pub/cbm/firmware/computers/c64/characters.901225-01.bin](characters.901225-01.bin) [4 kiB, binary]
* KERNAL ROM: [https://www.zimmers.net/anonftp/pub/cbm/firmware/computers/c64/kernal.901227-02.bin](kernal.901227-02.bin) [8 kiB, binary]

# Usage

Right now all you can do is start up the emulator and watch it process CPU instructions; video and audio aren't yet supported.

You can build and run the emulator with Cargo, just make sure to specify the paths to your ROM files:

```
$ cargo run -- -k roms/kernal.bin -b roms/basic.bin -r roms/char.bin
```

## Options

```
    -c, --clock TYPE    Clock speed to use. Options are PAL (default) or NTSC
    -k, --kernal FILE   Location of the KERNAL ROM file.
    -b, --basic FILE    Location of the BASIC ROM file.
    -r, --char FILE     Location of the charater ROM file.
    -d, --debug         Show debugging information
    -h, --help          Display this information
```


# C64 Documentation
## MOS 6510 CPU
* [MCS 6500 Microcomputer Family Programming Manual](http://archive.6502.org/books/mcs6500_family_programming_manual.pdf)
* [6502 Instruction Set](http://e-tradition.net/bytes/6502/6502_instruction_set.html)
* [All About Your 64](http://unusedino.de/ec64/technical/aay/c64/)
* [Commodore 64 Memory Map](http://sta.c64.org/cbm64mem.html)

## VIC-II
* [The MOS 6567/6569 video controller (VIC-II) and its application in the Commodore 64](http://vice-emu.sourceforge.net/plain/VIC-Article.txt)
* [VIC-II for beginners](http://dustlayer.com/vic-ii/2013/4/22/when-visibility-matters)
