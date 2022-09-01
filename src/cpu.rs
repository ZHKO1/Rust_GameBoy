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
            // NOP
            0x00 => {}
            // ADD (address)
            // ADD A r
            // ADD A n
            // ADD A (HL)
            // ADD HL rr
            // ADD SP dd
            0x09 | 0x19 | 0x29 | 0x39 => match opcode {
                0x09 => {
                    self.opc_add_hl(self.reg.get_bc());
                }
                0x19 => {
                    self.opc_add_hl(self.reg.get_de());
                }
                0x29 => {
                    self.opc_add_hl(self.reg.get_hl());
                }
                0x39 => {
                    self.opc_add_hl(self.reg.sp);
                }
                _ => {}
            },
            0x80..=0x87 => match opcode {
                0x80 => self.opc_add(self.reg.b),
                0x81 => self.opc_add(self.reg.c),
                0x82 => self.opc_add(self.reg.d),
                0x83 => self.opc_add(self.reg.e),
                0x84 => self.opc_add(self.reg.h),
                0x85 => self.opc_add(self.reg.l),
                0x86 => {
                    let hl = self.reg.get_hl();
                    let hl_v = self.memory.borrow().get(hl);
                    self.opc_add(hl_v);
                }
                0x87 => self.opc_add(self.reg.a),
                _ => {}
            },
            // ADD A D8
            0xC6 => {
                let d8 = self.imm();
                self.opc_add(d8);
            }
            // ADC A r
            0x88..=0x8F => match opcode {
                0x88 => self.opc_adc(self.reg.b),
                0x89 => self.opc_adc(self.reg.c),
                0x8A => self.opc_adc(self.reg.d),
                0x8B => self.opc_adc(self.reg.e),
                0x8C => self.opc_adc(self.reg.h),
                0x8D => self.opc_adc(self.reg.l),
                0x8E => {
                    let hl = self.reg.get_hl();
                    let hl_v = self.memory.borrow().get(hl);
                    self.opc_adc(hl_v);
                }
                0x8F => self.opc_adc(self.reg.a),
                _ => {}
            },
            // ADC A D8
            0xCE => {
                let d8 = self.imm();
                self.opc_adc(d8);
            }
            // PREFIX CB
            0xCB => {
                // TODO
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
            0xCD => {
                let a16 = self.imm_word();
                let pc = self.reg.pc;
                self.stack_push(pc);
                self.reg.pc = a16;
            }
            // CALL f,nn
            0xC4 | 0xCC | 0xD4 | 0xDC => {
                is_jump = match opcode {
                    0xC4 => !self.reg.get_flag(Z),
                    0xCC => self.reg.get_flag(Z),
                    0xD4 => !self.reg.get_flag(C),
                    0xDC => self.reg.get_flag(C),
                    _ => {
                        panic!("CALL f,nn. But cond?")
                    }
                };
                let a16 = self.imm_word();
                if is_jump {
                    let pc = self.reg.pc;
                    self.stack_push(pc);
                    self.reg.pc = a16;
                }
            }
            // CP R
            0xB8..=0xBF => match opcode {
                0xB8 => self.opc_cp(self.reg.b),
                0xB9 => self.opc_cp(self.reg.c),
                0xBA => self.opc_cp(self.reg.d),
                0xBB => self.opc_cp(self.reg.e),
                0xBC => self.opc_cp(self.reg.h),
                0xBD => self.opc_cp(self.reg.l),
                0xBE => {
                    let hl = self.reg.get_hl();
                    let hl_v = self.memory.borrow().get(hl);
                    self.opc_cp(hl_v);
                }
                0xBF => self.opc_cp(self.reg.a),
                _ => {}
            },
            // CP d8
            0xFE => {
                let d8 = self.imm();
                self.opc_cp(d8);
            }
            // DEC r
            0x05 | 0x3D | 0x0D | 0x15 | 0x1D | 0x25 | 0x2D | 0x35 => match opcode {
                0x3D => self.reg.a = self.opc_dec(self.reg.a),
                0x05 => self.reg.b = self.opc_dec(self.reg.b),
                0x0D => self.reg.c = self.opc_dec(self.reg.c),
                0x15 => self.reg.d = self.opc_dec(self.reg.d),
                0x1D => self.reg.e = self.opc_dec(self.reg.e),
                0x25 => self.reg.h = self.opc_dec(self.reg.h),
                0x2D => self.reg.l = self.opc_dec(self.reg.l),
                0x35 => {
                    let hl = self.reg.get_hl();
                    let hl_v = self.memory.borrow().get(hl);
                    let new_hl_v = self.opc_dec(hl_v);
                    self.memory.borrow_mut().set(hl, new_hl_v);
                }
                _ => {}
            },
            // DEC rr
            0x0B => {
                let bc = self.reg.get_bc();
                self.reg.set_bc(bc.wrapping_sub(1));
            }
            0x1B => {
                let de = self.reg.get_de();
                self.reg.set_de(de.wrapping_sub(1));
            }
            0x2B => {
                let hl = self.reg.get_hl();
                self.reg.set_hl(hl.wrapping_sub(1));
            }
            0x3B => {
                let sp = self.reg.sp;
                self.reg.sp = sp.wrapping_sub(1);
            }
            // INC r
            0x3C | 0x0C | 0x04 | 0x14 | 0x24 | 0x1C | 0x2C | 0x34 => match opcode {
                0x3C => self.reg.a = self.opc_inc(self.reg.a),
                0x04 => self.reg.b = self.opc_inc(self.reg.b),
                0x0C => self.reg.c = self.opc_inc(self.reg.c),
                0x14 => self.reg.d = self.opc_inc(self.reg.d),
                0x1C => self.reg.e = self.opc_inc(self.reg.e),
                0x24 => self.reg.h = self.opc_inc(self.reg.h),
                0x2C => self.reg.l = self.opc_inc(self.reg.l),
                0x34 => {
                    let hl = self.reg.get_hl();
                    let hl_v = self.memory.borrow().get(hl);
                    let new_hl_v = self.opc_inc(hl_v);
                    self.memory.borrow_mut().set(hl, new_hl_v);
                }
                _ => {}
            },
            // INC rr
            0x03 | 0x23 | 0x13 | 0x33 => match opcode {
                0x03 => self.reg.set_bc(self.reg.get_bc() + 1),
                0x23 => self.reg.set_hl(self.reg.get_hl() + 1),
                0x13 => self.reg.set_de(self.reg.get_de() + 1),
                0x33 => self.reg.sp = self.reg.sp + 1,
                _ => {}
            },
            // JR r8
            0x18 => {
                let r8 = self.imm() as i8;
                self.opc_jr(r8);
            }
            // JR cond,address
            0x20 | 0x28 | 0x30 | 0x38 => {
                is_jump = match opcode {
                    0x20 => !self.reg.get_flag(Z),
                    0x28 => self.reg.get_flag(Z),
                    0x30 => !self.reg.get_flag(C),
                    0x38 => self.reg.get_flag(C),
                    _ => {
                        panic!("JR cond,address. But cond?")
                    }
                };
                let r8 = self.imm() as i8;
                if is_jump {
                    self.opc_jr(r8);
                }
            }
            // JP NN
            0xC3 => {
                let r16 = self.imm_word();
                self.opc_jp(r16);
            }
            // JP cond,address
            0xC2 | 0xCA | 0xD2 | 0xDA => {
                is_jump = match opcode {
                    0xC2 => !self.reg.get_flag(Z),
                    0xCA => self.reg.get_flag(Z),
                    0xD2 => !self.reg.get_flag(C),
                    0xDA => self.reg.get_flag(C),
                    _ => {
                        panic!("JP cond,address. But cond?")
                    }
                };
                let r16 = self.imm_word();
                if is_jump {
                    self.opc_jp(r16);
                }
            }
            // JP (HL)
            0xE9 => {
                let hl = self.reg.get_hl();
                self.opc_jp(hl);
            }
            // LD (address), A
            0x02 | 0x12 | 0x77 | 0xEA | 0xE0 | 0xE2 => {
                let address = match opcode {
                    0x02 => self.reg.get_bc(),
                    0x12 => self.reg.get_de(),
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
            // LD A,(address)
            0x1a => {
                let de = self.reg.get_de();
                let de_v = self.memory.borrow_mut().get(de);
                self.reg.a = de_v;
            }
            0x0a => {
                let bc = self.reg.get_bc();
                let bc_v = self.memory.borrow_mut().get(bc);
                self.reg.a = bc_v;
            }
            0xF0 => {
                let address = 0xFF00 | (self.imm() as u16);
                let value = self.memory.borrow_mut().get(address);
                self.reg.a = value;
            }
            // LD r,r
            0x40..=0x4f | 0x50..=0x5f | 0x60..=0x6f | 0x70..=0x75 | 0x77..=0x7F => match opcode {
                0x40 => self.reg.b = self.reg.b,
                0x41 => self.reg.b = self.reg.c,
                0x42 => self.reg.b = self.reg.d,
                0x43 => self.reg.b = self.reg.e,
                0x44 => self.reg.b = self.reg.h,
                0x45 => self.reg.b = self.reg.l,
                0x46 => {
                    let hl = self.reg.get_hl();
                    self.reg.b = self.memory.borrow().get(hl);
                }
                0x47 => self.reg.b = self.reg.a,
                0x48 => self.reg.c = self.reg.b,
                0x49 => self.reg.c = self.reg.c,
                0x4A => self.reg.c = self.reg.d,
                0x4B => self.reg.c = self.reg.e,
                0x4C => self.reg.c = self.reg.h,
                0x4D => self.reg.c = self.reg.l,
                0x4E => {
                    let hl = self.reg.get_hl();
                    self.reg.c = self.memory.borrow().get(hl);
                }
                0x4F => self.reg.c = self.reg.a,
                0x50 => self.reg.d = self.reg.b,
                0x51 => self.reg.d = self.reg.c,
                0x52 => self.reg.d = self.reg.d,
                0x53 => self.reg.d = self.reg.e,
                0x54 => self.reg.d = self.reg.h,
                0x55 => self.reg.d = self.reg.l,
                0x56 => {
                    let hl = self.reg.get_hl();
                    self.reg.d = self.memory.borrow().get(hl);
                }
                0x57 => self.reg.d = self.reg.a,
                0x58 => self.reg.e = self.reg.b,
                0x59 => self.reg.e = self.reg.c,
                0x5A => self.reg.e = self.reg.d,
                0x5B => self.reg.e = self.reg.e,
                0x5C => self.reg.e = self.reg.h,
                0x5D => self.reg.e = self.reg.l,
                0x5E => {
                    let hl = self.reg.get_hl();
                    self.reg.e = self.memory.borrow().get(hl);
                }
                0x5F => self.reg.e = self.reg.a,
                0x60 => self.reg.h = self.reg.b,
                0x61 => self.reg.h = self.reg.c,
                0x62 => self.reg.h = self.reg.d,
                0x63 => self.reg.h = self.reg.e,
                0x64 => self.reg.h = self.reg.h,
                0x65 => self.reg.h = self.reg.l,
                0x66 => {
                    let hl = self.reg.get_hl();
                    self.reg.h = self.memory.borrow().get(hl);
                }
                0x67 => self.reg.h = self.reg.a,
                0x68 => self.reg.l = self.reg.b,
                0x69 => self.reg.l = self.reg.c,
                0x6A => self.reg.l = self.reg.d,
                0x6B => self.reg.l = self.reg.e,
                0x6C => self.reg.l = self.reg.h,
                0x6D => self.reg.l = self.reg.l,
                0x6E => {
                    let hl = self.reg.get_hl();
                    self.reg.l = self.memory.borrow().get(hl);
                }
                0x6F => self.reg.l = self.reg.a,
                0x70 => {
                    let hl = self.reg.get_hl();
                    self.memory.borrow_mut().set(hl, self.reg.b);
                }
                0x71 => {
                    let hl = self.reg.get_hl();
                    self.memory.borrow_mut().set(hl, self.reg.c);
                }
                0x72 => {
                    let hl = self.reg.get_hl();
                    self.memory.borrow_mut().set(hl, self.reg.d);
                }
                0x73 => {
                    let hl = self.reg.get_hl();
                    self.memory.borrow_mut().set(hl, self.reg.e);
                }
                0x74 => {
                    let hl = self.reg.get_hl();
                    self.memory.borrow_mut().set(hl, self.reg.h);
                }
                0x75 => {
                    let hl = self.reg.get_hl();
                    self.memory.borrow_mut().set(hl, self.reg.l);
                }
                0x77 => self.reg.h = self.reg.a,
                0x78 => self.reg.a = self.reg.b,
                0x79 => self.reg.a = self.reg.c,
                0x7A => self.reg.a = self.reg.d,
                0x7B => self.reg.a = self.reg.e,
                0x7C => self.reg.a = self.reg.h,
                0x7D => self.reg.a = self.reg.l,
                0x7E => {
                    let hl = self.reg.get_hl();
                    self.reg.a = self.memory.borrow().get(hl);
                }
                0x7F => self.reg.a = self.reg.a,
                _ => {}
            },
            // LD r,d8
            0x3E | 0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 => {
                let d8 = self.imm();
                match opcode {
                    0x3E => self.reg.a = d8,
                    0x06 => self.reg.b = d8,
                    0x0E => self.reg.c = d8,
                    0x16 => self.reg.d = d8,
                    0x1E => self.reg.e = d8,
                    0x26 => self.reg.h = d8,
                    0x2E => self.reg.l = d8,
                    0x36 => {
                        let hl = self.reg.get_hl();
                        self.memory.borrow_mut().set(hl, d8);
                    }
                    _ => {}
                }
            }
            // LD rr,d16
            0x01 | 0x11 | 0x21 | 0x31 => {
                let d16 = self.imm_word();
                match opcode {
                    0x01 => self.reg.set_bc(d16),
                    0x11 => self.reg.set_de(d16),
                    0x21 => self.reg.set_hl(d16),
                    0x31 => self.reg.sp = d16,
                    _ => {}
                }
            }
            // LD (a16) SP
            0x08 => {
                let a16 = self.imm_word();
                self.memory.borrow_mut().set_word(a16, self.reg.sp);
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
            // LD A,(HL+)
            0x2A => {
                let hl = self.reg.get_hl();
                self.reg.a = self.memory.borrow().get(hl);
                self.reg.set_hl(hl.wrapping_add(1));
            }
            // LD A,(HL-)
            0x3A => {
                let hl = self.reg.get_hl();
                self.reg.a = self.memory.borrow().get(hl);
                self.reg.set_hl(hl.wrapping_sub(1));
            }
            // POP rr
            0xC1 => {
                let bc = self.stack_pop();
                self.reg.set_bc(bc);
            }
            0xD1 => {
                let de = self.stack_pop();
                self.reg.set_de(de);
            }
            0xE1 => {
                let hl = self.stack_pop();
                self.reg.set_hl(hl);
            }
            0xF1 => {
                let af = self.stack_pop();
                self.reg.set_af(af);
            }
            // PUSH nn
            0xC5 => {
                let bc = self.reg.get_bc();
                self.stack_push(bc);
            }
            0xD5 => {
                let de = self.reg.get_de();
                self.stack_push(de);
            }
            0xE5 => {
                let hl = self.reg.get_hl();
                self.stack_push(hl);
            }
            0xF5 => {
                let af = self.reg.get_af();
                self.stack_push(af);
            }
            // RET f
            0xC0 | 0xC8 | 0xD0 | 0xD8 => {
                is_jump = match opcode {
                    0xC0 => !self.reg.get_flag(Z),
                    0xC8 => self.reg.get_flag(Z),
                    0xD0 => !self.reg.get_flag(C),
                    0xD8 => self.reg.get_flag(C),
                    _ => {
                        panic!("RET f. But cond?")
                    }
                };
                let pc = self.stack_pop();
                if is_jump {
                    self.reg.pc = pc;
                }
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
            // RLCA
            0x07 => {
                self.reg.a = self.opc_rlc(self.reg.a);
                self.reg.set_flag(Z, false);
            }
            // RRA
            0x1F => {
                self.reg.a = self.opc_rr(self.reg.a);
                self.reg.set_flag(Z, false);
            }
            // RRCA
            0x0F => {
                self.reg.a = self.opc_rrc(self.reg.a);
                self.reg.set_flag(Z, false);
            }
            // SUB R
            0x90..=0x97 => match opcode {
                0x90 => self.opc_sub(self.reg.b),
                0x91 => self.opc_sub(self.reg.c),
                0x92 => self.opc_sub(self.reg.d),
                0x93 => self.opc_sub(self.reg.e),
                0x94 => self.opc_sub(self.reg.h),
                0x95 => self.opc_sub(self.reg.l),
                0x96 => {
                    let hl = self.reg.get_hl();
                    let hl_v = self.memory.borrow().get(hl);
                    self.opc_sub(hl_v);
                }
                0x97 => self.opc_sub(self.reg.a),
                _ => {}
            },
            // SBC R
            0x98..=0x9F => match opcode {
                0x98 => self.opc_sbc(self.reg.b),
                0x99 => self.opc_sbc(self.reg.c),
                0x9A => self.opc_sbc(self.reg.d),
                0x9B => self.opc_sbc(self.reg.e),
                0x9C => self.opc_sbc(self.reg.h),
                0x9D => self.opc_sbc(self.reg.l),
                0x9E => {
                    let hl = self.reg.get_hl();
                    let hl_v = self.memory.borrow().get(hl);
                    self.opc_sbc(hl_v);
                }
                0x9F => self.opc_sbc(self.reg.a),
                _ => {}
            },
            // AND R
            0xA0..=0xA7 => match opcode {
                0xA0 => self.opc_and(self.reg.b),
                0xA1 => self.opc_and(self.reg.c),
                0xA2 => self.opc_and(self.reg.d),
                0xA3 => self.opc_and(self.reg.e),
                0xA4 => self.opc_and(self.reg.h),
                0xA5 => self.opc_and(self.reg.l),
                0xA6 => {
                    let hl = self.reg.get_hl();
                    let hl_v = self.memory.borrow().get(hl);
                    self.opc_and(hl_v);
                }
                0xA7 => self.opc_and(self.reg.a),
                _ => {}
            },
            // XOR R
            0xA8..=0xAF => match opcode {
                0xA8 => self.opc_xor(self.reg.b),
                0xA9 => self.opc_xor(self.reg.c),
                0xAA => self.opc_xor(self.reg.d),
                0xAB => self.opc_xor(self.reg.e),
                0xAC => self.opc_xor(self.reg.h),
                0xAD => self.opc_xor(self.reg.l),
                0xAE => {
                    let hl = self.reg.get_hl();
                    let hl_v = self.memory.borrow().get(hl);
                    self.opc_xor(hl_v);
                }
                0xAF => self.opc_xor(self.reg.a),
                _ => {}
            },
            // OR R
            0xB0..=0xB7 => match opcode {
                0xB0 => self.opc_or(self.reg.b),
                0xB1 => self.opc_or(self.reg.c),
                0xB2 => self.opc_or(self.reg.d),
                0xB3 => self.opc_or(self.reg.e),
                0xB4 => self.opc_or(self.reg.h),
                0xB5 => self.opc_or(self.reg.l),
                0xB6 => {
                    let hl = self.reg.get_hl();
                    let hl_v = self.memory.borrow().get(hl);
                    self.opc_or(hl_v);
                }
                0xB7 => self.opc_or(self.reg.a),
                _ => {}
            },
            // STOP 0
            0x10 => {
                let _ = self.imm();
                // TODO
                // Halt CPU & LCD display until button pressed
            }
            // DDA
            0x27 => {
                let a = self.reg.a;
                let a_low = a & 0x0F;
                let a_high = a & 0xF0 >> 4;
                // carry from low 4 bit
                let h = self.reg.get_flag(H);
                // carry from high 4 bit
                let c = self.reg.get_flag(C);
                // pre opcode is sub
                let n = self.reg.get_flag(N);
                let mut result = a;
                let mut adjust: u8 = 0;
                if a_low > 9 || h {
                    adjust |= 0x06;
                }
                if a_high > 9 || c {
                    adjust |= 0x60;
                }
                if n {
                    result = result.wrapping_sub(adjust);
                } else {
                    result = result.wrapping_add(adjust);
                }
                self.reg.a = result;
                self.reg.set_flag(Z, self.reg.a == 0);
                self.reg.set_flag(H, false);
                // 这里C Flag应该是指高四位BCD是否进行了加减6的操作，虽然我是不明白意义何在
                self.reg.set_flag(C, a_high > 9 || c);
            }
            // CPL
            0x2F => {
                self.reg.a = self.reg.a ^ 0xFF;
                self.reg.set_flag(N, true);
                self.reg.set_flag(H, true);
            }
            // SCF
            0x37 => {
                self.reg.set_flag(C, true);
                self.reg.set_flag(N, false);
                self.reg.set_flag(H, false);
            }
            // CCF
            0x3F => {
                let c = self.reg.get_flag(C);
                self.reg.set_flag(C, !c);
                self.reg.set_flag(N, false);
                self.reg.set_flag(H, false);
            }
            // HALT
            0x76 => {
                // TODO
            }
            // RST n
            0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF => {
                let address = match opcode {
                    0xC7 => 0x00,
                    0xCF => 0x08,
                    0xD7 => 0x10,
                    0xDF => 0x18,
                    0xE7 => 0x20,
                    0xEF => 0x28,
                    0xF7 => 0x30,
                    0xFF => 0x38,
                    _ => panic!("RST unknown n"),
                };
                let pc = self.reg.pc;
                self.stack_push(pc);
                self.reg.pc = address;
            }
            0xD3 | 0xDB | 0xDD | 0xE3 | 0xE4 | 0xEB | 0xEC | 0xED | 0xF4 | 0xFC | 0xFD => {
                panic!("this opcode not exist {:02x}", opcode);
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
                0x30 => 1,
                0x38 => 1,
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
    fn opc_adc(&mut self, value: u8) {
        let a = self.reg.a;
        let c = self.reg.get_flag(C) as u8;
        let result = a.wrapping_add(value).wrapping_add(c);
        self.reg.set_flag(Z, result == 0);
        self.reg.set_flag(N, false);
        self.reg
            .set_flag(H, a & 0x0f + value & 0x0f + c & 0x0f > 0x0f);
        self.reg
            .set_flag(C, a as u16 + value as u16 + c as u16 > 0xff);
        self.reg.a = result;
    }
    fn opc_add_hl(&mut self, value: u16) {
        let hl = self.reg.get_hl();
        let result = hl.wrapping_add(value);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, hl & 0xFFF + value & 0xFFF > 0x0FFF);
        self.reg.set_flag(C, hl as u32 + value as u32 > 0xFFFF);
        self.reg.set_hl(result);
    }
    fn opc_xor(&mut self, n: u8) {
        self.reg.a ^= n;
        self.reg.set_flag(Z, self.reg.a == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, false);
        self.reg.set_flag(C, false);
    }
    fn opc_and(&mut self, n: u8) {
        self.reg.a &= n;
        self.reg.set_flag(Z, self.reg.a == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, true);
        self.reg.set_flag(C, false);
    }
    fn opc_or(&mut self, n: u8) {
        self.reg.a |= n;
        self.reg.set_flag(Z, self.reg.a == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, false);
        self.reg.set_flag(C, false);
    }
    fn opc_cp(&mut self, value: u8) {
        let a = self.reg.a;
        self.reg.set_flag(Z, a == value);
        self.reg.set_flag(N, true);
        self.reg.set_flag(H, a & 0x0f < value & 0x0f);
        self.reg.set_flag(C, a < value);
    }
    fn opc_jr(&mut self, r8: i8) {
        self.reg.pc = self.reg.pc.wrapping_add(r8 as u16);
    }
    fn opc_jp(&mut self, r16: u16) {
        self.reg.pc = r16;
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
    fn opc_rlc(&mut self, value: u8) -> u8 {
        let c = value >> 7 == 0x01;
        let result = value << 1 | u8::from(c);
        self.reg.set_flag(Z, result == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, false);
        self.reg.set_flag(C, c);
        result
    }
    fn opc_rr(&mut self, value: u8) -> u8 {
        let old_c = self.reg.get_flag(C);
        let c = value & 0x01 == 0x01;
        let result = u8::from(old_c) << 7 | value >> 1;
        self.reg.set_flag(Z, result == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, false);
        self.reg.set_flag(C, c);
        result
    }
    fn opc_rrc(&mut self, value: u8) -> u8 {
        let c = value & 0x01 == 0x01;
        let result = u8::from(c) << 7 | value >> 1;
        self.reg.set_flag(Z, result == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, false);
        self.reg.set_flag(C, c);
        result
    }
    fn opc_sub(&mut self, value: u8) {
        let a = self.reg.a;
        let r = a.wrapping_sub(value);
        self.reg.set_flag(Z, r == 0);
        self.reg.set_flag(N, true);
        self.reg.set_flag(H, a & 0x0F < value & 0x0F);
        self.reg.set_flag(C, (a as u16) < (value as u16));
        self.reg.a = r;
    }
    fn opc_sbc(&mut self, value: u8) {
        let a = self.reg.a;
        let c = self.reg.get_flag(C) as u8;
        let r = a.wrapping_sub(value).wrapping_sub(c);
        self.reg.set_flag(Z, r == 0);
        self.reg.set_flag(N, true);
        self.reg.set_flag(H, a & 0x0F < value & 0x0F + c & 0x0F);
        self.reg.set_flag(C, (a as u16) < value as u16 + c as u16);
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
