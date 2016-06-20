// Copyright 2016 Peter Beard
// Distributed under the GNU GPL v2. For full terms, see the LICENSE file.
//
// Functions and datatypes related to the CPU

use std::fmt;
use memory::Memory;

const RESET_VECTOR_ADDR: u16 = 0xfce2;
const STACK_START_ADDR: u16 = 0x0100;

const IRQ_VEC_LO_ADDR: u16 = 0xfffe;
const IRQ_VEC_HI_ADDR: u16 = 0xffff;

#[derive(Debug)]
pub struct StatusRegister {
    negative: bool,
    overflow: bool,
    expansion: bool,
    break_cmd: bool,
    decimal: bool,
    int_disable: bool,
    zero_result: bool,
    carry: bool,
}

impl StatusRegister {
    pub fn from_u8(&mut self, value: u8) {
        self.negative = value & 128 == 128;
        self.overflow = value & 64 == 64;
        self.expansion = value & 32 == 32;
        self.break_cmd = value & 16 == 16;
        self.decimal = value & 8 == 8;
        self.int_disable = value & 4 == 4;
        self.zero_result = value & 2 == 2;
        self.carry = value & 1 == 1;
    }

    pub fn to_u8(&self) -> u8 {
        let mut val = 0u8;
        if self.negative {
            val += 128;
        }
        if self.overflow {
            val += 64;
        }
        if self.expansion {
            val += 32;
        }
        if self.break_cmd {
            val += 16;
        }
        if self.decimal {
            val += 8;
        }
        if self.int_disable {
            val += 4;
        }
        if self.zero_result {
            val += 2;
        }
        if self.carry {
            val += 1;
        }
        val
    }

    pub fn new() -> StatusRegister {
        StatusRegister {
            negative: false,
            overflow: false,
            expansion: false,
            break_cmd: false,
            decimal: false,
            int_disable: false,
            zero_result: false,
            carry: false,
        }
    }

    // Compare two values and store the results
    pub fn compare(&mut self, a: &u8, b: &u8) {
        if a < b {
            self.carry = false;
            self.zero_result = false;
        } else if a == b {
            self.negative = false;
            self.carry = true;
            self.zero_result = true;
        } else {
            self.carry = true;
            self.zero_result = false;
        }
    }

    // Determine whether a number is zero and set the corresponding status bit
    pub fn determine_zero(&mut self, value: u8) {
        self.zero_result = (value == 0);
    }

    // Determine whether a number is negative and set the corresponding status bit
    pub fn determine_negative(&mut self, value: u8) {
        self.negative = (value & 0x80 == 0x80);
    }

    // Determine whether to set the carry bit
    pub fn determine_carry(&mut self, value: u8) {
        self.carry = (value & 0x80 == 0x80);
    }
}

pub struct Cpu {
    pc: u16,
    a: u8,
    x: u8,
    y: u8,
    sr: StatusRegister,
    sp: u8
}

impl Cpu { 
    pub fn new() -> Cpu {
        Cpu {
            pc: 0u16,
            a: 0u8,
            x: 0u8,
            y: 0u8,
            sr: StatusRegister::new(),
            sp: 0u8
        }
    }

    // Reset sets the program counter to the address of the reset routine
    pub fn reset(&mut self) {
        self.pc = RESET_VECTOR_ADDR;
        self.sp = 0xfd; // The stack pointer ends up initialized to 0xfd
    }

    pub fn run(&mut self, ram: &mut Memory) {
        self.reset();
        loop {
            // Fetch an instruction from RAM
            let opcode = ram.read_byte(self.pc as usize);

            print!("${:0>4X}: ", self.pc);

            match opcode {
                // BRK -- force break
                0x00 => {
                    println!("BRK");
                    let pc = self.pc;
                    let sr = self.sr.to_u8() | 24;   // Set BRK flag in the stored SR

                    self.push_word(ram, pc + 2);
                    self.push(ram, sr);
                    self.sr.int_disable = true;

                    // Read interrupt vector into PC
                    let hi = ram.read_byte(IRQ_VEC_HI_ADDR as usize);
                    let lo = ram.read_byte(IRQ_VEC_LO_ADDR as usize);
                    self.pc = ((hi as u16) << 8) + lo as u16
                },

                // ASL -- shift left one
                // zeropage, X
                0x16 => {
                    let addr = ram.read_byte(self.pc as usize + 1);
                    println!("ASL, X ${:0>2X}", addr);
                    let value = ram.read_byte(addr as usize);
                    ram.write_byte(addr as usize, value.wrapping_shl(1));

                    self.sr.determine_zero(value);
                    self.sr.determine_negative(value);
                    self.sr.determine_carry(value);

                    self.pc += 2;
                },

                // JSR -- jump and save return addr
                0x20 => {
                    let old_addr = self.pc + 2;
                    self.push_word(ram, old_addr);
                    self.pc = ram.read_word(self.pc as usize + 1);
                    println!("JSR ${:0>4X}", self.pc);
                },

                // AND -- store A & M in A
                // immediate
                0x29 => {
                    let value = ram.read_byte(self.pc as usize + 1);
                    println!("AND #${:0>2X}", value);
                    self.a = self.a & value;
                    self.sr.determine_zero(self.a);
                    self.sr.determine_negative(self.a);
                    self.pc += 2;
                },
                // indirect
                0x31 => {
                    let addr = ram.read_byte(self.pc as usize + 1);
                    println!("AND (${:0>2X}, X)", addr);
                    let new_addr = ram.read_word(addr as usize + self.x as usize);
                    let value = ram.read_byte(new_addr as usize);
                    self.a = self.a & value;
                    self.sr.determine_zero(self.a);
                    self.sr.determine_negative(self.a);
                    self.pc += 2;
                },

                // PHA -- push A onto stack
                0x48 => {
                    println!("PHA");
                    let a = self.a;
                    self.push(ram, a);
                    self.pc += 1;
                },

                // RTS -- return from subroutine
                0x60 => {
                    println!("RTS");
                    self.pc = self.pop_word(ram);
                    self.pc += 1;
                },

                // ROR -- rotate one bit right
                // zero page
                0x66 => {
                    let addr = ram.read_byte(self.pc as usize + 1);
                    println!("ROR ${:0>2X}", addr);
                    let value = ram.read_byte(addr as usize).rotate_right(1);
                    ram.write_byte(addr as usize, value);
                    self.sr.determine_zero(value);
                    self.sr.determine_negative(value);
                    self.sr.determine_carry(value);
                    self.pc += 2;
                },

                // JMP -- jump to location
                0x6c => {
                    let addr = ram.read_word(self.pc as usize + 1);
                    println!("JMP ${:0>4X}", addr);
                    self.pc = addr;
                },

                // SEI -- disable interrupts
                0x78 => {
                    println!("SEI");
                    self.sr.int_disable = true;
                    self.pc += 1;
                },

                // STY -- store y
                0x84 => {
                    let addr = ram.read_byte(self.pc as usize + 1) as usize;
                    println!("STY ${:0>4X}", addr);
                    ram.write_byte(addr, self.y);
                    self.pc += 2;
                },

                // STX -- store x
                0x86 => {
                    let addr = ram.read_byte(self.pc as usize + 1) as usize;
                    println!("STX ${:0>4X}", addr);
                    ram.write_byte(addr, self.x);
                    self.pc += 2;
                },

                // TXA -- transfer X to A
                0x8a => {
                    println!("TXA");
                    self.a = self.x;
                    self.sr.determine_zero(self.a);
                    self.sr.determine_negative(self.a);
                    self.pc += 1;
                },

                // TYA -- transfer Y to A
                0x98 => {
                    println!("TYA");
                    self.a = self.y;
                    self.sr.determine_zero(self.a);
                    self.sr.determine_negative(self.a);
                    self.pc += 1;
                },
                // TXS -- transfer X to SP
                0x9a => {
                    println!("TXS");
                    self.sp = self.x;
                    self.pc += 1;
                },
                
                // LDX -- load into X
                // Immediate
                0xa2 => {
                    self.x = ram.read_byte(self.pc as usize + 1);
                    println!("LDX #${:0>2X}", self.x);
                    self.sr.determine_zero(self.x);
                    self.sr.determine_negative(self.x);
                    self.pc += 2;
                },

                // TSX -- transfer SP to X
                0xba => {
                    println!("TSX");
                    self.x = self.sp;
                    self.sr.determine_zero(self.x);
                    self.sr.determine_negative(self.x);
                    self.pc += 1;
                },

                // LDA -- load into accumulator
                // absolute, x
                0xbd => {
                    let addr = ram.read_word(self.pc as usize + 1).wrapping_add(self.x as u16) as usize;
                    println!("LDA ${:0>4X}, X", ram.read_word(self.pc as usize + 1));
                    self.a = ram.read_byte(addr);
                    self.sr.determine_zero(self.a);
                    self.sr.determine_negative(self.a);
                    self.pc += 3;
                },

                // BNE -- branch on result not zero
                0xd0 => {
                    let offset = ram.read_byte(self.pc as usize + 1);
                    println!("BNE ${:0>2X}", offset);
                    if !self.sr.zero_result {
                        self.pc = self.pc.wrapping_add(offset as u16);
                    } else {
                        self.pc += 2;
                    }
                },

                // CMP -- compare with accumulator
                // absolute, x
                0xdd => {
                    let addr = ram.read_word(self.pc as usize + 1).wrapping_add(self.x as u16) as usize;
                    let value = ram.read_byte(addr);

                    println!("CMP ${:0>4X}, X", ram.read_word(self.pc as usize + 1));
                    self.sr.compare(&self.a, &value);
                    self.pc += 3;
                },

                // CLD -- clear decimal mode
                0xd8 => {
                    println!("CLD");
                    self.sr.decimal = false;
                    self.pc += 1;
                },

                // NOP
                0xea => {
                    println!("NOP");
                    self.pc += 1;
                },

                // BEQ -- branch if zero
                0xf0 => {
                    let offset = ram.read_byte(self.pc as usize + 1);
                    println!("BEQ ${:0>2X}", offset);
                    if self.sr.zero_result {
                        self.pc = self.pc.wrapping_add(offset as u16);
                    } else {
                        self.pc += 2;
                    }
                },
                _ => panic!("Unrecognized opcode (${:0>2x})", opcode),
            };
            
            println!("  {:?}", self);
        }
    }

    // Push a word onto the stack {
    fn push_word(&mut self, ram: &mut Memory, value: u16) {
        let hi = (value >> 8) as u8;
        let lo = (value & 0xff) as u8;
        self.push(ram, hi);
        self.push(ram, lo);
    }

    // Push a byte onto the stack
    fn push(&mut self, ram: &mut Memory, value: u8) {
        ram.write_byte((self.sp as u16 + STACK_START_ADDR) as usize, value);
        self.sp = self.sp.wrapping_sub(1);
    }

    // Pop a word off the stack
    fn pop_word(&mut self, ram: &Memory) -> u16 {
        let lo = self.pop(ram);
        let hi = self.pop(ram);
        ((hi as u16) << 8) + lo as u16
    }

    // Pop a byte off the stack
    fn pop(&mut self, ram: &Memory) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        let value = ram.read_byte((self.sp as u16 + STACK_START_ADDR) as usize);
        value
    }
}

impl fmt::Debug for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "PC: ${:0>4X} // A: ${:0>2X} // X: ${:0>2X} // Y: ${:0>2X} // SP: ${:0>2X} // SR: {:0>8b}",
               self.pc, self.a, self.x, self.y, self.sp, self.sr.to_u8()
               )
    }
}
