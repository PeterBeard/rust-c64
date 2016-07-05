// Copyright 2016 Peter Beard
// Distributed under the GNU GPL v3. For full terms, see the LICENSE file.

mod cpu;
mod bus;
mod io;

use bus::Bus;

extern crate getopts;
use getopts::Options;
use std::env;

const RAM_IMAGE_FILE: &'static str = "src/ram-default-image.bin";

const ROM_DIR: &'static str = ".vice/c64";
const KERNAL_ROM_FILE: &'static str = "kernal";
const BASIC_ROM_FILE: &'static str = "basic";
const CHAR_ROM_FILE: &'static str = "chargen";

// Clock frequencies in mHz
const NTSC_CLK: u32 = 1022727714;
const PAL_CLK: u32 = 985248444;

struct C64 {
    ram_image_file: String,
    kernal_rom_file: String,
    basic_rom_file: String,
    char_rom_file: String,

    clock: u32,
    bus: Bus,
}

impl C64 {
    pub fn new(debug: bool) -> C64 {
        C64 {
            ram_image_file: String::new(),
            kernal_rom_file: String::new(),
            basic_rom_file: String::new(),
            char_rom_file: String::new(),

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

    pub fn set_ram_image_file(&mut self, fname: &str) {
        self.ram_image_file = fname.to_string();
    }

    pub fn set_kernal_rom(&mut self, fname: &str) {
        self.kernal_rom_file = fname.to_string();
    }

    pub fn set_basic_rom(&mut self, fname: &str) {
        self.basic_rom_file = fname.to_string();
    }

    pub fn set_char_rom(&mut self, fname: &str) {
        self.char_rom_file = fname.to_string();
    }

    pub fn run(&mut self) {
        self.bus.initialize(&self.ram_image_file);
        self.bus.load_roms(&self.kernal_rom_file, &self.basic_rom_file, &self.char_rom_file);
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
    opts.optopt("k", "kernal", "Location of the KERNAL ROM file.", "FILE");
    opts.optopt("b", "basic", "Location of the BASIC ROM file.", "FILE");
    opts.optopt("r", "char", "Location of the charater ROM file.", "FILE");

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

    // Set the locations of the ROM files
    commodore.set_ram_image_file(RAM_IMAGE_FILE);

    let mut home = env::home_dir().unwrap();
    home.push(ROM_DIR);

    match matches.opt_str("k") {
        Some(f) => {
            commodore.set_kernal_rom(&f);
        },
        None => {
            home.push(KERNAL_ROM_FILE);
            commodore.set_kernal_rom(home.to_str().unwrap());
            home.pop();
        },
    }

    match matches.opt_str("b") {
        Some(f) => {
            commodore.set_basic_rom(&f);
        },
        None => {
            home.push(BASIC_ROM_FILE);
            commodore.set_basic_rom(home.to_str().unwrap());
            home.pop();
        },
    }

    match matches.opt_str("r") {
        Some(f) => {
            commodore.set_char_rom(&f);
        },
        None => {
            home.push(CHAR_ROM_FILE);
            commodore.set_char_rom(home.to_str().unwrap());
        },
    }

    commodore.run();
}
