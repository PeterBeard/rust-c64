// Copyright 2016 Peter Beard
// Distributed under the GNU GPL v2. For full terms, see the LICENSE file.
//
// Functions and datatypes relating to RAM
use std::fs::File;
use std::io::Read;

const KERNAL_ROM_FILE: &'static str = "kernal.901227-03.bin";
const BASIC_ROM_FILE: &'static str = "basic.901226-01.bin";
const CHAR_ROM_FILE: &'static str = "characters.901225-01.bin";

const KERNAL_ROM_START: usize = 0xe000;
const BASIC_ROM_START: usize = 0xa000;
const CHAR_ROM_START: usize = 0xd000;

const KERNAL_ROM_SIZE: usize = 8192;
const BASIC_ROM_SIZE: usize = 8192;
const CHAR_ROM_SIZE: usize = 4096;

pub struct Memory {
    data: [u8; 65536],
    kernal_rom: [u8; KERNAL_ROM_SIZE],
    basic_rom: [u8; BASIC_ROM_SIZE],
    char_rom: [u8; CHAR_ROM_SIZE]
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            data: [0u8; 65536],
            kernal_rom: [0u8; KERNAL_ROM_SIZE],
            basic_rom: [0u8; BASIC_ROM_SIZE],
            char_rom: [0u8; CHAR_ROM_SIZE],
        }
    }

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
        if addr >= KERNAL_ROM_START && addr < KERNAL_ROM_START + KERNAL_ROM_SIZE
        {
            let offset_addr = addr - KERNAL_ROM_START;
            self.kernal_rom[offset_addr]

        } else if addr >= BASIC_ROM_START && addr < BASIC_ROM_START + BASIC_ROM_SIZE {
            let offset_addr = addr - BASIC_ROM_START;
            self.basic_rom[offset_addr]

        } else if addr >= CHAR_ROM_START && addr < CHAR_ROM_START + CHAR_ROM_SIZE {
            let offset_addr = addr - CHAR_ROM_START;
            self.char_rom[offset_addr]

        } else {
            self.data[addr as usize]
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
