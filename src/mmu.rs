use crate::cartridge::RoomBlank;
use crate::memory::Memory;
use crate::util::{u16_from_2u8, u8u8_from_u16};
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
    fn set(&mut self, index: u16, value: u8) -> bool {
        if index < self.start || index > self.end {
            return false;
        }
        self.memory[(index - self.start) as usize] = value;
        true
    }
}

pub struct Mmu {
    room_blank: RoomBlank,
    other: MemoryBlock,
}

impl Mmu {
    pub fn new() -> Self {
        let room_blank = RoomBlank::new();
        let other = MemoryBlock::new();
        Mmu { room_blank, other }
    }
}
impl Memory for Mmu {
    fn get(&self, index: u16) -> u8 {
        match index {
            0x0000..=0x7FFF => self.room_blank.get(index),
            0x8000..=0xFFFF => self.other.get(index),
        }
    }
    fn set(&mut self, index: u16, value: u8) -> bool {
        match index {
            0x0000..=0x7FFF => self.room_blank.set(index, value),
            0x8000..=0xFFFF => self.other.set(index, value),
        }
    }
    fn get_word(&self, index: u16) -> u16 {
        match index {
            0x0000..=0x7FFF => self.room_blank.get_word(index),
            0x8000..=0xFFFF => self.other.get_word(index),
        }
    }
    fn set_word(&mut self, index: u16, value: u16) -> bool {
        match index {
            0x0000..=0x7FFF => self.room_blank.set_word(index, value),
            0x8000..=0xFFFF => self.other.set_word(index, value),
        }
    }
}
