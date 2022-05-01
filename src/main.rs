#![feature(slice_as_chunks)]

mod emu;
mod font;
mod rng;
mod tests;

use emu::Chip8;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::fs::File;
use std::io::Read;

use crate::emu::Chip8Options;

fn main() {
    /*let mut chip8rom = [0u8; 0x1000 - 0x200];

    // ld v0, $12
    chip8rom[0] = 0x60;
    chip8rom[1] = 0x12;

    // ld v1, $56
    chip8rom[2] = 0x61;
    chip8rom[3] = 0x56;

    let mut chip8 = emu::Chip8::new(chip8rom);
    chip8.cycle();
    chip8.cycle();*/
    let usage = "chip8 <file> [speed multiplier] <options>";

    let arguments_length = std::env::args().len();
    if arguments_length < 2 {
        println!("{}", usage);
        println!("Possible options: savingIncreasesRegI, shiftVyNotVx");
        return;
    }
    
    let speed_multiplier: f32 = match str::parse(
        &std::env::args().nth(2).unwrap_or("1".to_string())[..]
    ) {
        Ok(x) => x,
        Err(_) => {
            println!("Incorrect speed multiplier");
            println!("Usage: {}", usage);
            std::process::exit(1);
        }
    };
    let default_cycles_per_frame = 8f32;
    let cycles_per_frame: usize = (default_cycles_per_frame * speed_multiplier) as usize;
    let min_cycles = 1f32;
    let max_cycles = 1000f32;
    let min_multiplier = min_cycles / default_cycles_per_frame;
    let max_multiplier = max_cycles / default_cycles_per_frame;
    if cycles_per_frame < min_cycles as usize {
        println!("Speed multiplier too low (min: {})", min_multiplier);
        println!("{}", usage);
        return;
    } else if cycles_per_frame > max_cycles as usize {
        println!("Speed multiplier too high (max: {})", max_multiplier);
        println!("{}", usage);
        return;
    }

    let mut saving_increases_reg_i= false;

    let mut shift_vy_not_vx = false;

    for i in 3..=4 {
        let option = std::env::args().nth(i).unwrap_or("none".to_string());
        if option == "savingIncreasesRegI" {
            saving_increases_reg_i = true;
        } else if option == "shiftVyNotVx" {
            shift_vy_not_vx = true;
        } else if option == "none" {
            
        } else {
            println!("Unknown option: {}", option);
            std::process::exit(1);
        }
    }

    let file_name = std::env::args().nth(1).expect("No file specified");

    let mut chip8rom = [0u8; 0x1000 - 0x200];
    let mut the_file = File::open(file_name).expect("Error opening file");
    the_file
        .read(&mut chip8rom)
        .expect("Error reading from file");

    let chip8options = Chip8Options {
        saving_increases_reg_i: saving_increases_reg_i,
        shift_vy_not_vx: shift_vy_not_vx
    };

    let mut chip8 = emu::Chip8::new(chip8rom, chip8options);

    const CHIP8_WIDTH: usize = 64;
    const CHIP8_HEIGHT: usize = 32;

    const SCR_MULTIPLIER: usize = 12;
    const X_MULT: usize = SCR_MULTIPLIER * 4;
    const Y_MULT: usize = SCR_MULTIPLIER;

    const WIDTH: usize = CHIP8_WIDTH * SCR_MULTIPLIER;
    const HEIGHT: usize = CHIP8_HEIGHT * SCR_MULTIPLIER;

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("chip8", WIDTH as u32, HEIGHT as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .unwrap();

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(
            sdl2::pixels::PixelFormatEnum::ARGB8888,
            WIDTH as u32,
            HEIGHT as u32,
        )
        .unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    
    //let mut frames = 0;
    //let mut start = std::time::Instant::now();
    //let mut last_printed = 0;

    'running: loop {
        //frames += 1;
        chip8.tick();
        for _ in 0..cycles_per_frame {
            chip8.cycle();
        }

        const ROWSIZE: usize = WIDTH * 4;
        texture
            .with_lock(None, |pixelarray, _| -> () {
                let (chunks, _remainder) = pixelarray.as_chunks_mut::<ROWSIZE>();
                for src_y in 0..CHIP8_HEIGHT {
                    for src_x in 0..CHIP8_WIDTH {
                        let pixel_value = chip8.display[src_y][src_x];
                        let to_set = if pixel_value == 1 { 255 } else { 0 };
                        let dest_y_start = src_y * Y_MULT;
                        let dest_x_start = src_x * X_MULT;
                        let dest_x_end = dest_x_start + X_MULT;
                        let dest_y_end = dest_y_start + Y_MULT;

                        for dest_y in dest_y_start..dest_y_end {
                            for dest_x in dest_x_start..dest_x_end {
                                chunks[dest_y][dest_x] = to_set;
                            }
                        }
                    }
                }
            })
            .unwrap();

        canvas.copy(&texture, None, None).unwrap();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                /*Event::KeyDown { keycode: Some(Keycode::J), .. } => {
                    chip8.tick();
                    chip8.cycle();
                },
                Event::KeyDown { keycode: Some(Keycode::K), .. } => {
                    chip8.tick();
                    for _ in 0..100 {
                        chip8.cycle();
                    }
                },*/
                Event::KeyDown {
                    keycode: Some(the_key),
                    ..
                } => try_handle_chip8_keycode(the_key, &mut chip8, true),
                Event::KeyUp {
                    keycode: Some(the_key),
                    ..
                } => try_handle_chip8_keycode(the_key, &mut chip8, false),
                _ => {}
            }
        }
        canvas.present();

        /*if start.elapsed() > std::time::Duration::new(1, 0) {
            let dt = start.elapsed().as_secs_f64();
            let fc = frames - last_printed;
            last_printed = frames;
            start = std::time::Instant::now();
            println!("\nfps: {:.1}\n", fc as f64/dt);
        }*/
    }
    /*let file_name = std::env::args().nth(1).expect("No file specified (chip8 file startpos)");
    let start_pos: usize = str::parse(
        &std::env::args().nth(2).unwrap_or("512".to_string())[..]
    ).expect("Incorrect start position (chip8 file startpos)");


    let mut file_binary = [0u8; 0x1000 - 0x200];
    let mut the_file = File::open(file_name).expect("Error opening file");
    the_file.read(&mut file_binary).expect("Error reading from file");

    let mut chip8 = emu::Chip8::new(start_pos, file_binary);*/
    /*


    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window(
        "chip8",
        800,
        600
    )
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();

    loop {
        canvas.present();
        std::thread::sleep(std::time::Duration::from_millis(16));
    }*/
}

fn try_handle_chip8_keycode(keycode: Keycode, chip8: &mut Chip8, down: bool) {
    match keycode {
        Keycode::KpPeriod => chip8.keys[0x0] = down,
        Keycode::Kp7 => chip8.keys[0x1] = down,
        Keycode::Kp8 => chip8.keys[0x2] = down,
        Keycode::Kp9 => chip8.keys[0x3] = down,
        Keycode::Kp4 => chip8.keys[0x4] = down,
        Keycode::Kp5 => chip8.keys[0x5] = down,
        Keycode::Kp6 => chip8.keys[0x6] = down,
        Keycode::Kp1 => chip8.keys[0x7] = down,
        Keycode::Kp2 => chip8.keys[0x8] = down,
        Keycode::Kp3 => chip8.keys[0x9] = down,
        Keycode::Kp0 => chip8.keys[0xA] = down,
        Keycode::KpEnter => chip8.keys[0xB] = down,
        Keycode::KpDivide => chip8.keys[0xC] = down,
        Keycode::KpMultiply => chip8.keys[0xD] = down,
        Keycode::KpMinus => chip8.keys[0xE] = down,
        Keycode::KpPlus => chip8.keys[0xF] = down,
        _ => {}
    }
}
