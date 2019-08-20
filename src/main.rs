#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]


use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::env;
use rand::prelude::*;
use mini_gl_fb::{MiniGlFb, BufferFormat};



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
    ic: u32,
    sp: u8,
    stack: [u16; 16],

    registers: [u8; 16],
    I: u16,

    // sound registers
    dt: u8,
    st: u8,
}

struct Display {
    //display_mode: DisplayMode,
    buffer: Vec<u8>,
    fb: MiniGlFb,
}
impl Display {
    fn new() -> Self {
        let config: mini_gl_fb::Config<&str> = mini_gl_fb::Config {
            buffer_size: (64, 32),
            .. Default::default()
        };
        let mut fb = mini_gl_fb::get_fancy(config);
        fb.change_buffer_format::<u8>(BufferFormat::R);
        let buffer = vec![0u8; 64 * 32];

        Display {
            buffer,
            fb,
        }

    }

    fn update(&mut self) {
        let mut rgba = vec![[0u8]; 64 * 32];
        for (src, dst) in self.buffer.iter().zip(rgba.iter_mut()) {
            *dst = [*src];
        }
        self.fb.update_buffer(&rgba);
    }

    fn xor(&mut self, x: usize, y: usize, v: u8) -> bool {
        if v == 1 {
            self.buffer[x * 32 + y] ^= 127;
            true
        }
        else {
            false
        }
    }
}

enum DisplayMode {
    Mode64x48,
    Mode128x64,
}

struct Emulator {
    cpu: Cpu,
    memory: Memory,
    display: Display,
}

impl Emulator {
    fn open<P: AsRef<Path>>(path: P) -> Self {
        let mut emulator = Emulator {
            cpu: Cpu::default(),
            memory: Memory::default(),
            display: Display::new(),
        };
        emulator.memory.load_default_sprites();
        emulator.memory.load_rom(path);
        emulator.cpu.pc = 0x200;

        emulator
    }

    fn run(&mut self) {
        loop {
            let quit = self.step();
            use std::{thread::sleep, time::Duration};
            sleep(Duration::from_millis(10));
            if quit {
                println!("ic: {}", self.cpu.ic);
                break;
            }
        }
        loop{}
    }

    fn step(&mut self) -> bool {
        self.cpu.ic += 1;

        fn xx_to_u8(a: u8, b: u8) -> u8 {
            (a << 4) + b
        }
        fn xxx_to_u16(a: u8, b: u8, c: u8) -> u16 {
            ((a as u16) << 8) + ((b as u16) << 4) + c as u16
        }

        let a = (self.memory.mem[self.cpu.pc as usize] & 0xf0) >> 4;
        let b = self.memory.mem[self.cpu.pc as usize] & 0x0f;
        let c = (self.memory.mem[self.cpu.pc as usize + 1] & 0xf0) >> 4;
        let d = self.memory.mem[self.cpu.pc as usize + 1] & 0x0f;

        print!("{:#04x?}: {:02x?} {:02x?} {:02x?} {:02x?};  ", self.cpu.pc, a, b, c, d);
        match (a, b, c, d) {
            (0, 0, 0xe, 0) => println!("CLS"),
            (0, 0, 0xe, 0xe) => {
                println!("RET");
                self.cpu.pc = self.cpu.stack[self.cpu.sp as usize];
                self.cpu.sp -= 1;
            },
            (1, b, c, d) => {
                let addr = xxx_to_u16(b, c, d);
                println!("JP    {:#04x?}", addr);

                if self.cpu.pc == addr {
                    return true;
                }
                self.cpu.pc = addr - 2;
            },
            (2, b, c, d) => {
                self.cpu.sp += 1;
                self.cpu.stack[self.cpu.sp as usize] = self.cpu.pc;

                let addr = xxx_to_u16(b, c, d);
                self.cpu.pc = addr;
                println!("CALL  {:#04x?}", addr);
            },
            (3, x, c, d) => {
                let byte = xx_to_u8(c, d);
                println!("SE    V{:x?} {:x?}", x, byte);

                if self.cpu.registers[x as usize] == byte {
                    self.cpu.pc += 2;
                }
            },
            (6, x, c, d) => {
                let byte = xx_to_u8(c, d);

                println!("LD    V{:x?} {:x?}", x, byte);
                self.cpu.registers[x as usize] = byte;
            },
            (7, x, c, d) => {
                let byte = xx_to_u8(c, d);
                self.cpu.registers[x as usize] += byte;

                println!("ADD   V{:x?} {:x?}", x, byte);
            },
            (8, x, y, 0) => {
                self.cpu.registers[x as usize] = self.cpu.registers[y as usize];
                println!("LD    V{:x?} V{:x?}", x, y);
            },
            (0xa, b, c, d) => {
                let addr = xxx_to_u16(b, c, d);
                self.cpu.I = addr;
                println!("LD    I {:x?}", addr);
            },
            (0xc, b, c, d) => {
                let byte = rand::random::<u8>();
                let val = xx_to_u8(c, d);
                let x = b;
                println!("RND   V{:x?} {:x?}", x, byte);
                self.cpu.registers[x as usize] = byte & val;
            },
            (0xd, Vx, Vy, n) => {
                //TODO
                let data = &self.memory.mem[self.cpu.I as usize .. self.cpu.I as usize + n as usize];
                let x = self.cpu.registers[Vx as usize] as usize;
                let y = self.cpu.registers[Vy as usize] as usize;

                println!("I={:?}", self.cpu.I as usize);
                println!("{:?}", data.len());
                println!("{:?}", data);

                for (j, byte) in data.iter().enumerate() {
                    //println!("byte: {}", byte);
                    for i in 0..7 {
                        let b = ((byte >> i) & 1) as u8;
                        print!("{} ", b);

                        self.display.xor(x+i, y+j, b);
                    }
                    println!("");
                }
                println!("DRW   V{:x?} V{:x?}, {}", Vx, Vy, n);
                self.display.update();
            },
            _ => panic!("unknown instruction {:x?} {:x?} {:x?} {:x?}", a, b, c, d),
        }

        self.cpu.pc += 2;

        return false;
    }
}


fn main() {
    let mut emulator = Emulator::open(env::args().nth(1).unwrap());
    emulator.run();
}
