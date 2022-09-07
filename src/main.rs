extern crate log;
extern crate simplelog;
use rust_gameboy::{display::Display, gameboy::GameBoy, joypad};
use simplelog::*;
use std::fs::File;

fn main() {
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Warn,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create("my_rust_binary.log").unwrap(),
        ),
    ])
    .unwrap();

    // let bios_path = "tests/DMG_ROM.bin";
    let bios_path = "";
    let rom_path = "tests/Tetris.gb";
    let mut gameboy = GameBoy::new(bios_path, rom_path);
    let mut display = Display::init(160, 144);
    let mut cycle: u32 = 0;
    // let mut start_time = Local::now().time();
    // let mut frames = 0;
    let mut buffer = vec![0; 160 * 144];

    let keys = vec![
        (minifb::Key::Right, joypad::JoyPadKey::Right),
        (minifb::Key::Up, joypad::JoyPadKey::Up),
        (minifb::Key::Left, joypad::JoyPadKey::Left),
        (minifb::Key::Down, joypad::JoyPadKey::Down),
        (minifb::Key::Z, joypad::JoyPadKey::A),
        (minifb::Key::X, joypad::JoyPadKey::B),
        (minifb::Key::Space, joypad::JoyPadKey::Select),
        (minifb::Key::Enter, joypad::JoyPadKey::Start),
    ];

    while display.is_open() {
        /*
        if cycle == 0 {
            if frames == 59 {
                println!("60帧所耗时间 = {}", Local::now().time() - start_time);
                start_time = Local::now().time();
                frames = 0;
            }
        }
         */
        cycle += 1;
        gameboy.trick();
        for (rk, vk) in &keys {
            if display.window.is_key_down(*rk) {
                gameboy.joypad.input(vk.clone(), true);
            } else {
                gameboy.joypad.input(vk.clone(), false);
            }
        }
        if cycle == 70224 {
            let frame_buffer = gameboy.get_frame_buffer();
            buffer.clone_from_slice(frame_buffer);
            display.update_with_buffer(&mut buffer);
            cycle = 0;
            // frames += 1;
        }
    }
}
