use crate::{memory::Memory, mmu::Mmu};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone)]
pub enum JoyPadKey {
    A,
    B,
    Left,
    Right,
    Up,
    Down,
    Start,
    Select,
}

pub struct JoyPad {
    mmu: Rc<RefCell<Mmu>>,
}

impl JoyPad {
    pub fn new(mmu: Rc<RefCell<Mmu>>) -> Self {
        let mut joypad = Self { mmu };
        joypad.set(0x3F);
        joypad
    }
    pub fn input(&mut self, key: JoyPadKey, is_pressed: bool) {
        let mut value = self.get();
        let act_dir_bit;
        let bit;
        match key {
            JoyPadKey::A => {
                act_dir_bit = 0b01_0000;
                bit = 0;
            }
            JoyPadKey::B => {
                act_dir_bit = 0b01_0000;
                bit = 1;
            }
            JoyPadKey::Left => {
                act_dir_bit = 0b10_0000;
                bit = 1;
            }
            JoyPadKey::Right => {
                act_dir_bit = 0b10_0000;
                bit = 0;
            }
            JoyPadKey::Up => {
                act_dir_bit = 0b10_0000;
                bit = 2;
            }
            JoyPadKey::Down => {
                act_dir_bit = 0b10_0000;
                bit = 3;
            }
            JoyPadKey::Start => {
                act_dir_bit = 0b01_0000;
                bit = 3;
            }
            JoyPadKey::Select => {
                act_dir_bit = 0b01_0000;
                bit = 2;
            }
        };
        value = (value & 0b00_1111) | act_dir_bit;
        if is_pressed {
            value = self.bit_res(value, bit);
            self.set_joypad_interrupt();
        } else {
            value = self.bit_set(value, bit);
        }
        self.set(value);
    }
    fn get(&self) -> u8 {
        self.mmu.borrow().get(0xFF00)
    }
    fn set(&mut self, value: u8) {
        self.mmu.borrow_mut().set(0xFF00, value);
    }
    fn bit_res(&mut self, n: u8, bit: u8) -> u8 {
        let r = n & !(1 << bit);
        r
    }
    fn bit_set(&mut self, n: u8, bit: u8) -> u8 {
        let r = n | (1 << bit);
        r
    }
    fn set_joypad_interrupt(&mut self) {
        let d8 = self.mmu.borrow_mut().get(0xFF0F);
        self.mmu.borrow_mut().set(0xFF0F, d8 | 0b10000);
    }
}
