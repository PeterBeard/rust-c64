// Copyright 2016 Peter Beard
// Distributed under the GNU GPL v3. For full terms, see the LICENSE file.
//
// Enum for the various addressing modes of the 6510 and a function for figuring out which one to
// use for a given opcode

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum AddressingMode {
    AbsoluteLo,
    AbsoluteLoX,
    AbsoluteLoY,
    AbsoluteHi,
    AbsoluteHiX,
    AbsoluteHiY,

    Zeropage,
    ZeropageX,
    ZeropageXAdd,
    ZeropageY,
    ZeropageYAdd,

    IndirectLo,
    IndirectHi,

    IndirectIndexed,
    IndirectIndexedLo,
    IndirectIndexedHi,
    IndirectIndexedPageCross,

    IndexedIndirect,
    IndexedIndirectAdd,
    IndexedIndirectLo,
    IndexedIndirectHi,

    Immediate,
    Implied,
}

impl AddressingMode {
    pub fn from_u8(code: u8) -> AddressingMode {
        // Opcodes are organized so that codes in the same column generally use one of two
        // addressing modes
        use self::AddressingMode::*;

        let row = code >> 4;
        let col = code % 16;
        match col {
            0 => {
                if row % 2 == 1 || row > 7{
                    Immediate
                } else {
                    if row == 0 || row == 4 || row == 6 {
                        Implied
                    } else {
                        AbsoluteLo
                    }
                }
            },
            1 | 3 => {
                if row % 2 == 1 {
                    IndirectIndexed
                } else {
                    IndexedIndirect
                }
            },
            2 => {
                match row {
                    8 | 0xa | 0xc | 0xe => Immediate,
                    _ => Implied,
                }
            },
            4 | 5 => {
                if row % 2 == 1 {
                    ZeropageX
                } else {
                    Zeropage
                }
            },
            6 => {
                if row % 2 == 0 {
                    Zeropage
                } else if row == 9 {
                    ZeropageY
                } else {
                    ZeropageX
                }
            },
            7 => {
                if row % 2 == 0 {
                    Zeropage
                } else if row == 9 || row == 0xa {
                    ZeropageY
                } else {
                    ZeropageX
                }
            },
            8 | 0xa => {
                Implied
            },
            9 | 0xb=> {
                if row % 2 == 0 {
                    Immediate
                } else {
                    AbsoluteLoY
                }
            },
            0xc | 0xd  => {
                if row % 2 == 1 {
                    AbsoluteLoX
                } else if row == 6 && col == 0xc {
                    IndirectLo
                } else {
                    AbsoluteLo
                }
            },
            0xe => {
                if row % 2 == 0 {
                    AbsoluteLo
                } else if row == 9 || row == 0xa {
                    AbsoluteLoY
                } else {
                    AbsoluteLoX
                }
            },
            _ => {
                panic!("Unknown addressing mode for instruction {:?}", code);
            },
        }
    }
}
