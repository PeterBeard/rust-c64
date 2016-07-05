// Copyright 2016 Peter Beard
// Distributed under the GNU GPL v2. For full terms, see the LICENSE file.
//
// Functions and datatypes relating to the system bus
use cpu::Cpu;

use io::vic;
use io::vic::Vic;

use io::sid;
use io::sid::Sid;

use io::cia::Cia;

use std::fs::File;
use std::io::{Read, Write, stdin, stdout};

use std::time::{Instant, SystemTime, Duration};
use std::thread::sleep;

const RAM_IMAGE_FILE: &'static str = "ram-default-image.bin";
const KERNAL_ROM_FILE: &'static str = "kernal.901227-03.bin";
const BASIC_ROM_FILE: &'static str = "basic.901226-01.bin";
const CHAR_ROM_FILE: &'static str = "characters.901225-01.bin";

const KERNAL_ROM_START: usize = 0xe000;
const BASIC_ROM_START: usize = 0xa000;
const CHAR_ROM_START: usize = 0xd000;

const KERNAL_ROM_SIZE: usize = 8192;
const BASIC_ROM_SIZE: usize = 8192;
const CHAR_ROM_SIZE: usize = 4096;

const IO_START: usize = 0xd000;
const IO_END: usize = 0xdfff;

const COLOR_RAM_START: usize = 0xd800;
const COLOR_RAM_END: usize = 0xdbff;

const CIA1_MIN_CONTROL_ADDR: usize = 0xdc00;
const CIA1_MAX_CONTROL_ADDR: usize = 0xdcff;
const CIA2_MIN_CONTROL_ADDR: usize = 0xdd00;
const CIA2_MAX_CONTROL_ADDR: usize = 0xddff;

#[derive(PartialEq, Eq)]
enum SystemMode {
    Run,
    DebugRun,
    DebugStep,
}

pub struct Bus {
    mode: SystemMode,
    ram: [u8; 65536],
    color_ram: [u8; 1024], // Only the 4 low bits of each byte are used
    kernal_rom: [u8; KERNAL_ROM_SIZE],
    basic_rom: [u8; BASIC_ROM_SIZE],
    char_rom: [u8; CHAR_ROM_SIZE],

    cpu: Cpu,
    vic: Vic,
    sid: Sid,
    cia_1: Cia,
    cia_2: Cia,
}

impl Bus {
    pub fn new(debug: bool) -> Bus {
        Bus {
            mode: if debug { SystemMode::DebugStep } else { SystemMode::Run },
            ram: [0u8; 65536],
            color_ram: [0u8; 1024],
            kernal_rom: [0u8; KERNAL_ROM_SIZE],
            basic_rom: [0u8; BASIC_ROM_SIZE],
            char_rom: [0u8; CHAR_ROM_SIZE],

            cpu: Cpu::new(),
            vic: Vic::new(),
            sid: Sid::new(),
            cia_1: Cia::new(CIA1_MIN_CONTROL_ADDR),
            cia_2: Cia::new(CIA2_MIN_CONTROL_ADDR),
        }
    }

    // Write default values into memory
    pub fn initialize(&mut self) {
        let mut file = File::open(RAM_IMAGE_FILE).unwrap();
        file.read(&mut self.ram).unwrap();
    }

    // Load data for the various ROM chips
    pub fn load_roms(&mut self) {
        let mut k_file = File::open(KERNAL_ROM_FILE).unwrap();
        k_file.read(&mut self.kernal_rom).unwrap();

        let mut b_file = File::open(BASIC_ROM_FILE).unwrap();
        b_file.read(&mut self.basic_rom).unwrap();

        let mut c_file = File::open(CHAR_ROM_FILE).unwrap();
        c_file.read(&mut self.char_rom).unwrap();
    }
    
    // Read a byte from the given address
    pub fn read_byte(&self, addr: usize) -> u8 {
        if addr == 0 {
            return self.cpu.read_ddr();
        } else if addr == 1 {
            return self.cpu.read_dataport();
        }

        if self.cpu.krom_enabled() && addr >= KERNAL_ROM_START && addr < KERNAL_ROM_START + KERNAL_ROM_SIZE
        {
            let offset_addr = addr - KERNAL_ROM_START;
            self.kernal_rom[offset_addr]

        } else if self.cpu.brom_enabled() && addr >= BASIC_ROM_START && addr < BASIC_ROM_START + BASIC_ROM_SIZE {
            let offset_addr = addr - BASIC_ROM_START;
            self.basic_rom[offset_addr]

        } else if self.cpu.crom_enabled() && addr >= CHAR_ROM_START && addr < CHAR_ROM_START + CHAR_ROM_SIZE {
            let offset_addr = addr - CHAR_ROM_START;
            self.char_rom[offset_addr]
        } else if self.cpu.io_enabled() && addr >= IO_START && addr <= IO_END {
            self.io_read(addr)
        } else {
            self.ram[addr]
        }
    }

    // Read from an I/O device
    fn io_read(&self, addr: usize) -> u8 {
        if addr >= vic::MIN_CONTROL_ADDR && addr <= vic::MAX_CONTROL_ADDR {
            self.vic.read_register(addr)
        } else if addr >= sid::MIN_CONTROL_ADDR && addr <= sid::MAX_CONTROL_ADDR {
            self.sid.read_register(addr)
        } else if addr >= COLOR_RAM_START && addr <= COLOR_RAM_END {
            self.color_ram[addr - COLOR_RAM_START]
        } else if addr >= CIA1_MIN_CONTROL_ADDR && addr <= CIA1_MAX_CONTROL_ADDR {
            self.cia_1.read_register(addr)
        } else if addr >= CIA2_MIN_CONTROL_ADDR && addr <= CIA2_MAX_CONTROL_ADDR {
            self.cia_2.read_register(addr)
        } else {
            panic!("Unimplemented I/O address: ${:0>4X}", addr);
        }
    }

    // Write a byte to the given address
    pub fn write_byte(&mut self, addr: usize, value: u8) {
        if addr == 0 {
            self.cpu.write_ddr(value);
        } else if addr == 1 {
            self.cpu.write_dataport(value);
        } else {
            let io_enabled = (self.cpu.read_dataport() & 7) > 4;

            if io_enabled && addr >= IO_START && addr <= IO_END {
                self.io_write(addr, value);
            } else {
                // System always writes to RAM even if it's masked by a ROM
                self.ram[addr] = value;
            }
        }
    }

    // Write to an I/O device
    fn io_write(&mut self, addr: usize, value: u8) {
        if addr >= vic::MIN_CONTROL_ADDR && addr <= vic::MAX_CONTROL_ADDR {
            self.vic.write_register(addr, value);
        } else if addr >= sid::MIN_CONTROL_ADDR && addr <= sid::MAX_CONTROL_ADDR {
            self.sid.write_register(addr, value);
        } else if addr >= COLOR_RAM_START && addr <= COLOR_RAM_END {
            self.color_ram[addr - COLOR_RAM_START] = value & 0x0f;
        } else if addr >= CIA1_MIN_CONTROL_ADDR && addr <= CIA1_MAX_CONTROL_ADDR {
            self.cia_1.write_register(addr, value);
        } else if addr >= CIA2_MIN_CONTROL_ADDR && addr <= CIA2_MAX_CONTROL_ADDR {
            self.cia_2.write_register(addr, value);
        } else {
            panic!("Unimplemented I/O address: ${:0>4X}", addr);
        }
    }

    // Convert a 14-bit VIC-II address to a 16-bit address
    fn convert_vic_ii_addr(&self, addr: u16) -> usize {
        // Two high bits come from port A on CIA 2
        let high_bits = (!self.read_byte(CIA2_MIN_CONTROL_ADDR)) & 0x03;
        let bank = 0x4000 * (high_bits as u16);
        (bank + (addr & 0x3ff)) as usize
    }

    pub fn run(&mut self, clock_speed_mhz: u32) {
        // Calculate the clock period in nanoseconds
        let clock_period_ns = Duration::new(0, (1_000_000_000_000f32/(clock_speed_mhz as f32)) as u32);

        self.cpu.reset();
        let mut cycles: u64 = 0;
        let mut speed = 0f32;

        let total_t = Instant::now();
        let mut idle_time = Duration::new(0, 0);
        let idle_step = Duration::new(0, 100);

        loop {

            // Run the VIC-II
            let addr = self.convert_vic_ii_addr(self.vic.read_addr_bus());
            let byte = self.read_byte(addr);
            let color = self.color_ram[addr & 0x03ff];  // Lowest 10 bits of addr always point to color RAM

            self.vic.data_in(byte);
            self.vic.color_in(color);

            if self.mode == SystemMode::Run {
                self.vic.rising_edge(false);
            } else {
                self.vic.rising_edge(true);
            }

            // Is the CPU allowed to use the bus or does the VIC need both clock edges?
            if self.vic.aec() {
                if !self.vic.irq() && self.vic.rdy() {
                    self.cpu.trigger_interrupt();
                }

                // Read/write the CPU data bus
                if self.cpu.addr_enable {
                    let addr = self.cpu.addr_bus as usize;
                    if self.cpu.rw {
                        let byte = self.read_byte(addr);
                        self.cpu.data_in(byte);
                    } else {
                        let data = self.cpu.data_out();
                        self.write_byte(addr, data);
                    }
                }
                if self.mode == SystemMode::Run {
                    self.cpu.cycle(false);
                } else {
                    self.cpu.cycle(true);
                }
            } else {
                if self.mode == SystemMode::Run {
                    self.vic.falling_edge(false);
                } else {
                    self.vic.falling_edge(true);
                }
            }

            if self.mode != SystemMode::Run {
                let elapsed = total_t.elapsed();
                let total_time_ms = (elapsed.as_secs() * 1000) + ((elapsed.subsec_nanos() / 1_000_000) as u64);
                let speed = (cycles as f32) / (total_time_ms as f32);
                println!("----------");
                println!("  Mean Clock speed: {:8.3} kHz", speed);
                println!("{:?}", self.cpu);
                println!("{:?}", self.vic);
                println!("----------");

                if self.mode == SystemMode::DebugStep {
                    print!("] ");
                    stdout().flush();
                    let mut input = String::new();
                    match stdin().read_line(&mut input) {
                        Ok(_) => { },
                        Err(e) => { panic!("Error reading STDIN: {}", e); },
                    }
                    
                    match input.trim() {
                        "r" | "run" => {
                            self.mode = SystemMode::DebugRun;
                        },
                        "h" | "help" => {
                            println!("Help not implemented");
                        },
                        "" => {
                        },
                        _ => {
                            println!("Invalid command");
                        }
                    }
                }
            } else {
                if idle_time.subsec_nanos() > 0 {
                    sleep(idle_time);
                }
            }

            cycles = cycles.wrapping_add(1);

            // Sample the speed every 10k cycles to make sure the clock speed isn't too fast
            if cycles % 10000 == 0 {
                let elapsed = total_t.elapsed();
                let total_time_ms = (elapsed.as_secs() * 1000) + ((elapsed.subsec_nanos() / 1_000_000) as u64);
                let speed = (cycles as f32) / (total_time_ms as f32);

                if speed > (clock_speed_mhz as f32) / 1_000_000f32 {
                    idle_time += idle_step;
                } else if idle_time > Duration::new(0, 0) {
                    idle_time -= idle_step;
                }

                println!("Ideal clock speed: {} kHz", clock_speed_mhz/1_000_000);
                println!("Mean clock speed:  {} kHz", speed);
                println!("Idle time: {} ns", idle_time.subsec_nanos());
                println!("{:?}", self.cpu);
            }
        }
    }
}
