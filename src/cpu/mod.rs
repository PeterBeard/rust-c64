// Copyright 2016 Peter Beard
// Distributed under the GNU GPL v2. For full terms, see the LICENSE file.
//
// Functions and datatypes related to the CPU

mod opcode;
mod addressing_mode;
mod instruction;
mod status_register;

use self::opcode::Opcode;
use self::instruction::Instruction;

use self::status_register::StatusRegister;

use std::fmt;

const RESET_VECTOR_ADDR: u16 = 0xfce2;
const STACK_START_ADDR: u16 = 0x0100;
const IRQ_VEC_ADDR: u16 = 0xfffe;

#[derive(Eq, PartialEq, Debug)]
enum CpuState {
    Interrupt,
    InterruptLo,
    InterruptHi,

    Fetch,
    Load,
    Store,
    PushWordLo,
    PushWordHi,
    PullWordLo,
    PullWordHi,

    Address,

    ToLoad,
    Halt,
}

pub struct Cpu {
    // Input pins
    irq: bool,

    // Registers
    pc: u16,
    a: u8,
    x: u8,
    y: u8,
    sr: StatusRegister,
    sp: u8,
    dataport: u8,
    // ROM status flags derived from the dataport value
    kernal_rom_enabled: bool,
    basic_rom_enabled: bool,
    char_rom_enabled: bool,
    io_enabled: bool,

    data_direction_reg: u8,

    cycles: u64,
    curr_instr: Instruction,

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
            irq: false,

            pc: 0u16,
            a: 0u8,
            x: 0u8,
            y: 0u8,
            sr: StatusRegister::new(),
            sp: 0u8,
            dataport: 0u8,
            kernal_rom_enabled: false,
            basic_rom_enabled: false,
            char_rom_enabled: false,
            io_enabled: false,

            data_direction_reg: 0u8,

            cycles: 0u64,
            curr_instr: Instruction::new(),

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
        self.write_dataport(0x37);

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
        use self::addressing_mode::AddressingMode::*;

        match (self.curr_instr.opcode, self.curr_instr.addr_mode) {
            // ADC -- add with carry
            (ADC, _) => {
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
            (AND, _) => {
                if debug {
					println!("AND #${:0>2X}", self.data_bus);
				}
                self.a = self.a & self.data_bus;
                self.sr.determine_zero(self.a);
                self.sr.determine_negative(self.a);
                Fetch
            },

            // ASL -- shift left one
            (ASL, addr_mode) => {
                if debug {
					println!("ASL");
				}
                if addr_mode == Implied {
                    self.sr.determine_carry(self.a);
                    self.a <<= 1;
                    self.sr.determine_zero(self.a);
                    self.sr.determine_negative(self.a);
                    Fetch
                } else if addr_mode == AbsoluteHiX {
                    // Kill a cycle for absolute, x
                    self.curr_instr.addr_mode = AbsoluteHi;
                    Load
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
            (BCC, _) => {
                if debug {
					println!("BCC ${:0>2X}", self.data_bus);
				}

                if !self.sr.carry {
                    self.relative_branch();
                }
                Fetch
            },

            // BCS -- branch if carry set
            (BCS, _) => {
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
            (BEQ, _) => {
                if debug {
					println!("BEQ ${:0>2X}", self.data_bus);
				}

                if self.sr.zero_result {
                    self.relative_branch();
                }
                Fetch
            },

            // BIT -- test bits against A
            (BIT, _) => {
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
            (BMI, _) => {
                if debug {
					println!("BMI ${:0>2X}", self.data_bus);
				}

                if self.sr.negative {
                    self.relative_branch();
                }
                Fetch
            },
            
            // BNE -- branch on result not zero
            (BNE, _) => {
                if debug {
					println!("BNE ${:0>2X}", self.data_bus);
				}

                if !self.sr.zero_result {
                    self.relative_branch();
                }
                Fetch
            },

            // BPL -- branch if plus
            (BPL, _) => {
                if debug {
					println!("BPL ${:0>2X}", self.data_bus);
				}

                if !self.sr.negative {
                    self.relative_branch();
                }
                Fetch
            },

            // BRK -- force break
            // TODO: This should take 7 cycles, not 10
            (BRK, addr_mode) => {
                if debug {
					println!("BRK");
				}
                if self.state == Address && addr_mode == Implied {
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
                        self.pc = IRQ_VEC_ADDR;
                        self.set_addr_bus(IRQ_VEC_ADDR);
                        self.curr_instr.addr_mode = AbsoluteLo;

                        Address
                    }
                } else {
                    self.pc = self.addr_from_hi_lo();

                    Fetch
                }

            },

            // BVC -- branck on overflow clear
            (BVC, _) => {
                if debug {
					println!("BVC ${:0>2X}", self.read_data_bus());
				}

                if !self.sr.overflow {
                    self.relative_branch();
                }
                Fetch
            },

            // BVS -- branch on overflow set
            (BVS, _) => {
                if debug {
					println!("BVS ${:0>2X}", self.read_data_bus());
				}

                if self.sr.overflow {
                    self.relative_branch();
                }
                Fetch
            },

            // CLC -- clear carry flag
            (CLC, _) => {
                if debug {
					println!("CLC");
				}
                self.sr.carry = false;
                Fetch
            },

            // CLD -- clear decimal mode
            (CLD, _) => {
                if debug {
					println!("CLD");
				}
                self.sr.decimal = false;
                Fetch
            },

            // CLI -- clear interrupt disable
            (CLI, _) => {
                if debug {
                    println!("CLI");
                }
                self.sr.int_disable = false;
                Fetch
            },

            // CLV -- clear overflow
            (CLV, _) => {
                if debug {
                    println!("CLV");
                }
                self.sr.overflow = false;
                Fetch
            },

            // CMP -- compare with accumulator
            (CMP, _) => {
                if debug {
					println!("CMP (${:0>2X}), Y", self.addr_bus);
				}
                self.sr.compare(&self.a, &self.data_bus);
                Fetch
            },

            // CPX -- compare X to memory
            (CPX, _) => {
                if debug {
					println!("CPX #${:0>2X}", self.data_bus);
				}
                self.sr.compare(&self.x, &self.data_bus);
                Fetch
            },

            // CPY -- compare Y to memory
            (CPY, _) => {
                if debug {
					println!("CPY #${:0>2X}", self.read_data_bus());
				}
                self.sr.compare(&self.y, &self.data_bus);
                Fetch
            },

            // DEC -- decrement
            (DEC, addr_mode) => {
                if debug {
					println!("DEC ${:0>2X}", self.addr_lo);
				}
                if addr_mode == AbsoluteHiX {
                    // Kill a cycle for absolute, x
                    self.curr_instr.addr_mode = AbsoluteHi;
                    Load
                } else {
                    let data = self.read_data_bus().wrapping_sub(1);
                    self.sr.determine_negative(self.data_bus);
                    self.sr.determine_zero(self.data_bus);
                    self.set_data_bus(data);
                    Store
                }
            },

            // DEX -- decrement X
            (DEX, _) => {
                if debug {
					println!("DEX");
				}
                self.x = self.x.wrapping_sub(1);
                self.sr.determine_negative(self.x);
                self.sr.determine_zero(self.x);
                Fetch
            },

            // DEY -- decrement Y
            (DEY, _) => {
                if debug {
					println!("DEY");
				}
                self.y = self.y.wrapping_sub(1);
                self.sr.determine_negative(self.y);
                self.sr.determine_zero(self.y);
                Fetch
            },

            // EOR -- A XOR value
            (EOR, _) => {
                if debug {
					println!("EOR ${:0>2X}", self.read_data_bus());
				}
                self.a ^= self.read_data_bus();
                self.sr.determine_zero(self.a);
                self.sr.determine_negative(self.a);
                Fetch
            },

            // INC -- increment
            (INC, addr_mode) => {
                if debug {
					println!("INC ${:0>2X}", self.addr_lo);
				}
                if addr_mode == AbsoluteHiX {
                    // Kill a cycle for absolute, x
                    self.curr_instr.addr_mode = AbsoluteHi;
                    Load
                } else {
                    let data = self.read_data_bus().wrapping_add(1);
                    self.sr.determine_negative(self.data_bus);
                    self.sr.determine_zero(self.data_bus);
                    self.set_data_bus(data);
                    Store
                }
            },

            // INX -- increment X
            (INX, _) => {
                if debug {
					println!("INX");
				}
                self.x = self.x.wrapping_add(1);
                self.sr.determine_zero(self.x);
                self.sr.determine_negative(self.x);
                Fetch
            },

            // INY -- increment Y
            (INY, _) => {
                if debug {
					println!("INY");
				}
                self.y = self.y.wrapping_add(1);
                self.sr.determine_negative(self.y);
                self.sr.determine_zero(self.y);
                Fetch
            },

            // JMP -- jump
            (JMP, _) => {
                if debug {
					println!("JMP ${:0>4X}", self.addr_from_hi_lo());
				}
                self.pc = self.addr_from_hi_lo();
                Fetch
            },

            // JSR -- jump and save return addr
            (JSR, _) => {
                self.stack_word = self.pc;
                self.pc = self.addr_from_hi_lo();
                if debug {
					println!("JSR ${:0>4X}", self.pc);
				}
                PushWordHi
            },

            // LDA -- load into A
            (LDA, _) => {
                if debug {
					println!("LDA ${:0>2X}", self.addr_lo);
				}
                self.a = self.data_bus;
                self.sr.determine_zero(self.a);
                self.sr.determine_negative(self.a);
                Fetch
            },
            
            // LDX -- load into X
            (LDX, _) => {
                if debug {
					println!("LDX #${:0>2X}", self.data_bus);
				}
                self.x = self.data_bus;
                self.sr.determine_zero(self.x);
                self.sr.determine_negative(self.x);
                Fetch
            },

            // LDY -- load into Y
            (LDY, _) => {
                if debug {
					println!("LDY #${:0>2X}", self.data_bus);
				}
                self.y = self.data_bus;
                self.sr.determine_zero(self.y);
                self.sr.determine_negative(self.y);
                Fetch
            },
            
            // LSR -- shift right one
            (LSR, addr_mode) => {
                if debug {
					println!("LSR");
				}
                if addr_mode == Implied {
                    self.sr.determine_carry(self.a);
                    self.a >>= 1;
                    self.sr.determine_zero(self.a);
                    self.sr.determine_negative(self.a);
                    Fetch
                } else if addr_mode == AbsoluteHiX {
                    // Kill a cycle for absolute, x
                    self.curr_instr.addr_mode = AbsoluteHi;
                    Load
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
            (NOP, _) => {
                Fetch
            },

            // ORA -- A | v
            (ORA, _) => {
                if debug {
					println!("ORA (${:0>2X}, X)", self.addr_lo);
				}
                self.a |= self.read_data_bus();
                self.sr.determine_zero(self.a);
                self.sr.determine_negative(self.a);
                Fetch
            },

            // PHA -- push A on stack
            // TODO: Cycle counts are wrong for the four stack functions
            (PHA, _) => {
                if debug {
					println!("PHA");
				}
                let a = self.a;
                self.set_data_bus(a);
                let sp = self.get_stack_addr();
                self.set_addr_bus(sp);
                self.sp  = self.sp.wrapping_sub(1);
                self.pc = self.pc.wrapping_add(1);

                Store
            },

            // PHP -- push SR on stack
            (PHP, _) => {
                if debug {
					println!("PHP");
				}
                let sr = self.sr.to_u8();
                self.set_data_bus(sr);
                let sp = self.get_stack_addr();
                self.set_addr_bus(sp);
                self.sp  = self.sp.wrapping_sub(1);
                self.pc = self.pc.wrapping_add(1);

                Store
            },

            // PLA -- pull A from stack
            (PLA, addr_mode) => {
                if debug {
                    println!("PLA");
                }
                if addr_mode == Implied {
                    self.sp.wrapping_add(1);
                    let sp = self.get_stack_addr();
                    self.set_addr_bus(sp);
                    self.pc = self.pc.wrapping_add(1);

                    Load
                } else {
                    self.a = self.read_data_bus();
                    self.sr.determine_zero(self.a);
                    self.sr.determine_negative(self.a);
                    Fetch
                }
            },

            // PLP -- pull SR from stack
            (PLP, addr_mode) => {
                if debug {
                    println!("PLA");
                }
                if addr_mode == Implied {
                    self.sp.wrapping_add(1);
                    let sp = self.get_stack_addr();
                    self.set_addr_bus(sp);
                    self.pc = self.pc.wrapping_add(1);

                    Load
                } else {
                    let data = self.read_data_bus();
                    self.sr.from_u8(data);
                    Fetch
                }
            },
            
            // ROL -- rotate left
            (ROL, addr_mode) => {
                if debug {
					println!("ROL");
				}
                if addr_mode == Implied {
                    self.sr.determine_negative(self.a);
                    self.a = self.a.rotate_left(1);
                    self.sr.determine_zero(self.a);
                    self.sr.determine_carry(self.a);
                    Fetch
                } else if addr_mode == AbsoluteHiX {
                    // Kill a cycle for absolute, x
                    self.curr_instr.addr_mode = AbsoluteHi;
                    Load
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
            (ROR, addr_mode) => {
                if debug {
					println!("ROR ${:0>2X}", self.addr_lo);
				}
                if addr_mode == Implied {
                    self.sr.determine_negative(self.a);
                    self.a = self.a.rotate_right(1);
                    self.sr.determine_zero(self.a);
                    self.sr.determine_carry(self.a);
                    Fetch
                } else if addr_mode == AbsoluteHiX {
                    // Kill a cycle for absolute, x
                    self.curr_instr.addr_mode = AbsoluteHi;
                    Load
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
            (RTI, _) => {
                panic!();
            },

            // RTS -- return from subroutine
            (RTS, _) => {
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
            (SBC, _) => {
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
            (SEC, _) => {
                if debug {
                    println!("SEC");
                }
                self.sr.carry = true;
                Fetch
            },

            // SED -- set decimal mode
            (SED, _) => {
                if debug {
                    println!("SED");
                }
                self.sr.decimal = true;
                Fetch
            },


            // SEI -- disable interrupts
            (SEI, _) => {
                if debug {
					println!("SEI");
				}
                self.sr.int_disable = true;
                Fetch
            },

            // STA -- store A
            // TODO: All addressing modes for STA take a few cycles too long
            (STA, _) => {
                if debug {
					println!("STA ${:0>4X}", self.addr_bus);
				}
                let a = self.a;
                self.set_data_bus(a);
                Store
            },
            
            // STX -- store x
            (STX, _) => {
                if debug {
					println!("STX ${:0>4X}", self.addr_bus);
				}
                let x = self.x;
                self.set_data_bus(x);
                Store
            },

            // STY -- store y
            (STY, _) => {
                if debug {
					println!("STY ${:0>2X}", self.addr_lo);
				}
                let y = self.y;
                self.set_data_bus(y);
                Store
            },

            // TAX -- transfer A to X
            (TAX, _) => {
                if debug {
					println!("TAX");
				}
                self.x = self.a;
                self.sr.determine_negative(self.x);
                self.sr.determine_zero(self.x);
                Fetch
            }

            // TAY -- transfer A to Y
            (TAY, _) => {
                if debug {
					println!("TAY");
				}
                self.y = self.a;
                self.sr.determine_negative(self.y);
                self.sr.determine_zero(self.y);
                Fetch
            }
            
            // TYA -- transfer Y to A
            (TYA, _) => {
                if debug {
					println!("TYA");
				}
                self.a = self.y;
                self.sr.determine_zero(self.a);
                self.sr.determine_negative(self.a);
                Fetch
            },

            // TSX -- transfer SP to X
            (TSX, _) => {
                if debug {
					println!("TSX");
				}
                self.x = self.sp;
                self.sr.determine_zero(self.x);
                self.sr.determine_negative(self.x);
                Fetch
            },

            // TXA -- transfer X to A
            (TXA, _) => {
                if debug {
					println!("TXA");
				}
                self.a = self.x;
                self.sr.determine_zero(self.a);
                self.sr.determine_negative(self.a);
                Fetch
            },

            // TXS -- transfer X to SP
            (TXS, _) => {
                if debug {
					println!("TXS");
				}
                self.sp = self.x;
                Fetch
            },

            // - Undocumented Instructions - //
            
            // ALR -- combination of AND and LSR
            (ALR, _) => {
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
            (ANC, _) => {
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
            (ARR, _) => {
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
            (AXS, _) => {
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
            (DCP, _) => {
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
            (LAX, _) => {
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
            (SAX, _) => {
                if debug {
                    println!("!! SAX");
                }
                let ax = self.a & self.x;
                self.set_data_bus(ax);

                Store
            },

            // KIL -- halt the CPU
            (KIL, _) => {
                Halt
            },

            (_, _) => {
                panic!("Unimplemented instruction {:?}", self.curr_instr)
            }
        }
    }

    pub fn cycle(&mut self, debug: bool) {
        use self::CpuState::*;

        self.increment_pc();
        let next_state = match self.state {
            ToLoad => {
                // Switch to read mode
                // BRK is a special case
                if self.curr_instr.opcode != Opcode::BRK {
                    let pc = self.pc;
                    self.set_addr_bus(pc);
                    Fetch
                } else {
                    self.do_instr(debug)
                }
            },
            Interrupt => {
                // Ignore the interrupt if disabled
                if self.sr.int_disable {
                    self.irq = false;
                    Fetch
                } else {
                    // Trigger a BRK and load the IRQ routine address
                    if self.curr_instr.opcode != Opcode::BRK {
                        self.curr_instr = Instruction::from_u8(0x00);

                        Address
                    } else {
                        self.pc = IRQ_VEC_ADDR;

                        InterruptLo
                    }
                }
            },
            InterruptLo => {
                self.addr_lo = self.read_data_bus();
                InterruptHi
            },
            InterruptHi => {
                self.addr_hi = self.read_data_bus();
                let addr = self.addr_from_hi_lo();
                self.pc = self.addr_from_hi_lo();
                self.set_addr_bus(addr);

                self.irq = false;
                Fetch
            },
            Fetch => {

                if !self.irq {
                    self.curr_instr = Instruction::from_u8(self.read_data_bus());
                    Address
                } else {
                    Interrupt
                }
            },
            Load => {
                let s = self.do_instr(debug);
                if s == Fetch {
                    let pc = self.pc;
                    self.set_addr_bus(pc);
                }
                s
            },
            Store => {
                self.rw = false;
                ToLoad
            },
            Address => {
                use self::addressing_mode::AddressingMode::*;
                match self.curr_instr.addr_mode {
                    Zeropage | ZeropageX | ZeropageY => {
                        self.addr_hi = 0u8;
                        self.addr_lo = self.read_data_bus();
                        let addr = self.addr_from_hi_lo();
                        self.set_addr_bus(addr);

                        if self.curr_instr.addr_mode == ZeropageX {
                            self.curr_instr.addr_mode = ZeropageXAdd;
                            Address
                        } else if self.curr_instr.addr_mode == ZeropageY {
                            self.curr_instr.addr_mode = ZeropageYAdd;
                            Address
                        } else {
                            Load
                        }
                    },
                    ZeropageXAdd => {
                        self.addr_lo = self.addr_lo.wrapping_add(self.x);
                        let addr = self.addr_from_hi_lo();
                        self.set_addr_bus(addr);

                        Load
                    },
                    ZeropageYAdd => {
                        self.addr_lo = self.addr_lo.wrapping_add(self.y);
                        let addr = self.addr_from_hi_lo();
                        self.set_addr_bus(addr);

                        Load
                    },
                    AbsoluteLo => {
                        self.addr_lo = self.read_data_bus();
                        self.curr_instr.addr_mode = AbsoluteHi;
                        Address
                    },
                    AbsoluteLoX => {
                        self.addr_lo = self.read_data_bus();
                        self.curr_instr.addr_mode = AbsoluteHiX;
                        Address
                    },
                    AbsoluteLoY => {
                        self.addr_lo = self.read_data_bus();
                        self.curr_instr.addr_mode = AbsoluteHiY;
                        Address
                    }
                    AbsoluteHi => {
                        self.addr_hi = self.read_data_bus();
                        let addr = self.addr_from_hi_lo();
                        self.set_addr_bus(addr);

                        // JMP and JSR are special cases since we don't care what's on the data bus
                        if self.curr_instr.opcode == Opcode::JMP || self.curr_instr.opcode == Opcode::JSR {
                            self.do_instr(debug)
                        } else {
                            Load
                        }
                    },
                    AbsoluteHiX => {
                        self.addr_hi = self.read_data_bus();
                        let addr = self.addr_from_hi_lo().wrapping_add(self.x as u16);
                        self.set_addr_bus(addr);

                        Load
                    },
                    AbsoluteHiY => {
                        self.addr_hi = self.read_data_bus();
                        let addr = self.addr_from_hi_lo().wrapping_add(self.y as u16);
                        self.set_addr_bus(addr);

                        Load
                    },
                    IndirectLo => {
                        self.addr_lo = self.read_data_bus();
                        self.curr_instr.addr_mode = IndirectHi;

                        Address
                    },
                    IndirectHi => {
                        self.addr_hi = self.read_data_bus();
                        let addr = self.addr_from_hi_lo();
                        self.pc = addr;
                        self.set_addr_bus(addr);

                        self.curr_instr.addr_mode = AbsoluteLo;

                        Address
                    },
                    IndexedIndirect => {
                        self.addr_hi = 0u8;
                        self.addr_lo = self.read_data_bus();
                        let addr = self.addr_from_hi_lo();
                        self.set_addr_bus(addr);

                        self.curr_instr.addr_mode = IndexedIndirectAdd;
                        Address
                    },
                    IndexedIndirectAdd => {
                        let addr = self.addr_bus.wrapping_add(self.x as u16);
                        self.set_addr_bus(addr);

                        self.curr_instr.addr_mode = IndexedIndirectLo;
                        Address
                    },
                    IndexedIndirectLo => {
                        self.addr_lo = self.read_data_bus();
                        let addr = self.addr_bus.wrapping_add(1);
                        self.set_addr_bus(addr);

                        self.curr_instr.addr_mode = IndexedIndirectHi;
                        Address
                    },
                    IndexedIndirectHi => {
                        self.addr_hi = self.read_data_bus();
                        let addr = self.addr_from_hi_lo();
                        self.set_addr_bus(addr);

                        self.pc = self.pc.wrapping_add(1);
                        Load
                    },
                    IndirectIndexed => {
                        self.addr_hi = 0u8;
                        self.addr_lo = self.read_data_bus();
                        let addr = self.addr_from_hi_lo();
                        self.set_addr_bus(addr);

                        self.curr_instr.addr_mode = IndirectIndexedLo;
                        Address
                    },
                    IndirectIndexedLo => {
                        self.addr_lo = self.addr_lo.wrapping_add(1);
                        let addr = self.addr_from_hi_lo();

                        self.addr_lo = self.read_data_bus();
                        self.set_addr_bus(addr);

                        self.curr_instr.addr_mode = IndirectIndexedHi;
                        Address
                    },
                    IndirectIndexedHi => {
                        self.addr_hi = self.read_data_bus();
                        self.addr_lo = self.addr_lo.wrapping_add(self.y);
                        let addr = self.addr_from_hi_lo();
                        self.set_addr_bus(addr);

                        self.pc = self.pc.wrapping_add(1);

                        // Determine whether we crossed to the next page
                        if (self.addr_lo as u16) + (self.y as u16) > 0xff {
                            self.curr_instr.addr_mode = IndirectIndexedPageCross;
                            Address
                        } else {
                            Load
                        }
                    },
                    IndirectIndexedPageCross => {
                        self.addr_hi = self.addr_hi.wrapping_add(1);
                        let addr = self.addr_from_hi_lo();
                        self.set_addr_bus(addr);

                        Load
                    },

                    Implied => {
                        let s = self.do_instr(debug);
                        if s != Fetch {
                            // Program counter shouldn't have been incremented
                            self.pc = self.pc.wrapping_sub(1);
                        }
                        s
                    },
                    Immediate => {
                        self.do_instr(debug)
                    },
                }
            },
            PushWordLo => {
                let sp = self.get_stack_addr();
                self.set_addr_bus(sp);
                let lo_byte = (self.stack_word & 0xff) as u8;
                self.set_data_bus(lo_byte);
                self.sp = self.sp.wrapping_sub(1);

                ToLoad
            },
            PushWordHi => {
                let sp = self.get_stack_addr();
                self.set_addr_bus(sp);
                let hi_byte = (self.stack_word >> 8) as u8;
                self.set_data_bus(hi_byte);
                self.sp = self.sp.wrapping_sub(1);

                PushWordLo
            },
            PullWordHi => {
                if self.stack_word_ready {
                    self.stack_word += (self.data_bus as u16) << 8;
                    self.do_instr(debug)
                } else {
                    self.sp = self.sp.wrapping_add(1);
                    let sp = self.get_stack_addr();
                    self.set_addr_bus(sp);

                    self.stack_word = self.data_bus as u16;
                    self.stack_word_ready = true;
                    PullWordHi
                }
            },
            PullWordLo => {
                self.sp = self.sp.wrapping_add(1);
                let sp = self.get_stack_addr();
                self.set_addr_bus(sp);

                self.stack_word_ready = false;
                self.stack_word = 0u16;

                PullWordHi
            },
            Halt => {
                panic!("CPU halted");
            },
        };
        self.state = next_state;
        self.cycles = self.cycles.wrapping_add(1);
    }

    fn read_data_bus(&self) -> u8 {
        self.data_bus
    }

    fn set_data_bus(&mut self, value: u8) {
        self.data_bus = value;
        self.rw = false;
    }

    pub fn trigger_interrupt(&mut self) {
        self.irq = true;
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
        self.dataport = self.data_direction_reg & value;
        
        // Reset rom statuses
        let rom_status = self.read_dataport() & 7;
        self.kernal_rom_enabled = rom_status % 4 > 1;
        self.basic_rom_enabled = rom_status % 4 == 3;
        self.char_rom_enabled = rom_status < 4 && rom_status > 0;
        self.io_enabled = rom_status > 4;
    }

    pub fn krom_enabled(&self) -> bool {
        self.kernal_rom_enabled
    }

    pub fn brom_enabled(&self) -> bool {
        self.basic_rom_enabled
    }

    pub fn crom_enabled(&self) -> bool {
        self.char_rom_enabled
    }

    pub fn io_enabled(&self) -> bool {
        self.io_enabled
    }

    pub fn read_dataport(&self) -> u8 {
        self.dataport
    }

    fn get_stack_addr(&self) -> u16 {
        (self.sp as u16) + STACK_START_ADDR
    }

    fn increment_pc(&mut self) {
        use self::CpuState::*;
        match self.state {
            Fetch | InterruptLo => {
                self.pc = self.pc.wrapping_add(1);
                let pc = self.pc;
                self.set_addr_bus(pc);
            },
            Address => {
                use self::addressing_mode::AddressingMode::*;
                match self.curr_instr.addr_mode {
                    AbsoluteLo | AbsoluteLoX | AbsoluteLoY | AbsoluteHi | AbsoluteHiX | AbsoluteHiY |
                    Zeropage | ZeropageX | ZeropageY | Immediate | IndirectLo => {
                        self.pc = self.pc.wrapping_add(1);
                        let pc = self.pc;
                        self.set_addr_bus(pc);
                    },
                    _ => {
                        // Do nothing
                    },
                }
            },
            _ => {
                // Do nothing
            }
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
               "  Cycle {:0>5} :: PC: ${:0>4X} // A: ${:0>2X} // X: ${:0>2X} // Y: ${:0>2X} // SP: ${:0>2X} // SR: {:0>8b}\n                 DB: ${:0>2X} // AB: ${:0>4X} // CI: {:?} // RW: {:?} // S: {:?}",
               self.cycles, self.pc, self.a, self.x, self.y, self.sp, self.sr.to_u8(),
               self.data_bus, self.addr_bus, self.curr_instr, self.rw, self.state
               )
    }
}

#[cfg(test)]
mod test_mod;
