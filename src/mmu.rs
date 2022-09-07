use std::path::Path;

use crate::cartridge::{open, Cartridge};
use crate::memory::Memory;
use crate::util::read_rom;
struct MemoryBlock {
    memory: [u8; 0xFFFF - 0x8000 + 1],
    start: u16,
    end: u16,
}

impl MemoryBlock {
    fn new() -> Self {
        let start = 0x8000;
        let end = 0xFFFF;
        MemoryBlock {
            start,
            end,
            memory: [0; 0xFFFF - 0x8000 + 1],
        }
    }
}

impl Memory for MemoryBlock {
    fn get(&self, index: u16) -> u8 {
        if index < self.start || index > self.end {
            return 0xFF;
        }
        self.memory[(index - self.start) as usize]
    }
    fn set(&mut self, index: u16, value: u8) {
        if index < self.start || index > self.end {
            return;
        }
        self.memory[(index - self.start) as usize] = value;
    }
}

pub struct Mmu {
    boot: [u8; 0x100],
    cartridge: Box<dyn Cartridge>,
    other: MemoryBlock,
    pub log_msg: String,
}

impl Mmu {
    pub fn new(bios_path: impl AsRef<Path>, rom_path: impl AsRef<Path>) -> Self {
        let cartridge = open(rom_path);
        let other = MemoryBlock::new();
        let mut boot = [0; 0x100];
        let skip_boot = bios_path.as_ref().to_str().unwrap().is_empty();
        let mut mmu = Self {
            boot,
            cartridge,
            other,
            log_msg: String::new(),
        };
        if skip_boot {
            mmu.set(0xFF50, 1);
        } else {
            let boot_rom = read_rom(bios_path).unwrap();
            boot.copy_from_slice(&boot_rom[..0x100]);
        };
        mmu
    }
    pub fn is_boot(&self) -> bool {
        let v = self.get(0xFF50);
        v == 0
    }
    pub fn bind_event(&mut self, index: u16, value: u8) {
        match index {
            0xFF02 => {
                if value == 0x81 {
                    let v = self.get(0xFF01);
                    self.log_msg.push(v as char);
                }
            }
            _ => {}
        };
    }
}
impl Memory for Mmu {
    fn get(&self, index: u16) -> u8 {
        match index {
            0x0000..=0x00ff => {
                if self.is_boot() {
                    self.boot[index as usize]
                } else {
                    self.cartridge.get(index)
                }
            }
            0x0100..=0x7FFF => self.cartridge.get(index),
            0x8000..=0x9FFF => self.other.get(index),
            0xA000..=0xBFFF => self.cartridge.get(index),
            0xC000..=0xFFFF => self.other.get(index),
        }
    }
    fn set(&mut self, index: u16, value: u8) {
        self.bind_event(index, value);
        match index {
            0x0000..=0x7FFF => self.cartridge.set(index, value),
            0x8000..=0x9FFF => self.other.set(index, value),
            0xA000..=0xBFFF => self.cartridge.set(index, value),
            0xC000..=0xFFFF => self.other.set(index, value),
        }
    }
}
