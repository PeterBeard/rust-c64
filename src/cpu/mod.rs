// Copyright 2016 Peter Beard
// Distributed under the GNU GPL v2. For full terms, see the LICENSE file.
//
// Functions and datatypes related to the CPU

mod opcode;
mod status_register;

use self::opcode::Opcode;
use self::status_register::StatusRegister;

use std::fmt;

const RESET_VECTOR_ADDR: u16 = 0xfce2;
const STACK_START_ADDR: u16 = 0x0100;

const IRQ_VEC_LO_ADDR: u16 = 0xfffe;
const IRQ_VEC_HI_ADDR: u16 = 0xffff;

#[derive(Eq, PartialEq, Debug)]
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
    AddressZeropageXAdd,
    AddressZeropageY,
    AddressZeropageYAdd,

    AddressIndirectIndexed,
    AddressIndirectIndexedLo,
    AddressIndirectIndexedHi,
    AddressIndirectIndexedPageCross,

    AddressIndexedIndirect,
    AddressIndexedIndirectAdd,
    AddressIndexedIndirectLo,
    AddressIndexedIndirectHi,

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
    curr_op: Opcode,

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
            curr_op: Opcode::KIL,

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

    // Write an address to the address bus
    fn set_addr_bus(&mut self, addr: u16) {
        self.addr_bus = addr;
        self.addr_enable = true;
        self.rw = true;
    }

    // Do the action associated with an opcode
    fn do_instr(&mut self, debug: bool) -> CpuState {
        use self::CpuState::*;
        use self::opcode::Opcode::*;
        match self.curr_op {
            // ADC -- add with carry
            ADC => {
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

            // AND -- store A & M in A
            AND => {
                if debug {
					println!("AND #${:0>2X}", self.data_bus);
				}
                self.a = self.a & self.data_bus;
                self.sr.determine_zero(self.a);
                self.sr.determine_negative(self.a);
                Fetch
            },

            // ASL -- shift left one
            ASL => {
                // TODO: This should take an extra cycle when using absolute, x addressing
                if debug {
					println!("ASL");
				}
                if self.state == Implied {
                    self.sr.determine_carry(self.a);
                    self.a <<= 1;
                    self.sr.determine_zero(self.a);
                    self.sr.determine_negative(self.a);
                    Fetch
                } else {
                    let data = self.read_data_bus();
                    self.sr.determine_carry(data);
                    let data = data << 1;
                    self.set_data_bus(data);
                    self.sr.determine_zero(data);
                    self.sr.determine_negative(data);
                    Store
                }
            },

            // BCC -- branch if carry clear
            BCC => {
                if debug {
					println!("BCC ${:0>2X}", self.data_bus);
				}

                if !self.sr.carry {
                    self.relative_branch();
                }
                Fetch
            },

            // BCS -- branch if carry set
            BCS => {
                if debug {
					println!("BCS ${:0>2X}", self.data_bus);
				}
                self.pc += 2;
                if self.sr.carry {
                    self.relative_branch();
                }
                Fetch
            },
            
            // BEQ -- branch if zero
            BEQ => {
                if debug {
					println!("BEQ ${:0>2X}", self.data_bus);
				}

                if self.sr.zero_result {
                    self.relative_branch();
                }
                Fetch
            },

            // BIT -- test bits against A
            BIT => {
                if debug {
                    println!("BIT ${:0>2X}", self.read_data_bus());
                }

                let data = self.read_data_bus();
                self.a &= data;
                self.sr.overflow = (data & 0x80) == 0x80;
                self.sr.determine_negative(data);
                self.sr.determine_zero(self.a);
                Fetch
            },

            // BMI -- branch on minus
            BMI => {
                if debug {
					println!("BMI ${:0>2X}", self.data_bus);
				}

                if self.sr.negative {
                    self.relative_branch();
                }
                Fetch
            },
            
            // BNE -- branch on result not zero
            BNE => {
                if debug {
					println!("BNE ${:0>2X}", self.data_bus);
				}

                if !self.sr.zero_result {
                    self.relative_branch();
                }
                Fetch
            },

            // BPL -- branch if plus
            BPL => {
                if debug {
					println!("BPL ${:0>2X}", self.data_bus);
				}

                if !self.sr.negative {
                    self.relative_branch();
                }
                Fetch
            },

            // BRK -- force break
            BRK => {
                if debug {
					println!("BRK");
				}
                if self.state == Implied {
                    self.stack_word_ready = false;
                    self.stack_word = self.pc.wrapping_add(2);
                    PushWordHi
                } else if self.state == ToLoad {
                    if !self.stack_word_ready {
                        self.stack_word_ready = true;

                        let sp = self.get_stack_addr();
                        self.sp = self.sp.wrapping_sub(1);
                        self.set_addr_bus(sp);

                        let sr = self.sr.to_u8() | 24;  // Set BRK flag in the stored SR
                        self.set_data_bus(sr);
                        self.sr.int_disable = true;

                        Store
                    } else {
                        // Read interrupt vector
                        self.pc = IRQ_VEC_LO_ADDR;
                        self.set_addr_bus(IRQ_VEC_LO_ADDR);

                        AddressLo
                    }
                } else {
                    self.pc = self.addr_from_hi_lo();

                    Fetch
                }

            },

            // BVC -- branck on overflow clear
            BVC => {
                if debug {
					println!("BVC ${:0>2X}", self.read_data_bus());
				}

                if !self.sr.overflow {
                    self.relative_branch();
                }
                Fetch
            },

            // BVS -- branch on overflow set
            BVS => {
                if debug {
					println!("BVS ${:0>2X}", self.read_data_bus());
				}

                if self.sr.overflow {
                    self.relative_branch();
                }
                Fetch
            },

            // CLC -- clear carry flag
            CLC => {
                if debug {
					println!("CLC");
				}
                self.sr.carry = false;
                Fetch
            },

            // CLD -- clear decimal mode
            CLD => {
                if debug {
					println!("CLD");
				}
                self.sr.decimal = false;
                Fetch
            },

            // CLI -- clear interrupt disable
            CLI => {
                if debug {
                    println!("CLI");
                }
                self.sr.int_disable = false;
                Fetch
            },

            // CLV -- clear overflow
            CLV => {
                if debug {
                    println!("CLV");
                }
                self.sr.overflow = false;
                Fetch
            },

            // CMP -- compare with accumulator
            CMP => {
                if debug {
					println!("CMP (${:0>2X}), Y", self.addr_bus);
				}
                self.sr.compare(&self.a, &self.data_bus);
                Fetch
            },

            // CPX -- compare X to memory
            CPX => {
                if debug {
					println!("CPX #${:0>2X}", self.data_bus);
				}
                self.sr.compare(&self.x, &self.data_bus);
                Fetch
            },

            // CPY -- compare Y to memory
            CPY => {
                if debug {
					println!("CPY #${:0>2X}", self.read_data_bus());
				}
                self.sr.compare(&self.y, &self.data_bus);
                Fetch
            },

            // DEC -- decrement
            DEC => {
                if debug {
					println!("DEC ${:0>2X}", self.addr_lo);
				}
                let data = self.read_data_bus().wrapping_sub(1);
                self.sr.determine_negative(self.data_bus);
                self.sr.determine_zero(self.data_bus);
                self.set_data_bus(data);
                Store
            },

            // DEX -- decrement X
            DEX => {
                if debug {
					println!("DEX");
				}
                self.x = self.x.wrapping_sub(1);
                self.sr.determine_negative(self.x);
                self.sr.determine_zero(self.x);
                Fetch
            },

            // DEY -- decrement Y
            DEY => {
                if debug {
					println!("DEY");
				}
                self.y = self.y.wrapping_sub(1);
                self.sr.determine_negative(self.y);
                self.sr.determine_zero(self.y);
                Fetch
            },

            // EOR -- A XOR value
            EOR => {
                if debug {
					println!("EOR ${:0>2X}", self.read_data_bus());
				}
                self.a ^= self.read_data_bus();
                self.sr.determine_zero(self.a);
                self.sr.determine_negative(self.a);
                Fetch
            },

            // INC -- increment
            INC => {
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
            INX => {
                if debug {
					println!("INX");
				}
                self.x = self.x.wrapping_add(1);
                self.sr.determine_zero(self.x);
                self.sr.determine_negative(self.x);
                Fetch
            },

            // INY -- increment Y
            INY => {
                if debug {
					println!("INY");
				}
                self.y = self.y.wrapping_add(1);
                self.sr.determine_negative(self.y);
                self.sr.determine_zero(self.y);
                Fetch
            },

            // JMP -- jump
            JMP => {
                if debug {
					println!("JMP ${:0>4X}", self.addr_from_hi_lo());
				}
                self.pc = self.addr_from_hi_lo();
                Fetch
            },

            // JSR -- jump and save return addr
            JSR => {
                self.stack_word = self.pc;
                self.pc = self.addr_from_hi_lo();
                if debug {
					println!("JSR ${:0>4X}", self.pc);
				}
                PushWordHi
            },

            // LDA -- load into A
            LDA => {
                if debug {
					println!("LDA ${:0>2X}", self.addr_lo);
				}
                self.a = self.data_bus;
                self.sr.determine_zero(self.a);
                self.sr.determine_negative(self.a);
                Fetch
            },
            
            // LDX -- load into X
            LDX => {
                if debug {
					println!("LDX #${:0>2X}", self.data_bus);
				}
                self.x = self.data_bus;
                self.sr.determine_zero(self.x);
                self.sr.determine_negative(self.x);
                Fetch
            },

            // LDY -- load into Y
            LDY => {
                if debug {
					println!("LDY #${:0>2X}", self.data_bus);
				}
                self.y = self.data_bus;
                self.sr.determine_zero(self.y);
                self.sr.determine_negative(self.y);
                Fetch
            },
            
            // LSR -- shift right one
            LSR => {
                if debug {
					println!("LSR");
				}
                if self.state == Implied {
                    self.sr.determine_carry(self.a);
                    self.a >>= 1;
                    self.sr.determine_zero(self.a);
                    self.sr.determine_negative(self.a);
                    Fetch
                } else {
                    let data = self.read_data_bus();
                    self.sr.determine_carry(data);
                    let data = data >> 1;
                    self.set_data_bus(data);
                    self.sr.determine_zero(data);
                    self.sr.determine_negative(data);
                    Store
                }
            },

            // NOP -- no op
            NOP => {
                Fetch
            },

            // ORA -- A | v
            ORA => {
                if debug {
					println!("ORA (${:0>2X}, X)", self.addr_lo);
				}
                self.a |= self.read_data_bus();
                self.sr.determine_zero(self.a);
                self.sr.determine_negative(self.a);
                Fetch
            },

            // PHA -- push A on stack
            PHA => {
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

            // PHP -- push SR on stack
            PHP => {
                if debug {
					println!("PHP");
				}
                let sr = self.sr.to_u8();
                self.set_data_bus(sr);
                let sp = self.get_stack_addr();
                self.set_addr_bus(sp);
                self.sp  = self.sp.wrapping_sub(1);
                Store
            },

            // PLA -- pull A from stack
            PLA => {
                if debug {
                    println!("PLA");
                }
                if self.state == Implied {
                    self.sp.wrapping_add(1);
                    let sp = self.get_stack_addr();
                    self.set_addr_bus(sp);
                    Load
                } else {
                    self.a = self.read_data_bus();
                    self.sr.determine_zero(self.a);
                    self.sr.determine_negative(self.a);
                    Fetch
                }
            },

            // PLP -- pull SR from stack
            PLP => {
                if debug {
                    println!("PLA");
                }
                if self.state == Implied {
                    self.sp.wrapping_add(1);
                    let sp = self.get_stack_addr();
                    self.set_addr_bus(sp);
                    Load
                } else {
                    let data = self.read_data_bus();
                    self.sr.from_u8(data);
                    Fetch
                }
            },
            
            // ROL -- rotate left
            ROL => {
                if debug {
					println!("ROL");
				}
                if self.state == Implied {
                    self.sr.determine_negative(self.a);
                    self.a = self.a.rotate_left(1);
                    self.sr.determine_zero(self.a);
                    self.sr.determine_carry(self.a);
                    Fetch
                } else {
                    let data = self.read_data_bus();
                    self.sr.determine_negative(data);
                    let data = data.rotate_left(1);
                    self.set_data_bus(data);
                    self.sr.determine_zero(data);
                    self.sr.determine_carry(data);
                    Store
                }
            },

            // ROR -- rotate one bit right
            ROR => {
                if debug {
					println!("ROR ${:0>2X}", self.addr_lo);
				}
                if self.state == Implied {
                    self.sr.determine_negative(self.a);
                    self.a = self.a.rotate_right(1);
                    self.sr.determine_zero(self.a);
                    self.sr.determine_carry(self.a);
                    Fetch
                } else {
                    let data = self.read_data_bus();
                    self.sr.determine_negative(data);
                    let data = data.rotate_right(1);
                    self.set_data_bus(data);
                    self.sr.determine_zero(data);
                    self.sr.determine_carry(data);
                    Store
                }
            },

            // RTI -- return from interrupt
            RTI => {
                panic!();
            },

            // RTS -- return from subroutine
            RTS => {
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

            // SBC -- subtract with carry
            SBC => {
                if debug {
                    println!("SBC #${:0>2X}", self.read_data_bus());
                }

                let data = if self.sr.carry {
                    !self.read_data_bus()
                } else {
                    (!self.read_data_bus()).wrapping_add(1)
                };

                // Determine whether a borrow will be required
                self.sr.carry = self.read_data_bus() > self.a;

                self.a = self.a.wrapping_add(data);

                self.sr.determine_negative(self.a);
                self.sr.determine_zero(self.a);
                let result =(self.a as i16) - (self.read_data_bus() as i16);
                self.sr.overflow = result < -128 || result > 127;
                    
                Fetch
            },

            // SEC -- set carry flag
            SEC => {
                if debug {
                    println!("SEC");
                }
                self.sr.carry = true;
                Fetch
            },

            // SED -- set decimal mode
            SED => {
                if debug {
                    println!("SED");
                }
                self.sr.decimal = true;
                Fetch
            },


            // SEI -- disable interrupts
            SEI => {
                if debug {
					println!("SEI");
				}
                self.sr.int_disable = true;
                Fetch
            },

            // STA -- store A
            STA => {
                if debug {
					println!("STA ${:0>4X}", self.addr_bus);
				}
                let a = self.a;
                self.set_data_bus(a);
                Store
            },
            
            // STX -- store x
            STX => {
                if debug {
					println!("STX ${:0>4X}", self.addr_bus);
				}
                let x = self.x;
                self.set_data_bus(x);
                Store
            },

            // STY -- store y
            STY => {
                if debug {
					println!("STY ${:0>2X}", self.addr_lo);
				}
                let y = self.y;
                self.set_data_bus(y);
                Store
            },

            // TAX -- transfer A to X
            TAX => {
                if debug {
					println!("TAX");
				}
                self.x = self.a;
                self.sr.determine_negative(self.x);
                self.sr.determine_zero(self.x);
                Fetch
            }

            // TAY -- transfer A to Y
            TAY => {
                if debug {
					println!("TAY");
				}
                self.y = self.a;
                self.sr.determine_negative(self.y);
                self.sr.determine_zero(self.y);
                Fetch
            }
            
            // TYA -- transfer Y to A
            TYA => {
                if debug {
					println!("TYA");
				}
                self.a = self.y;
                self.sr.determine_zero(self.a);
                self.sr.determine_negative(self.a);
                Fetch
            },

            // TSX -- transfer SP to X
            TSX => {
                if debug {
					println!("TSX");
				}
                self.x = self.sp;
                self.sr.determine_zero(self.x);
                self.sr.determine_negative(self.x);
                Fetch
            },

            // TXA -- transfer X to A
            TXA => {
                if debug {
					println!("TXA");
				}
                self.a = self.x;
                self.sr.determine_zero(self.a);
                self.sr.determine_negative(self.a);
                Fetch
            },

            // TXS -- transfer X to SP
            TXS => {
                if debug {
					println!("TXS");
				}
                self.sp = self.x;
                Fetch
            },

            // - Undocumented Instructions - //
            
            // ALR -- combination of AND and LSR
            ALR => {
                if debug {
                    println!("!! ALR $#{:0>2X}", self.read_data_bus());
                }
                self.a &= self.read_data_bus();
                self.sr.determine_carry(self.a);
                self.a >>= 1;
                self.sr.determine_zero(self.a);
                self.sr.determine_negative(self.a);

                Fetch
            },

            // ANC -- AND with carry
            ANC => {
                if debug {
                    println!("!! ANC $#{:0>2X}", self.read_data_bus());
                }
                self.a &= self.read_data_bus();
                self.sr.determine_zero(self.a);
                self.sr.determine_negative(self.a);
                self.sr.carry = self.sr.negative;

                Fetch
            },

            // ARR -- Combination of AND and ROR
            ARR => {
                if debug {
                    println!("!! ARR $#{:0>2X}", self.read_data_bus());
                }
                self.a &= self.read_data_bus();
                self.sr.determine_negative(self.a);

                self.a = self.a.rotate_right(1);
                self.sr.determine_zero(self.a);
                self.sr.carry = self.a & 0x40 == 0x40;
                self.sr.overflow = (self.a ^ (self.a << 1)) & 0x20 == 0x20;

                Fetch
            },

            // AXS -- Combination of AND and SBC without borrow
            AXS => {
                if debug {
                    println!("!! AXS $#{:0>2X}", self.read_data_bus());
                }
                self.a &= self.x;
                self.a = self.a.wrapping_sub(self.read_data_bus());
                self.sr.determine_negative(self.a);
                self.sr.determine_zero(self.a);
                self.sr.determine_carry(self.a);

                Fetch
            },

            // DCP -- DEC then CMP
            DCP => {
                if debug {
                    println!("!! DCP");
                }
                self.a = self.a.wrapping_sub(1);
                let data = self.read_data_bus().wrapping_sub(1);

                self.sr.determine_negative(data);
                self.sr.determine_zero(data);

                self.sr.compare(&self.a, &data);

                Fetch
            },

            // LAX -- LDA then TAX
            LAX => {
                if debug {
                    println!("!! LAX $#{:0>2X}", self.read_data_bus());
                }
                self.a = self.read_data_bus();
                self.x = self.read_data_bus();
                self.sr.determine_zero(self.x);
                self.sr.determine_negative(self.x);

                Fetch
            },

            // SAX -- store A & X
            SAX => {
                if debug {
                    println!("!! SAX");
                }
                let ax = self.a & self.x;
                self.set_data_bus(ax);

                Store
            },

            // KIL -- halt the CPU
            KIL => {
                Halt
            },

            _ => {
                panic!("Unimplemented instruction {:?}", self.curr_op)
            }
        }
    }

    // Determine the addressing mode of the current instruction
    fn addressing_mode(&self) -> CpuState {
        // Opcodes are organized so that codes in the same column generally use one of two
        // addressing modes
        use self::CpuState::*;

        let row = self.read_data_bus() >> 4;
        let col = self.read_data_bus() % 16;
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
                    AddressIndirectIndexed
                } else {
                    AddressIndexedIndirect
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
                panic!("Unknown addressing mode for instruction {:?}", self.curr_op);
            },
        }
    }

    pub fn cycle(&mut self, debug: bool) {
        use self::CpuState::*;

        self.increment_pc();
        match self.state {
            ToLoad => {
                // Switch to read mode
                // BRK is a special case
                if self.curr_op != Opcode::BRK {
                    let pc = self.pc;
                    self.set_addr_bus(pc);
                    self.state = Fetch;
                } else {
                    self.state = self.do_instr(debug);
                }
            },
            Fetch => {
                self.curr_op = Opcode::from_u8(self.read_data_bus());
                self.state = self.addressing_mode();
            },
            Implied => {
                self.state = self.do_instr(debug);
                if self.state == Fetch {
                    self.curr_op = Opcode::from_u8(self.read_data_bus());
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
                let addr = self.addr_from_hi_lo();
                self.set_addr_bus(addr);

                if self.state == AddressZeropageX {
                    self.state = AddressZeropageXAdd;
                } else if self.state == AddressZeropageY {
                    self.state = AddressZeropageYAdd;
                } else {
                    self.state = Load;
                }
            },
            AddressZeropageXAdd => {
                self.addr_lo = self.addr_lo.wrapping_add(self.x);
                let addr = self.addr_from_hi_lo();
                self.set_addr_bus(addr);

                self.state = Load;
            },
            AddressZeropageYAdd => {
                self.addr_lo = self.addr_lo.wrapping_add(self.y);
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
            AddressIndexedIndirect => {
                self.addr_hi = 0u8;
                self.addr_lo = self.read_data_bus();
                let addr = self.addr_from_hi_lo();
                self.set_addr_bus(addr);

                self.state = AddressIndexedIndirectAdd;
            },
            AddressIndexedIndirectAdd => {
                let addr = self.addr_bus.wrapping_add(self.x as u16);
                self.set_addr_bus(addr);

                self.state = AddressIndexedIndirectLo;
            },
            AddressIndexedIndirectLo => {
                self.addr_lo = self.read_data_bus();
                let addr = self.addr_bus.wrapping_add(1);
                self.set_addr_bus(addr);

                self.state = AddressIndexedIndirectHi;
            },
            AddressIndexedIndirectHi => {
                self.addr_hi = self.read_data_bus();
                let addr = self.addr_from_hi_lo();
                self.set_addr_bus(addr);

                self.state = Load;
            },
            AddressIndirectIndexed => {
                self.addr_hi = 0u8;
                self.addr_lo = self.read_data_bus();
                let addr = self.addr_from_hi_lo();
                self.set_addr_bus(addr);

                self.state = AddressIndirectIndexedLo;
            },
            AddressIndirectIndexedLo => {
                self.addr_lo = self.addr_lo.wrapping_add(1);
                let addr = self.addr_from_hi_lo();

                self.addr_lo = self.read_data_bus();
                self.set_addr_bus(addr);

                self.state = AddressIndirectIndexedHi;
            },
            AddressIndirectIndexedHi => {
                self.addr_hi = self.read_data_bus();
                self.addr_lo = self.addr_lo.wrapping_add(self.y);
                let addr = self.addr_from_hi_lo();
                self.set_addr_bus(addr);

                // Determine whether we crossed to the next page
                if (self.addr_lo as u16) + (self.y as u16) > 0xff {
                    self.state = AddressIndirectIndexedPageCross;
                } else {
                    self.state = Load;
                }
            },
            AddressIndirectIndexedPageCross => {
                self.addr_hi = self.addr_hi.wrapping_add(1);
                let addr = self.addr_from_hi_lo();
                self.set_addr_bus(addr);

                self.state = Load;
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
        if self.state == Fetch || self.state == AddressLo || self.state == AddressLoX || 
            self.state == AddressLoY || self.state == AddressHi || self.state == AddressHiX ||
            self.state == AddressHiY || self.state == AddressZeropage || self.state == AddressZeropageX ||
            self.state == AddressZeropageY || self.state == Immediate || self.state == Implied {
            self.pc = self.pc.wrapping_add(1);
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
               "  Cycle {:0>5} :: PC: ${:0>4X} // A: ${:0>2X} // X: ${:0>2X} // Y: ${:0>2X} // SP: ${:0>2X} // SR: {:0>8b}\n                 DB: ${:0>2X} // AB: ${:0>4X} // CO: {:?} // RW: {:?} // S: {:?}",
               self.cycles, self.pc, self.a, self.x, self.y, self.sp, self.sr.to_u8(),
               self.data_bus, self.addr_bus, self.curr_op, self.rw, self.state
               )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Run a program consisting of a single instruction
    fn run_program(program: &[u8], cpu: &mut Cpu) {
        let mut ram: [u8; 65536] = [0u8; 65536];

        for addr in 0..program.len() {
            ram[super::RESET_VECTOR_ADDR as usize + addr] = program[addr];
        }

        cpu.reset();

        loop {
            let addr = cpu.addr_bus as usize;

            if cpu.pc != super::RESET_VECTOR_ADDR && cpu.state == super::CpuState::Fetch {
                break;
            } else if cpu.state == super::CpuState::Implied {
                break;
            }

            if cpu.rw {
                cpu.data_in(ram[addr]);
            } else {
                ram[addr] = cpu.data_out();
            }
            println!("{:?}", cpu);
            cpu.cycle(false);
        }
    }
    // Test cycle-accuracy of instructions
    // ADC
    #[test]
    fn adc_imm_cycles() {
        let mut cpu = Cpu::new();

        let program = [0x69, 0x10];
        run_program(&program[..], &mut cpu);

        assert_eq!(2, cpu.cycles)
    }

    #[test]
    fn adc_zp_cycles() {
        let mut cpu = Cpu::new();

        let program = [0x65, 0x00];
        run_program(&program[..], &mut cpu);

        assert_eq!(3, cpu.cycles)
    }

    #[test]
    fn adc_zpx_cycles() {
        let mut cpu = Cpu::new();

        let program = [0x75, 0x00];
        run_program(&program[..], &mut cpu);

        assert_eq!(4, cpu.cycles)
    }

    #[test]
    fn adc_abs_cycles() {
        let mut cpu = Cpu::new();

        let program = [0x6d, 0x00, 0x0f];
        run_program(&program[..], &mut cpu);

        assert_eq!(4, cpu.cycles)
    }

    #[test]
    fn adc_absx_cycles() {
        let mut cpu = Cpu::new();

        let program = [0x7d, 0x00, 0x0f];
        run_program(&program[..], &mut cpu);

        assert_eq!(4, cpu.cycles)
    }

    #[test]
    fn adc_absy_cycles() {
        let mut cpu = Cpu::new();

        let program = [0x79, 0x00, 0x0f];
        run_program(&program[..], &mut cpu);

        assert_eq!(4, cpu.cycles)
    }

    #[test]
    fn adc_indx_cycles() {
        let mut cpu = Cpu::new();

        let program = [0x61, 0x00];
        run_program(&program[..], &mut cpu);

        assert_eq!(6, cpu.cycles)
    }

    #[test]
    fn adc_indy_cycles() {
        let mut cpu = Cpu::new();

        let program = [0x71, 0x00];
        run_program(&program[..], &mut cpu);

        assert_eq!(5, cpu.cycles)
    }

    // AND
    #[test]
    fn and_imm_cycles() {
        let mut cpu = Cpu::new();

        let program = [0x29, 0x00];
        run_program(&program[..], &mut cpu);

        assert_eq!(2, cpu.cycles)
    }

    #[test]
    fn and_zp_cycles() {
        let mut cpu = Cpu::new();

        let program = [0x25, 0x00];
        run_program(&program[..], &mut cpu);

        assert_eq!(3, cpu.cycles)
    }

    #[test]
    fn and_zpx_cycles() {
        let mut cpu = Cpu::new();

        let program = [0x35, 0x00];
        run_program(&program[..], &mut cpu);

        assert_eq!(4, cpu.cycles)
    }

    #[test]
    fn and_abs_cycles() {
        let mut cpu = Cpu::new();

        let program = [0x2d, 0x00, 0x0f];
        run_program(&program[..], &mut cpu);

        assert_eq!(4, cpu.cycles)
    }

    #[test]
    fn and_absx_cycles() {
        let mut cpu = Cpu::new();

        let program = [0x3d, 0x00, 0x0f];
        run_program(&program[..], &mut cpu);

        assert_eq!(4, cpu.cycles)
    }

    #[test]
    fn and_absy_cycles() {
        let mut cpu = Cpu::new();

        let program = [0x39, 0x00];
        run_program(&program[..], &mut cpu);

        assert_eq!(4, cpu.cycles)
    }

    #[test]
    fn and_indx_cycles() {
        let mut cpu = Cpu::new();

        let program = [0x21, 0x00];
        run_program(&program[..], &mut cpu);

        assert_eq!(6, cpu.cycles)
    }

    #[test]
    fn and_indy_cycles() {
        let mut cpu = Cpu::new();

        let program = [0x31, 0x00];
        run_program(&program[..], &mut cpu);

        assert_eq!(5, cpu.cycles)
    }

    // ASL
    //#[test]
    fn asl_impl_cycles() {
        let mut cpu = Cpu::new();

        let program = [0x0a];
        run_program(&program[..], &mut cpu);

        assert_eq!(2, cpu.cycles);
    }

    #[test]
    fn asl_zp_cycles() {
        let mut cpu = Cpu::new();

        let program = [0x06, 0x00];
        run_program(&program[..], &mut cpu);

        assert_eq!(5, cpu.cycles);
    }

    #[test]
    fn asl_zpx_cycles() {
        let mut cpu = Cpu::new();

        let program = [0x16, 0x00];
        run_program(&program[..], &mut cpu);

        assert_eq!(6, cpu.cycles);
    }

    #[test]
    fn asl_abs_cycles() {
        let mut cpu = Cpu::new();

        let program = [0x0e, 0x00, 0x0f];
        run_program(&program[..], &mut cpu);

        assert_eq!(6, cpu.cycles);
    }

    //#[test]
    fn asl_absx_cycles() {
        let mut cpu = Cpu::new();

        let program = [0x1e, 0x00, 0x0f];
        run_program(&program[..], &mut cpu);

        assert_eq!(7, cpu.cycles);
    }
}
