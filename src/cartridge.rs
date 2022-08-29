use crate::memory::Memory;
use crate::util::{read_rom, u16_from_2u8, u8u8_from_u16};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use MBC1Mode::{Ram, Rom};

pub struct RoomBlank {
    rom: [u8; 0x8000],
    boot_rom: [u8; 0x100],
    start: u16,
    end: u16,
    is_boot_rom: bool,
}

impl RoomBlank {
    pub fn new() -> Self {
        let start = 0;
        let end = 0x7FFF;
        let mut room_blank = RoomBlank {
            start,
            end,
            rom: [0xFF; 0x8000],
            boot_rom: [0; 0x100],
            is_boot_rom: true,
        };
        let boot_rom = read_rom("./tests/DMG_ROM.bin");
        for (index, data) in boot_rom.unwrap().iter().enumerate() {
            room_blank.boot_rom[index] = *data;
        }
        room_blank
    }
}
impl Memory for RoomBlank {
    fn get(&self, index: u16) -> u8 {
        match index {
            0x0000..=0x00ff => {
                if self.is_boot_rom == true {
                    self.boot_rom[(index - self.start) as usize]
                } else {
                    self.rom[(index - self.start) as usize]
                }
            }
            0x0100..=0x7FFF => self.rom[(index - self.start) as usize],
            _ => {
                return 0xFF;
            }
        }
    }
    fn set(&mut self, index: u16, value: u8) {
        match index {
            0x0000..=0x00ff => {
                if self.is_boot_rom == true {
                    self.boot_rom[(index - self.start) as usize] = value;
                } else {
                    self.rom[(index - self.start) as usize] = value;
                }
            }
            0x0100..=0x7FFF => {
                self.rom[(index - self.start) as usize] = value;
            }
            _ => {}
        };
    }
    fn get_word(&self, index: u16) -> u16 {
        match index {
            0x0000..=0x00ff => {
                if self.is_boot_rom == true {
                    u16_from_2u8(
                        self.boot_rom[(index - self.start) as usize],
                        self.boot_rom[(index - self.start + 1) as usize],
                    )
                } else {
                    u16_from_2u8(
                        self.rom[(index - self.start) as usize],
                        self.rom[(index - self.start + 1) as usize],
                    )
                }
            }
            0x0100..=0x7FFF => u16_from_2u8(
                self.rom[(index - self.start) as usize],
                self.rom[(index - self.start + 1) as usize],
            ),
            _ => {
                return 0xFFFF;
            }
        }
    }
    fn set_word(&mut self, index: u16, value: u16) {
        let (value_low, value_high) = u8u8_from_u16(value);
        match index {
            0x0000..=0x00ff => {
                if self.is_boot_rom == true {
                    self.boot_rom[(index - self.start) as usize] = value_low;
                    self.boot_rom[(index - self.start + 1) as usize] = value_high;
                } else {
                    self.rom[(index - self.start) as usize] = value_low;
                    self.rom[(index - self.start + 1) as usize] = value_high;
                }
            }
            0x0100..=0x7FFF => {
                self.rom[(index - self.start) as usize] = value_low;
                self.rom[(index - self.start + 1) as usize] = value_high;
            }
            _ => {}
        };
    }
}

pub trait Stable {
    fn save(&self) {}
}

struct RomOnly {
    rom: Vec<u8>,
}
impl RomOnly {
    fn new(rom: Vec<u8>) -> Self {
        RomOnly { rom }
    }
}
impl Memory for RomOnly {
    fn get(&self, index: u16) -> u8 {
        self.rom[index as usize]
    }
    fn set(&mut self, index: u16, value: u8) {}
}

enum MBC1Mode {
    Rom, //  16Mbit ROM/8KByte RAM
    Ram, // false 4Mbit ROM/32KByte RAM
}
struct MBC1 {
    mode: MBC1Mode,
    rom: Vec<u8>,
    ram: Vec<u8>,
    rom_blank_bit: u8,
    ram_blank_bit: u8,
    ram_enable: bool,
    save_path: PathBuf,
}
impl MBC1 {
    fn new(rom: Vec<u8>, ram: Vec<u8>, path: impl AsRef<Path>) -> Self {
        MBC1 {
            mode: Rom,
            rom,
            ram,
            rom_blank_bit: 0b00001,
            ram_blank_bit: 0b00,
            ram_enable: false,
            save_path: PathBuf::from(path.as_ref()),
        }
    }
    fn get_rom_blank_index(&self) -> u8 {
        let result = match self.mode {
            Rom => self.ram_blank_bit << 5 + self.rom_blank_bit,
            Ram => self.rom_blank_bit,
        };
        match result {
            0x00 => 0x01,
            0x20 => 0x21,
            0x40 => 0x41,
            0x60 => 0x61,
            _ => result,
        }
    }
    fn get_ram_blank_index(&self) -> u8 {
        match self.mode {
            Rom => 0,
            Ram => self.ram_blank_bit,
        }
    }
}
impl Memory for MBC1 {
    fn get(&self, index: u16) -> u8 {
        let rom_blank_index = self.get_rom_blank_index();
        let ram_blank_index = self.get_ram_blank_index();
        match index {
            0..=0x3FFF => self.rom[index as usize],
            0x4000..=0x7FFF => {
                let rom_index =
                    rom_blank_index as usize * 0x4000 as usize + (index - 0x4000) as usize;
                self.rom[rom_index]
            }
            0xA000..=0xBFFF => {
                if self.ram_enable {
                    let ram_index =
                        ram_blank_index as usize * 0x2000 as usize + (index - 0xA000) as usize;
                    self.ram[ram_index]
                } else {
                    0x00
                }
            }
            _ => panic!("out range of MC1"),
        }
    }
    fn set(&mut self, index: u16, value: u8) {
        match index {
            0x0000..=0x1FFF => {
                if value & 0x0A == 0x0A {
                    self.ram_enable = true;
                } else {
                    self.ram_enable = false;
                    self.save();
                }
            }
            0x2000..=0x3FFF => {
                self.rom_blank_bit = value & 0x1F;
            }
            0x4000..=0x5FFF => {
                self.ram_blank_bit = value & 0x03;
            }
            0x6000..=0x7FFF => match value {
                0x00 => self.mode = Rom,
                0x01 => self.mode = Ram,
                _ => {}
            },
            0xA000..=0xBFFF => {
                if self.ram_enable {
                    let ram_blank_index = self.get_ram_blank_index();
                    let ram_index =
                        ram_blank_index as usize * 0x2000 as usize + (index - 0xA000) as usize;
                    self.ram[ram_index] = value;
                }
            }
            _ => panic!("out range of MC1"),
        }
    }
}
impl Stable for MBC1 {
    fn save(&self) {
        if self.save_path.to_str().unwrap().is_empty() {
            return;
        }
        File::create(self.save_path.clone())
            .and_then(|mut file| file.write_all(&self.ram))
            .unwrap();
    }
}

struct MBC2 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    rom_blank: u8,
    ram_enable: bool,
    save_path: PathBuf,
}
impl MBC2 {
    fn new(rom: Vec<u8>, ram: Vec<u8>, path: impl AsRef<Path>) -> Self {
        MBC2 {
            rom,
            ram,
            rom_blank: 1,
            ram_enable: false,
            save_path: PathBuf::from(path.as_ref()),
        }
    }
}
impl Memory for MBC2 {
    fn get(&self, index: u16) -> u8 {
        match index {
            0..=0x3FFF => self.rom[index as usize],
            0x4000..=0x7FFF => {
                let rom_index =
                    self.rom_blank as usize * 0x4000 as usize + (index - 0x4000) as usize;
                self.rom[rom_index]
            }
            0xA000..=0xA1FF => self.ram[index as usize],
            0xA200..=0xBFFF => {
                if self.ram_enable {
                    let ram_index = 0xA000 + (index - 0xA000 - 1) % 0x01FF;
                    self.ram[ram_index as usize]
                } else {
                    0x00
                }
            }
            _ => panic!("out range of MC1"),
        }
    }
    fn set(&mut self, index: u16, value: u8) {
        match index {
            0x0000..=0x3FFF => {
                let bit8 = index & 0x100 >> 8;
                if bit8 == 0 {
                    if value & 0x0A == 0x0A {
                        self.ram_enable = true;
                    } else {
                        self.ram_enable = false;
                        self.save();
                    }
                } else {
                    if value != 0 {
                        self.rom_blank = value;
                    }
                }
            }
            0x4000..=0x7FFF => {}
            0xA000..=0xBFFF => {
                if self.ram_enable {
                    let ram_index = 0xA000 + (index - 0xA000 - 1) % 0x01FF;
                    self.ram[ram_index as usize] = value;
                }
            }
            _ => panic!("out range of MC1"),
        }
    }
}
impl Stable for MBC2 {
    fn save(&self) {
        if self.save_path.to_str().unwrap().is_empty() {
            return;
        }
        File::create(self.save_path.clone())
            .and_then(|mut file| file.write_all(&self.ram))
            .unwrap();
    }
}
