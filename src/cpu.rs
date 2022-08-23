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

    /*
    ADD (address)      DEC r               LD A,(address)      LD rr,d16         RET
    BIT n,r            INC r               LD r,r              LD (HL+),A        RLA
    CALL address       INC rr              LD r,d8             LD (HL-),A        RL r
    CP d8              JR cond,address     LD r,(address)      POP rr            SUB r
    CP (HL)            LD (address),A      LD (address),r      PUSH rr           XOR r
    */
    fn run_opcode(&mut self, opcode: u8) {
        match opcode {
            // PREFIX CB
            0xCB => {
                let opcode = self.imm();
                match opcode {
                    0x11 => {
                        self.reg.c = self.opc_rl(self.reg.c);
                    }
                    0x7c => {
                        self.opc_cb_bit(7, self.reg.h);
                    }
                    _ => {
                        println!("{:02x}", opcode);
                        panic!("unkown opcode CB");
                    }
                }
            }
            // CALL a16
            0xcd => {
                let a16 = self.imm_word();
                let pc = self.reg.pc;
                self.stack_push(pc);
                self.reg.pc = a16;
            }
            // INC r
            0x0c => match opcode {
                0x0c => self.reg.c = self.opc_inc(self.reg.c),
                _ => {}
            },
            // JR NZ,r8
            0x20 => {
                let is_jump = match opcode {
                    0x20 => !self.reg.get_flag(Z),
                    _ => {
                        panic!("JR NZ,r8")
                    }
                };
                let r8 = self.imm() as i8;
                if is_jump {
                    self.opc_jr(r8);
                }
            }
            // LD (HL), A
            0x77 => {
                let hl = self.reg.get_hl();
                let a = self.reg.a;
                self.memory.borrow_mut().set(hl, a);
            }
            // LD ($FF00+a8),A
            0xE0 => {
                let a8 = self.imm();
                self.memory
                    .borrow_mut()
                    .set(0xFF00 | (a8 as u16), self.reg.a);
            }
            // LD ($FF00+C),A
            0xE2 => {
                let c = self.reg.c;
                self.memory
                    .borrow_mut()
                    .set(0xFF00 | (c as u16), self.reg.a);
            }
            // LD A,(DE)
            0x1a => {
                let de = self.reg.get_de();
                let de_v = self.memory.borrow_mut().get(de);
                self.reg.a = de_v;
            }
            // LD r,r
            0x4f => match opcode {
                0x4f => self.reg.c = self.reg.a,
                _ => {}
            },
            // LD r,d8
            0x06 | 0x0E | 0x3E => {
                let d8 = self.imm();
                match opcode {
                    0x06 => self.reg.b = d8,
                    0x0E => self.reg.c = d8,
                    0x3E => self.reg.a = d8,
                    _ => {}
                }
            }
            // LD rr,d16
            0x11 | 0x21 | 0x31 => {
                let d16 = self.imm_word();
                match opcode {
                    0x11 => self.reg.set_de(d16),
                    0x21 => self.reg.set_hl(d16),
                    0x31 => self.reg.sp = d16,
                    _ => {}
                }
            }
            // LD (HL-),A
            0x32 => {
                let hl = self.reg.get_hl();
                let a = self.reg.a;
                self.memory.borrow_mut().set(hl, a);
                self.reg.set_hl(hl.wrapping_sub(1));
            }
            // POP rr
            0xC1 => {
                let bc = self.stack_pop();
                self.reg.set_bc(bc);
            }
            // PUSH nn
            0xC5 => {
                let bc = self.reg.get_bc();
                self.stack_push(bc);
            }
            // RLA
            0x17 => {
                self.reg.a = self.opc_rl(self.reg.a);
                self.reg.set_flag(Z, false);
            }
            // XOR A
            0xAF => {
                self.opc_xor(self.reg.a);
            }
            _ => {
                println!("{:02x}  PC:{:04x}", opcode, self.reg.pc - 1);
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
    fn opc_jr(&mut self, r8: i8) {
        self.reg.pc = self.reg.pc.wrapping_add(r8 as u16);
        println!("jump to {:04x}", self.reg.pc);
    }
    fn opc_inc(&mut self, reg: u8) -> u8 {
        let result = reg.wrapping_add(1);
        self.reg.set_flag(Z, result == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, (reg & 0x0f) + 0x01 > 0x0f);
        result
    }
    fn opc_rl(&mut self, value: u8) -> u8 {
        let c = value >> 7 == 0x01;
        let old_c = self.reg.get_flag(C);
        let result = value << 1 | u8::from(old_c);
        self.reg.set_flag(Z, result == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, false);
        self.reg.set_flag(C, c);
        result
    }
    fn opc_cb_bit(&mut self, bit: u8, r: u8) {
        let z = ((1 << bit) & r) == 0;
        self.reg.set_flag(Z, z);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, true);
    }
    fn stack_push(&mut self, value: u16) {
        self.memory.borrow_mut().set_word(self.reg.sp - 1, value);
        self.reg.sp = self.reg.sp - 2;
    }
    fn stack_pop(&mut self) -> u16 {
        let value = self.memory.borrow_mut().get_word(self.reg.sp + 1);
        self.memory.borrow_mut().set_word(self.reg.sp + 1, 0);
        self.reg.sp = self.reg.sp + 2;
        value
    }
}
