// Copyright 2016 Peter Beard
// Distributed under the GNU GPL v3. For full terms, see the LICENSE file.

mod cpu;
mod memory;

use cpu::Cpu;
use memory::Memory;

struct C64 {
    cpu: Cpu,
    ram: Memory,
}

impl C64 {
    pub fn new() -> C64 {
        C64 {
            cpu: Cpu::new(),
            ram: Memory::new(),
        }
    }

    pub fn run(&mut self) {
        self.ram.load_roms();
        self.cpu.run(&mut self.ram);
    }
}

fn main() {
    let mut commodore = C64::new();
    commodore.run();
}
