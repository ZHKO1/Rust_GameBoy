use crate::util::{u16_from_2u8, u8u8_from_u16};

pub trait Memory {
    fn get(&self, index: u16) -> u8;
    fn set(&mut self, index: u16, value: u8) -> bool;
    fn get_word(&self, index: u16) -> u16 {
        let low = self.get(index);
        let high = self.get(index + 1);
        u16_from_2u8(low, high)
    }
    fn set_word(&mut self, index: u16, value: u16) -> bool {
        let (value_low, value_high) = u8u8_from_u16(value);
        self.set(index, value_low);
        self.set(index + 1, value_high);
        true
    }
}
