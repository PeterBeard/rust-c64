// Copyright 2016 Peter Beard
// Distributed under the GNU GPL v2. For full terms, see the LICENSE file.
//
// Functions and datatypes relating to the system bus
use io::vic;
use io::vic::Vic;

use io::sid;
use io::sid::Sid;

use io::cia;
use io::cia::Cia;

use std::fs::File;
use std::io::Read;

const RAM_IMAGE_FILE: &'static str = "ram-default-image.bin";
const KERNAL_ROM_FILE: &'static str = "kernal.901227-03.bin";
const BASIC_ROM_FILE: &'static str = "basic.901226-01.bin";
const CHAR_ROM_FILE: &'static str = "characters.901225-01.bin";

const KERNAL_ROM_START: usize = 0xe000;
const BASIC_ROM_START: usize = 0xa000;
const CHAR_ROM_START: usize = 0xd000;

const KERNAL_ROM_SIZE: usize = 8192;
const BASIC_ROM_SIZE: usize = 8192;
const CHAR_ROM_SIZE: usize = 4096;

const IO_START: usize = 0xd000;
const IO_END: usize = 0xdfff;

const COLOR_RAM_START: usize = 0xd800;
const COLOR_RAM_END: usize = 0xdbff;

const CIA1_MIN_CONTROL_ADDR: usize = 0xdc00;
const CIA1_MAX_CONTROL_ADDR: usize = 0xdcff;
const CIA2_MIN_CONTROL_ADDR: usize = 0xdd00;
const CIA2_MAX_CONTROL_ADDR: usize = 0xddff;

pub struct Bus {
    ram: [u8; 65536],
    kernal_rom: [u8; KERNAL_ROM_SIZE],
    basic_rom: [u8; BASIC_ROM_SIZE],
    char_rom: [u8; CHAR_ROM_SIZE],

    vic: Vic,
    sid: Sid,
    cia_1: Cia,
    cia_2: Cia,
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            ram: [0u8; 65536],
            kernal_rom: [0u8; KERNAL_ROM_SIZE],
            basic_rom: [0u8; BASIC_ROM_SIZE],
            char_rom: [0u8; CHAR_ROM_SIZE],

            vic: Vic::new(),
            sid: Sid::new(),
            cia_1: Cia::new(CIA1_MIN_CONTROL_ADDR),
            cia_2: Cia::new(CIA2_MIN_CONTROL_ADDR),
        }
    }

    // Write default values into memory
    pub fn initialize(&mut self) {
        let mut file = File::open(RAM_IMAGE_FILE).unwrap();
        file.read(&mut self.ram).unwrap();
    }

    // Load data for the various ROM chips
    pub fn load_roms(&mut self) {
        let mut k_file = File::open(KERNAL_ROM_FILE).unwrap();
        k_file.read(&mut self.kernal_rom).unwrap();

        let mut b_file = File::open(BASIC_ROM_FILE).unwrap();
        b_file.read(&mut self.basic_rom).unwrap();

        let mut c_file = File::open(CHAR_ROM_FILE).unwrap();
        c_file.read(&mut self.char_rom).unwrap();
    }
    
    // Read a byte from the given address
    pub fn read_byte(&self, addr: usize) -> u8 {
        // Determine whether to read from ROM or RAM
        let rom_status = (self.ram[1] & 7);
        let kernal_rom_enabled = rom_status % 4 > 1;
        let basic_rom_enabled = rom_status % 4 == 3;
        let char_rom_enabled = rom_status < 4 && rom_status > 0;
        let io_enabled = rom_status > 4;

        if kernal_rom_enabled && addr >= KERNAL_ROM_START && addr < KERNAL_ROM_START + KERNAL_ROM_SIZE
        {
            let offset_addr = addr - KERNAL_ROM_START;
            self.kernal_rom[offset_addr]

        } else if basic_rom_enabled && addr >= BASIC_ROM_START && addr < BASIC_ROM_START + BASIC_ROM_SIZE {
            let offset_addr = addr - BASIC_ROM_START;
            self.basic_rom[offset_addr]

        } else if char_rom_enabled && addr >= CHAR_ROM_START && addr < CHAR_ROM_START + CHAR_ROM_SIZE {
            let offset_addr = addr - CHAR_ROM_START;
            self.char_rom[offset_addr]
        } else if io_enabled && addr >= IO_START && addr <= IO_END && !(addr >= COLOR_RAM_START && addr < COLOR_RAM_END) {
            self.io_read(addr)
        } else {
            self.ram[addr]
        }
    }

    // Read from an I/O device
    fn io_read(&self, addr: usize) -> u8 {
        if addr >= vic::MIN_CONTROL_ADDR && addr <= vic::MAX_CONTROL_ADDR {
            self.vic.read_register(addr)
        } else if addr >= sid::MIN_CONTROL_ADDR && addr <= sid::MAX_CONTROL_ADDR {
            self.sid.read_register(addr)
        } else if addr >= CIA1_MIN_CONTROL_ADDR && addr <= CIA1_MAX_CONTROL_ADDR {
            self.cia_1.read_register(addr)
        } else if addr >= CIA2_MIN_CONTROL_ADDR && addr <= CIA2_MAX_CONTROL_ADDR {
            self.cia_2.read_register(addr)
        } else {
            panic!("Unimplemented I/O address: ${:0>4X}", addr);
        }
    }

    // Read a word (little endian) from the given address
    pub fn read_word(&self, addr: usize) -> u16 {
        let lo = self.read_byte(addr);
        let hi = self.read_byte(addr + 1);
        ((hi as u16) << 8) + lo as u16
    }

    // Write a byte to the given address
    pub fn write_byte(&mut self, addr: usize, value: u8) {
        let io_enabled = (self.ram[1] & 7) > 4;

        if io_enabled && addr >= IO_START && addr <= IO_END && !(addr >= COLOR_RAM_START && addr < COLOR_RAM_END) {
            self.io_write(addr, value);
        } else {
            // System always writes to RAM even if it's masked by a ROM
            self.ram[addr] = value;
        }
    }

    // Write to an I/O device
    fn io_write(&mut self, addr: usize, value: u8) {
        if addr >= vic::MIN_CONTROL_ADDR && addr <= vic::MAX_CONTROL_ADDR {
            self.vic.write_register(addr, value);
        } else if addr >= sid::MIN_CONTROL_ADDR && addr <= sid::MAX_CONTROL_ADDR {
            self.sid.write_register(addr, value);
        } else if addr >= CIA1_MIN_CONTROL_ADDR && addr <= CIA1_MAX_CONTROL_ADDR {
            self.cia_1.write_register(addr, value);
        } else if addr >= CIA2_MIN_CONTROL_ADDR && addr <= CIA2_MAX_CONTROL_ADDR {
            self.cia_2.write_register(addr, value);
        } else {
            panic!("Unimplemented I/O address: ${:0>4X}", addr);
        }
    }
}
