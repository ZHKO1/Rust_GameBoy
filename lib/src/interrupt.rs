use crate::memory::Memory;

pub enum InterruptFlag {
    VBlank = 0,
    LCDSTAT = 1,
    Timer = 2,
    Serial = 3,
    Joypad = 4,
}

pub struct Interrupt {
    data: u8,
}

impl Interrupt {
    pub fn new() -> Self {
        Self { data: 0x00 }
    }
    pub fn set_flag(&mut self, flag: InterruptFlag) {
        self.data |= 1 << (flag as u8);
    }
}

impl Memory for Interrupt {
    fn set(&mut self, index: u16, value: u8) {
        assert_eq!(index, 0xFF0F);
        self.data = value;
    }
    fn get(&self, index: u16) -> u8 {
        assert_eq!(index, 0xFF0F);
        self.data
    }
}
