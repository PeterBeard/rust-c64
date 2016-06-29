// Copyright 2016 Peter Beard
// Distributed under the GNU GPL v3. For full terms, see the LICENSE file.

mod cpu;
mod bus;
mod io;

use bus::Bus;

extern crate getopts;
use getopts::Options;
use std::env;

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

fn print_usage(pname: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", pname);
    print!("{}", opts.usage(&brief));
}

fn main() {
    // Read and parse command line arguments
    let args: Vec<String> = env::args().collect();
    let pname = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("c", "clock", "Clock speed to use. Options are PAL (default) or NTSC", "TYPE");
    opts.optflag("d", "debug", "Show debugging information");
    opts.optflag("h", "help", "Display this information");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => panic!(e.to_string()),
    };

    if matches.opt_present("h") {
        print_usage(&pname, opts);
        return;
    }

    let debug = matches.opt_present("d");
    let clocktype = match matches.opt_str("c") {
        Some(s) => s,
        None => "PAL".to_string(),
    };

    let mut commodore = match clocktype.as_ref() {
        "PAL" | "pal" => C64::new_pal(debug),
        "NTSC" | "ntsc" => C64::new_ntsc(debug),
        _ => panic!("Invalid clock type. See --help for options"),
    };

    commodore.run();
}
