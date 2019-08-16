#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]


use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::env;
use rand::prelude::*;



fn display() {
    let config: mini_gl_fb::Config<&str> = mini_gl_fb::Config {
        //window_size: (64., 64.),
        buffer_size: (10, 10),
        .. Default::default()
    };
    let mut fb = mini_gl_fb::get_fancy(config);
    let mut buffer = vec![[128u8, 0, 0, 255]; 10 * 10];

    for (idx, x) in buffer.iter_mut().enumerate() {
        x[0] = (idx as u8) * 8;
    }

    fb.update_buffer(&buffer);
    fb.persist();
}


struct Memory {
    mem: [u8; 0x1000],
}

impl Default for Memory {
    fn default() -> Self {
        Self {
            mem: [0; 0x1000],
        }
    }
}

impl Memory {
    fn load_default_sprites(&mut self) {
        let sprites = [
            0xf0, 0x90, 0x90, 0x90, 0xf0,
            0x20, 0x60, 0x20, 0x20, 0x70,
            0xf0, 0x10, 0xf0, 0x80, 0xf0,
            0xf0, 0x10, 0xf0, 0x10, 0xf0,
            0x90, 0x90, 0xf0, 0x10, 0x10,
            0xf0, 0x80, 0xf0, 0x10, 0xf0,
            0xf0, 0x80, 0xf0, 0x90, 0xf0,
            0xf0, 0x10, 0x20, 0x40, 0x40,
            0xf0, 0x90, 0xf0, 0x90, 0xf0,
            0xf0, 0x90, 0xf0, 0x10, 0xf0,
            0xf0, 0x90, 0xf0, 0x90, 0x90,
            0xe0, 0x90, 0xe0, 0x90, 0xe0,
            0xf0, 0x80, 0x80, 0x80, 0xf0,
            0xe0, 0x90, 0x90, 0x90, 0xe0,
            0xf0, 0x80, 0xf0, 0x80, 0xf0,
            0xf0, 0x80, 0xf0, 0x80, 0x80,
        ];

        for (src, dst) in sprites.iter().zip(self.mem.iter_mut()) {
            *dst = *src;
        }
    }

    fn load_rom<P: AsRef<Path>>(&mut self, path: P) {
        let mut f = File::open(path).unwrap();
        let mut buffer = Vec::new();

        f.read_to_end(&mut buffer).unwrap();

        for (src, dst) in buffer.iter().zip(self.mem.iter_mut().skip(0x200)) {
            *dst = *src;
        }
    }
}

#[derive(Default)]
struct Cpu {
    pc: u16,
    sp: u8,
    stack: [u16; 16],

    registers: [u8; 16],
    I: u16,

    // sound registers
    dt: u8,
    st: u8,
}

struct Display {
    display_mode: DisplayMode,
    graphics: Vec<u8>,
}

enum DisplayMode {
    Mode64x48,
    Mode128x64,
}

#[derive(Default)]
struct Emulator {
    cpu: Cpu,
    memory: Memory,
}

impl Emulator {
    fn open<P: AsRef<Path>>(path: P) -> Self {
        let mut emulator = Emulator::default();
        emulator.memory.load_rom(path);
        emulator.cpu.pc = 0x200;

        emulator
    }

    fn run(&mut self) {
        while true {
            self.step();
        }
    }

    fn step(&mut self) {
        fn xx_to_u16(a: u8, b: u8) -> u16 {
            ((a as u16) << 4) + b as u16
        }
        fn xxx_to_u16(a: u8, b: u8, c: u8) -> u16 {
            ((a as u16) << 8) + ((b as u16) << 4) + c as u16
        }

        let a = (self.memory.mem[self.cpu.pc as usize] & 0xf0) >> 4;
        let b = (self.memory.mem[self.cpu.pc as usize] & 0x0f);
        let c = (self.memory.mem[self.cpu.pc as usize + 1] & 0xf0) >> 4;
        let d = (self.memory.mem[self.cpu.pc as usize + 1] & 0x0f);

        print!("{:#04x?}: {:02x?} {:02x?} {:02x?} {:02x?};  ", self.cpu.pc, a, b, c, d);
        match (a, b, c, d) {
            (0, 0, 0xe, 0) => println!("CLS"),
            (0, 0, 0xe, 0xe) => println!("RET"),
            (1, b, c, d) => println!("JP    {:#04x?}", xxx_to_u16(b, c, d)),
            (2, b, c, d) => println!("CALL  {:#04x?}", xxx_to_u16(b, c, d)),
            (3, b, c, d) => println!("SE    V{:x?} {:x?}", b, xx_to_u16(c, d)),
            (6, b, c, d) => println!("LD    V{:x?} {:x?}", b, xx_to_u16(c, d)),
            (7, b, c, d) => println!("ADD   V{:x?} {:x?}", b, xx_to_u16(c, d)),
            (8, b, c, 0) => println!("LD    V{:x?} V{:x?}", b, c),
            (0xa, b, c, d) => println!("LD    I {:x?}", xxx_to_u16(b, c, d)),
            (0xc, b, c, d) => println!("RND   V{:x?} {:x?}", b, xx_to_u16(c, d)),
            (0xd, b, c, d) => println!("DRW   V{:x?} V{:x?}, {}", b, c, d),
            _ => panic!("unknown instruction {:x?} {:x?} {:x?} {:x?}", a, b, c, d),
        }

        self.cpu.pc += 2;

        // decode
        // execute
    }
}


fn main() {
    let mut emulator = Emulator::open(env::args().nth(1).unwrap());
    emulator.run();
}
