#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]


use std::fs::File;
use std::io::prelude::*;
use std::path::Path;



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
            0xF0, 0x90, 0x90, 0x90, 0xF0,
            0x20, 0x60, 0x20, 0x20, 0x70,
            0xF0, 0x10, 0xF0, 0x80, 0xF0,
            0xF0, 0x10, 0xF0, 0x10, 0xF0,
            0x90, 0x90, 0xF0, 0x10, 0x10,
            0xF0, 0x80, 0xF0, 0x10, 0xF0,
            0xF0, 0x80, 0xF0, 0x90, 0xF0,
            0xF0, 0x10, 0x20, 0x40, 0x40,
            0xF0, 0x90, 0xF0, 0x90, 0xF0,
            0xF0, 0x90, 0xF0, 0x10, 0xF0,
            0xF0, 0x90, 0xF0, 0x90, 0x90,
            0xE0, 0x90, 0xE0, 0x90, 0xE0,
            0xF0, 0x80, 0x80, 0x80, 0xF0,
            0xE0, 0x90, 0x90, 0x90, 0xE0,
            0xF0, 0x80, 0xF0, 0x80, 0xF0,
            0xF0, 0x80, 0xF0, 0x80, 0x80,
        ];

        for (src, dst) in sprites.iter().zip(self.mem.iter_mut()) {
            *dst = *src;
        }
    }

    fn load_rom<P: AsRef<Path>>(&mut self, path: P) {
        let mut f = File::open(path).unwrap();
        let mut buffer = Vec::new();

        f.read_to_end(&mut buffer).unwrap();

        for (src, dst) in buffer.iter().zip(self.mem.iter_mut().skip(0x1ff)) {
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
    fn run(&mut self) {
        while true {
            self.step();
        }
    }

    fn step(&mut self) {
        let instruction = self.memory.mem[self.cpu.pc as usize];
        // decode
        // execute
    }
}


fn main() {
}
