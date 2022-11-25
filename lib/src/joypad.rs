use crate::memory::Memory;

#[derive(Clone)]
pub enum JoyPadKey {
    Right = 0b0000_0001,
    Left = 0b0000_0010,
    Up = 0b0000_0100,
    Down = 0b0000_1000,
    A = 0b0001_0000,
    B = 0b0010_0000,
    Select = 0b0100_0000,
    Start = 0b1000_0000,
}
#[derive(serde::Deserialize, serde::Serialize)]
pub struct JoyPad {
    matrix: u8,
    select: u8,
    pub interrupt_flag: bool,
}

impl JoyPad {
    pub fn new() -> Self {
        Self {
            select: 0x00,
            matrix: 0xFF,
            interrupt_flag: false,
        }
    }
    pub fn input(&mut self, key: JoyPadKey, is_pressed: bool) {
        if is_pressed {
            self.matrix &= !(key as u8);
            if self.select & 0x30 > 0 {
                self.interrupt_flag = true;
            }
        } else {
            self.matrix |= key as u8;
        }
    }
}

impl Memory for JoyPad {
    fn get(&self, index: u16) -> u8 {
        assert_eq!(index, 0xFF00);
        if (self.select & 0b01_0000) == 0x00 {
            return self.select | (self.matrix & 0x0f);
        }
        if (self.select & 0b10_0000) == 0x00 {
            return self.select | (self.matrix >> 4);
        }
        self.select | 0x0f
    }
    fn set(&mut self, index: u16, value: u8) {
        assert_eq!(index, 0xFF00);
        self.select = value & 0x30;
    }
}
