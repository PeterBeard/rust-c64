// Copyright 2016 Peter Beard
// Distributed under the GNU GPL v3. For full terms, see the LICENSE file.
//
// Data structures and functions related to CIA #1

use super::{write_high_byte, write_low_byte};

const CONTROL_REG_COUNT: usize = 0x10;

pub struct Cia {
    port_a: u8,         // Port A (keybord col and joystick 2)
    port_b: u8,         // Port B (keybord row and joystick 1)
    port_a_dir: u8,     // Port A data direction
    port_b_dir: u8,     // Port B data direction
    timer_a: u16,       // Timer A
    timer_b: u16,       // Timer B
    tod_ds: u8,         // Time of day in hundreds of ms (BCD)
    tod_s: u8,          // Time of day in seconds (BCD)
    tod_m: u8,          // Time of day in minutes (BCD)
    tod_h: u8,          // Time of day in hours (BCD)
    serial_shift: u8,   // Serial shift register
    int_enable: u8,     // Interrupt enable status
    int_status: u8,     // Interrupt status
    timer_a_ctl: u8,    // Timer A control register
    timer_b_ctl: u8,    // Timer B control register

    base_addr: usize,   // Base memory address for this CIA
}

impl Cia {
    pub fn new(base_addr: usize) -> Cia {
        Cia {
            port_a: 0,
            port_b: 0,
            port_a_dir: 0,
            port_b_dir: 0,
            timer_a: 0,
            timer_b: 0,
            tod_ds: 0,
            tod_s: 0,
            tod_m: 0,
            tod_h: 0,
            serial_shift: 0,
            int_enable: 0,
            int_status: 0,
            timer_a_ctl: 0,
            timer_b_ctl: 0,

            base_addr: base_addr,
        }
    }

    // Translate a memory address to a register index
    fn translate_addr(&self, addr: usize) -> u8 {
        if addr >= (self.base_addr + CONTROL_REG_COUNT) || addr < self.base_addr {
            panic!("Invalid address for CIA control register: ${:0>4X}", addr);
        }
        if (addr - self.base_addr) > CONTROL_REG_COUNT {
            return self.translate_addr(addr - CONTROL_REG_COUNT);
        }
        (addr - self.base_addr) as u8
    }


    pub fn read_register(&self, addr: usize) -> u8 {
        let reg = self.translate_addr(addr);

        match reg {
            0 => self.port_a,
            1 => self.port_b,
            2 => self.port_a_dir,
            3 => self.port_b_dir,
            4 => {
                // Low byte
                (self.timer_a & 0x0f) as u8
            },
            5 => {
                // High byte
                (self.timer_a >> 8) as u8
            },
            6 => {
                // Low byte
                (self.timer_b & 0x0f) as u8
            },
            7 => {
                // High byte
                (self.timer_b >> 8) as u8
            },
            8 => self.tod_ds,
            9 => self.tod_s,
            10 => self.tod_m,
            11 => self.tod_h,
            12 => self.serial_shift,
            13 => self.int_status,
            14 => self.timer_a_ctl,
            15 => self.timer_b_ctl,
            _ => 0
        }
    }

    pub fn write_register(&mut self, addr: usize, value: u8) {
        let reg = self.translate_addr(addr);
        // TODO: This is completely wrong and bad
        match reg {
            0 => { self.port_a = value; },
            1 => { self.port_b = value; },
            2 => { self.port_a_dir = value; },
            3 => { self.port_b_dir = value; },
            4 => { self.timer_a = write_low_byte(self.timer_a, value); },
            5 => { self.timer_a = write_high_byte(self.timer_a, value); },
            6 => { self.timer_b = write_low_byte(self.timer_b, value); },
            7 => { self.timer_b = write_high_byte(self.timer_b, value); },
            8 => { self.tod_ds = value; },
            9 => { self.tod_s = value; },
            10 => { self.tod_m = value; },
            11 => { self.tod_h = value; },
            12 => { self.serial_shift = value; },
            13 => { self.int_enable = value; },
            14 => { self.timer_a_ctl = value; },
            15 => { self.timer_b_ctl = value; },
            _ => { },
        }
    }
}
