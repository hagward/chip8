extern crate sdl2;

mod emulator;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::collections::HashMap;
use std::thread;
use std::time::{Duration, SystemTime};

static KEY_MAP: [(Keycode, usize); 16] = [
    (Keycode::Num1, 1),
    (Keycode::Num2, 2),
    (Keycode::Num3, 3),
    (Keycode::Num4, 0xc),
    (Keycode::Q, 4),
    (Keycode::W, 5),
    (Keycode::E, 6),
    (Keycode::R, 0xd),
    (Keycode::A, 7),
    (Keycode::S, 8),
    (Keycode::D, 9),
    (Keycode::F, 0xe),
    (Keycode::Z, 0xa),
    (Keycode::X, 0),
    (Keycode::C, 0xb),
    (Keycode::V, 0xf),
];

fn main() {
    let mut emulator = emulator::Emulator::new();
    emulator.init("./roms/IBM Logo.ch8");
    // emulator.init("./roms/test_opcode.ch8");
    // emulator.init("./roms/random_number_test.ch8");
    // emulator.init("./roms/morse_demo.ch8");
    // emulator.init("./roms/br8kout.ch8");
    // emulator.init("./roms/octogon.ch8");

    let key_map = HashMap::from(KEY_MAP);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let pixel_size: usize = 20;
    let width: usize = pixel_size * 64;
    let height: usize = pixel_size * 32;

    let window = video_subsystem
        .window("chip8", width as u32, height as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut iterations: usize = 0;
    let mut now = SystemTime::now();
    let sleep_duration = Duration::from_nanos(10_000_000 / 8);

    'running: loop {
        let elapsed_ms = now.elapsed().unwrap().as_millis();

        if elapsed_ms >= 1000 {
            println!("Emulating at {iterations} hz");
            iterations = 0;
            now = SystemTime::now();
        }

        if emulator.gfx_updated {
            canvas.set_draw_color(Color::RGB(0, 0, 0));
            canvas.clear();
            canvas.set_draw_color(Color::RGB(255, 255, 255));

            for i in 0..emulator.gfx.len() {
                let row = &emulator.gfx[i];
                for j in 0..row.len() {
                    if !row[j] {
                        continue;
                    }
                    canvas
                        .fill_rect(Rect::new(
                            (j * pixel_size) as i32,
                            (i * pixel_size) as i32,
                            pixel_size as u32,
                            pixel_size as u32,
                        ))
                        .unwrap();
                }
            }

            canvas.present();
        }

        // Run timers and stuff 60 times per second.
        if elapsed_ms % 17 == 0 {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    Event::KeyDown {
                        keycode: Some(keycode),
                        ..
                    } => {
                        if let Some(key) = key_map.get(&keycode) {
                            emulator.keypress[*key] = true;
                        }
                    }
                    Event::KeyUp {
                        keycode: Some(keycode),
                        ..
                    } => {
                        if let Some(key) = key_map.get(&keycode) {
                            emulator.keypress[*key] = false;
                        }
                    }
                    _ => {}
                }
            }

            emulator.tick_timers();
        }

        emulator.decode_next();
        iterations += 1;

        thread::sleep(sleep_duration)
    }
}
