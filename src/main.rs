// Copyright 2016 Peter Beard
// Distributed under the GNU GPL v3. For full terms, see the LICENSE file.

mod cpu;
mod bus;
mod io;

use bus::Bus;

// Clock frequencies in mHz
const NTSC_CLK: u32 = 1022727714;
const PAL_CLK: u32 = 985248444;

struct C64 {
    clock: u32,
    bus: Bus,
}

impl C64 {
    pub fn new(debug: bool) -> C64 {
        C64 {
            clock: 0,
            bus: Bus::new(debug),
        }
    }

    pub fn new_ntsc(debug: bool) -> C64 {
        let mut c = C64::new(debug);
        c.clock = NTSC_CLK;
        c
    }

    pub fn new_pal(debug: bool) -> C64 {
        let mut c = C64::new(debug);
        c.clock = PAL_CLK;
        c
    }

    pub fn run(&mut self) {
        self.bus.initialize();
        self.bus.load_roms();
        self.bus.run(self.clock);
    }
}

fn main() {
    let mut commodore = C64::new_pal(true);
    commodore.run();
}
