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
        // NV-BDIZC
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
        let diff = a.wrapping_sub(*b);
        self.negative = diff < 0;
        self.zero_result = false;

        if a < b {
            self.carry = false;
        } else if a == b {
            self.carry = true;
            self.zero_result = true;
        } else {
            self.carry = true;
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

                // ORA -- A | v
                // indirect, x
                0x01 => {
                    let addr = ram.read_byte(self.pc as usize + 1);
                    println!("ORA (${:0>2X}, X)", addr);
                    let new_addr = ram.read_word(addr.wrapping_add(self.x) as usize) as usize;
                    let value = ram.read_byte(new_addr);
                    self.a |= value;
                    self.sr.determine_zero(self.a);
                    self.sr.determine_negative(self.a);
                    self.pc += 2;
                },

                // KIL -- halt the CPU
                0x02 => {
                    break;
                },

                // SLO -- combination of ASL and ORA
                // indirect, x
                0x03 => {
                    panic!("Illegal opcode SLO ($03)");
                }

                // NOP -- no op
                // zeropage
                0x04 => {
                    panic!("Illegal opcode NOP ($04)");
                },

                // ORA -- A | v
                // zeropage
                0x05 => {
                    let addr = ram.read_byte(self.pc as usize + 1) as usize;
                    println!("ORA ${:0>2X}", addr);
                    let value = ram.read_byte(addr);
                    self.a |= value;
                    self.sr.determine_zero(self.a);
                    self.sr.determine_negative(self.a);
                    self.pc += 2;
                },

                // ASL -- shift left one
                // zeropage
                0x06 => {
                    let addr = ram.read_byte(self.pc as usize + 1);
                    println!("ASL ${:0>2X}", addr);
                    let value = ram.read_byte(addr as usize);
                    ram.write_byte(addr as usize, value << 1);

                    self.sr.determine_carry(value);
                    let value = value << 1;
                    self.sr.determine_zero(value);
                    self.sr.determine_negative(value);

                    self.pc += 2;
                },

                // SLO -- combination of ASL and ORA
                // zeropage
                0x07 => {
                    panic!("Illegal opcode SLO ($07)");
                },

                // PHP -- push a on stack
                0x08 => {
                    println!("PHA");
                    let v = self.a;
                    self.push(ram, v);
                    self.pc += 1;
                },

                // ORA -- A | v
                // immediate
                0x09 => {
                    let value = ram.read_byte(self.pc as usize + 1);
                    println!("ORA #${:0>2X}", value);
                    self.a |= value;
                    self.sr.determine_zero(self.a);
                    self.sr.determine_negative(self.a);
                    self.pc += 2;
                },

                // ASL -- shift accumulator left one
                0x0a => {
                    println!("ASL");
                    self.sr.determine_carry(self.a);
                    self.a <<= 1;
                    self.sr.determine_zero(self.a);
                    self.sr.determine_negative(self.a);

                    self.pc += 1;
                },

                // ANC -- combination of AND and ASL
                0x0b => {
                    panic!("Illegal opcode ANC ($0b)");
                },

                // NOP
                0x0c => {
                    panic!("Illegal opcode NOP ($0c)");
                },

                // ORA -- A | v
                // absolute
                0x0d => {
                    let addr = ram.read_word(self.pc as usize + 1) as usize;
                    println!("ORA ${:0>4X}", addr);
                    let value = ram.read_byte(addr);
                    self.a |= value;
                    self.sr.determine_zero(self.a);
                    self.sr.determine_negative(self.a);
                    self.pc += 3;
                },

                // ASL -- shift left one
                // absolute
                0x0e => {
                    let addr = ram.read_word(self.pc as usize + 1) as usize;
                    println!("ORA ${:0>4X}", addr);
                    let value = ram.read_byte(addr);
                    self.sr.determine_carry(value);
                    let value = value << 1;
                    self.sr.determine_zero(value);
                    self.sr.determine_negative(value);
                    self.pc += 3;
                },

                // SLO -- combination of ASL and ORA
                0x0f => {
                    panic!("Illegal opcode SLO ($0f)");
                },

                // BPL -- branch if plus
                0x10 => {
                    let offset = ram.read_byte(self.pc as usize + 1);
                    println!("BPL ${:0>2X}", offset);
                    self.pc += 2;

                    if !self.sr.negative {
                        self.relative_branch(offset);
                    }
                },

                // ORA -- A | v
                // indirect, y
                0x11 => {
                    let addr = ram.read_byte(self.pc as usize + 1);
                    println!("ORA (${:0>2X}, Y)", addr);
                    let new_addr = ram.read_word(addr.wrapping_add(self.y) as usize) as usize;
                    let value = ram.read_byte(new_addr);
                    self.a |= value;
                    self.sr.determine_zero(self.a);
                    self.sr.determine_negative(self.a);
                    self.pc += 2;
                },

                // KIL -- halt the CPU
                0x12 => {
                    break;
                },

                // SLO -- combination of ASL and ORA
                0x13 => {
                    panic!("Illegal instruction SLO ($13)");
                },

                // NOP
                0x14 => {
                    panic!("Illegal instruction NOP ($14)");
                },

                // ORA -- A | v
                // zeropage, x
                0x15 => {
                    let addr = ram.read_byte(self.pc as usize + 1);
                    println!("ORA ${:0>2X}, X", addr);
                    let value = ram.read_byte(addr.wrapping_add(self.x) as usize);
                    self.a |= value;
                    self.sr.determine_zero(self.a);
                    self.sr.determine_negative(self.a);
                    self.pc += 2;
                },

                // ASL -- shift left one
                // zeropage, X
                0x16 => {
                    let addr = ram.read_byte(self.pc as usize + 1);
                    println!("ASL ${:0>2X}, X", addr);
                    let addr = addr.wrapping_add(self.x);
                    let value = ram.read_byte(addr as usize);
                    ram.write_byte(addr as usize, value << 1);

                    self.sr.determine_carry(value);
                    let value = value << 1;
                    self.sr.determine_zero(value);
                    self.sr.determine_negative(value);

                    self.pc += 2;
                },

                // SLO -- combination of ASL and ORA
                0x17 => {
                    panic!("Illegal opcode SLO ($17)");
                },

                // CLC -- clear carry flag
                0x18 => {
                    println!("CLC");
                    self.sr.carry = false;
                    self.pc += 1;
                },

                // ORA -- A | v
                // absolute, y
                0x19 => {
                    let addr = ram.read_word(self.pc as usize + 1);
                    println!("ORA ${:0>4X}, Y", addr);
                    let value = ram.read_byte(addr.wrapping_add(self.y as u16) as usize);
                    self.a |= value;
                    self.sr.determine_zero(self.a);
                    self.sr.determine_negative(self.a);
                    self.pc += 3;
                },

                // JSR -- jump and save return addr
                0x20 => {
                    let old_addr = self.pc + 2;
                    self.push_word(ram, old_addr);
                    self.pc = ram.read_word(self.pc as usize + 1);
                    println!("JSR ${:0>4X}", self.pc);
                },

                // KIL -- halt the CPU
                0x22 => {
                    break;
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
                
                // ROL -- rotate left
                // ROL A
                0x2a => {
                    println!("ROL");
                    self.sr.determine_negative(self.a);
                    self.sr.carry = self.a & 0x80 == 0x80;
                    self.a = self.a.rotate_left(1);
                    self.sr.determine_zero(self.a);
                    self.pc += 1;
                },

                // BMI -- branch on minus
                0x30 => {
                    let offset = ram.read_byte(self.pc as usize + 1);
                    println!("BMI ${:0>2X}", offset);
                    self.pc += 2;

                    if self.sr.negative {
                        self.relative_branch(offset);
                    }
                },

                // AND -- store A & M in A
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


                // KIL -- halt the CPU
                0x32 => {
                    break;
                },

                // KIL -- halt the CPU
                0x42 => {
                    break;
                },

                // PHA -- push A onto stack
                0x48 => {
                    println!("PHA");
                    let a = self.a;
                    self.push(ram, a);
                    self.pc += 1;
                },

                // JMP -- jump
                // absolute
                0x4c => {
                    let addr = ram.read_word(self.pc as usize + 1);
                    println!("JMP ${:0>4X}", addr);
                    self.pc = addr;
                },

                // KIL -- halt the CPU
                0x52 => {
                    break;
                },

                // RTS -- return from subroutine
                0x60 => {
                    println!("RTS");
                    self.pc = self.pop_word(ram);
                    self.pc += 1;
                },

                // KIL -- halt the CPU
                0x62 => {
                    break;
                },

                // ROR -- rotate one bit right
                // zero page
                0x66 => {
                    let addr = ram.read_byte(self.pc as usize + 1);
                    println!("ROR ${:0>2X}", addr);
                    let value = ram.read_byte(addr as usize).rotate_right(1);
                    ram.write_byte(addr as usize, value);
                    self.sr.determine_zero(value);
                    if self.sr.zero_result {
                        self.sr.determine_negative(value.rotate_left(1));
                    }
                    if value.rotate_left(1) % 2 == 0 {
                        self.sr.carry = false;
                    } else {
                        self.sr.carry = true;
                    }

                    self.pc += 2;
                },

                // ADC -- add with carry
                // immediate
                0x69 => {
                    let value = ram.read_byte(self.pc as usize + 1);
                    println!("ADC #${:0>2X}", value);
                    let old_sign = self.a & 0x80;
                    let result = (self.a as u16) + (value as u16);
                    if self.sr.decimal {
                        self.sr.carry = result > 99;
                    } else {
                        self.sr.carry = result > 0xff;
                    }
                    self.a = self.a.wrapping_add(value);

                    self.sr.overflow = old_sign != (self.a & 0x80);
                    self.sr.determine_zero(self.a);

                    self.pc += 2;
                },

                // JMP -- jump to location
                0x6c => {
                    let addr = ram.read_word(self.pc as usize + 1);
                    println!("JMP ${:0>4X}", addr);
                    self.pc = addr;
                },

                // KIL -- halt the CPU
                0x72 => {
                    break;
                },

                // SEI -- disable interrupts
                0x78 => {
                    println!("SEI");
                    self.sr.int_disable = true;
                    self.pc += 1;
                },

                // STY -- store y
                // zeropage
                0x84 => {
                    let addr = ram.read_byte(self.pc as usize + 1) as usize;
                    println!("STY ${:0>2X}", addr);
                    ram.write_byte(addr, self.y);
                    self.pc += 2;
                },

                // STA -- store A
                // zeropage
                0x85 => {
                    let addr = ram.read_byte(self.pc as usize + 1) as usize;
                    println!("STA ${:0>2X}", addr);
                    ram.write_byte(addr, self.a);
                    self.pc += 2;
                },

                // STX -- store x
                // zeropage
                0x86 => {
                    let addr = ram.read_byte(self.pc as usize + 1) as usize;
                    println!("STX ${:0>2X}", addr);
                    ram.write_byte(addr, self.x);
                    self.pc += 2;
                },

                // DEY -- decrement Y
                0x88 => {
                    println!("DEY");
                    self.y = self.y.wrapping_sub(1);
                    self.sr.determine_negative(self.y);
                    self.sr.determine_zero(self.y);
                    self.pc += 1;
                },

                // TXA -- transfer X to A
                0x8a => {
                    println!("TXA");
                    self.a = self.x;
                    self.sr.determine_zero(self.a);
                    self.sr.determine_negative(self.a);
                    self.pc += 1;
                },

                // STY -- store Y
                // absolute
                0x8c => {
                    let addr = ram.read_word(self.pc as usize + 1) as usize;
                    println!("STY ${:0>4X}", addr);
                    ram.write_byte(addr, self.y);
                    self.pc += 3;
                }

                // STA -- store A
                // absolute
                0x8d => {
                    let addr = ram.read_word(self.pc as usize + 1) as usize;
                    println!("STA ${:0>4X}", addr);
                    ram.write_byte(addr, self.a);
                    self.pc += 3;
                },
                
                // STX -- store X
                // absolute
                0x8e => {
                    let addr = ram.read_word(self.pc as usize + 1) as usize;
                    println!("STX ${:0>4X}", addr);
                    ram.write_byte(addr, self.x);
                    self.pc += 3;
                },

                // BCC -- branch if carry clear
                0x90 => {
                    let offset = ram.read_byte(self.pc as usize + 1);
                    println!("BCC ${:0>2X}", offset);
                    self.pc += 2;

                    if !self.sr.carry {
                        self.relative_branch(offset);
                    }
                },

                // STA -- store A
                // indirect, y
                0x91 => {
                    let addr = ram.read_byte(self.pc as usize + 1);
                    println!("STA (${:0>2X}), Y", addr);
                    let direct_addr = ram.read_word(addr.wrapping_add(self.y) as usize);
                    ram.write_byte(direct_addr as usize, self.a);
                    self.pc += 2;
                },

                // KIL -- halt the CPU
                0x92 => {
                    break;
                },

                // STY -- store Y
                // zeropage, X
                0x94 => {
                    let addr = ram.read_byte(self.pc as usize + 1);
                    println!("STY ${:0>2X}, X", addr);
                    let addr = addr.wrapping_add(self.x) as usize;
                    ram.write_byte(addr, self.y);
                    self.pc += 2;
                },

                // STA -- store A
                // zeropage, X
                0x95 => {
                    let addr = ram.read_byte(self.pc as usize + 1);
                    println!("STA ${:0>2X}, X", addr);
                    let addr = addr.wrapping_add(self.x) as usize;
                    ram.write_byte(addr, self.a);
                    self.pc += 2;
                },

                // TYA -- transfer Y to A
                0x98 => {
                    println!("TYA");
                    self.a = self.y;
                    self.sr.determine_zero(self.a);
                    self.sr.determine_negative(self.a);
                    self.pc += 1;
                },

                // STA -- store A
                // absolute, y
                0x99 => {
                    let addr = ram.read_word(self.pc as usize + 1);
                    println!("STA ${:0>4X}, Y", addr);
                    let addr = addr.wrapping_add(self.y as u16);
                    ram.write_byte(addr as usize, self.a);
                    self.pc += 3;
                },

                // TXS -- transfer X to SP
                0x9a => {
                    println!("TXS");
                    self.sp = self.x;
                    self.pc += 1;
                },

                // STA -- store A
                // absolute, X
                0x9d => {
                    let addr = ram.read_word(self.pc as usize + 1);
                    println!("STA ${:0>4X}, X", addr);
                    let addr = addr.wrapping_add(self.x as u16);
                    ram.write_byte(addr as usize, self.a);
                    self.pc += 3;
                },

                // LDY -- load into Y
                // immediate
                0xa0 => {
                    self.y = ram.read_byte(self.pc as usize + 1);
                    println!("LDY #${:0>2X}", self.y);
                    self.sr.determine_zero(self.y);
                    self.sr.determine_negative(self.y);
                    self.pc += 2;
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

                // LDY -- load into Y
                // zeropage
                0xa4 => {
                    let addr = ram.read_byte(self.pc as usize + 1) as usize;
                    println!("LDA {:0>2X}", addr);
                    self.y = ram.read_byte(addr);
                    self.sr.determine_negative(self.y);
                    self.sr.determine_zero(self.y);
                    self.pc += 2;
                },

                // LDA -- load into A
                // zeropage
                0xa5 => {
                    let addr = ram.read_byte(self.pc as usize + 1) as usize;
                    println!("LDA ${:0>2X}", addr);
                    self.a = ram.read_byte(addr);
                    self.sr.determine_zero(self.a);
                    self.sr.determine_negative(self.a);
                    self.pc += 2;
                },

                // LDX -- load into X
                // zeropage
                0xa6 => {
                    let addr = ram.read_byte(self.pc as usize + 1) as usize;
                    println!("LDX ${:0>2X}", addr);
                    self.x = ram.read_byte(addr);
                    self.sr.determine_zero(self.x);
                    self.sr.determine_negative(self.x);
                    self.pc += 2;
                },

                // TAY -- transfer A to Y
                0xa8 => {
                    println!("TAY");
                    self.y = self.a;
                    self.sr.determine_negative(self.y);
                    self.sr.determine_zero(self.y);
                    self.pc += 1;
                }

                // LDA -- load into A
                // immediate
                0xa9 => {
                    let value = ram.read_byte(self.pc as usize + 1);
                    println!("LDA #${:0>2X}", value);
                    self.a = value;
                    self.sr.determine_zero(self.a);
                    self.sr.determine_negative(self.a);
                    self.pc += 2;
                },

                // TAX -- transfer A to X
                0xaa => {
                    println!("TAX");
                    self.x = self.a;
                    self.sr.determine_negative(self.x);
                    self.sr.determine_zero(self.x);
                    self.pc += 1;
                }

                // absolute
                0xad => {
                    let addr = ram.read_word(self.pc as usize + 1) as usize;
                    println!("LDA ${:0>4X}", addr);
                    let value = ram.read_byte(addr);
                    self.a = value;
                    self.sr.determine_zero(self.a);
                    self.sr.determine_negative(self.a);
                    self.pc += 3;
                },

                // BCS -- branch if carry set
                0xb0 => {
                    let offset = ram.read_byte(self.pc as usize + 1);
                    println!("BCS ${:0>2X}", offset);
                    self.pc += 2;
                    if self.sr.carry {
                        self.relative_branch(offset);
                    }
                },

                // LDA -- load into A
                // indirect, y
                0xb1 => {
                    let addr = ram.read_byte(self.pc as usize + 1);
                    println!("LDA (${:0>2X}), Y", addr);
                    let direct_addr = ram.read_word(addr.wrapping_add(self.y) as usize);
                    let value = ram.read_byte(direct_addr as usize);
                    self.a = value;
                    self.sr.determine_zero(self.a);
                    self.sr.determine_negative(self.a);
                    self.pc += 2;
                },

                // KIL -- halt the CPU
                0xb2 => {
                    break;
                },

                // LDY -- load into Y
                // zeropage, X
                0xb4 => {
                    let addr = ram.read_byte(self.pc as usize + 1);
                    println!("LDY ${:0>2X}, X", addr);
                    let addr = addr.wrapping_add(self.x);
                    self.y = ram.read_byte(addr as usize);
                    self.sr.determine_zero(self.y);
                    self.sr.determine_negative(self.y);
                    self.pc += 2;
                },

                // LDA -- load into A
                // zeropage, X
                0xb5 => {
                    let addr = ram.read_byte(self.pc as usize + 1);
                    println!("LDA ${:0>2X}, X", addr);
                    let addr = addr.wrapping_add(self.x);
                    self.a = ram.read_byte(addr as usize);
                    self.sr.determine_zero(self.a);
                    self.sr.determine_negative(self.a);
                    self.pc += 2;
                },

                // LDA -- load into A
                // absolute, Y
                0xb9 => {
                    let addr = ram.read_word(self.pc as usize + 1);
                    println!("LDA ${:0>4X}, Y", addr);
                    self.a = ram.read_byte(addr.wrapping_add(self.y as u16) as usize);
                    self.sr.determine_zero(self.a);
                    self.sr.determine_negative(self.a);
                    self.pc += 3;
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

                // INY -- increment Y
                0xc8 => {
                    println!("INY");
                    self.y = self.y.wrapping_add(1);
                    self.sr.determine_negative(self.y);
                    self.sr.determine_zero(self.y);

                    self.pc += 1;
                },

                // DEX -- decrement X
                0xca => {
                    println!("DEX");
                    self.x = self.x.wrapping_sub(1);
                    self.sr.determine_negative(self.x);
                    self.sr.determine_zero(self.x);

                    self.pc += 1;
                },

                // BNE -- branch on result not zero
                0xd0 => {
                    let offset = ram.read_byte(self.pc as usize + 1);
                    println!("BNE ${:0>2X}", offset);
                    self.pc += 2;
                    if !self.sr.zero_result {
                        self.relative_branch(offset);
                    }
                },

                // CMP -- compare with accumulator
                // indirect, y
                0xd1 => {
                    let addr = ram.read_byte(self.pc as usize + 1).wrapping_add(self.y) as usize;
                    let direct_addr = ram.read_word(addr);
                    let value = ram.read_byte(direct_addr as usize);

                    println!("CMP (${:0>2X}), Y", ram.read_byte(self.pc as usize + 1));
                    self.sr.compare(&self.a, &value);
                    self.pc += 2;
                },

                // KIL -- halt the CPU
                0xd2 => {
                    break;
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

                // CPX compare X to memory
                // immediate
                0xe0 => {
                    let value = ram.read_byte(self.pc as usize + 1);
                    println!("CPX #${:0>2X}", value);
                    self.sr.compare(&self.x, &value);
                    self.pc += 2;
                },

                // INC -- increment
                // zeropage
                0xe6 => {
                    let addr = ram.read_byte(self.pc as usize + 1) as usize;
                    println!("INC ${:0>2X}", addr);
                    let value = ram.read_byte(addr).wrapping_add(1);
                    ram.write_byte(addr, value);
                    self.sr.determine_negative(value);
                    self.sr.determine_zero(value);
                    
                    self.pc += 2;
                },

                // INX -- increment X
                0xe8 => {
                    println!("INX");
                    self.x = self.x.wrapping_add(1);
                    self.sr.determine_zero(self.x);
                    self.sr.determine_negative(self.x);
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
                    self.pc += 2;
                    if self.sr.zero_result {
                        self.relative_branch(offset);
                    }
                },

                // KIL -- halt the CPU
                0xf2 => {
                    break;
                },

                // INC -- increment memory by 1
                // absolute, x
                0xfe => {
                    let addr = ram.read_word(self.pc as usize + 1).wrapping_add(self.x as u16) as usize;
                    println!("INC ${:0>4X}, X", ram.read_word(self.pc as usize + 1));

                    let value = ram.read_byte(addr).wrapping_add(1);
                    ram.write_byte(addr, value);

                    self.sr.determine_zero(value);
                    self.sr.determine_negative(value);
                    self.pc += 3;
                },
                _ => panic!("Unrecognized opcode (${:0>2X})", opcode),
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

    // Apply an offset for relative addressing
    fn relative_branch(&mut self, offset: u8) {
        if offset < 0x80 {
            self.pc = self.pc.wrapping_add(offset as u16);
        } else {
            self.pc = self.pc.wrapping_sub(0x100 - offset as u16);
        }
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
