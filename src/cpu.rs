use crate::util::{u16_from_2u8, u8u8_from_u16};
use crate::{memory::Memory, mmu::Mmu};
use std::{cell::RefCell, rc::Rc};
struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
    sp: u16,
    pc: u16,
}
impl Registers {
    fn new() -> Self {
        Registers {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: 0,
            h: 0,
            l: 0,
            sp: 0,
            pc: 0,
        }
    }
    fn setAF(&mut self, value: u16) {
        let (value_low, value_high) = u8u8_from_u16(value);
        self.a = value_high;
        self.f = value_low;
    }
    fn getAF(&self) -> u16 {
        u16_from_2u8(self.f, self.a)
    }
    fn setBC(&mut self, value: u16) {
        let (value_low, value_high) = u8u8_from_u16(value);
        self.b = value_high;
        self.c = value_low;
    }
    fn getBC(&self) -> u16 {
        u16_from_2u8(self.c, self.b)
    }
    fn setDE(&mut self, value: u16) {
        let (value_low, value_high) = u8u8_from_u16(value);
        self.d = value_high;
        self.e = value_low;
    }
    fn getDE(&self) -> u16 {
        u16_from_2u8(self.e, self.d)
    }
    fn setHL(&mut self, value: u16) {
        let (value_low, value_high) = u8u8_from_u16(value);
        self.h = value_high;
        self.l = value_low;
    }
    fn getHL(&self) -> u16 {
        u16_from_2u8(self.l, self.h)
    }
}
pub struct Cpu {
    reg: Registers,
    cycles: usize,
    memory: Rc<RefCell<dyn Memory>>,
}

impl Cpu {
    pub fn new(mmu: Rc<RefCell<dyn Memory>>) -> Self {
        let reg = Registers::new();
        Cpu {
            reg,
            memory: mmu,
            cycles: 0,
        }
    }
    pub fn run(&mut self) {
        loop {
            self.step();
        }
    }
    pub fn step(&mut self) {
        let opcode = self.get_opcode();
        self.run_opcode(opcode);
    }
    fn get_opcode(&self) -> u8 {
        self.memory.borrow().get(self.reg.pc)
    }
    fn run_opcode(&mut self, opcode: u8) {
        match opcode {
            0x31 => {
                println!("LD SP,d16");
                let cycle = 12;
                let d16 = self.memory.borrow().get_word(self.reg.pc + 1);
                self.reg.sp = d16;
                self.reg.pc += 3;
            }
            0xAF => {
                println!("XOR A");
                let cycle = 4;
                self.reg.a ^= self.reg.a;
                self.reg.pc += 1;
            }
            _ => {
                println!("{:02x}", opcode);
                panic!("unkown opcode");
                self.reg.pc += 1;
            }
        }
    }
}
