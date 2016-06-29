// Copyright 2016 Peter Beard
// Distributed under the GNU GPL v2. For full terms, see the LICENSE file.
//
// Functions and datatypes related to the CPU status register

#[derive(Debug)]
pub struct StatusRegister {
    pub negative: bool,
    pub overflow: bool,
    pub expansion: bool,
    pub break_cmd: bool,
    pub decimal: bool,
    pub int_disable: bool,
    pub zero_result: bool,
    pub carry: bool,
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

