// Copyright 2016 Peter Beard
// Distributed under the GNU GPL v3. For full terms, see the LICENSE file.
//
// A 6510 instruction consists of an opcode and its addressing mode
use super::opcode::Opcode;
use super::addressing_mode::AddressingMode;

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub struct Instruction {
    pub opcode: Opcode,
    pub addr_mode: AddressingMode,
}

impl Instruction {
    pub fn new() -> Instruction {
        Instruction {
            opcode: Opcode::KIL,
            addr_mode: AddressingMode::Implied,
        }
    }
    pub fn from_u8(code: u8) -> Instruction {
        Instruction {
            opcode: Opcode::from_u8(code),
            addr_mode: AddressingMode::from_u8(code),
        }
    }
}
