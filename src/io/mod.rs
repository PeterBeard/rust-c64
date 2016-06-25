// Copyright 2016 Peter Beard
// Distributed under the GNU GPL v3. For full terms, see the LICENSE file.
//
// Data structures and functions related to I/O devices

pub mod vic;
pub mod sid;
pub mod cia;

fn write_low_byte(word: u16, byte: u8) -> u16 {
    (word & 0xf0) + byte as u16
}

fn write_high_byte(word: u16, byte: u8) -> u16 {
    ((byte as u16) << 8) + (word >> 8)
}
