use crate::memory::Memory;
use crate::util::{u16_from_2u8, u8u8_from_u16};
use std::io::{self, Read};

fn get_boot_rom(path: &str) -> io::Result<Vec<u8>> {
    let mut rom = vec![];
    let mut file = std::fs::File::open(path)?;
    file.read_to_end(&mut rom)?;
    Ok(rom)
}

pub struct RoomBlank {
    memory: [u8; 0x8000],
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
            memory: [0xFF; 0x8000],
            boot_rom: [0; 0x100],
            is_boot_rom: true,
        };
        let boot_rom = get_boot_rom("./tests/DMG_ROM.bin");
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
                    self.memory[(index - self.start) as usize]
                }
            }
            0x0100..=0x7FFF => self.memory[(index - self.start) as usize],
            _ => {
                return 0xFF;
            }
        }
    }
    fn set(&mut self, index: u16, value: u8) -> bool {
        match index {
            0x0000..=0x00ff => {
                if self.is_boot_rom == true {
                    self.boot_rom[(index - self.start) as usize] = value;
                } else {
                    self.memory[(index - self.start) as usize] = value;
                }
                true
            }
            0x0100..=0x7FFF => {
                self.memory[(index - self.start) as usize] = value;
                true
            }
            _ => false,
        }
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
                        self.memory[(index - self.start) as usize],
                        self.memory[(index - self.start + 1) as usize],
                    )
                }
            }
            0x0100..=0x7FFF => u16_from_2u8(
                self.memory[(index - self.start) as usize],
                self.memory[(index - self.start + 1) as usize],
            ),
            _ => {
                return 0xFFFF;
            }
        }
    }
    fn set_word(&mut self, index: u16, value: u16) -> bool {
        let (value_low, value_high) = u8u8_from_u16(value);
        match index {
            0x0000..=0x00ff => {
                if self.is_boot_rom == true {
                    self.boot_rom[(index - self.start) as usize] = value_low;
                    self.boot_rom[(index - self.start + 1) as usize] = value_high;
                } else {
                    self.memory[(index - self.start) as usize] = value_low;
                    self.memory[(index - self.start + 1) as usize] = value_high;
                }
                true
            }
            0x0100..=0x7FFF => {
                self.memory[(index - self.start) as usize] = value_low;
                self.memory[(index - self.start + 1) as usize] = value_high;
                true
            }
            _ => false,
        }
    }
}
