extern crate sdl2;

mod emulator;

use clap::Parser;
use emulator::Emulator;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::collections::HashMap;
use std::thread;
use std::time::{Duration, SystemTime};

/// CHIP-8 emulator
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// File name of the ROM file to run
    #[arg()]
    file_name: String,

    /// Emulation speed in cycles/frame
    #[arg(short, long, default_value_t = 27)]
    speed: usize,
}

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

const PIXEL_SIZE: u32 = 20;
const WIDTH: u32 = PIXEL_SIZE * 64;
const HEIGHT: u32 = PIXEL_SIZE * 32;

fn main() {
    let args = Args::parse();
    let key_map = HashMap::from(KEY_MAP);

    let mut emulator = Emulator::init(&args.file_name);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("CHIP-8", WIDTH, HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let frame_time = Duration::from_nanos(1_000_000_000 / 60);

    'running: loop {
        let now = SystemTime::now();

        for _ in 0..args.speed {
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
                                (j * (PIXEL_SIZE as usize)) as i32,
                                (i * (PIXEL_SIZE as usize)) as i32,
                                PIXEL_SIZE,
                                PIXEL_SIZE,
                            ))
                            .unwrap();
                    }
                }

                canvas.present();
            }

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

            emulator.tick();
        }

        emulator.tick_timers();

        let diff = frame_time.saturating_sub(now.elapsed().unwrap());
        if !diff.is_zero() {
            thread::sleep(diff);
        }
    }
}
