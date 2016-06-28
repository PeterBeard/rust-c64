// Copyright 2016 Peter Beard
// Distributed under the GNU GPL v2. For full terms, see the LICENSE file.
//
// Functions and datatypes related to the CPU

use bus::Bus;
use std::fmt;

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
        self.negative = diff > 0x80;
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

#[derive(PartialEq, Debug)]
enum CpuState {
    Fetch,
    Load,
    Store,
    PushWordLo,
    PushWordHi,
    PullWordLo,
    PullWordHi,

    AddressLo,
    AddressLoX,
    AddressLoY,
    AddressHi,
    AddressHiX,
    AddressHiY,

    AddressZeropage,
    AddressZeropageX,
    AddressZeropageY,

    AddressIndirect,
    AddressIndirectX,
    AddressIndirectY,
    AddressIndirectLo,
    AddressIndirectHi,

    Immediate,
    Implied,
    ToLoad,
    Halt,
}

pub struct Cpu {
    pc: u16,
    a: u8,
    x: u8,
    y: u8,
    sr: StatusRegister,
    sp: u8,
    dataport: u8,
    data_direction_reg: u8,

    cycles: u64,
    curr_op: u8,

    addr_lo: u8,
    addr_hi: u8,

    data_bus: u8,
    pub rw: bool,          // Bus read/write - true for read, false for write
    pub addr_enable: bool,
    pub addr_bus: u16,

    stack_word_ready: bool,
    stack_word: u16,
    state: CpuState,
}

impl Cpu { 
    pub fn new() -> Cpu {
        Cpu {
            pc: 0u16,
            a: 0u8,
            x: 0u8,
            y: 0u8,
            sr: StatusRegister::new(),
            sp: 0u8,
            dataport: 0u8,
            data_direction_reg: 0u8,

            cycles: 0u64,
            curr_op: 0u8,

            addr_lo: 0u8,
            addr_hi: 0u8,

            rw: true,
            addr_enable: false,
            addr_bus: 0u16,
            data_bus: 0u8,

            stack_word_ready: false,
            stack_word: 0u16,
            state: CpuState::Halt,
        }
    }

    // Reset sets the program counter to the address of the reset routine
    pub fn reset(&mut self) {
        self.pc = RESET_VECTOR_ADDR;
        self.a = 0xaa;
        self.x = 0;
        self.y = 0;
        self.sp = 0xfd; // The stack pointer ends up initialized to 0xfd

        self.data_direction_reg = 0x2f;
        self.dataport = 0x37;

        self.addr_bus = self.pc;
        self.addr_enable = true;
        self.rw = true;

        self.state = CpuState::Fetch;
    }

    // Fetch the next instruction from RAM
    fn fetch_instr(&mut self) {
        let pc = self.pc;
        self.set_addr_bus(pc);
    }

    // Write an address to the address bus
    fn set_addr_bus(&mut self, addr: u16) {
        self.addr_bus = addr;
        self.addr_enable = true;
        self.rw = true;
    }

    // Do the action associated with an opcode
    fn do_instr(&mut self, debug: bool) -> CpuState {
        use self::CpuState::*;
        match self.curr_op {
            // BRK -- force break
            /*
            0x00 => {
                if debug {
					println!("BRK");
				}
                let pc = self.pc;
                let sr = self.sr.to_u8() | 24;   // Set BRK flag in the stored SR

                self.push_word(ram, pc + 2);
                self.push(ram, sr);
                self.sr.int_disable = true;

                // Read interrupt vector into PC
                let hi = ram.read_byte(IRQ_VEC_HI_ADDR as usize);
                let lo = ram.read_byte(IRQ_VEC_LO_ADDR as usize);
                self.pc = ((hi as u16) << 8) + lo as u16;
                SINGLE
            },
            */

            // ORA -- A | v
            // indirect, x
            0x01 => {
                if debug {
					println!("ORA (${:0>2X}, X)", self.addr_lo);
				}
                self.a |= self.data_bus;
                self.sr.determine_zero(self.a);
                self.sr.determine_negative(self.a);
                Fetch
            },

            // KIL -- halt the CPU
            0x02 => {
                Halt
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
                if debug {
					println!("ORA ${:0>2X}", self.addr_lo);
				}
                self.a |= self.data_bus;
                self.sr.determine_zero(self.a);
                self.sr.determine_negative(self.a);
                Fetch
            },

            // ASL -- shift left one
            // zeropage
            0x06 => {
                if debug {
					println!("ASL ${:0>2X}", self.addr_lo);
				}

                let data = self.read_data_bus();
                self.sr.determine_carry(self.data_bus);

                self.set_data_bus(data << 1);
                self.sr.determine_zero(self.data_bus);
                self.sr.determine_negative(self.data_bus);
                Store
            },

            // SLO -- combination of ASL and ORA
            // zeropage
            0x07 => {
                panic!("Illegal opcode SLO ($07)");
            },

            // PHP -- push A on stack
            0x08 => {
                if debug {
					println!("PHA");
				}
                let a = self.a;
                self.set_data_bus(a);
                let sp = self.get_stack_addr();
                self.set_addr_bus(sp);
                self.sp  = self.sp.wrapping_sub(1);
                Store
            },

            // ORA -- A | v
            // immediate
            0x09 => {
                if debug {
					println!("ORA #${:0>2X}", self.data_bus);
				}
                self.a |= self.data_bus;
                self.sr.determine_zero(self.a);
                self.sr.determine_negative(self.a);
                Fetch
            },

            // ASL -- shift accumulator left one
            0x0a => {
                if debug {
					println!("ASL");
				}
                self.sr.determine_carry(self.a);
                self.a <<= 1;
                self.sr.determine_zero(self.a);
                self.sr.determine_negative(self.a);
                Fetch
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
                if debug {
					println!("ORA ${:0>4X}", self.addr_bus);
				}
                self.a |= self.data_bus;
                self.sr.determine_zero(self.a);
                self.sr.determine_negative(self.a);
                Fetch
            },

            // ASL -- shift left one
            // absolute
            0x0e => {
                if debug {
					println!("ASL ${:0>4X}", self.addr_bus);
				}
                self.sr.determine_carry(self.data_bus);
                let data = self.read_data_bus() << 1;
                self.set_data_bus(data);
                self.sr.determine_zero(self.data_bus);
                self.sr.determine_negative(self.data_bus);
                Store
            },

            // SLO -- combination of ASL and ORA
            0x0f => {
                panic!("Illegal opcode SLO ($0f)");
            },

            // BPL -- branch if plus
            0x10 => {
                if debug {
					println!("BPL ${:0>2X}", self.data_bus);
				}

                if !self.sr.negative {
                    self.relative_branch();
                }
                Fetch
            },

            // ORA -- A | v
            // indirect, y
            0x11 => {
                if debug {
					println!("ORA (${:0>2X}, Y)", self.addr_lo);
				}
                self.a |= self.data_bus;
                self.sr.determine_zero(self.a);
                self.sr.determine_negative(self.a);
                Fetch
            },

            // KIL -- halt the CPU
            0x12 => {
                Halt
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
                if debug {
					println!("ORA ${:0>2X}, X", self.addr_lo);
				}
                self.a |= self.data_bus;
                self.sr.determine_zero(self.a);
                self.sr.determine_negative(self.a);
                Fetch
            },

            // ASL -- shift left one
            // zeropage, X
            0x16 => {
                if debug {
					println!("ASL ${:0>2X}, X", self.addr_lo);
				}

                self.sr.determine_carry(self.data_bus);
                let data = self.read_data_bus() << 1;
                self.sr.determine_zero(self.data_bus);
                self.sr.determine_negative(self.data_bus);
                self.set_data_bus(data);
                Store
            },

            // SLO -- combination of ASL and ORA
            0x17 => {
                panic!("Illegal opcode SLO ($17)");
            },

            // CLC -- clear carry flag
            0x18 => {
                if debug {
					println!("CLC");
				}
                self.sr.carry = false;
                Fetch
            },

            // ORA -- A | v
            // absolute, y
            0x19 => {
                if debug {
					println!("ORA ${:0>4X}, Y", self.addr_bus);
				}
                self.a |= self.data_bus;
                self.sr.determine_zero(self.a);
                self.sr.determine_negative(self.a);
                Fetch
            },

            // JSR -- jump and save return addr
            0x20 => {
                let old_addr = self.pc + 2;
                self.stack_word = self.pc;
                self.pc = self.addr_from_hi_lo();
                if debug {
					println!("JSR ${:0>4X}", self.pc);
				}
                PushWordHi
            },

            // KIL -- halt the CPU
            0x22 => {
                Halt
            },

            // AND -- store A & M in A
            // immediate
            0x29 => {
                if debug {
					println!("AND #${:0>2X}", self.data_bus);
				}
                self.a = self.a & self.data_bus;
                self.sr.determine_zero(self.a);
                self.sr.determine_negative(self.a);
                Fetch
            },
            
            // ROL -- rotate left
            // ROL A
            0x2a => {
                if debug {
					println!("ROL");
				}
                self.sr.determine_negative(self.a);
                self.sr.carry = self.a & 0x80 == 0x80;
                self.a = self.a.rotate_left(1);
                self.sr.determine_zero(self.a);
                Fetch
            },

            // BMI -- branch on minus
            0x30 => {
                if debug {
					println!("BMI ${:0>2X}", self.data_bus);
				}

                if self.sr.negative {
                    self.relative_branch();
                }
                Fetch
            },

            // AND -- store A & M in A
            // indirect
            0x31 => {
                if debug {
					println!("AND (${:0>2X}, X)", self.addr_lo);
				}
                self.a = self.a & self.data_bus;
                self.sr.determine_zero(self.a);
                self.sr.determine_negative(self.a);
                Fetch
            },


            // KIL -- halt the CPU
            0x32 => {
                Halt
            },

            // KIL -- halt the CPU
            0x42 => {
                Halt
            },

            // PHA -- push A onto stack
            0x48 => {
                if debug {
					println!("PHA");
				}
                let a = self.a;
                self.set_data_bus(a);
                let sp = self.get_stack_addr();
                self.set_addr_bus(sp);
                Store
            },

            // JMP -- jump
            // absolute
            0x4c => {
                if debug {
					println!("JMP ${:0>4X}", self.addr_bus);
				}
                self.pc = self.addr_bus;
                Fetch
            },

            // KIL -- halt the CPU
            0x52 => {
                Halt
            },

            // RTS -- return from subroutine
            0x60 => {
                if debug {
					println!("RTS");
				}
                if self.stack_word_ready {
                    self.pc = self.stack_word;
                    self.stack_word_ready = false;
                    ToLoad
                } else {
                    PullWordLo
                }
            },

            // KIL -- halt the CPU
            0x62 => {
                Halt
            },

            // ROR -- rotate one bit right
            // zero page
            0x66 => {
                if debug {
					println!("ROR ${:0>2X}", self.addr_lo);
				}
                self.sr.determine_zero(self.data_bus.rotate_right(1));
                if self.sr.zero_result {
                    self.sr.determine_negative(self.data_bus);
                }
                if self.data_bus % 2 == 0 {
                    self.sr.carry = false;
                } else {
                    self.sr.carry = true;
                }
                let data = self.data_bus.rotate_right(1);
                self.set_data_bus(data);
                Store
            },

            // ADC -- add with carry
            // immediate
            0x69 => {
                if debug {
					println!("ADC #${:0>2X}", self.data_bus);
				}
                let old_sign = self.a & 0x80;
                let result = (self.a as u16) + (self.data_bus as u16);
                if self.sr.decimal {
                    self.sr.carry = result > 99;
                } else {
                    self.sr.carry = result > 0xff;
                }
                self.a = self.a.wrapping_add(self.data_bus);

                self.sr.overflow = old_sign != (self.a & 0x80);
                self.sr.determine_zero(self.a);
                Fetch
            },

            // JMP -- jump to location
            0x6c => {
                if debug {
					println!("JMP ${:0>4X}", self.addr_bus);
				}
                self.pc = self.addr_bus;
                Fetch
            },

            // KIL -- halt the CPU
            0x72 => {
                Halt
            },

            // SEI -- disable interrupts
            0x78 => {
                if debug {
					println!("SEI");
				}
                self.sr.int_disable = true;
                Fetch
            },

            // STY -- store y
            // zeropage
            0x84 => {
                if debug {
					println!("STY ${:0>2X}", self.addr_lo);
				}
                let y = self.y;
                self.set_data_bus(y);
                Store
            },

            // STA -- store A
            // zeropage
            0x85 => {
                if debug {
					println!("STA ${:0>2X}", self.addr_lo);
				}
                let a = self.a;
                self.set_data_bus(a);
                Store
            },

            // STX -- store x
            // zeropage
            0x86 => {
                if debug {
					println!("STX ${:0>2X}", self.addr_lo);
				}
                let x = self.x;
                self.set_data_bus(x);
                Store
            },

            // DEY -- decrement Y
            0x88 => {
                if debug {
					println!("DEY");
				}
                self.y = self.y.wrapping_sub(1);
                self.sr.determine_negative(self.y);
                self.sr.determine_zero(self.y);
                Fetch
            },

            // TXA -- transfer X to A
            0x8a => {
                if debug {
					println!("TXA");
				}
                self.a = self.x;
                self.sr.determine_zero(self.a);
                self.sr.determine_negative(self.a);
                Fetch
            },

            // STY -- store Y
            // absolute
            0x8c => {
                if debug {
					println!("STY ${:0>4X}", self.addr_bus);
				}
                let y = self.y;
                self.set_data_bus(y);
                Store
            }

            // STA -- store A
            // absolute
            0x8d => {
                if debug {
					println!("STA ${:0>4X}", self.addr_bus);
				}
                let a = self.a;
                self.set_data_bus(a);
                Store
            },
            
            // STX -- store X
            // absolute
            0x8e => {
                if debug {
					println!("STX ${:0>4X}", self.addr_bus);
				}
                let x = self.x;
                self.set_data_bus(x);
                Store
            },

            // BCC -- branch if carry clear
            0x90 => {
                if debug {
					println!("BCC ${:0>2X}", self.data_bus);
				}

                if !self.sr.carry {
                    self.relative_branch();
                }
                Fetch
            },

            // STA -- store A
            // indirect, y
            0x91 => {
                if debug {
					println!("STA (${:0>2X}), Y", self.addr_lo);
				}
                let a = self.a;
                self.set_data_bus(a);
                Store
            },

            // KIL -- halt the CPU
            0x92 => {
                Halt
            },

            // STY -- store Y
            // zeropage, X
            0x94 => {
                if debug {
					println!("STY ${:0>2X}, X", self.addr_lo);
				}
                let y = self.y;
                self.set_data_bus(y);
                Store
            },

            // STA -- store A
            // zeropage, X
            0x95 => {
                if debug {
					println!("STA ${:0>2X}, X", self.addr_lo);
				}
                let a = self.a;
                self.set_data_bus(a);
                Store
            },

            // TYA -- transfer Y to A
            0x98 => {
                if debug {
					println!("TYA");
				}
                self.a = self.y;
                self.sr.determine_zero(self.a);
                self.sr.determine_negative(self.a);
                Fetch
            },

            // STA -- store A
            // absolute, y
            0x99 => {
                if debug {
					println!("STA ${:0>4X}, Y", self.addr_bus);
				}
                let a = self.a;
                self.set_data_bus(a);
                Store
            },

            // TXS -- transfer X to SP
            0x9a => {
                if debug {
					println!("TXS");
				}
                self.sp = self.x;
                Fetch
            },

            // STA -- store A
            // absolute, X
            0x9d => {
                if debug {
					println!("STA ${:0>4X}, X", self.addr_bus);
				}
                let a = self.a;
                self.set_data_bus(a);
                Store
            },

            // LDY -- load into Y
            // immediate
            0xa0 => {
                if debug {
					println!("LDY #${:0>2X}", self.data_bus);
				}
                self.y = self.data_bus;
                self.sr.determine_zero(self.y);
                self.sr.determine_negative(self.y);
                Fetch
            },
            
            // LDX -- load into X
            // Immediate
            0xa2 => {
                if debug {
					println!("LDX #${:0>2X}", self.data_bus);
				}
                self.x = self.data_bus;
                self.sr.determine_zero(self.x);
                self.sr.determine_negative(self.x);
                Fetch
            },

            // LDY -- load into Y
            // zeropage
            0xa4 => {
                if debug {
					println!("LDA {:0>2X}", self.addr_lo);
				}
                self.y = self.data_bus;
                self.sr.determine_negative(self.y);
                self.sr.determine_zero(self.y);
                Fetch
            },

            // LDA -- load into A
            // zeropage
            0xa5 => {
                if debug {
					println!("LDA ${:0>2X}", self.addr_lo);
				}
                self.a = self.data_bus;
                self.sr.determine_zero(self.a);
                self.sr.determine_negative(self.a);
                Fetch
            },

            // LDX -- load into X
            // zeropage
            0xa6 => {
                if debug {
					println!("LDX ${:0>2X}", self.addr_lo);
				}
                self.x = self.data_bus;
                self.sr.determine_zero(self.x);
                self.sr.determine_negative(self.x);
                Fetch
            },

            // TAY -- transfer A to Y
            0xa8 => {
                if debug {
					println!("TAY");
				}
                self.y = self.a;
                self.sr.determine_negative(self.y);
                self.sr.determine_zero(self.y);
                Fetch
            }

            // LDA -- load into A
            // immediate
            0xa9 => {
                if debug {
					println!("LDA #${:0>2X}", self.data_bus);
				}
                self.a = self.data_bus;
                self.sr.determine_zero(self.a);
                self.sr.determine_negative(self.a);
                Fetch
            },

            // TAX -- transfer A to X
            0xaa => {
                if debug {
					println!("TAX");
				}
                self.x = self.a;
                self.sr.determine_negative(self.x);
                self.sr.determine_zero(self.x);
                Fetch
            }

            // LDA -- load into A
            // absolute
            0xad => {
                if debug {
					println!("LDA ${:0>4X}", self.addr_bus);
				}
                self.a = self.data_bus;
                self.sr.determine_zero(self.a);
                self.sr.determine_negative(self.a);
                Fetch
            },

            // BCS -- branch if carry set
            0xb0 => {
                if debug {
					println!("BCS ${:0>2X}", self.data_bus);
				}
                self.pc += 2;
                if self.sr.carry {
                    self.relative_branch();
                }
                Fetch
            },

            // LDA -- load into A
            // indirect, y
            0xb1 => {
                if debug {
					println!("LDA (${:0>2X}), Y", self.addr_bus);
				}
                self.a = self.data_bus;
                self.sr.determine_zero(self.a);
                self.sr.determine_negative(self.a);
                Fetch
            },

            // KIL -- halt the CPU
            0xb2 => {
                Halt
            },

            // LDY -- load into Y
            // zeropage, X
            0xb4 => {
                if debug {
					println!("LDY ${:0>2X}, X", self.addr_lo);
				}
                self.y = self.data_bus;
                self.sr.determine_zero(self.y);
                self.sr.determine_negative(self.y);
                Fetch
            },

            // LDA -- load into A
            // zeropage, X
            0xb5 => {
                if debug {
					println!("LDA ${:0>2X}, X", self.addr_lo);
				}
                self.a = self.data_bus;
                self.sr.determine_zero(self.a);
                self.sr.determine_negative(self.a);
                Fetch
            },

            // LDA -- load into A
            // absolute, Y
            0xb9 => {
                if debug {
					println!("LDA ${:0>4X}, Y", self.addr_bus);
				}
                self.a = self.data_bus;
                self.sr.determine_zero(self.a);
                self.sr.determine_negative(self.a);
                Fetch
            },

            // TSX -- transfer SP to X
            0xba => {
                if debug {
					println!("TSX");
				}
                self.x = self.sp;
                self.sr.determine_zero(self.x);
                self.sr.determine_negative(self.x);
                Fetch
            },

            // LDA -- load into accumulator
            // absolute, x
            0xbd => {
                if debug {
					println!("LDA ${:0>4X}, X", self.addr_bus);
				}
                self.a = self.data_bus;
                self.sr.determine_zero(self.a);
                self.sr.determine_negative(self.a);
                Fetch
            },

            // INY -- increment Y
            0xc8 => {
                if debug {
					println!("INY");
				}
                self.y = self.y.wrapping_add(1);
                self.sr.determine_negative(self.y);
                self.sr.determine_zero(self.y);
                Fetch
            },

            // DEX -- decrement X
            0xca => {
                if debug {
					println!("DEX");
				}
                self.x = self.x.wrapping_sub(1);
                self.sr.determine_negative(self.x);
                self.sr.determine_zero(self.x);
                Fetch
            },

            // BNE -- branch on result not zero
            0xd0 => {
                if debug {
					println!("BNE ${:0>2X}", self.data_bus);
				}

                if !self.sr.zero_result {
                    self.relative_branch();
                }
                Fetch
            },

            // CMP -- compare with accumulator
            // indirect, y
            0xd1 => {
                if debug {
					println!("CMP (${:0>2X}), Y", self.addr_bus);
				}
                self.sr.compare(&self.a, &self.data_bus);
                Fetch
            },

            // KIL -- halt the CPU
            0xd2 => {
                Halt
            },

            // CMP -- compare with accumulator
            // absolute, x
            0xdd => {
                if debug {
					println!("CMP ${:0>4X}, X", self.addr_bus);
				}
                self.sr.compare(&self.a, &self.data_bus);
                Fetch
            },

            // CLD -- clear decimal mode
            0xd8 => {
                if debug {
					println!("CLD");
				}
                self.sr.decimal = false;
                Fetch
            },

            // CPX compare X to memory
            // immediate
            0xe0 => {
                if debug {
					println!("CPX #${:0>2X}", self.data_bus);
				}
                self.sr.compare(&self.x, &self.data_bus);
                Fetch
            },

            // INC -- increment
            // zeropage
            0xe6 => {
                if debug {
					println!("INC ${:0>2X}", self.addr_lo);
				}
                let data = self.read_data_bus().wrapping_add(1);
                self.sr.determine_negative(self.data_bus);
                self.sr.determine_zero(self.data_bus);
                self.set_data_bus(data);
                Store
            },

            // INX -- increment X
            0xe8 => {
                if debug {
					println!("INX");
				}
                self.x = self.x.wrapping_add(1);
                self.sr.determine_zero(self.x);
                self.sr.determine_negative(self.x);
                Fetch
            },

            // NOP
            0xea => {
                if debug {
					println!("NOP");
				}
                Fetch
            },

            // BEQ -- branch if zero
            0xf0 => {
                if debug {
					println!("BEQ ${:0>2X}", self.data_bus);
				}

                if self.sr.zero_result {
                    self.relative_branch();
                }
                Fetch
            },

            // KIL -- halt the CPU
            0xf2 => {
                Halt
            },

            // INC -- increment memory by 1
            // absolute, x
            0xfe => {
                if debug {
					println!("INC ${:0>4X}, X", self.addr_bus);
				}

                let data = self.read_data_bus().wrapping_add(1);
                self.sr.determine_zero(self.data_bus);
                self.sr.determine_negative(self.data_bus);
                self.set_data_bus(data);
                Store
            },
            _ => panic!("Unrecognized opcode (${:0>2X})", self.curr_op),
        }
    }

    // Determine the addressing mode of the current instruction
    fn addressing_mode(&self) -> CpuState {
        // Opcodes are organized so that codes in the same column generally use one of two
        // addressing modes
        use self::CpuState::*;

        let row = self.curr_op >> 4;
        let col = self.curr_op % 16;
        match col {
            0 => {
                if row % 2 == 1 || row > 7{
                    Immediate
                } else {
                    if row == 0 || row == 4 || row == 6 {
                        Implied
                    } else {
                        AddressLo
                    }
                }
            },
            1 | 3 => {
                if row % 2 == 1 {
                    AddressIndirectY
                } else {
                    AddressIndirectX
                }
            },
            2 => {
                match row {
                    8 | 0xa | 0xc | 0xe => Immediate,
                    _ => Halt,
                }
            },
            4 | 5 => {
                if row % 2 == 1 {
                    AddressZeropageX
                } else {
                    AddressZeropage
                }
            },
            6 => {
                if row % 2 == 0 {
                    AddressZeropage
                } else if row == 9 {
                    AddressZeropageY
                } else {
                    AddressZeropageX
                }
            },
            7 => {
                if row % 2 == 0 {
                    AddressZeropage
                } else if row == 9 || row == 0xa {
                    AddressZeropageY
                } else {
                    AddressZeropageX
                }
            },
            8 | 0xa => {
                Implied
            },
            9 | 0xb=> {
                if row % 2 == 0 {
                    Immediate
                } else {
                    AddressLoY
                }
            },
            0xc | 0xd  => {
                if row % 2 == 1 {
                    AddressLoX
                } else {
                    AddressLo
                }
            },
            0xe => {
                if row % 2 == 0 {
                    AddressLo
                } else if row == 9 || row == 0xa {
                    AddressLoY
                } else {
                    AddressLoX
                }
            },
            _ => {
                panic!("Unknown addressing mode for instruction ${:0>2X}", self.curr_op);
            },
        }
    }

    pub fn cycle(&mut self, debug: bool) {
        use self::CpuState::*;

        self.increment_pc();
        match self.state {
            ToLoad => {
                // Switch to read mode
                let pc = self.pc;
                self.set_addr_bus(pc);
                self.state = Fetch;
            },
            Fetch => {
                self.curr_op = self.data_bus;
                self.state = self.addressing_mode();
            },
            Implied => {
                self.state = self.do_instr(debug);
                if self.state == Fetch {
                    self.curr_op = self.data_bus;
                    self.state = self.addressing_mode();
                }
            },
            Immediate => {
                self.state = self.do_instr(debug);
            },
            Load => {
                self.state = self.do_instr(debug);
                if self.state == Fetch {
                    let pc = self.pc;
                    self.set_addr_bus(pc);
                }
            },
            Store => {
                self.rw = false;
                self.state = ToLoad;
            },
            AddressZeropage | AddressZeropageX | AddressZeropageY => {
                self.addr_hi = 0u8;
                self.addr_lo = self.read_data_bus();
                if self.state == AddressZeropageX {
                    self.addr_lo = self.addr_lo.wrapping_add(self.x);
                } else if self.state == AddressZeropageY {
                    self.addr_lo = self.addr_lo.wrapping_add(self.y);
                }
                let addr = self.addr_from_hi_lo();
                self.set_addr_bus(addr);
                self.state = Load;
            },
            AddressLo => {
                self.addr_lo = self.read_data_bus();
                self.state = AddressHi
            },
            AddressLoX => {
                self.addr_lo = self.read_data_bus();
                self.state = AddressHiX
            },
            AddressLoY => {
                self.addr_lo = self.read_data_bus();
                self.state = AddressHiY
            }
            AddressHi => {
                self.addr_hi = self.read_data_bus();
                let addr = self.addr_from_hi_lo();
                self.set_addr_bus(addr);
                self.state = Load;
            },
            AddressHiX => {
                self.addr_hi = self.read_data_bus();
                let addr = self.addr_from_hi_lo().wrapping_add(self.x as u16);
                self.set_addr_bus(addr);
                self.state = Load;
            },
            AddressHiY => {
                self.addr_hi = self.read_data_bus();
                let addr = self.addr_from_hi_lo().wrapping_add(self.y as u16);
                self.set_addr_bus(addr);
                self.state = Load;
            },
            PushWordHi => {
                let sp = self.get_stack_addr();
                self.set_addr_bus(sp);
                let hi_byte = (self.stack_word >> 8) as u8;
                self.set_data_bus(hi_byte);
                self.sp = self.sp.wrapping_sub(1);

                self.state = PushWordLo;
            },
            PushWordLo => {
                let sp = self.get_stack_addr();
                self.set_addr_bus(sp);
                let lo_byte = (self.stack_word & 0xff) as u8;
                self.set_data_bus(lo_byte);
                self.sp = self.sp.wrapping_sub(1);

                self.state = ToLoad;
            },
            PullWordHi => {
                if self.stack_word_ready {
                    self.stack_word += (self.data_bus as u16) << 8;
                    self.state = Immediate;
                } else {
                    self.sp = self.sp.wrapping_add(1);
                    let sp = self.get_stack_addr();
                    self.set_addr_bus(sp);

                    self.stack_word = self.data_bus as u16;
                    self.stack_word_ready = true;
                }
            },
            PullWordLo => {
                self.sp = self.sp.wrapping_add(1);
                let sp = self.get_stack_addr();
                self.set_addr_bus(sp);

                self.state = PullWordHi;

                self.stack_word_ready = false;
                self.stack_word = 0u16;
            },
            AddressIndirect => {
                self.addr_hi = 0;
                self.addr_lo = self.read_data_bus();
                let addr = self.addr_from_hi_lo();
                self.set_addr_bus(addr);

                self.state = AddressIndirectLo;
            },
            AddressIndirectX => {
                self.addr_hi = 0;
                self.addr_lo = self.read_data_bus().wrapping_add(self.x);
                let addr = self.addr_from_hi_lo();
                self.set_addr_bus(addr);

                self.state = AddressIndirectLo;
            },
            AddressIndirectY => {
                self.addr_hi = 0;
                self.addr_lo = self.read_data_bus().wrapping_add(self.y);
                let addr = self.addr_from_hi_lo();
                self.set_addr_bus(addr);

                self.state = AddressIndirectLo;
            },
            AddressIndirectLo => {
                self.addr_lo = self.read_data_bus();

                let hi_addr = self.addr_bus.wrapping_add(1);
                self.set_addr_bus(hi_addr);

                self.state = AddressIndirectHi;
            },
            AddressIndirectHi => {
                self.addr_hi = self.read_data_bus();
                let addr = self.addr_from_hi_lo();
                self.set_addr_bus(addr);

                self.state = Load;
            },
            Halt => {
                panic!("CPU halted");
            },
        };
        self.cycles += 1;
    }

    fn read_data_bus(&self) -> u8 {
        self.data_bus
    }

    fn set_data_bus(&mut self, value: u8) {
        self.data_bus = value;
        self.rw = false;
    }

    pub fn data_in(&mut self, value: u8) {
        if self.rw {
            self.data_bus = value;
        }
    }

    pub fn data_out(&self) -> u8 {
        self.data_bus
    }

    pub fn write_ddr(&mut self, value: u8) {
        self.data_direction_reg = value;
    }

    pub fn read_ddr(&self) -> u8 {
        self.data_direction_reg
    }

    pub fn write_dataport(&mut self, value: u8) {
        // TODO: This is not quite how the DDR masking works
        self.dataport = (self.data_direction_reg & value);
    }

    pub fn read_dataport(&self) -> u8 {
        self.dataport
    }

    fn get_stack_addr(&self) -> u16 {
        (self.sp as u16) + STACK_START_ADDR
    }

    fn increment_pc(&mut self) {
        use self::CpuState::*;
        if self.state != ToLoad && self.state != Load && self.state != Store && self.state != PushWordLo && self.state != PushWordHi && self.state != PullWordLo && self.state != PullWordHi && self.state != AddressIndirect && self.state != AddressIndirectLo && self.state != AddressIndirectHi {
            self.pc += 1;
            let pc = self.pc;
            self.set_addr_bus(pc);
        }
    }

    fn addr_from_hi_lo(&self) -> u16 {
        ((self.addr_hi as u16) << 8) + (self.addr_lo as u16)
    }

    // Apply an offset for relative addressing
    fn relative_branch(&mut self) {
        let offset = self.data_bus;
        if offset < 0x80 {
            self.pc = self.pc.wrapping_add(offset as u16);
        } else {
            self.pc = self.pc.wrapping_sub(0x100 - offset as u16);
        }
        let pc = self.pc;
        self.set_addr_bus(pc);
    }
}

impl fmt::Debug for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "  Cycle {:0>5} :: PC: ${:0>4X} // A: ${:0>2X} // X: ${:0>2X} // Y: ${:0>2X} // SP: ${:0>2X} // SR: {:0>8b}\n                 DB: ${:0>2X} // AB: ${:0>4X} // CO: {:0>2X} // RW: {:?} // S: {:?}",
               self.cycles, self.pc, self.a, self.x, self.y, self.sp, self.sr.to_u8(),
               self.data_bus, self.addr_bus, self.curr_op, self.rw, self.state
               )
    }
}
