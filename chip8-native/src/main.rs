extern crate sdl2;
extern crate chip8;

use chip8::Chip8;
use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::Color,
    rect::{
        Rect,
    },
};

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Chip8", 64 * 10, 32 * 10)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window
        .into_canvas()
        .target_texture()
        .present_vsync()
        .build()
        .unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    //let file_data = std::fs::read("BLINKY.c8").unwrap();
    let file_data = include_bytes!("../../roms/tetris.c8").to_vec();
    let mut chip8 = Chip8::new();
    chip8.init();
    chip8.load(&file_data);
    let mut cycles = 0;
    let limit = 3_000_000;
    let speed = 8;
    let mut printed_data = false;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(code),
                    ..
                } => match code {
                    Keycode::X => chip8.set_key(0, true),
                    Keycode::Num1 => chip8.set_key(1, true),
                    Keycode::Num2 => chip8.set_key(2, true),
                    Keycode::Num3 => chip8.set_key(3, true),
                    Keycode::Q => chip8.set_key(4, true),
                    Keycode::W => chip8.set_key(5, true),
                    Keycode::E => chip8.set_key(6, true),
                    Keycode::A => chip8.set_key(7, true),
                    Keycode::S => chip8.set_key(8, true),
                    Keycode::D => chip8.set_key(9, true),
                    Keycode::Z => chip8.set_key(10, true),
                    Keycode::C => chip8.set_key(11, true),
                    Keycode::Num4 => chip8.set_key(12, true),
                    Keycode::R => chip8.set_key(13, true),
                    Keycode::F => chip8.set_key(14, true),
                    Keycode::V => chip8.set_key(15, true),
                    _ => {}
                },
                Event::KeyUp {
                    keycode: Some(code),
                    ..
                } => match code {
                    Keycode::X => chip8.set_key(0, false),
                    Keycode::Num1 => chip8.set_key(1, false),
                    Keycode::Num2 => chip8.set_key(2, false),
                    Keycode::Num3 => chip8.set_key(3, false),
                    Keycode::Q => chip8.set_key(4, false),
                    Keycode::W => chip8.set_key(5, false),
                    Keycode::E => chip8.set_key(6, false),
                    Keycode::A => chip8.set_key(7, false),
                    Keycode::S => chip8.set_key(8, false),
                    Keycode::D => chip8.set_key(9, false),
                    Keycode::Z => chip8.set_key(10, false),
                    Keycode::C => chip8.set_key(11, false),
                    Keycode::Num4 => chip8.set_key(12, false),
                    Keycode::R => chip8.set_key(13, false),
                    Keycode::F => chip8.set_key(14, false),
                    Keycode::V => chip8.set_key(15, false),
                    _ => {}
                },
                _ => {}
            }
        }

        if cycles != limit {
            chip8.update_timers();
            for _ in 0..speed {
                println!("{}", chip8.cycle().unwrap());
                cycles += 1;
            }
        } else if !printed_data {
            println!("{}", chip8);
            printed_data = true;
        }

        canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
        canvas.clear();

        for (i, &el) in chip8.gfx.iter().enumerate() {
            let x = (i as i32 % 64) * 10;
            let y = (i / 64) as i32 * 10;
            if el {
                canvas.set_draw_color(Color::RGBA(255, 255, 255, 255));
                canvas
                    .fill_rect(Rect::new(x, y, 10, 10))
                    .expect("could not fill rect");
            }
            if i % 2 == 0 {
                canvas.set_draw_color(Color::RGBA(255, 0, 255, 255));
                //canvas.fill_rect(Rect::new(x, y, 10, 10));
            }
        }

        canvas.present();
    }
}
