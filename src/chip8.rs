use std::fs;
use rand::random;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use crate::FONT_BITMAP;

pub struct VM<'a> {
    pub op: u16,
    pub v: [u8; 16],
    pub i: u16,
    pub pc: u16,
    pub stack: [u16; 16],
    pub sp: u16,
    pub delay: u8,
    pub sound: u8,
    pub memory: [u8; 4096],
    pub display: [u8; 64 * 32],
    pub drawflag: bool,
    pub keypad: [bool; 16],
    pub canvas: WindowCanvas,
    pub display_texture: Option<Texture<'a>>,
    pub texture_creator: &'a TextureCreator<WindowContext>,
}

impl<'a> VM<'a> {
    pub fn new(canvas: WindowCanvas, texture_creator: &'a TextureCreator<WindowContext>) -> Self {
        Self {
            op: 0,
            v: [0; 16],
            i: 0,
            pc: 0x200,
            stack: [0; 16],
            sp: 0,
            delay: 0,
            sound: 0,
            memory: [0; 4096],
            display: [0; 64 * 32],
            drawflag: false,
            keypad: [false; 16],
            canvas,
            display_texture: None, // Initialize as None, create later
            texture_creator,
        }
    }

    pub fn initialize_texture(&mut self) -> Result<(), String> {
        let display_texture = self
            .texture_creator
            .create_texture_streaming(PixelFormatEnum::RGB24, 64, 32)
            .map_err(|e| e.to_string())?;

        self.display_texture = Some(display_texture);
        Ok(())
    }

    pub fn init_font_set(&mut self) {
        for i in 0..80 {
            self.memory[i as usize] = FONT_BITMAP[i as usize];
        }
    }
}

impl VM<'_> {
    pub fn emulate_cycle(&mut self) {
        self.op = (self.memory[self.pc as usize] as u16) << 8 | self.memory[(self.pc + 1) as usize] as u16;
        parse_op_code(self);
    }

    pub fn read_input(&self) {}

    pub fn load_rom(&mut self, rom: &str) {
        let rom_content = match fs::read(rom) {
            Ok(b) => { b }
            Err(e) => { panic!("Error loading rom, {}", e) }
        };

        if rom_content.len() > self.memory.len() - 0x200 {
            panic!("Selected rom is too large for chip8, exiting")
        }

        for (i, e) in rom_content.iter().enumerate() {
            //self.memory.offset()
            self.memory[0x200 + i] = *e;
        }

        println!("Loaded rom \"{}\" of length {}", rom, rom_content.len())
    }

    // display | drawing
    pub fn draw_display(&mut self, window_scale: u32) {
        self.display_texture.as_mut().unwrap().with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0..32 {
                for x in 0..64 {
                    let offset = y * pitch + x * 3; // Each pixel occupies 3 bytes (RGB)
                    let pixel_value = if self.display[y * 64 + x] == 1 { 0xFF } else { 0x00 }; // white or black

                    // Set the RGB values for the pixel
                    buffer[offset] = pixel_value;     // R
                    buffer[offset + 1] = pixel_value; // G
                    buffer[offset + 2] = pixel_value; // B
                }
            }
        }).unwrap();

        self.canvas.clear();
        self.canvas.copy(&self.display_texture.as_ref().unwrap(), None, Some(Rect::new(0, 0, 64 * window_scale, 32 * window_scale))).unwrap();
        self.canvas.present();
    }

    // OpCodes
    fn _0x00e0(&mut self) {
        self.display = [0; 64 * 32];
        self.pc += 2;
    }

    fn _0x00ee(&mut self) {
        self.pc = self.stack[self.sp as usize] + 2;
        self.sp -= 1;
    }
    fn _1nnn(&mut self, nnn: u16) {
        self.pc = nnn;
    }

    fn _2nnn(&mut self, nnn: u16) {
        self.sp += 1;
        self.stack[self.sp as usize] = self.pc;
        self.pc = nnn;
    }

    fn _3xkk(&mut self, x: u16, kk: u8) {
        if self.v[x as usize] == kk {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn _4xkk(&mut self, x: u16, kk: u8) {
        if self.v[x as usize] != kk {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn _5xy0(&mut self, x: u16, y: u16) {
        if x == y {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn _6xkk(&mut self, x: u16, kk: u8) {
        self.v[x as usize] = kk;
        self.pc += 2;
    }

    fn _7xkk(&mut self, x: u16, kk: u8) {
        self.v[x as usize] = self.v[x as usize].wrapping_add(kk);
        self.pc += 2;
    }

    fn _0x8000(&mut self, x: u16, y: u16) {
        self.v[x as usize] = self.v[y as usize];
    }

    fn _8xy0(&mut self, x: u16, y: u16) {
        self.v[x as usize] = self.v[y as usize];
        self.pc += 2;
    }

    fn _8xy1(&mut self, x: u16, y: u16) {
        self.v[x as usize] = self.v[x as usize] | self.v[y as usize];
        self.pc += 2;
    }

    fn _8xy2(&mut self, x: u16, y: u16) {
        self.v[x as usize] = self.v[x as usize] & self.v[y as usize];
        self.pc += 2;
    }

    fn _8xy3(&mut self, x: u16, y: u16) {
        self.v[x as usize] = self.v[x as usize] ^ self.v[y as usize];
        self.pc += 2;
    }

    fn _8xy4(&mut self, x: u16, y: u16) {
        let (result, carry) = self.v[x as usize].overflowing_add(self.v[y as usize]);
        self.v[0xF] = if carry { 1 } else { 0 };
        self.v[x as usize] = result;
        self.pc += 2;
    }

    fn _8xy5(&mut self, x: u16, y: u16) {
        let (result, borrow) = self.v[x as usize].overflowing_sub(self.v[y as usize]);
        self.v[0xF] = if borrow { 0 } else { 1 };
        self.v[x as usize] = result;
        self.pc += 2;
    }

    fn _8xy6(&mut self, x: u16, y: u16) {
        self.v[x as usize] = self.v[y as usize] >> 1;
        self.v[0xF] = self.v[y as usize] & 0x01;
        self.pc += 2;
    }

    fn _8xy7(&mut self, x: u16, y: u16) {
        self.v[0xF] = match self.v[y as usize] > self.v[x as usize] {
            true => { 1 }
            false => { 0 }
        };

        let (result, ..) =  self.v[y as usize].overflowing_sub(self.v[x as usize]);
        self.v[x as usize] = result;
        self.pc += 2;
    }

    fn _8xye(&mut self, x: u16, y: u16) {
        self.v[x as usize] = self.v[y as usize] << 1;
        self.v[0xF] = self.v[y as usize] & 0x80;
        self.pc += 2;
    }

    fn _9xy0(&mut self, x: u16, y: u16) {
        if self.v[x as usize] == self.v[y as usize] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn _annn(&mut self, nnn: u16) {
        self.i = nnn;
        self.pc += 2;
    }

    fn _bnnn(&mut self, nnn: u16) {
        self.pc = nnn + self.v[0x0] as u16;
        self.pc += 2;
    }

    fn _cxkk(&mut self, x: u16, kk: u8) {
        self.v[x as usize] = random::<u8>() & kk;
        self.pc += 2;
    }

    // Thank you chatgpt-san for your kind contribution
    fn _dxyn(&mut self, x: u16, y: u16) {
        let x_pos = self.v[x as usize] as usize;
        let y_pos = self.v[y as usize] as usize;
        let height = self.op & 0x000F;
        self.v[0xF] = 0;

        for y_line in 0..height {
            let pixel = self.memory[(self.i + y_line) as usize];
            for x_line in 0..8 {
                let index = ((y_pos + y_line as usize) % 32) * 64 + (x_pos + x_line as usize) % 64;
                let sprite_pixel = (pixel >> (7 - x_line)) & 1;
                let screen_pixel = &mut self.display[index];

                if *screen_pixel == 1 && sprite_pixel == 1 {
                    self.v[0xF] = 1;
                }
                *screen_pixel ^= sprite_pixel;
            }
        }

        self.drawflag = true;
        self.pc += 2;
    }

    fn _ex9e(&mut self, x: u16) {
        if self.keypad[self.v[x as usize] as usize] == true {
            //self.keypad[self.v[x as usize] as usize] = 0;
            self.pc += 4
        } else {
            self.pc += 2;
        }
    }

    fn _exa1(&mut self, x: u16) {
        if !self.keypad[self.v[x as usize] as usize] {
            self.pc += 4;
        } else {
            self.keypad[self.v[x as usize] as usize] = false;
            self.pc += 2;
        }
    }

    fn _fx07(&mut self, x: u16) {
        self.v[x as usize] = self.delay;
        self.pc += 2;
    }

    fn _fx0a(&mut self, x: u16) {
        for key in self.keypad {
            if key == true {
                self.v[x as usize] = key as u8;
                self.pc += 2;
                //return;
            }
        }
        self.keypad[self.v[x as usize] as usize] = false;
    }

    fn _fx15(&mut self, x: u16) {
        self.delay = self.v[x as usize];
        self.pc += 2;
    }

    fn _fx18(&mut self, x: u16) {
        self.sound = self.v[x as usize];
        self.pc += 2;
    }

    fn _fx1e(&mut self, x: u16) {
        self.i += self.v[x as usize] as u16;
        self.pc += 2;
    }

    fn _fx29(&mut self, x: u16) {
        self.i = (self.v[x as usize] * 5) as u16;
        self.pc += 2;
    }

    fn _fx33(&mut self, x: u16) {
        // I'm way too stupid for this function. Thank you bradford-hamilton.
        self.memory[self.i as usize] = self.v[x as usize] / 100;
        self.memory[(self.i + 1) as usize] = (self.v[x as usize] / 10) % 10;
        self.memory[(self.i + 2) as usize] = (self.v[x as usize] % 100) % 10;
        self.pc += 2;
    }

    fn _fx55(&mut self, x: u16) {
        for register_index in 0..x {
            self.memory[(self.i + register_index) as usize] = self.v[register_index as usize];
        }
        self.pc += 2;
    }

    fn _fx65(&mut self, x: u16) {
        for register_index in 0..x {
            self.v[register_index as usize] = self.memory[(self.i + register_index) as usize];
        }
        self.pc += 2;
    }
}

pub fn parse_op_code(vm: &mut VM) {
    let x = (vm.op & 0x0F00) >> 8;
    let y = (vm.op & 0x00F0) >> 4;
    let nn: u8 = (vm.op & 0x00FF) as u8;
    let nnn = vm.op & 0x0FFF;

    println!("Op: {:#06x} | x: {} y: {} nn: {} nnn: {}", vm.op, x, y, nn, nnn);

    match vm.op & 0xF000 {
        0x0000 => {
            match vm.op & 0x00FF {
                0x00E0 => { vm._0x00e0() }
                0x00EE => { vm._0x00ee() }
                _ => {}
            }
        }
        0x1000 => { vm._1nnn(nnn) }
        0x2000 => { vm._2nnn(nnn) }
        0x3000 => { vm._3xkk(x, nn) }
        0x4000 => { vm._4xkk(x, nn) }
        0x5000 => { vm._5xy0(x, y) }
        0x6000 => { vm._6xkk(x, nn) }
        0x7000 => { vm._7xkk(x, nn) }
        0x8000 => {
            match vm.op & 0x000f {
                0x0000 => { vm._8xy0(x, y) }
                0x0001 => { vm._8xy1(x, y) }
                0x0002 => { vm._8xy2(x, y) }
                0x0003 => { vm._8xy3(x, y) }
                0x0004 => { vm._8xy4(x, y) }
                0x0005 => { vm._8xy5(x, y) }
                0x0006 => { vm._8xy6(x, y) }
                0x0007 => { vm._8xy7(x, y) }
                0x0008 => { vm._8xye(x, y) }
                _ => {}
            }
        }
        0x9000 => { vm._9xy0(x, y) }
        0xa000 => { vm._annn(nnn) }
        0xb000 => { vm._bnnn(nnn) }
        0xc000 => { vm._cxkk(x, nn) }
        0xd000 => { vm._dxyn(x, y) }
        0xe000 => {
            match vm.op & 0x00FF {
                0x009e => { vm._ex9e(x) }
                0x00a1 => { vm._exa1(x) }
                _ => {}
            }
        }
        0xf000 => {
            match vm.op & 0x00FF {
                0x00A1 => { vm._exa1(x) }
                0x0007 => { vm._fx07(x) }
                0x000A => { vm._fx0a(x) }
                0x0015 => { vm._fx15(x) }
                0x0018 => { vm._fx18(x) }
                0x001E => { vm._fx1e(x) }
                0x0029 => { vm._fx29(x) }
                0x0033 => { vm._fx33(x) }
                0x0055 => { vm._fx55(x) }
                0x0065 => { vm._fx65(x) }
                _ => { panic!("Unknown opcode {:#06x}", vm.op) }
            }
        }

        _ => { panic!("Unknown opcode {:#06x}", vm.op) }
    }
}