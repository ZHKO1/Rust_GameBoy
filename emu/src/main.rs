use minifb::KeyRepeat;
use rust_gameboy::display::Display;
use rust_gameboy_core::cartridge::Stable;
use rust_gameboy_core::gameboy::{GameBoy, HEIGHT, WIDTH};
use rust_gameboy_core::joypad;
use rust_gameboy_core::util::read_rom;
use std::io::Write;
use std::path::Path;
use std::{fs::File, path::PathBuf};
// use std::time::SystemTime;
use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Reach new heights.
struct Args {
    #[argh(subcommand)]
    nested: Option<Subcommands>,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum Subcommands {
    Info(InfoArgs),
    Run(RunArgs),
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "run")]
/// Start Game
struct RunArgs {
    #[argh(option, short = 'b')]
    /// path to bios file
    bios_path: Option<String>,
    #[argh(positional)]
    /// path to rom file    
    rom_path: String,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "info")]
/// Show Info
struct InfoArgs {
    #[argh(positional)]
    /// path to rom file
    rom_path: String,
}

fn start_game(bios_path: impl AsRef<Path>, rom_path: impl AsRef<Path>) {
    let ram_path = PathBuf::from(rom_path.as_ref()).with_extension("sav");
    let status_path = PathBuf::from(rom_path.as_ref()).with_extension("status");

    let bios = read_rom(bios_path).unwrap_or(vec![]);
    let rom = read_rom(rom_path).unwrap();
    let cartridge = GameBoy::get_cartridge(rom.clone());
    let mut gameboy = GameBoy::new(bios, cartridge);
    let ram_path = ram_path.to_str().unwrap();
    let ram_result = read_rom(ram_path);
    if let Ok(ram) = ram_result {
        gameboy.load_sav(ram);
    }
    let status_path = status_path.to_str().unwrap();
    /*
    let status_result = read_rom(status_path);
    if let Ok(status) = status_result {
        gameboy = gameboy
            .load(&status, GameBoy::get_cartridge(rom.clone()))
            .unwrap();
    }
    */
    let mut display = Display::init(WIDTH, HEIGHT);
    /*
    let mut start_time = SystemTime::now()
    .duration_since(SystemTime::UNIX_EPOCH)
    .unwrap()
    .as_millis();
    let mut frames = 0;
    */
    let mut buffer = vec![0; WIDTH * HEIGHT];

    let keys = vec![
        (minifb::Key::Right, joypad::JoyPadKey::Right),
        (minifb::Key::Up, joypad::JoyPadKey::Up),
        (minifb::Key::Left, joypad::JoyPadKey::Left),
        (minifb::Key::Down, joypad::JoyPadKey::Down),
        (minifb::Key::X, joypad::JoyPadKey::B),
        (minifb::Key::Z, joypad::JoyPadKey::A),
        (minifb::Key::Space, joypad::JoyPadKey::Select),
        (minifb::Key::Enter, joypad::JoyPadKey::Start),
    ];

    let mut gameboy_status: Vec<u8> = vec![];

    while display.is_open() {
        /*
        if frames == 59 {
            println!(
                "60帧所耗时间 = {}",
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
                    - start_time
            );
            start_time = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis();
            frames = 0;
        }
        */

        let is_refresh = gameboy.trick();
        if is_refresh {
            let frame_buffer = gameboy.get_frame_buffer();
            buffer.clone_from_slice(frame_buffer);
            display.update_with_buffer(&mut buffer);
            // frames += 1;

            if !gameboy.flip() {
                continue;
            }
            for (rk, vk) in &keys {
                if display.window.is_key_down(*rk) {
                    gameboy.input(vk.clone(), true);
                } else {
                    gameboy.input(vk.clone(), false);
                }
            }
            if display.window.is_key_pressed(minifb::Key::O, KeyRepeat::No) {
                let ram = gameboy.save_sav();
                File::create(ram_path)
                    .and_then(|mut file| file.write_all(&ram))
                    .unwrap();
            }

            if display.window.is_key_pressed(minifb::Key::Y, KeyRepeat::No) {
                if let Ok(status) = gameboy.save() {
                    gameboy_status = status;
                    File::create(status_path)
                        .and_then(|mut file| file.write_all(&gameboy_status))
                        .unwrap();
                }
            }
            if display.window.is_key_pressed(minifb::Key::U, KeyRepeat::No) {
                let cartridge = GameBoy::get_cartridge(rom.clone());
                if let Ok(gameboy_new) = gameboy.load(&gameboy_status, cartridge) {
                    gameboy = gameboy_new;
                }
            }
        }
    }
}

fn main() {
    let args: Args = argh::from_env();
    let command = args.nested.unwrap_or_else(|| {
        println!("Row! Row! Fight The Power!\n");
        std::process::exit(0);
    });

    match command {
        Subcommands::Run(subargs) => {
            start_game(subargs.bios_path.unwrap_or("".to_owned()), subargs.rom_path);
        }
        Subcommands::Info(subargs) => {
            let rom = read_rom(subargs.rom_path).unwrap();
            let cartridge = GameBoy::get_cartridge(rom);
            let title = cartridge.title();
            let gbc_flag = cartridge.gbc_flag();
            let ram_size = cartridge.get_ram_size();
            let cartridge_type = cartridge.get_cartridge_type();
            println!("title: {}", title);
            println!("gbc_flag: {}", gbc_flag);
            println!("ram_size: {}", ram_size);
            println!("cartridge_type: {}", cartridge_type);
        }
    }
}
