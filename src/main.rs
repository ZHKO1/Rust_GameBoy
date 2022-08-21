use std::io::{self, Read};
extern crate minifb;
use minifb::{Key, Window, WindowOptions};
const WIDTH: usize = 640;
const HEIGHT: usize = 360;
fn main() {}

fn show_pixel_window() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });
    window.limit_update_rate(Some(std::time::Duration::from_micros(166000)));
    while window.is_open() && !window.is_key_down(Key::Escape) {
        for i in buffer.iter_mut() {
            *i = 0xffffff00; // write something more funny here!
                             // j = j.wrapping_add(1);
        }
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}

fn init() {
    let result_rom = get_rom_buffer("./tests/SML.gb");
    let rom = match result_rom {
        Err(error) => panic!("read rom error{:?}", error),
        Ok(rom) => rom,
    };

    /*
    for data in rom[0x104..0x134].iter() {
        println!("{:02x}", data);
    }
    */

    let title = get_rom_title(&rom);
    println!("{:?}", title);
}

fn get_rom_buffer(path: &str) -> io::Result<Vec<u8>> {
    let mut rom = vec![];
    let mut file = std::fs::File::open(path)?;
    file.read_to_end(&mut rom)?;
    Ok(rom)
}

fn get_rom_slice(rom: &Vec<u8>, start: u16, end: u16) -> &[u8] {
    &rom[(start as usize)..((end + 1) as usize)]
}

fn get_rom_title(rom: &Vec<u8>) -> String {
    let start = 0x134;
    let end = match &rom[0x0143] {
        &0x80 => 0x013E,
        _ => 0x0143,
    };
    let str_buff = get_rom_slice(rom, start, end);
    let mut result = String::new();
    for ch in str_buff.iter() {
        result.push(*ch as char);
    }
    result
}
