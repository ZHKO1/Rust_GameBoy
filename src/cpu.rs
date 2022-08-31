use crate::memory::Memory;
use crate::util::{u16_from_2u8, u8u8_from_u16};
use std::{cell::RefCell, rc::Rc};
use Flag::{C, H, N, Z};

//  0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f
const OP_CYCLES: [u32; 256] = [
    1, 3, 2, 2, 1, 1, 2, 1, 5, 2, 2, 2, 1, 1, 2, 1, // 0
    0, 3, 2, 2, 1, 1, 2, 1, 3, 2, 2, 2, 1, 1, 2, 1, // 1
    2, 3, 2, 2, 1, 1, 2, 1, 2, 2, 2, 2, 1, 1, 2, 1, // 2
    2, 3, 2, 2, 3, 3, 3, 1, 2, 2, 2, 2, 1, 1, 2, 1, // 3
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 4
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 5
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 6
    2, 2, 2, 2, 2, 2, 0, 2, 1, 1, 1, 1, 1, 1, 2, 1, // 7
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 8
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 9
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // a
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // b
    2, 3, 3, 4, 3, 4, 2, 4, 2, 4, 3, 0, 3, 6, 2, 4, // c
    2, 3, 3, 0, 3, 4, 2, 4, 2, 4, 3, 0, 3, 0, 2, 4, // d
    3, 3, 2, 0, 0, 4, 2, 4, 4, 1, 4, 0, 0, 0, 2, 4, // e
    3, 3, 2, 1, 0, 4, 2, 4, 3, 2, 4, 1, 0, 0, 2, 4, // f
];

//  0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f
const CB_CYCLES: [u32; 256] = [
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // 0
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // 1
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // 2
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // 3
    2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2, // 4
    2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2, // 5
    2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2, // 6
    2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2, // 7
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // 8
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // 9
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // a
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // b
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // c
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // d
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // e
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // f
];

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
    cycles: u32,
    cur_opcode_cycles: u32,
    memory: Rc<RefCell<dyn Memory>>,
}

impl Cpu {
    pub fn new(mmu: Rc<RefCell<dyn Memory>>) -> Self {
        let reg = Registers::new();
        Cpu {
            reg,
            memory: mmu,
            cycles: 0,
            cur_opcode_cycles: 0,
        }
    }
    pub fn run(&mut self) {
        loop {
            self.step();
        }
    }
    pub fn step(&mut self) -> u32 {
        let opcode = self.imm();
        let cycles = self.run_opcode(opcode);
        cycles * 4
    }
    pub fn trick(&mut self) {
        if self.cycles == 0 {
            self.cur_opcode_cycles = self.step();
        }
        if self.cycles == self.cur_opcode_cycles - 1 {
            self.cycles = 0;
            self.cur_opcode_cycles = 0;
        } else if self.cycles < self.cur_opcode_cycles - 1 {
            self.cycles += 1;
        } else {
            panic!("NOP or HALT?")
        }
    }
    /*
    ADD (address)      DEC r               LD A,(address)      LD rr,d16         RET
    BIT n,r            INC r               LD r,r              LD (HL+),A        RLA
    CALL address       INC rr              LD r,d8             LD (HL-),A        RL r
    CP d8              JR cond,address     LD r,(address)      POP rr            SUB r
    CP (HL)            LD (address),A      LD (address),r      PUSH rr           XOR r
    */
    fn run_opcode(&mut self, opcode: u8) -> u32 {
        // println!("{:02x}  PC:{:04x}", opcode, self.reg.pc - 1);
        let mut cb_opcode = 0;
        let mut is_jump = false;
        match opcode {
            // ADD (address)
            0x86 => match opcode {
                0x86 => {
                    let hl = self.reg.get_hl();
                    let hl_v = self.memory.borrow().get(hl);
                    self.opc_add(hl_v);
                }
                _ => {}
            },
            // PREFIX CB
            0xCB => {
                cb_opcode = self.imm();
                match cb_opcode {
                    0x11 => {
                        self.reg.c = self.opc_rl(self.reg.c);
                    }
                    0x7c => {
                        self.opc_cb_bit(7, self.reg.h);
                    }
                    _ => {
                        println!("{:02x}", cb_opcode);
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
            // CP d8
            0xFE | 0xBE => {
                match opcode {
                    0xFE => {
                        let d8 = self.imm();
                        self.opc_cp(d8);
                    }
                    0xBE => {
                        let hl = self.reg.get_hl();
                        let d8 = self.memory.borrow().get(hl);
                        self.opc_cp(d8);
                    }
                    _ => {}
                };
            }
            // DEC r
            0x05 | 0x3D | 0x0D | 0x15 | 0x1D => match opcode {
                0x3D => self.reg.a = self.opc_dec(self.reg.a),
                0x05 => self.reg.b = self.opc_dec(self.reg.b),
                0x0D => self.reg.c = self.opc_dec(self.reg.c),
                0x15 => self.reg.d = self.opc_dec(self.reg.d),
                0x1D => self.reg.e = self.opc_dec(self.reg.e),
                _ => {}
            },
            // INC r
            0x0c | 0x04 | 0x24 => match opcode {
                0x04 => self.reg.b = self.opc_inc(self.reg.b),
                0x0c => self.reg.c = self.opc_inc(self.reg.c),
                0x24 => self.reg.h = self.opc_inc(self.reg.h),
                _ => {}
            },
            // INC rr
            0x23 | 0x13 => match opcode {
                0x23 => self.reg.set_hl(self.reg.get_hl() + 1),
                0x13 => self.reg.set_de(self.reg.get_de() + 1),
                _ => {}
            },
            // JR r8
            0x18 => {
                let r8 = self.imm() as i8;
                self.opc_jr(r8);
            }
            // JR cond,address
            0x20 | 0x28 => {
                is_jump = match opcode {
                    0x20 => !self.reg.get_flag(Z),
                    0x28 => self.reg.get_flag(Z),
                    _ => {
                        panic!("JR NZ,r8. But cond?")
                    }
                };
                let r8 = self.imm() as i8;
                if is_jump {
                    self.opc_jr(r8);
                }
            }
            // LD (address), A
            0x77 | 0xEA | 0xE0 | 0xE2 => {
                let address = match opcode {
                    0x77 => self.reg.get_hl(),
                    0xEA => self.imm_word(),
                    0xE0 => (0xFF00 | (self.imm() as u16)),
                    0xE2 => (0xFF00 | (self.reg.c as u16)),
                    _ => {
                        panic!("LD (address), A. But address?")
                    }
                };
                let a = self.reg.a;
                self.memory.borrow_mut().set(address, a);
            }
            // LD A,(DE)
            0x1a => {
                let de = self.reg.get_de();
                let de_v = self.memory.borrow_mut().get(de);
                self.reg.a = de_v;
            }
            // LD A,(address)
            0xF0 => {
                let address = 0xFF00 | (self.imm() as u16);
                let value = self.memory.borrow_mut().get(address);
                self.reg.a = value;
            }
            // LD r,r
            0x4f | 0x7b | 0x67 | 0x57 | 0x7C | 0x78 | 0x7D => match opcode {
                0x4f => self.reg.c = self.reg.a,
                0x7b => self.reg.a = self.reg.e,
                0x67 => self.reg.h = self.reg.a,
                0x57 => self.reg.d = self.reg.a,
                0x7C => self.reg.a = self.reg.h,
                0x78 => self.reg.a = self.reg.b,
                0x7D => self.reg.a = self.reg.l,
                _ => {}
            },
            // LD r,d8
            0x3E | 0x06 | 0x0E | 0x16 | 0x1E | 0x2E => {
                let d8 = self.imm();
                match opcode {
                    0x3E => self.reg.a = d8,
                    0x06 => self.reg.b = d8,
                    0x0E => self.reg.c = d8,
                    0x16 => self.reg.d = d8,
                    0x1E => self.reg.e = d8,
                    0x2E => self.reg.l = d8,
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
            // LD (HL+),A
            0x22 => {
                let hl = self.reg.get_hl();
                let a = self.reg.a;
                self.memory.borrow_mut().set(hl, a);
                self.reg.set_hl(hl.wrapping_add(1));
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
            // RET
            0xC9 => {
                let pc = self.stack_pop();
                self.reg.pc = pc;
            }
            // RLA
            0x17 => {
                self.reg.a = self.opc_rl(self.reg.a);
                self.reg.set_flag(Z, false);
            }
            // SUB R
            0x90 => match opcode {
                0x90 => self.opc_sub(self.reg.b),
                _ => {}
            },
            // XOR A
            0xAF => {
                self.opc_xor(self.reg.a);
            }
            _ => {
                println!("{:02x}  PC:{:04x}", opcode, self.reg.pc - 1);
                panic!("unkown opcode");
            }
        };
        let mut ecycle = 0;
        if is_jump {
            ecycle = match opcode {
                0x20 => 1,
                0x28 => 1,
                0xC0 => 3,
                0xC2 => 1,
                0xC4 => 3,
                0xC8 => 3,
                0xCA => 1,
                0xCC => 3,
                0xD0 => 3,
                0xD2 => 1,
                0xD4 => 3,
                0xD8 => 3,
                0xDA => 1,
                0xDC => 3,
                _ => 0,
            }
        }
        if opcode == 0xCB {
            CB_CYCLES[cb_opcode as usize]
        } else {
            OP_CYCLES[opcode as usize] + ecycle
        }
    }
    fn opc_add(&mut self, value: u8) {
        let a = self.reg.a;
        let result = a.wrapping_add(value);
        self.reg.set_flag(Z, result == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, a & 0x0f + value & 0x0f > 0x0f);
        self.reg.set_flag(C, a as u16 + value as u16 > 0xff);
        self.reg.a = result;
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
        // println!("jump to {:04x}", self.reg.pc);
    }
    fn opc_inc(&mut self, reg: u8) -> u8 {
        let result = reg.wrapping_add(1);
        self.reg.set_flag(Z, result == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, (reg & 0x0f) == 0x0f);
        result
    }
    fn opc_dec(&mut self, reg: u8) -> u8 {
        let result = reg.wrapping_sub(1);
        self.reg.set_flag(Z, result == 0);
        self.reg.set_flag(N, true);
        self.reg.set_flag(H, (reg & 0x0F) == 0x00);
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
    fn opc_cp(&mut self, value: u8) {
        let a = self.reg.a;
        self.reg.set_flag(Z, a == value);
        self.reg.set_flag(N, true);
        self.reg.set_flag(H, a & 0x0f < value & 0x0f);
        self.reg.set_flag(C, a < value);
    }
    fn opc_sub(&mut self, value: u8) {
        let a = self.reg.a;
        let r = a.wrapping_sub(value);
        self.reg.set_flag(Z, r == 0);
        self.reg.set_flag(N, true);
        self.reg.set_flag(H, a & 0x0f < value & 0x0f);
        self.reg.set_flag(C, (a as u16) < (value as u16));
        self.reg.a = r;
    }
    fn opc_cb_bit(&mut self, bit: u8, r: u8) {
        let z = ((1 << bit) & r) == 0;
        self.reg.set_flag(Z, z);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, true);
    }
    fn stack_push(&mut self, value: u16) {
        self.reg.sp = self.reg.sp - 2;
        self.memory.borrow_mut().set_word(self.reg.sp, value);
    }
    fn stack_pop(&mut self) -> u16 {
        let value = self.memory.borrow_mut().get_word(self.reg.sp);
        self.memory.borrow_mut().set_word(self.reg.sp, 0);
        self.reg.sp = self.reg.sp + 2;
        value
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
}
