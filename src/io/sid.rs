// Copyright 2016 Peter Beard
// Distributed under the GNU GPL v3. For full terms, see the LICENSE file.
//
// Data structures and functions related to the SID sound chip

use super::{write_high_byte, write_low_byte};

pub const MIN_CONTROL_ADDR: usize = 0xd400;
pub const MAX_CONTROL_ADDR: usize = 0xd7ff;
const CONTROL_REG_COUNT: usize = 0x20;

pub struct Sid {
    v1_f: u16,       // Voice 1 frequency
    v1_pw: u16,      // Voice 1 pulse width
    v1_ctl: u8,     // Voice 1 control register
    v1_ad: u8,      // Voice 1 attack/decay register
    v1_sr: u8,      // Voice 1 sustain/release

    v2_f: u16,       // Voice 2 frequency
    v2_pw: u16,      // Voice 2 pulse width
    v2_ctl: u8,     // Voice 2 control register
    v2_ad: u8,      // Voice 2 attack/decay register
    v2_sr: u8,      // Voice 2 sustain/release

    v3_f: u16,       // Voice 3 frequency
    v3_pw: u16,      // Voice 3 pulse width
    v3_ctl: u8,     // Voice 3 control register
    v3_ad: u8,      // Voice 3 attack/decay register
    v3_sr: u8,      // Voice 3 sustain/release

    filter_co: u16,  // Filter cutoff frequency (11 bits)
    filter_ctl: u8, // Filter control
    vol_mode: u8,   // Volume/filter mode

    paddle_x: u8,   // X value of paddle at $DD00
    paddle_y: u8,   // Y value of paddle at $DD00

    v3_wave: u8,    // Voice 3 waveform
    v3_adsr: u8,    // Voice 3 envelope
}

impl Sid {
    pub fn new() -> Sid {
        Sid {
            v1_f: 0,
            v1_pw: 0,
            v1_ctl: 0,
            v1_ad: 0,
            v1_sr: 0,

            v2_f: 0,
            v2_pw: 0,
            v2_ctl: 0,
            v2_ad: 0,
            v2_sr: 0,

            v3_f: 0,
            v3_pw: 0,
            v3_ctl: 0,
            v3_ad: 0,
            v3_sr: 0,

            filter_co: 0,
            filter_ctl: 0,
            vol_mode: 0,

            paddle_x: 0,
            paddle_y: 0,

            v3_wave: 0,
            v3_adsr: 0,
        }
    }

    // Translate a memory address to a register index
    fn translate_addr(&self, addr: usize) -> u8 {
        if addr > MAX_CONTROL_ADDR || addr < MIN_CONTROL_ADDR {
            panic!("Invalid address for SID control register: ${:0>4X}", addr);
        }
        if (addr - MIN_CONTROL_ADDR) > CONTROL_REG_COUNT {
            return self.translate_addr(addr - CONTROL_REG_COUNT);
        }
        (addr - MIN_CONTROL_ADDR) as u8
    }


    pub fn read_register(&self, addr: usize) -> u8 {
        let reg = self.translate_addr(addr);

        // Most of the SID's registers are write-only
        match reg {
            0x19 => self.paddle_x,
            0x1a => self.paddle_y,
            0x1b => self.v3_wave,
            0x1c => self.v3_adsr,
            _ => 0
        }
    }

    pub fn write_register(&mut self, addr: usize, value: u8) {
        let reg = self.translate_addr(addr);

        match reg {
            0 => {
                self.v1_f = write_low_byte(self.v1_f, value);
            },
            1 => {
                self.v1_f = write_high_byte(self.v1_f, value);
            },
            2 => {
                self.v1_pw = write_low_byte(self.v1_pw, value);
            },
            3 => {
                self.v1_pw = write_high_byte(self.v1_pw, value);
            },
            4 => { self.v1_ctl = value; },
            5 => { self.v1_ad = value; },
            6 => { self.v1_sr = value; },


            7 => {
                self.v2_f = write_low_byte(self.v2_f, value);
            },
            8 => {
                self.v2_f = write_high_byte(self.v2_f, value);
            },
            9 => {
                self.v2_pw = write_low_byte(self.v2_pw, value);
            },
            10 => {
                self.v2_pw = write_high_byte(self.v2_pw, value);
            },
            11 => { self.v2_ctl = value; },
            12 => { self.v2_ad = value; },
            13 => { self.v2_sr = value; },

            14 => {
                self.v3_f = write_low_byte(self.v3_f, value);
            },
            15 => {
                self.v3_f = write_high_byte(self.v3_f, value);
            },
            16 => {
                self.v3_pw = write_low_byte(self.v3_pw, value);
            },
            17 => {
                self.v3_pw = write_high_byte(self.v3_pw, value);
            },
            18 => { self.v3_ctl = value; },
            19 => { self.v3_ad = value; },
            20 => { self.v3_sr = value; },

            21 => {
                // Write lower 3 bits
                self.filter_co = (self.filter_co & 0xf8) & ((value as u16) & 7);
            },
            22 => {
                // Write upper 8 bits
                self.filter_co = (self.filter_co & 0x07) & ((value as u16) << 3);
            },
            23 => { self.filter_ctl = value; },
            24 => { self.vol_mode = value; },
            _ => { /* Remaining registers are non-existent or read-only */ },
        };
    }
}
