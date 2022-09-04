extern crate log;
extern crate simplelog;
use rust_gameboy::{display::Display, gameboy::GameBoy};
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
    let rom_path = "tests/gb-test-roms/cpu_instrs/individual/01-special.gb";
    let mut gameboy = GameBoy::new(bios_path, rom_path);
    let mut display = Display::init(160, 144);
    let mut cycle: u32 = 0;
    // let mut start_time = Local::now().time();
    // let mut frames = 0;
    let mut buffer = vec![0; 160 * 144];
    while display.window.is_open() {
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
        let frame_buffer = gameboy.get_frame_buffer();
        if cycle == 70224 {
            buffer.clone_from_slice(frame_buffer);
            display.update_with_buffer(&mut buffer);
            cycle = 0;
            // frames += 1;
        }
    }
}
