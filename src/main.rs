extern crate rand;
extern crate sdl2;

use std::io;
use std::io::prelude::*;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::env;
use std::fs::File;

use sdl2::keyboard;

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
    keyboard::Scancode::X,
    keyboard::Scancode::Num1,
    keyboard::Scancode::Num2,
    keyboard::Scancode::Num3,
    keyboard::Scancode::Q,
    keyboard::Scancode::W,
    keyboard::Scancode::E,
    keyboard::Scancode::A,
    keyboard::Scancode::S,
    keyboard::Scancode::D,
    keyboard::Scancode::Z,
    keyboard::Scancode::C,
    keyboard::Scancode::Num4,
    keyboard::Scancode::R,
    keyboard::Scancode::F,
    keyboard::Scancode::V,
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
    pub delay_between_instructions: std::time::Duration,
    pub delay_between_cycles: std::time::Duration,
    pub skip_next_cycle: bool,
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
            delay_between_instructions: std::time::Duration::from_micros(16600),
            delay_between_cycles: std::time::Duration::from_micros(2000),
            skip_next_cycle: false,
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

                // calculate size of a pixel so window resizing works properly
                let width = renderer.viewport().width();
                let height= renderer.viewport().height();
                let pixel_width = width / 64;
                let pixel_height = height / 32;

                let rect = sdl2::rect::Rect::new(x as i32 * pixel_width as i32, y as i32 * pixel_height as i32, pixel_width, pixel_height);
                renderer.fill_rect(rect).unwrap();
            }
        }
        renderer.present();
    }
    pub fn cycle(&mut self) {
        std::thread::sleep(self.delay_between_cycles);
        if self.skip_next_cycle == false {
            self.pc += 2;
        } else {
            self.skip_next_cycle = false;
        }
    }
    pub fn faster(&mut self) {
        self.delay_between_instructions += std::time::Duration::from_micros(500);
    }
    pub fn slower(&mut self) {
        self.delay_between_instructions -= std::time::Duration::from_micros(500);
    }
    pub fn debug_print(&self) {
        println!("PC: {:X}", self.pc);
        for num in 0..16 {
            print!("{:X}: [{:02X}] ", num, self.r[num]);
            if num % 4 == 3 && num > 0 {
                println!();
            }
        }
        println!();

        let _ = std::io::stdin().read(&mut [0u8]).unwrap();
    }
    pub fn reset_emu(&mut self) {
        self.pc = 0x200;
        self.r = [0; 16];
        self.i = 0;
        self.sp = 0;
        self.stack = [0; 16];
        self.display = [[0; 64]; 32];
        self.delay_timer = 0;
        self.sound_timer = 0;
    }
    //// opcode functions
    // clear the screen - 00E0
    pub fn cls_00e0(&mut self) {
        self.display = [[0; 64]; 32];
    }
    // return - 00EE
    pub fn ret_00ee(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
    }
    // jmp nnn - 1nnn
    pub fn jmp_1nnn(&mut self, nnn: u16) {
        self.pc = nnn;
        self.skip_next_cycle = true;
    }
    // call nnn - 2nnn
    pub fn call_2nnn(&mut self, nnn: u16) {
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        self.pc = nnn;
        self.skip_next_cycle = true;
    }
    // skip next if rx == nn - 3xnn
    pub fn se_3xnn(&mut self, x: u8, nn: u8) {
        if self.r[x as usize] == nn {
            self.pc += 2;
        }
    }
    // skip next if rx != nn - 4xnn
    pub fn sne_4xnn(&mut self, x: u8, nn: u8) {
        if self.r[x as usize] != nn {
            self.pc += 2;
        }
    }
    // skip next if rx == ry - 5xy0
    pub fn se_5xy0(&mut self, x: u8, y: u8) {
        if self.r[x as usize] == self.r[y as usize] {
            self.pc += 2;
        }
    }
    // load nn into rx - 6xnn
    pub fn ld_6xnn(&mut self, x: u8, nn: u8) {
        self.r[x as usize] = nn;
    }
    // add rx = rx + kk - 7xnn
    pub fn add_7xnn(&mut self, x: u8, nn: u8) {
        self.r[x as usize] = self.r[x as usize].wrapping_add(nn);
    }
    // load rx = ry - 8xy0
    pub fn ld_8xy0(&mut self, x: u8, y: u8) {
        self.r[x as usize] = self.r[y as usize];
    }
    // or rx = rx | ry - 8xy1
    pub fn or_8xy1(&mut self, x: u8, y: u8) {
        self.r[x as usize] |= self.r[y as usize];
    }
    // and rx = rx & ry - 8xy2
    pub fn and_8xy2(&mut self, x: u8, y: u8) {
        self.r[x as usize] &= self.r[y as usize];
    }
    // xor rx = rx ^ ry - 8xy3
    pub fn xor_8xy3(&mut self, x: u8, y: u8) {
        self.r[x as usize] ^= self.r[y as usize];
    }
    // carry add rx = rx - ry - 8xy4
    pub fn c_add_8xy4(&mut self, x: u8, y: u8) {
        let (result, carry) = self.r[x as usize].overflowing_add(self.r[y as usize]);
        self.r[0xF] = if carry {1} else {0};
        self.r[x as usize] = result;
    }
    // carry sub rx = rx - ry - 8xy5
    pub fn c_sub_8xy5(&mut self, x: u8, y: u8) {
        let (result, carry) = self.r[x as usize].overflowing_sub(self.r[y as usize]);
        self.r[0xF] = if carry {1} else {0};
        self.r[x as usize] = result;
    }
    // shift right rx = ry >> 1 (lsb into r[0xF]) - 8xy6
    pub fn shr_8xy6(&mut self, x: u8, y: u8) {
        self.r[0xF] = 1 & self.r[y as usize];
        self.r[x as usize] = self.r[y as usize] >> 1;
    }
    // backwards carry sub ry - rx into rx - 8xy7
    pub fn bc_sub_8xy7(&mut self, x: u8, y: u8) {
        let (result, carry) = self.r[y as usize].overflowing_sub(self.r[x as usize]);
        self.r[0xF] = if carry {1} else {0};
        self.r[x as usize] = result;
    }
    // shift left rx = ry << 1 (msb into r[0xF]) - 8xyE
    pub fn shl_8xye(&mut self, x: u8, y: u8) {
        self.r[0xF] = (self.r[y as usize] & 0x80) >> 7;
        self.r[x as usize] = self.r[y as usize] << 1;
    }
    // skip next if rx != ry - 9xy0
    pub fn sne_9xy0(&mut self, x: u8, y: u8) {
        if self.r[x as usize] != self.r[y as usize] {
            self.pc += 2;
        }
    }
    // load i = nnn - Annn
    pub fn ld_annn(&mut self, nnn: u16) {
        self.i = nnn;
    }
    // jmp nnn + r[0] - Bnnn
    pub fn jmp_bnnn(&mut self, nnn: u16) {
        self.pc = nnn + self.r[0] as u16;
        self.skip_next_cycle = true;
    }
    // rand rx = rand and nn - Cxnn
    pub fn rand_cxnn(&mut self, x: u8, nn: u8, rand: u8) {
        self.r[x as usize] = rand & nn;
    }
    // draw sprite at rx, ry, n rows high
    pub fn draw_dxyn(&mut self, x: u8, y: u8, n: u8) {
        let mut collision = false;

        for row in 0..n as u16 {
            let current_byte: u8 = self.memory[(self.i + row) as usize];

            for bit in 0..8 {
                let x = ((self.r[x as usize] as u16 + bit) % 64) as usize;
                let y = ((self.r[y as usize] as u16 + row) % 32) as usize;

                let old_pixel = self.display[y][x];
                let new_pixel = (current_byte & (0x80 >> bit)) >> (7 - bit);

                if old_pixel == 1 && new_pixel == 1 {
                    collision = true;
                }

                self.display[y][x] ^= new_pixel;
            }
        }

        if collision {
            self.r[0xF] = 1;
        } else {
            self.r[0xF] = 0;
        }
    }
    // load rx = delay_timer - FX07
    pub fn ld_dt_fx07(&mut self, x: u8) {
        self.r[x as usize] = self.delay_timer;
    }
    // load delay_timer = rx - Fx15
    pub fn ld_dt_fx15(&mut self, x: u8) {
        self.delay_timer = self.r[x as usize];
    }
    // load sound_timer = rx - Fx18
    pub fn ld_st_fx18(&mut self, x: u8) {
        self.sound_timer = self.r[x as usize];
    }
    // add I = I + rx - Fx1E
    pub fn add_i_fx1e(&mut self, x: u8) {
        self.i += self.r[x as usize] as u16;
    }
    // load font character into I - FX29
    pub fn ld_f_fx29(&mut self, x: u8) {
        self.i = (self.r[x as usize] * 0x5) as u16;
    }
    // load digits into memory starting at I - FX33
    pub fn ld_b_fx33(&mut self, x: u8) {
        self.memory[self.i as usize] = self.r[x as usize] / 100;
        self.memory[(self.i + 1) as usize] = (self.r[x as usize] % 100) / 10;
        self.memory[(self.i + 2) as usize] = self.r[x as usize] % 10;
    }
    // load registers into memory starting at I to x - FX55
    pub fn ld_reg_fx55(&mut self, x: u8) {
        for num in 0..=x as u16 {
            self.memory[(self.i + num) as usize] = self.r[num as usize];
        }
    }
    // load memory into registers starting at I to x - FX65
    pub fn ld_reg_fx65(&mut self, x: u8) {
        for num in 0..=x as u16 {
            self.r[num as usize] = self.memory[(self.i + num) as usize];
        }
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() >= 1 {
        if args.len() != 2 {
            println!("usage: <program name> <rom name>");
            return Ok(());
        }
        if args.len() == 1 {
            return Ok(());
        }
    }
    
    println!("--{:?}--", args);

    let f = File::open(&args[1])?;

    let mut cpu = CPU::new();
    cpu.load_fondset();

    let mut rng =  rand::thread_rng();

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
    let window = video.window("chip 8", 640, 320).resizable().build().unwrap();
    let mut renderer = window.renderer().accelerated().build().unwrap();

    // async setup because properly decrementing counters at 60hz is hard
    let (tx, rx): (Sender<u8>, Receiver<u8>) = mpsc::channel();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_micros(16600));
            tx.send(1).unwrap();
        }
    });
    
    // debug nonsense
    // let mut used: [u8; 20] = [0; 20];
    
    'main: loop {
        // cpu.debug_print();

        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit {..} => break 'main,
                _ => {},
            }
        }
        
        for key in keyboard::KeyboardState::new(&event_pump).pressed_scancodes() {
            match key {
                keyboard::Scancode::Escape => {
                    break 'main;
                },
                keyboard::Scancode::Backspace => {
                    cpu.reset_emu();
                },
                keyboard::Scancode::Minus => {
                    cpu.slower();
                },
                keyboard::Scancode::Equals => {
                    cpu.faster();
                },
                _ => {},
            }
        }
        if keyboard::KeyboardState::new(&event_pump).is_scancode_pressed(keyboard::Scancode::Backspace) {
            cpu.reset_emu();
            continue;
        }

        let op_code = cpu.read_op_code();
        let op_1: u8 = ((op_code & 0xF000) >> 12) as u8;
        let op_2: u8 = ((op_code & 0x0F00) >> 8) as u8;
        let op_3: u8 = ((op_code & 0x00F0) >> 4) as u8;
        let op_4: u8 = (op_code & 0x000F) as u8;
        let nn: u8 = (op_code & 0x00FF) as u8;
        let nnn: u16 = (op_code & 0x0FFF) as u16;

        println!("{:X}::{:X}{:X}{:X}{:X}", cpu.pc, op_1, op_2, op_3, op_4);
        // used[op_1 as usize] = 1;

        match (op_1, op_2, op_3, op_4) {
            // 0nnn - sys nnn - only used by old chip8
            (0x0, 0x0, 0xE, 0x0) => cpu.cls_00e0(),
            (0x0, 0x0, 0xE, 0xE) => cpu.ret_00ee(),
            (0x1, _, _, _) => cpu.jmp_1nnn(nnn),
            (0x2, _, _, _) => cpu.call_2nnn(nnn),
            (0x3, _, _, _) => cpu.se_3xnn(op_2, nn),
            (0x4, _, _, _) => cpu.sne_4xnn(op_2, nn),
            (0x5, _, _, 0x0) => cpu.se_5xy0(op_2, op_3),
            (0x6, _, _, _) => cpu.ld_6xnn(op_2, nn),
            (0x7, _, _, _) => cpu.add_7xnn(op_2, nn),
            (0x8, _, _, 0x0) => cpu.ld_8xy0(op_2, op_3),
            (0x8, _, _, 0x1) => cpu.or_8xy1(op_2, op_3),
            (0x8, _, _, 0x2) => cpu.and_8xy2(op_2, op_3),
            (0x8, _, _, 0x3) => cpu.xor_8xy3(op_2, op_3),
            (0x8, _, _, 0x4) => cpu.c_add_8xy4(op_2, op_3),
            (0x8, _, _, 0x5) => cpu.c_sub_8xy5(op_2, op_3),
            (0x8, _, _, 0x6) => cpu.shr_8xy6(op_2, op_3),
            (0x8, _, _, 0x7) => cpu.bc_sub_8xy7(op_2, op_3),
            (0x8, _, _, 0xE) => cpu.shl_8xye(op_2, op_3),
            (0x9, _, _, 0x0) => cpu.sne_9xy0(op_2, op_3),
            (0xA, _, _, _) => cpu.ld_annn(nnn),
            (0xB, _, _, _) => cpu.jmp_bnnn(nnn),
            (0xC, _, _, _) => cpu.rand_cxnn(op_2, nn, rng.gen_range(0, 255)),
            (0xD, _, _, _) => {
                cpu.draw_dxyn(op_2, op_3, op_4);
                cpu.draw_screen(&mut renderer);
            },
            (0xE, _, 0x9, 0xE) => {
                let keys = keyboard::KeyboardState::new(&event_pump);
                if keys.is_scancode_pressed(SCANCODES[cpu.r[op_2 as usize] as usize]) == true {
                    cpu.pc += 2;
                }
            },
            (0xE, _, 0xA, 0x1) => {
                let keys = keyboard::KeyboardState::new(&event_pump);
                if keys.is_scancode_pressed(SCANCODES[cpu.r[op_2 as usize] as usize]) == false {
                    cpu.pc += 2;
                }
            },
            (0xF, _, 0x0, 0x7) => cpu.ld_dt_fx07(op_2),
            (0xF, _, 0x0, 0xA) => {
                let keys = keyboard::KeyboardState::new(&event_pump);
                for key in SCANCODES.iter() {
                    if !keys.is_scancode_pressed(*key) {
                        cpu.skip_next_cycle = true;
                    } else {
                        cpu.r[op_2 as usize] = SCANCODES.iter().position(|&r| r == *key).unwrap() as u8;
                    }
                }
            },
            (0xF, _, 0x1, 0x5) => cpu.ld_dt_fx15(op_2),
            (0xF, _, 0x1, 0x8) => cpu.ld_st_fx18(op_2),
            (0xF, _, 0x1, 0xE) => cpu.add_i_fx1e(op_2),
            (0xF, _, 0x2, 0x9) => cpu.ld_f_fx29(op_2),
            (0xF, _, 0x3, 0x3) => cpu.ld_b_fx33(op_2),
            (0xF, _, 0x5, 0x5) => cpu.ld_reg_fx55(op_2),
            (0xF, _, 0x6, 0x5) => cpu.ld_reg_fx65(op_2),
            // unimplemented
            (_, _, _, _) => {
                println!("Broke on: {:X} - {:X}", cpu.pc, cpu.memory[cpu.pc as usize]);
                break;
            },
        }
    
        // check to see if our thread told us its been time to decrement this
        let answer = rx.recv_timeout(std::time::Duration::from_micros(0));
        if answer.is_ok() {
            if cpu.delay_timer > 0 {
                cpu.delay_timer -= 1;
            }
            if cpu.sound_timer > 0 {
                cpu.sound_timer -= 1;
            }
        }
        cpu.cycle();
    }

    // for x in 0..20 {
    //     if used[x] == 1 {
    //         println!("{:X}", x);
    //     }
    // }

    return Ok(());
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_00e0() {
        let mut cpu = CPU::new();
        cpu.display[1][1] = 1;
        cpu.cls_00e0();
        for y in 0..32 {
            for x in 0..64 {
                assert_eq!(cpu.display[y][x], 0);
            }
        }
    }
    #[test]
    fn test_00ee() {
        let mut cpu = CPU::new();
        cpu.pc = 0x1000;
        cpu.sp = 5;
        cpu.stack[4] = 0x200;
        cpu.ret_00ee();
        assert_eq!(cpu.pc, 0x200);
    }
    #[test]
    fn test_1nnn() {
        let mut cpu = CPU::new();
        cpu.pc = 0x1000;
        cpu.jmp_1nnn(0x200);
        assert_eq!(cpu.pc, 0x200);
    }
    #[test]
    fn test_2nnn() {
        let mut cpu = CPU::new();
        cpu.call_2nnn(0x400);
        assert_eq!(cpu.pc, 0x400);
        assert_eq!(cpu.sp, 1);
        assert_eq!(cpu.stack[0], 0x200);
    }
    #[test]
    fn test_3xnn() {
        let mut cpu = CPU::new();
        cpu.pc = 0x400;
        cpu.r[0] = 0x45;
        cpu.se_3xnn(0, 0x45);
        assert_eq!(cpu.pc, 0x402);
    }
    #[test]
    fn test_4xnn() {
        let mut cpu = CPU::new();
        cpu.pc = 0x400;
        cpu.r[0] = 0x45;
        cpu.sne_4xnn(0, 0x46);
        assert_eq!(cpu.pc, 0x400 + 2);
    }
}