use crate::util::{u16_from_2u8, u8u8_from_u16};
use crate::{memory::Memory, mmu::Mmu};
use std::{cell::RefCell, rc::Rc};
use Flag::{C, H, N, Z};
pub enum Flag {
    Z = 0b1000_0000,
    N = 0b0100_0000,
    H = 0b0010_0000,
    C = 0b0001_0000,
}

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
    fn set_af(&mut self, value: u16) {
        let (value_low, value_high) = u8u8_from_u16(value);
        self.a = value_high;
        self.f = value_low;
    }
    fn get_af(&self) -> u16 {
        u16_from_2u8(self.f, self.a)
    }
    fn set_bc(&mut self, value: u16) {
        let (value_low, value_high) = u8u8_from_u16(value);
        self.b = value_high;
        self.c = value_low;
    }
    fn get_bc(&self) -> u16 {
        u16_from_2u8(self.c, self.b)
    }
    fn set_de(&mut self, value: u16) {
        let (value_low, value_high) = u8u8_from_u16(value);
        self.d = value_high;
        self.e = value_low;
    }
    fn get_de(&self) -> u16 {
        u16_from_2u8(self.e, self.d)
    }
    fn set_hl(&mut self, value: u16) {
        let (value_low, value_high) = u8u8_from_u16(value);
        self.h = value_high;
        self.l = value_low;
    }
    fn get_hl(&self) -> u16 {
        u16_from_2u8(self.l, self.h)
    }
    fn get_flag(&self, flag: Flag) -> bool {
        self.f & (flag as u8) != 0
    }
    fn set_flag(&mut self, flag: Flag, value: bool) {
        if value {
            self.f = self.f | (flag as u8)
        } else {
            self.f = self.f & !(flag as u8)
        }
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
        let opcode = self.imm();
        self.run_opcode(opcode);
    }
    fn imm(&mut self) -> u8 {
        let v = self.memory.borrow().get(self.reg.pc);
        self.reg.pc += 1;
        v
    }
    fn imm_word(&mut self) -> u16 {
        let low = self.memory.borrow().get(self.reg.pc);
        let high = self.memory.borrow().get(self.reg.pc + 1);
        self.reg.pc += 2;
        u16_from_2u8(low, high)
    }
    fn run_opcode(&mut self, opcode: u8) {
        match opcode {
            // LD rr,d16
            0x21 | 0x31 => {
                println!("LD rr,d16");
                let d16 = self.imm_word();
                match opcode {
                    0x21 => self.reg.set_hl(d16),
                    0x31 => self.reg.sp = d16,
                    _ => {}
                }
            }
            // LD (HL-),A
            0x32 => {
                println!("LD (HL-),A");
                
            }
            // XOR A
            0xAF => {
                println!("XOR A");
                self.opc_xor(self.reg.a);
            }
            _ => {
                println!("{:02x}", opcode);
                panic!("unkown opcode");
            }
        }
    }
    fn opc_xor(&mut self, n: u8) {
        self.reg.a ^= n;
        self.reg.set_flag(Z, self.reg.a == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, false);
        self.reg.set_flag(C, false);
    }
}
