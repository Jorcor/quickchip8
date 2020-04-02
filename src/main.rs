extern crate rand;
extern crate sdl2;

use std::io;
use std::io::prelude::*;
use std::fs::File;

use rand::Rng;

const FONTSET: [u8; 80] = 
[
    0xF0, 0x90, 0x90, 0x90, 0xF0, //0
    0x20, 0x60, 0x20, 0x20, 0x70, //1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, //2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, //3
    0x90, 0x90, 0xF0, 0x10, 0x10, //4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, //5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, //6
    0xF0, 0x10, 0x20, 0x40, 0x40, //7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, //8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, //9
    0xF0, 0x90, 0xF0, 0x90, 0x90, //A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, //B
    0xF0, 0x80, 0x80, 0x80, 0xF0, //C
    0xE0, 0x90, 0x90, 0x90, 0xE0, //D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, //E
    0xF0, 0x80, 0xF0, 0x80, 0x80  //F
];

// scancodes mapped to reasonable keys
const SCANCODES: [sdl2::keyboard::Scancode; 16] = [
    sdl2::keyboard::Scancode::Num1,
    sdl2::keyboard::Scancode::Num2,
    sdl2::keyboard::Scancode::Num3,
    sdl2::keyboard::Scancode::Num4,
    sdl2::keyboard::Scancode::Q,
    sdl2::keyboard::Scancode::W,
    sdl2::keyboard::Scancode::E,
    sdl2::keyboard::Scancode::R,
    sdl2::keyboard::Scancode::A,
    sdl2::keyboard::Scancode::S,
    sdl2::keyboard::Scancode::D,
    sdl2::keyboard::Scancode::F,
    sdl2::keyboard::Scancode::Z,
    sdl2::keyboard::Scancode::X,
    sdl2::keyboard::Scancode::C,
    sdl2::keyboard::Scancode::V,
];

pub struct CPU {
    pub pc: u16,
    pub memory: [u8; 4097],
    pub r: [u8; 16],
    pub i: u16,
    pub sp: u8,
    pub stack: [u16; 16],
    pub display: [[u8; 64]; 32],
    pub delay_timer: u8,
    pub sound_timer: u8,
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            pc: 0x200,
            memory: [0; 4097],
            r: [0; 16],
            i: 0,
            sp: 0,
            stack: [0; 16],
            display: [[0; 64]; 32],
            delay_timer: 0,
            sound_timer: 0,
        } 
    }
    pub fn load_fondset(&mut self) {
        for num in 0..80 {
            self.memory[num as usize] = FONTSET[num as usize];
        }
    }
    pub fn read_op_code(&mut self) -> u16 {
        let upper: u8 = self.memory[self.pc as usize];
        let lower: u8 = self.memory[(self.pc + 1) as usize];
        return ((upper as u16) << 8 | lower as u16).into();
    }
    pub fn draw_screen(&mut self, renderer: &mut sdl2::render::Renderer) {
        renderer.clear();
        for y in 0..32 {
            for x in 0..64 {
                if self.display[y][x] == 1 {
                    renderer.set_draw_color(sdl2::pixels::Color::RGB(255, 255, 255));
                } else {
                    renderer.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
                }
                let rect = sdl2::rect::Rect::new(x as i32 * 10, y as i32 * 10, 10, 10);
                renderer.fill_rect(rect).unwrap();
            }
        }
        renderer.present();
    }
}

fn main() -> io::Result<()> {
    let f = File::open("pong.rom")?;

    let mut cpu = CPU::new();
    cpu.load_fondset();

    // load rom into cpu.memory
    let mut index = 0x200;
    for byte in f.bytes() {
        cpu.memory[index] = byte.unwrap();
        index += 1;
    }
    
    // sdl2 setup
    let sdl = sdl2::init().unwrap();
    let mut event_pump = sdl.event_pump().unwrap();
    event_pump.disable_event(sdl2::event::EventType::KeyDown);
    event_pump.disable_event(sdl2::event::EventType::KeyUp);
    let video = sdl.video().unwrap();
    let window = video.window("chip 8", 640, 320).build().unwrap();
    let mut renderer = window.renderer().accelerated().build().unwrap();

    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit {..} => break 'main,
                _ => {},
            }
        }

        let op_code = cpu.read_op_code();
        let op_1 = (op_code & 0xF000) >> 12;
        let op_2 = (op_code & 0x0F00) >> 8;
        let op_3 = (op_code & 0x00F0) >> 4;
        let op_4 = op_code & 0x000F;

        // println!("{:X}::{:X}{:X}{:X}{:X}", cpu.pc, op_1, op_2, op_3, op_4);

        match (op_1, op_2, op_3, op_4) {
            // 00E0 - cls
            (0x0, 0x0, 0xE, 0x0) => {
                for y in 0..32 {
                    for x in 0..64 {
                        cpu.display[y][x] = 0;
                    }
                }
            },
            // 00EE - ret
            (0x0, 0x0, 0xE, 0xE) => {
                cpu.pc = cpu.stack[cpu.sp as usize];
                cpu.sp -= 1;
            },
            // 1nnn - jp nnn
            (0x1, _, _, _) => {
                cpu.pc = op_2 << 8 | op_3 << 4 | op_4;
                continue;
            },
            // 2nnn - call nnn
            (0x2, _, _, _) => {
                cpu.stack[cpu.sp as usize] = cpu.pc;
                cpu.sp += 1;
                cpu.pc = op_2 << 8 | op_3 << 4 | op_4;
                continue;
            }
            // 3xkk - se rx, kk
            (0x3, _, _, _) => {
                if cpu.r[op_2 as usize] == (op_3 << 4) as u8 | op_4 as u8 {
                    cpu.pc += 2;
                }
            }
            // 4xkk - sne rx, kk
            (0x4, _, _, _) => {
                if cpu.r[op_2 as usize] != ((op_3 << 4) | op_4) as u8 {
                    cpu.pc += 2;
                }
            },
            // 5xy0 - se rx, ry
            (0x5, _, _, 0x0) => {
                if cpu.r[op_2 as usize] == cpu.r[op_3 as usize] {
                    cpu.pc += 2;
                }
            },
            // 6xkk - ld rx, kk
            (0x6, _, _, _) => {
                cpu.r[op_2 as usize] = (op_3 << 4 | op_4) as u8;
            },
            // 7xkk add rx, kk
            (0x7, _, _, _) => { 
                cpu.r[op_2 as usize] = cpu.r[op_2 as usize].wrapping_add((op_3 << 4 | op_4) as u8);
            }
            // 8xy0 - ld rx, ry
            (0x8, _, _, 0x0) => {
                cpu.r[op_2 as usize] = cpu.r[op_3 as usize];
            },
            // 8xy1 - or rx, ry
            (0x8, _, _, 0x1) => {
                cpu.r[op_2 as usize] |= cpu.r[op_3 as usize];
            },
            // 8xy2 - and rx, ry
            (0x8, _, _, 0x2) => {
                cpu.r[op_2 as usize] &= cpu.r[op_3 as usize];
            },
            // 8xy3 - xor rx, ry
            (0x8, _, _, 0x3) => {
                cpu.r[op_2 as usize] ^= cpu.r[op_3 as usize]; 
            },
            // 8xy4 - add rx, ry
            (0x8, _, _, 0x4) => {
                let (result, carry) = cpu.r[op_2 as usize].overflowing_add(cpu.r[op_3 as usize]);
                cpu.r[0xF] = if carry {1} else {0};
                cpu.r[op_2 as usize] = result;
            },
            // 8xy5 - sub rx, ry
            (0x8, _, _, 0x5) => {
                let (result, carry) = cpu.r[op_2 as usize].overflowing_sub(cpu.r[op_3 as usize]);
                cpu.r[0xF] = if carry {1} else {0};
                cpu.r[op_2 as usize] = result;
            },
            // 8xy6 - shr rx
            (0x8, _, _, 0x6) => {
                cpu.r[0xF] = 1 & cpu.r[op_2 as usize];
                cpu.r[op_2 as usize] = cpu.r[op_2 as usize] >> 1;
            },
            // 8xy7 - subn rx, ry
            (0x8, _, _, 0x7) => {

            },
            // Annn - ld i, nnn
            (0xA, _, _, _) => {
                cpu.i = op_2 << 8 | op_3 << 4 | op_4;
                // println!("\tcpu.i = {:04X}", cpu.i);
            },
            // Cxkk - rnd rx, kk
            (0xC, _, _, _) => {
                let mut rng =  rand::thread_rng();
                let rand: u8 = rng.gen_range(0, 255);
                cpu.r[op_2 as usize] = rand & (op_3 << 4) as u8 | op_4 as u8;
                // println!("\t{:X} and {:X} = {:X}", rand, (op_3 << 4) as u8 | op_4 as u8, rand & (op_3 << 4) as u8 | op_4 as u8);
            },
            // Dxyn - drw rx, ry, w
            (0xD, _, _, _) => {
                for offset in 0..op_4 {
                    for bit in 0..8 {
                        // needs collision detection
                        let x = ((cpu.r[op_2 as usize] + bit as u8) % 64 as u8) as usize;
                        let y = ((cpu.r[op_3 as usize] + offset as u8) % 32 as u8) as usize;
                        if cpu.display[y][x] == 1 && (cpu.memory[(cpu.i+offset) as usize] & 0x1 << (7 - bit)) >> (7 - bit) == 1 {
                            cpu.r[0xF] = 1;
                        } else {
                            cpu.r[0xF] = 0;
                        }
                        cpu.display[y][x] ^= (cpu.memory[(cpu.i+offset) as usize] & 0x1 << (7 - bit)) >> (7 - bit)
                    }
                }
                cpu.draw_screen(&mut renderer);
            },
            // ExA1 - sknp rx - skip next if key not pressed
            (0xE, _, 0xA, 0x1) => {
                let keys = sdl2::keyboard::KeyboardState::new(&event_pump);
                if keys.is_scancode_pressed(SCANCODES[cpu.r[op_2 as usize] as usize]) == false {
                    cpu.pc += 2;
                    // println!("\tskipped because {:X} is not pressed", cpu.r[op_2 as usize]);
                } else {
                    // println!("\tnot skipped because {:X} is pressed", cpu.r[op_2 as usize]);
                }
            },
            // Fx07 - ld rx, dt
            (0xF, _, 0x0, 0x7) => {
                cpu.r[op_2 as usize] = cpu.delay_timer;
                // println!("\tdelay timer was {} set into r[{}]", cpu.delay_timer, op_2);
            },
            // Fx15 - ld dt, rx
            (0xF, _, 0x1, 0x5) => {
                cpu.delay_timer = cpu.r[op_2 as usize];
                // println!("\tdelay timer set to {}", cpu.delay_timer);
            },
            // Fx18 - ld st, rx
            (0xF, _, 0x1, 0x8) => {
                cpu.sound_timer = cpu.r[op_2 as usize];
                // println!("\tsound timer set to {}", cpu.sound_timer);
            },
            // Fx29 - ld f, rx
            (0xF, _, 0x2, 0x9) => {
                cpu.i = (cpu.r[op_2 as usize] * 0x5) as u16;
                // println!("\tcpu.i = {:X}, char: {:X}", cpu.i, cpu.r[op_2 as usize]);
            },
            // Fx33 - ld b, rx // hundreds digit = I, tens digit = I+1, ones digit = I+2
            (0xF, _, 0x3, 0x3) => {
                cpu.memory[cpu.i as usize] = cpu.r[op_2 as usize] / 100;
                cpu.memory[(cpu.i+1) as usize] = (cpu.r[op_2 as usize] / 10) % 10 as u8;
                cpu.memory[(cpu.i+2) as usize] = cpu.r[op_2 as usize] % 10 as u8;
                // println!("\thundreds: {}, tens: {}, ones: {}", 
                //     cpu.memory[cpu.i as usize], 
                //     cpu.memory[(cpu.i+1) as usize], 
                //     cpu.memory[(cpu.i+2) as usize]
                // );
            },
            // Fx65 - ld rx, [I]
            (0xF, _, 0x6, 0x5) => {
                for num in 0..=op_2 {
                    cpu.r[num as usize] = cpu.memory[(cpu.i+num) as usize];
                    // println!("\t{:X}", cpu.r[num as usize]);
                }
            },
            (_, _, _, _) => break,
        }
        
        if cpu.delay_timer > 0 {
            cpu.delay_timer -= 1;   // this needs to be 60hz
        }
        cpu.pc += 2;
        // cpu.draw_screen(&mut renderer);
        std::thread::sleep(std::time::Duration::from_millis(1))
    }

    println!("\nbroke on {:04X}", cpu.read_op_code());

    Ok(())
}