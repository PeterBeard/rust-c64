// Copyright 2016 Peter Beard
// Distributed under the GNU GPL v2. For full terms, see the LICENSE file.
//
// Functions and datatypes relating to the VIC-II video chip

use std::fmt;

use super::super::Screen;

pub const MIN_CONTROL_ADDR: usize = 0xd000;
pub const MAX_CONTROL_ADDR: usize = 0xd3ff;
const CONTROL_REG_COUNT: usize = 0x40;

// TODO: Add code for NTSC
const HORZ_CYCLE_COUNT: u8 = 63;    // Number of cycles per line

// Mapping from color nybble to gamma-corrected color
// Values from Philip "Pepto" Timmermann's research here: http://www.pepto.de/projects/colorvic/
const COLOR: [(u8, u8, u8); 16] = [
    (0x00, 0x00, 0x00), // Black        0
    (0xff, 0xff, 0xff), // White        1
    (0x68, 0x37, 0x2b), // Red          2
    (0x70, 0xa4, 0xb2), // Cyan         3
    (0x6f, 0x3d, 0x86), // Purple       4
    (0x58, 0x8d, 0x43), // Green        5
    (0x35, 0x28, 0x79), // Blue         6
    (0xb8, 0xc7, 0x6f), // Yellow       7
    (0x6f, 0x4f, 0x25), // Orange       8
    (0x43, 0x39, 0x00), // Brown        9
    (0x9a, 0x67, 0x59), // Light red    a
    (0x44, 0x44, 0x44), // Dark grey    b
    (0x6c, 0x6c, 0x6c), // Grey         c
    (0x9a, 0xd2, 0x84), // Light green  d
    (0x6c, 0x5e, 0xb5), // Light blue   e
    (0x95, 0x95, 0x95), // Light grey   f
];

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
enum VicState {
    Idle,
    MatrixRead,
}

pub struct Vic {
    // Output pins (active low)
    irq: bool,  // IRQ pin triggers interrupts in the CPU
    rdy: bool,  // RDY stuns the CPU when the VIC needs more bus cycles
    aec: bool,  // AEC deactivates the CPU address bus

    // Registers
    sx0: u8,        // Sprite 0 x coord
    sy0: u8,        // Sprite 0 y coord
    sx1: u8,        // Sprite 1 x coord
    sy1: u8,        // Sprite 1 y coord
    sx2: u8,        // Sprite 2 x coord
    sy2: u8,        // Sprite 2 y coord
    sx3: u8,        // Sprite 3 x coord
    sy3: u8,        // Sprite 3 y coord
    sx4: u8,        // Sprite 4 x coord
    sy4: u8,        // Sprite 4 y coord
    sx5: u8,        // Sprite 5 x coord
    sy5: u8,        // Sprite 5 y coord
    sx6: u8,        // Sprite 6 x coord
    sy6: u8,        // Sprite 6 y coord
    sx7: u8,        // Sprite 7 x coord
    sy7: u8,        // Sprite 7 y coord
    msbx: u8,       // MSBs of X coordinates
    cr1: u8,        // Control register 1
    raster: u8,     // Raster counter
    lpx: u8,        // Light pen x
    lpy: u8,        // Light pen y
    s_enable: u8,   // Sprite enabled
    cr2: u8,        // Control register 2
    sye: u8,        // Sprite y expension
    mem: u8,        // Memory pointers
    int: u8,        // Interrupt register
    int_enable: u8, // Interrupt enabled
    s_priority: u8, // Sprite priority
    s_multi: u8,    // Sprite multicolor
    sxe: u8,        // Sprite x expansion
    ss_coll: u8,    // Sprite-sprite collision
    sd_coll: u8,    // Sprite-data collision
    border: u8,     // Border color
    bg0: u8,        // Background color 0
    bg1: u8,        // Background color 1
    bg2: u8,        // Background color 2
    bg3: u8,        // Background color 3
    sm0: u8,        // Sprite multicolor 0
    sm1: u8,        // Sprite multicolor 1
    s0c: u8,        // Sprite 0 color
    s1c: u8,        // Sprite 1 color
    s2c: u8,        // Sprite 2 color
    s3c: u8,        // Sprite 3 color
    s4c: u8,        // Sprite 4 color
    s5c: u8,        // Sprite 5 color
    s6c: u8,        // Sprite 6 color
    s7c: u8,        // Sprite 7 color

    state: VicState,// Current state of the processor
    addr_bus: u16,  // Address bus (only the lower 14 bits are used)
    data_bus: u16,   // Data bus -- lower nybble of upper byte is for color ram
    matrix_pos: u16,// Current position in the video matrix

    xpos: u8,       // X-position on the current raster line
    cycles: u64,    // Number of cycles since startup
    raster_int: u8, // Value of raster to interrupt on
}

impl Vic {
    pub fn new() -> Vic {
        Vic {
            irq: true,
            rdy: true,
            aec: true,

            sx0: 0,
            sy0: 0,
            sx1: 0,
            sy1: 0,
            sx2: 0,
            sy2: 0,
            sx3: 0,
            sy3: 0,
            sx4: 0,
            sy4: 0,
            sx5: 0,
            sy5: 0,
            sx6: 0,
            sy6: 0,
            sx7: 0,
            sy7: 0,
            msbx: 0,
            cr1: 0x1b,
            raster: 0,
            lpx: 0,
            lpy: 0,
            s_enable: 0,
            cr2: 0xc8,
            sye: 0,
            mem: 0,
            int: 0,
            int_enable: 0,
            s_priority: 0,
            s_multi: 0,
            sxe: 0,
            ss_coll: 0,
            sd_coll: 0,
            border: 0,
            bg0: 0,
            bg1: 0,
            bg2: 0,
            bg3: 0,
            sm0: 0,
            sm1: 0,
            s0c: 0,
            s1c: 0,
            s2c: 0,
            s3c: 0,
            s4c: 0,
            s5c: 0,
            s6c: 0,
            s7c: 0,

            state: VicState::Idle,
            addr_bus: 0x3fffu16,
            data_bus: 0u16,
            matrix_pos: 0u16,

            xpos: 0u8,
            raster_int: 0xff,
            cycles: 0u64,
        }
    }

    // Translate a memory address to a register index
    fn translate_addr(&self, addr: usize) -> u8 {
        if addr > MAX_CONTROL_ADDR || addr < MIN_CONTROL_ADDR {
            panic!("Invalid address for VIC-II control register: ${:0>4X}", addr);
        }
        ((addr - MIN_CONTROL_ADDR) % CONTROL_REG_COUNT) as u8
    }

    pub fn read_register(&self, addr: usize) -> u8 {
        let reg = self.translate_addr(addr);

        match reg {
            0 => self.sx0,
            1 => self.sy0,
            2 => self.sx1,
            3 => self.sy1,
            4 => self.sx2,
            5 => self.sy2,
            6 => self.sx3,
            7 => self.sy3,
            8 => self.sx4,
            9 => self.sy4,
            10 => self.sx5,
            11 => self.sy5,
            12 => self.sx6,
            13 => self.sy6,
            14 => self.sx7,
            15 => self.sy7,
            16 => self.msbx,
            17 => self.cr1,
            18 => self.raster,
            19 => self.lpx,
            20 => self.lpy,
            21 => self.s_enable,
            22 => self.cr2,
            23 => self.sye,
            24 => self.mem,
            25 => self.int,
            26 => self.int_enable,
            27 => self.s_priority,
            28 => self.s_multi,
            29 => self.sxe,
            30 => self.ss_coll,
            31 => self.sd_coll,
            32 => self.border,
            33 => self.bg0,
            34 => self.bg1,
            35 => self.bg2,
            36 => self.bg3,
            37 => self.sm0,
            38 => self.sm1,
            39 => self.s0c,
            40 => self.s1c,
            41 => self.s2c,
            42 => self.s3c,
            43 => self.s4c,
            44 => self.s5c,
            45 => self.s6c,
            46 => self.s7c,
            _ => 0xff,
        }
    }

    pub fn write_register(&mut self, addr: usize, value: u8) {
        let reg = self.translate_addr(addr);

        match reg {
            0 => { self.sx0 = value; },
            1 => { self.sy0 = value; },
            2 => { self.sx1 = value; },
            3 => { self.sy1 = value; },
            4 => { self.sx2 = value; },
            5 => { self.sy2 = value; },
            6 => { self.sx3 = value; },
            7 => { self.sy3 = value; },
            8 => { self.sx4 = value; },
            9 => { self.sy4 = value; },
            10 => { self.sx5 = value; },
            11 => { self.sy5 = value; },
            12 => { self.sx6 = value; },
            13 => { self.sy6 = value; },
            14 => { self.sx7 = value; },
            15 => { self.sy7 = value; },
            16 => { self.msbx = value; },
            17 => { self.cr1 = value | 0xc0; },
            18 => { self.raster_int = value; },
            19 => { self.lpx = value; },
            20 => { self.lpy = value; },
            21 => { self.s_enable = value; },
            22 => { self.cr2 = value; },
            23 => { self.sye = value; },
            24 => { self.mem = value | 1; },
            25 => { self.int = value | 0x70; },
            26 => { self.int_enable = value | 0x70; },
            27 => { self.s_priority = value; },
            28 => { self.s_multi = value; },
            29 => { self.sxe = value; },
            30 => { self.ss_coll = value; },
            31 => { self.sd_coll = value; },
            32 => { self.border = value | 0xf0; },
            33 => { self.bg0 = value | 0xf0; },
            34 => { self.bg1 = value | 0xf0; },
            35 => { self.bg2 = value | 0xf0; },
            36 => { self.bg3 = value | 0xf0; },
            37 => { self.sm0 = value | 0xf0; },
            38 => { self.sm1 = value | 0xf0; },
            39 => { self.s0c = value | 0xf0; },
            40 => { self.s1c = value | 0xf0; },
            41 => { self.s2c = value | 0xf0; },
            42 => { self.s3c = value | 0xf0; },
            43 => { self.s4c = value | 0xf0; },
            44 => { self.s5c = value | 0xf0; },
            45 => { self.s6c = value | 0xf0; },
            46 => { self.s7c = value | 0xf0; },
            _ => { /* ignore writes to non-existent registers */ },
        }
    }

    pub fn read_addr_bus(&self) -> u16 {
        // Only use the lower 14 bits of the address
        self.addr_bus & 0x40
    }

    // Calculate the current 14-bit video matrix address
    fn matrix_addr(&self) -> u16 {
        let addr = ((self.mem & 0xf0) as u16) << 6;
        addr + (self.matrix_pos & 0x3ff)
    }

    // Calculate a 14-bit character pointer address
    fn char_addr(&self, pointer: u8) -> u16 {
        let addr = ((self.mem & 0x0e) as u16) << 10;
        let addr = addr + ((pointer as u16) << 3);
        addr + (self.raster % 8) as u16
    }

    pub fn rising_edge(&mut self, screen: &mut Screen, debug: bool) {
        use self::VicState::*;

        self.aec = false;

        match self.state {
            Idle => {
                self.state = MatrixRead;
            },
            MatrixRead => {
                self.addr_bus = self.matrix_addr();
                self.matrix_pos = self.matrix_pos.wrapping_add(1);
            },
        }

        if self.raster == self.raster_int {
            // Do interrupt
        }
        self.xpos = self.xpos.wrapping_add(1);
        if self.xpos == HORZ_CYCLE_COUNT {
            self.xpos = 0;
            self.raster = self.raster.wrapping_add(1);
        }
        if self.raster > 100 {
            self.raster = 0;
        }
        screen.set_pixel_at(self.xpos as usize, self.raster as usize, COLOR[6]);

        self.aec = true;
        self.cycles = self.cycles.wrapping_add(1);
    }

    pub fn falling_edge(&mut self, screen: &mut Screen, debug: bool) {

    }

    // Write a color nybble to the data bus
    pub fn color_in(&mut self, byte: u8) {
        self.data_bus &= 0x00ff;
        self.data_bus &= ((byte as u16) & 0x0f) << 8;
    }

    // Write a byte to the data bus
    pub fn data_in(&mut self, byte: u8) {
        self.data_bus &= 0x0f00;
        self.data_bus &= (byte as u16);
    }

    // Read the color nybble of the data bus
    fn read_color_nybble(&self) -> u8 {
        ((self.data_bus & 0x0f00) >> 8) as u8
    }

    // Read the low byte of the data bus
    fn read_data_bus(&self) -> u8 {
        (self.data_bus & 0xff) as u8
    }

    pub fn frame_ready(&self) -> bool {
        self.xpos == 0 && self.raster == 0
    }

    pub fn irq(&self) -> bool {
        self.irq
    }

    pub fn rdy(&self) -> bool {
        self.rdy
    }

    pub fn aec(&self) -> bool {
        self.aec
    }
}

impl fmt::Debug for Vic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "  Cycle {:0>5} :: AB: ${:0>4X} // DB: ${:0>3X} // X: ${:0>2X} // Raster: ${:0>2X} // S: {:?}",
               self.cycles, self.addr_bus, self.data_bus, self.xpos, self.raster, self.state
               )
    }

}
