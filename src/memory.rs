// Copyright 2016 Peter Beard
// Distributed under the GNU GPL v2. For full terms, see the LICENSE file.
//
// Functions and datatypes relating to RAM
use vic;
use vic::Vic;

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

pub struct Memory {
    data: [u8; 65536],
    kernal_rom: [u8; KERNAL_ROM_SIZE],
    basic_rom: [u8; BASIC_ROM_SIZE],
    char_rom: [u8; CHAR_ROM_SIZE],

    vic: Vic
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            data: [0u8; 65536],
            kernal_rom: [0u8; KERNAL_ROM_SIZE],
            basic_rom: [0u8; BASIC_ROM_SIZE],
            char_rom: [0u8; CHAR_ROM_SIZE],

            vic: Vic::new(),
        }
    }

    // Write default values into memory
    pub fn initialize(&mut self) {
        let mut file = File::open(RAM_IMAGE_FILE).unwrap();
        file.read(&mut self.data).unwrap();
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
        let rom_status = (self.data[1] & 7);
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
        } else if io_enabled && addr >= IO_START && addr <= IO_END {
            if addr >= vic::MIN_CONTROL_ADDR && addr <= vic::MAX_CONTROL_ADDR {
                self.vic.read_register(addr)
            } else {
                panic!("Unimplemented I/O address: ${:0>4X}", addr);
            }
        } else {
            self.data[addr]
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
        self.data[addr] = value;
    }
}
