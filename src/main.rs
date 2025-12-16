// Copyright 2016 Peter Beard
// Distributed under the GNU GPL v3. For full terms, see the LICENSE file.

mod bus;
mod cpu;
mod io;

use bus::Bus;

extern crate sdl2;
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Mod};
use sdl2::pixels::PixelFormatEnum;
use sdl2::surface::Surface;

extern crate getopts;
use getopts::Options;
use std::env;

use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

const SCREEN_X: u32 = 320;
const SCREEN_Y: u32 = 240;

const RAM_IMAGE_FILE: &'static str = "src/ram-default-image.bin";

const ROM_DIR: &'static str = ".vice/c64";
const KERNAL_ROM_FILE: &'static str = "kernal";
const BASIC_ROM_FILE: &'static str = "basic";
const CHAR_ROM_FILE: &'static str = "chargen";

// Clock frequencies in mHz
const NTSC_CLK: u32 = 1022727714;
const PAL_CLK: u32 = 985248444;

#[derive(Clone)]
pub struct Screen {
    width: u32,
    height: u32,
    pixels: Vec<(u8, u8, u8)>,
}

impl Screen {
    pub fn new(w: u32, h: u32) -> Screen {
        let mut p: Vec<(u8, u8, u8)> = Vec::with_capacity((w * h) as usize);
        for _ in 0..w * h {
            p.push((0, 0, 0));
        }

        Screen {
            width: w,
            height: h,
            pixels: p,
        }
    }

    pub fn set_pixel_at(&mut self, x: usize, y: usize, pixel: (u8, u8, u8)) {
        let index = y * (self.width as usize) + x;
        self.pixels[index] = pixel;
    }

    pub fn pixel_data(&self) -> Vec<u8> {
        // Convert pixel data to surface data
        let mut data: Vec<u8> = Vec::with_capacity(self.pixels.len() * 3);
        for i in 0..self.pixels.len() {
            data.push(self.pixels[i].0);
            data.push(self.pixels[i].1);
            data.push(self.pixels[i].2);
        }
        data
    }
}

pub enum EmulatorEvent {
    Quit,
    Key(Keycode, Mod),
}

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

    pub fn run(&mut self, screen_tx: Sender<Screen>, event_rx: Receiver<EmulatorEvent>) {
        self.bus.initialize(&self.ram_image_file);
        self.bus.load_roms(
            &self.kernal_rom_file,
            &self.basic_rom_file,
            &self.char_rom_file,
        );
        self.bus.run(self.clock, screen_tx, event_rx);
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
    opts.optopt(
        "c",
        "clock",
        "Clock speed to use. Options are PAL (default) or NTSC",
        "TYPE",
    );
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
        }
        None => {
            home.push(KERNAL_ROM_FILE);
            commodore.set_kernal_rom(home.to_str().unwrap());
            home.pop();
        }
    }

    match matches.opt_str("b") {
        Some(f) => {
            commodore.set_basic_rom(&f);
        }
        None => {
            home.push(BASIC_ROM_FILE);
            commodore.set_basic_rom(home.to_str().unwrap());
            home.pop();
        }
    }

    match matches.opt_str("r") {
        Some(f) => {
            commodore.set_char_rom(&f);
        }
        None => {
            home.push(CHAR_ROM_FILE);
            commodore.set_char_rom(home.to_str().unwrap());
        }
    }

    // Set up the screen
    let sdl2_context = sdl2::init().unwrap();
    let video_subsystem = sdl2_context.video().expect("Failed to get video context");
    let window = video_subsystem
        .window("rust-c64", SCREEN_X, SCREEN_Y)
        .build()
        .expect("Failed to build window");
    let mut canvas = window.into_canvas().build().expect("Failed to get canvas");
    let texture_creator = canvas.texture_creator();

    // Spawn a thread to run the emulator
    let (screen_tx, screen_rx) = mpsc::channel::<Screen>();
    let (event_tx, event_rx) = mpsc::channel::<EmulatorEvent>();
    let emulator = thread::spawn(move || {
        commodore.run(screen_tx, event_rx);
    });

    // Loop until quit event
    let mut events = sdl2_context.event_pump().unwrap();
    loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    event_tx.send(EmulatorEvent::Quit).unwrap();
                    break;
                }
                Event::KeyDown {
                    keycode: Some(keycode),
                    keymod: m,
                    ..
                }
                | Event::KeyUp {
                    keycode: Some(keycode),
                    keymod: m,
                    ..
                } => match event_tx.send(EmulatorEvent::Key(keycode, m)) {
                    Ok(_) => continue,
                    Err(e) => panic!("Error sending event to emulator: {}", e),
                },
                _ => {
                    continue;
                }
            }
        }

        // This will block until it gets a frame from the emulator. Is that what it should do?
        let scr = match screen_rx.recv() {
            Ok(s) => s,
            Err(_) => break,
        };

        let mut data = scr.pixel_data();
        let surf = Surface::from_data(
            &mut data[..],
            scr.width,
            scr.height,
            0,
            PixelFormatEnum::RGB24,
        )
        .unwrap();
        let tex = texture_creator.create_texture_from_surface(&surf).unwrap();

        canvas.clear();
        canvas
            .copy(&tex, None, None)
            .expect("Failed to copy texture");
        canvas.present();
    }
}
