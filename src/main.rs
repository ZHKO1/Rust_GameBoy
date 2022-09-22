extern crate log;
extern crate simplelog;
use rust_gameboy::cartridge::Stable;
use rust_gameboy::util::{read_rom};
use rust_gameboy::{display::Display, gameboy::GameBoy, joypad};
use simplelog::*;
use std::io::Write;
use std::{fs::File, path::PathBuf};
// use std::time::SystemTime;

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
    // let rom_path = "tests/Red.gb";
    let bios = read_rom(bios_path).unwrap_or(vec![]);
    let rom = read_rom(rom_path).unwrap();
    let mut gameboy = GameBoy::new(bios, rom);
    let ram_path = PathBuf::from(rom_path).with_extension("sav");
    let ram_path = ram_path.to_str().unwrap();
    let ram_result = read_rom(ram_path);
    if let Ok(ram) = ram_result {
        gameboy.load_sav(ram);
    }
    let mut display = Display::init(160, 144);
    let mut cycle: u32 = 0;
    /*
    let mut start_time = SystemTime::now()
    .duration_since(SystemTime::UNIX_EPOCH)
    .unwrap()
    .as_millis();
    let mut frames = 0;
    */
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
                println!("60帧所耗时间 = {}", SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() - start_time);
                start_time = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis();;
                frames = 0;
            }
        }
         */
        if display.window.is_key_down(minifb::Key::O) {
            let ram = gameboy.save_sav();
            File::create(ram_path)
            .and_then(|mut file| file.write_all(&ram))
            .unwrap();
        } 
        cycle += 1;
        gameboy.trick();
        for (rk, vk) in &keys {
            if display.window.is_key_down(*rk) {
                gameboy.input(vk.clone(), true);
            } else {
                gameboy.input(vk.clone(), false);
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
