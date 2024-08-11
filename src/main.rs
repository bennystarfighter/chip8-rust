// https://multigesture.net/articles/how-to-write-an-emulator-chip-8-interpreter/
// https://github.com/bradford-hamilton/chippy/blob/master/internal/chip8/instructions.go#L25
// http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#Dxyn


extern crate sdl2;
use std::time::{Duration, Instant};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

use crate::chip8::VM;

pub mod chip8;

const FONT_BITMAP: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];


pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window_scale = 10;
    let window = video_subsystem.window("CHIP-8", 64 * window_scale, 32 * window_scale)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.clear();
    canvas.present();

    let texture_creator = canvas.texture_creator();
    let mut vm = VM::new(canvas, &texture_creator);
    vm.initialize_texture()?;
    vm.init_font_set();
    vm.load_rom("D:\\Downloads\\IBM Logo.ch8");

    let mut last_timer_update = Instant::now();
    let timer_interval = Duration::from_secs_f64(1.0 / 60.0);
    let emulation_interval = Duration::from_secs_f64(1.0 / 500.0);
    let mut last_emulation_cycle = Instant::now();

    // SDL event loop to keep the window open
    let mut event_pump = sdl_context.event_pump()?;
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => { break 'running }
                Event::KeyDown { keycode, .. } => {
                    if let Some(k) = keycode {
                        println!("Key down: {}", k);
                        update_keypad(&mut vm, k, true);
                    }
                }
                Event::KeyUp { keycode, .. } => {
                    if let Some(k) = keycode {
                        println!("Key up: {}", k);
                        update_keypad(&mut vm, k, false);
                    }
                }
                _ => {}
            }
        }

        let now = Instant::now();
        if now.duration_since(last_emulation_cycle) >= emulation_interval {
            vm.emulate_cycle();
            if vm.drawflag { vm.draw_display(window_scale) }
            last_emulation_cycle = now;
        }

        if now.duration_since(last_timer_update) >= timer_interval {
            if vm.delay > 0 {
                vm.delay -= 1;
            }
            last_timer_update = now;
        }
    }

    Ok(())
}

fn update_keypad(vm: &mut VM, keycode: Keycode, pressed: bool) {
    let key_mapping = match keycode {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xc),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xd),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xe),
        Keycode::Z => Some(0xa),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xb),
        Keycode::V => Some(0xf),
        _ => None,
    };

    if let Some(key) = key_mapping {
        vm.keypad[key] = pressed;
    }
}