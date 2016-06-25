// Copyright 2016 Peter Beard
// Distributed under the GNU GPL v3. For full terms, see the LICENSE file.

mod cpu;
mod bus;
mod vic;

use cpu::Cpu;
use bus::Bus;

struct C64 {
    cpu: Cpu,
    bus: Bus,
}

impl C64 {
    pub fn new() -> C64 {
        C64 {
            cpu: Cpu::new(),
            bus: Bus::new(),
        }
    }

    pub fn run(&mut self) {
        self.bus.initialize();
        self.bus.load_roms();
        self.cpu.run(&mut self.bus);
    }
}

fn main() {
    let mut commodore = C64::new();
    commodore.run();
}
