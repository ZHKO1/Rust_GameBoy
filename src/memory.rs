pub trait Memory {
    fn get(&self, index: u16) -> u8;
    fn set(&mut self, index: u16, value: u8) -> bool;
    fn get_word(&self, index: u16) -> u16;
    fn set_word(&mut self, index: u16, value: u16) -> bool;
}