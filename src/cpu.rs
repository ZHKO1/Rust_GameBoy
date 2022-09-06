use crate::memory::Memory;
use crate::util::{check_bit, u16_from_2u8, u8u8_from_u16};
// use log::info;
use std::{cell::RefCell, rc::Rc};
use Flag::{C, H, N, Z};

//  0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f
const OP_CYCLES: [u32; 256] = [
    1, 3, 2, 2, 1, 1, 2, 1, 5, 2, 2, 2, 1, 1, 2, 1, // 0
    1, 3, 2, 2, 1, 1, 2, 1, 3, 2, 2, 2, 1, 1, 2, 1, // 1
    2, 3, 2, 2, 1, 1, 2, 1, 2, 2, 2, 2, 1, 1, 2, 1, // 2
    2, 3, 2, 2, 3, 3, 3, 1, 2, 2, 2, 2, 1, 1, 2, 1, // 3
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 4
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 5
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 6
    2, 2, 2, 2, 2, 2, 1, 2, 1, 1, 1, 1, 1, 1, 2, 1, // 7
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
        self.f = value_low & 0xF0;
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
        (self.f & (flag as u8)) != 0
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
    cycles: u32,
    cur_opcode_cycles: u32,
    ime: bool, // true:enable; false:disable
    ime_next: Option<bool>,
    is_halted: bool,
    reg: Registers,
    mmu: Rc<RefCell<dyn Memory>>,
}

impl Cpu {
    pub fn new(mmu: Rc<RefCell<dyn Memory>>) -> Self {
        let reg = Registers::new();
        Cpu {
            reg,
            mmu: mmu,
            cycles: 0,
            cur_opcode_cycles: 0,
            ime: false,
            ime_next: None,
            is_halted: false,
        }
    }
    pub fn skip_bios(&mut self) {
        self.reg.pc = 0x0100;
        self.reg.sp = 0xFFFE;
    }
    fn interrupt_check_pending(&mut self) -> u8 {
        let m_ie = self.mmu.borrow().get(0xFFFF);
        let m_if = self.mmu.borrow().get(0xFF0F);
        let m_r = m_ie & m_if;
        m_r & 0x1F
    }
    fn interrupt_handle(&mut self, m_r: u8) -> u32 {
        for index in 0..=4 {
            let bit = check_bit(m_r, index);
            if bit {
                let address: u16 = match index {
                    0 => 0x0040,
                    1 => 0x0048,
                    2 => 0x0050,
                    3 => 0x0058,
                    4 => 0x0060,
                    _ => panic!("index is out of range"),
                };
                self.ime = false;
                let m_if = self.mmu.borrow().get(0xFF0F);
                let m_if = self.opc_res(index, m_if);
                self.mmu.borrow_mut().set(0xFF0F, m_if);

                let a16 = address;
                let pc = self.reg.pc;
                self.stack_push(pc);
                self.reg.pc = a16;
                return 5;
            }
        }
        return 0;
    }
    fn step(&mut self) -> u32 {
        let interrupts = self.interrupt_check_pending();
        if self.is_halted {
            self.step_halt(interrupts) * 4
        } else {
            self.step_run(interrupts) * 4
        }
    }
    fn step_halt(&mut self, interrupts: u8) -> u32 {
        if self.ime {
            if interrupts > 0 {
                self.is_halted = false;
                self.interrupt_handle(interrupts)
            } else {
                1
            }
        } else {
            if interrupts > 0 {
                self.is_halted = false;
                1
            } else {
                1
            }
        }
    }
    fn step_run(&mut self, interrupts: u8) -> u32 {
        let ime_next = self.ime_next.clone();
        let mut cycles = if self.ime && interrupts > 0 {
            self.interrupt_handle(interrupts)
        } else {
            0
        };
        if cycles == 0 {
            let opcode = self.imm();
            cycles = self.run_opcode(opcode);
            if let Some(ime) = ime_next {
                self.ime = ime;
                self.ime_next = None;
            }
        };
        cycles
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
            panic!("trick error!")
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
        // info!("{:02x}  PC:{:04x}", opcode, self.reg.pc - 1);
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
                    let hl_v = self.mmu.borrow().get(hl);
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
            // ADD SP,r8
            0xE8 => {
                let sp = self.reg.sp;
                let r8 = self.imm() as i8;
                let value = (r8 as i16) as u16;
                self.reg.set_flag(Z, false);
                self.reg.set_flag(N, false);
                self.reg
                    .set_flag(H, (sp & 0x000f) + (value & 0x000f) > 0x000f);
                self.reg
                    .set_flag(C, (sp & 0x00ff) + (value & 0x00ff) > 0x00ff);
                self.reg.sp = sp.wrapping_add(value);
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
                    let hl_v = self.mmu.borrow().get(hl);
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
                cb_opcode = self.imm();
                match cb_opcode {
                    0x00 => self.reg.b = self.opc_rlc(self.reg.b),
                    0x01 => self.reg.c = self.opc_rlc(self.reg.c),
                    0x02 => self.reg.d = self.opc_rlc(self.reg.d),
                    0x03 => self.reg.e = self.opc_rlc(self.reg.e),
                    0x04 => self.reg.h = self.opc_rlc(self.reg.h),
                    0x05 => self.reg.l = self.opc_rlc(self.reg.l),
                    0x06 => {
                        let hl = self.reg.get_hl();
                        let hl_v = self.mmu.borrow().get(hl);
                        let r = self.opc_rlc(hl_v);
                        self.mmu.borrow_mut().set(hl, r);
                    }
                    0x07 => self.reg.a = self.opc_rlc(self.reg.a),

                    0x08 => self.reg.b = self.opc_rrc(self.reg.b),
                    0x09 => self.reg.c = self.opc_rrc(self.reg.c),
                    0x0A => self.reg.d = self.opc_rrc(self.reg.d),
                    0x0B => self.reg.e = self.opc_rrc(self.reg.e),
                    0x0C => self.reg.h = self.opc_rrc(self.reg.h),
                    0x0D => self.reg.l = self.opc_rrc(self.reg.l),
                    0x0E => {
                        let hl = self.reg.get_hl();
                        let hl_v = self.mmu.borrow().get(hl);
                        let r = self.opc_rrc(hl_v);
                        self.mmu.borrow_mut().set(hl, r);
                    }
                    0x0F => self.reg.a = self.opc_rrc(self.reg.a),

                    0x10 => self.reg.b = self.opc_rl(self.reg.b),
                    0x11 => self.reg.c = self.opc_rl(self.reg.c),
                    0x12 => self.reg.d = self.opc_rl(self.reg.d),
                    0x13 => self.reg.e = self.opc_rl(self.reg.e),
                    0x14 => self.reg.h = self.opc_rl(self.reg.h),
                    0x15 => self.reg.l = self.opc_rl(self.reg.l),
                    0x16 => {
                        let hl = self.reg.get_hl();
                        let hl_v = self.mmu.borrow().get(hl);
                        let r = self.opc_rl(hl_v);
                        self.mmu.borrow_mut().set(hl, r);
                    }
                    0x17 => self.reg.a = self.opc_rl(self.reg.a),

                    0x18 => self.reg.b = self.opc_rr(self.reg.b),
                    0x19 => self.reg.c = self.opc_rr(self.reg.c),
                    0x1A => self.reg.d = self.opc_rr(self.reg.d),
                    0x1B => self.reg.e = self.opc_rr(self.reg.e),
                    0x1C => self.reg.h = self.opc_rr(self.reg.h),
                    0x1D => self.reg.l = self.opc_rr(self.reg.l),
                    0x1E => {
                        let hl = self.reg.get_hl();
                        let hl_v = self.mmu.borrow().get(hl);
                        let r = self.opc_rr(hl_v);
                        self.mmu.borrow_mut().set(hl, r);
                    }
                    0x1F => self.reg.a = self.opc_rr(self.reg.a),

                    0x20 => self.reg.b = self.opc_sla(self.reg.b),
                    0x21 => self.reg.c = self.opc_sla(self.reg.c),
                    0x22 => self.reg.d = self.opc_sla(self.reg.d),
                    0x23 => self.reg.e = self.opc_sla(self.reg.e),
                    0x24 => self.reg.h = self.opc_sla(self.reg.h),
                    0x25 => self.reg.l = self.opc_sla(self.reg.l),
                    0x26 => {
                        let hl = self.reg.get_hl();
                        let hl_v = self.mmu.borrow().get(hl);
                        let r = self.opc_sla(hl_v);
                        self.mmu.borrow_mut().set(hl, r);
                    }
                    0x27 => self.reg.a = self.opc_sla(self.reg.a),

                    0x28 => self.reg.b = self.opc_sra(self.reg.b),
                    0x29 => self.reg.c = self.opc_sra(self.reg.c),
                    0x2A => self.reg.d = self.opc_sra(self.reg.d),
                    0x2B => self.reg.e = self.opc_sra(self.reg.e),
                    0x2C => self.reg.h = self.opc_sra(self.reg.h),
                    0x2D => self.reg.l = self.opc_sra(self.reg.l),
                    0x2E => {
                        let hl = self.reg.get_hl();
                        let hl_v = self.mmu.borrow().get(hl);
                        let r = self.opc_sra(hl_v);
                        self.mmu.borrow_mut().set(hl, r);
                    }
                    0x2F => self.reg.a = self.opc_sra(self.reg.a),

                    0x30 => self.reg.b = self.opc_swap(self.reg.b),
                    0x31 => self.reg.c = self.opc_swap(self.reg.c),
                    0x32 => self.reg.d = self.opc_swap(self.reg.d),
                    0x33 => self.reg.e = self.opc_swap(self.reg.e),
                    0x34 => self.reg.h = self.opc_swap(self.reg.h),
                    0x35 => self.reg.l = self.opc_swap(self.reg.l),
                    0x36 => {
                        let hl = self.reg.get_hl();
                        let hl_v = self.mmu.borrow().get(hl);
                        let r = self.opc_swap(hl_v);
                        self.mmu.borrow_mut().set(hl, r);
                    }
                    0x37 => self.reg.a = self.opc_swap(self.reg.a),

                    0x38 => self.reg.b = self.opc_srl(self.reg.b),
                    0x39 => self.reg.c = self.opc_srl(self.reg.c),
                    0x3A => self.reg.d = self.opc_srl(self.reg.d),
                    0x3B => self.reg.e = self.opc_srl(self.reg.e),
                    0x3C => self.reg.h = self.opc_srl(self.reg.h),
                    0x3D => self.reg.l = self.opc_srl(self.reg.l),
                    0x3E => {
                        let hl = self.reg.get_hl();
                        let hl_v = self.mmu.borrow().get(hl);
                        let r = self.opc_srl(hl_v);
                        self.mmu.borrow_mut().set(hl, r);
                    }
                    0x3F => self.reg.a = self.opc_srl(self.reg.a),

                    0x40 => self.opc_bit(0, self.reg.b),
                    0x41 => self.opc_bit(0, self.reg.c),
                    0x42 => self.opc_bit(0, self.reg.d),
                    0x43 => self.opc_bit(0, self.reg.e),
                    0x44 => self.opc_bit(0, self.reg.h),
                    0x45 => self.opc_bit(0, self.reg.l),
                    0x46 => {
                        let hl = self.reg.get_hl();
                        let hl_v = self.mmu.borrow().get(hl);
                        self.opc_bit(0, hl_v);
                    }
                    0x47 => self.opc_bit(0, self.reg.a),

                    0x48 => self.opc_bit(1, self.reg.b),
                    0x49 => self.opc_bit(1, self.reg.c),
                    0x4A => self.opc_bit(1, self.reg.d),
                    0x4B => self.opc_bit(1, self.reg.e),
                    0x4C => self.opc_bit(1, self.reg.h),
                    0x4D => self.opc_bit(1, self.reg.l),
                    0x4E => {
                        let hl = self.reg.get_hl();
                        let hl_v = self.mmu.borrow().get(hl);
                        self.opc_bit(1, hl_v);
                    }
                    0x4F => self.opc_bit(1, self.reg.a),

                    0x50 => self.opc_bit(2, self.reg.b),
                    0x51 => self.opc_bit(2, self.reg.c),
                    0x52 => self.opc_bit(2, self.reg.d),
                    0x53 => self.opc_bit(2, self.reg.e),
                    0x54 => self.opc_bit(2, self.reg.h),
                    0x55 => self.opc_bit(2, self.reg.l),
                    0x56 => {
                        let hl = self.reg.get_hl();
                        let hl_v = self.mmu.borrow().get(hl);
                        self.opc_bit(2, hl_v);
                    }
                    0x57 => self.opc_bit(2, self.reg.a),

                    0x58 => self.opc_bit(3, self.reg.b),
                    0x59 => self.opc_bit(3, self.reg.c),
                    0x5A => self.opc_bit(3, self.reg.d),
                    0x5B => self.opc_bit(3, self.reg.e),
                    0x5C => self.opc_bit(3, self.reg.h),
                    0x5D => self.opc_bit(3, self.reg.l),
                    0x5E => {
                        let hl = self.reg.get_hl();
                        let hl_v = self.mmu.borrow().get(hl);
                        self.opc_bit(3, hl_v);
                    }
                    0x5F => self.opc_bit(3, self.reg.a),

                    0x60 => self.opc_bit(4, self.reg.b),
                    0x61 => self.opc_bit(4, self.reg.c),
                    0x62 => self.opc_bit(4, self.reg.d),
                    0x63 => self.opc_bit(4, self.reg.e),
                    0x64 => self.opc_bit(4, self.reg.h),
                    0x65 => self.opc_bit(4, self.reg.l),
                    0x66 => {
                        let hl = self.reg.get_hl();
                        let hl_v = self.mmu.borrow().get(hl);
                        self.opc_bit(4, hl_v);
                    }
                    0x67 => self.opc_bit(4, self.reg.a),

                    0x68 => self.opc_bit(5, self.reg.b),
                    0x69 => self.opc_bit(5, self.reg.c),
                    0x6A => self.opc_bit(5, self.reg.d),
                    0x6B => self.opc_bit(5, self.reg.e),
                    0x6C => self.opc_bit(5, self.reg.h),
                    0x6D => self.opc_bit(5, self.reg.l),
                    0x6E => {
                        let hl = self.reg.get_hl();
                        let hl_v = self.mmu.borrow().get(hl);
                        self.opc_bit(5, hl_v);
                    }
                    0x6F => self.opc_bit(5, self.reg.a),

                    0x70 => self.opc_bit(6, self.reg.b),
                    0x71 => self.opc_bit(6, self.reg.c),
                    0x72 => self.opc_bit(6, self.reg.d),
                    0x73 => self.opc_bit(6, self.reg.e),
                    0x74 => self.opc_bit(6, self.reg.h),
                    0x75 => self.opc_bit(6, self.reg.l),
                    0x76 => {
                        let hl = self.reg.get_hl();
                        let hl_v = self.mmu.borrow().get(hl);
                        self.opc_bit(6, hl_v);
                    }
                    0x77 => self.opc_bit(6, self.reg.a),

                    0x78 => self.opc_bit(7, self.reg.b),
                    0x79 => self.opc_bit(7, self.reg.c),
                    0x7A => self.opc_bit(7, self.reg.d),
                    0x7B => self.opc_bit(7, self.reg.e),
                    0x7C => self.opc_bit(7, self.reg.h),
                    0x7D => self.opc_bit(7, self.reg.l),
                    0x7E => {
                        let hl = self.reg.get_hl();
                        let hl_v = self.mmu.borrow().get(hl);
                        self.opc_bit(7, hl_v);
                    }
                    0x7F => self.opc_bit(7, self.reg.a),

                    0x80 => self.reg.b = self.opc_res(0, self.reg.b),
                    0x81 => self.reg.c = self.opc_res(0, self.reg.c),
                    0x82 => self.reg.d = self.opc_res(0, self.reg.d),
                    0x83 => self.reg.e = self.opc_res(0, self.reg.e),
                    0x84 => self.reg.h = self.opc_res(0, self.reg.h),
                    0x85 => self.reg.l = self.opc_res(0, self.reg.l),
                    0x86 => {
                        let hl = self.reg.get_hl();
                        let hl_v = self.mmu.borrow().get(hl);
                        let r = self.opc_res(0, hl_v);
                        self.mmu.borrow_mut().set(hl, r);
                    }
                    0x87 => self.reg.a = self.opc_res(0, self.reg.a),

                    0x88 => self.reg.b = self.opc_res(1, self.reg.b),
                    0x89 => self.reg.c = self.opc_res(1, self.reg.c),
                    0x8A => self.reg.d = self.opc_res(1, self.reg.d),
                    0x8B => self.reg.e = self.opc_res(1, self.reg.e),
                    0x8C => self.reg.h = self.opc_res(1, self.reg.h),
                    0x8D => self.reg.l = self.opc_res(1, self.reg.l),
                    0x8E => {
                        let hl = self.reg.get_hl();
                        let hl_v = self.mmu.borrow().get(hl);
                        let r = self.opc_res(1, hl_v);
                        self.mmu.borrow_mut().set(hl, r);
                    }
                    0x8F => self.reg.a = self.opc_res(1, self.reg.a),

                    0x90 => self.reg.b = self.opc_res(2, self.reg.b),
                    0x91 => self.reg.c = self.opc_res(2, self.reg.c),
                    0x92 => self.reg.d = self.opc_res(2, self.reg.d),
                    0x93 => self.reg.e = self.opc_res(2, self.reg.e),
                    0x94 => self.reg.h = self.opc_res(2, self.reg.h),
                    0x95 => self.reg.l = self.opc_res(2, self.reg.l),
                    0x96 => {
                        let hl = self.reg.get_hl();
                        let hl_v = self.mmu.borrow().get(hl);
                        let r = self.opc_res(2, hl_v);
                        self.mmu.borrow_mut().set(hl, r);
                    }
                    0x97 => self.reg.a = self.opc_res(2, self.reg.a),

                    0x98 => self.reg.b = self.opc_res(3, self.reg.b),
                    0x99 => self.reg.c = self.opc_res(3, self.reg.c),
                    0x9A => self.reg.d = self.opc_res(3, self.reg.d),
                    0x9B => self.reg.e = self.opc_res(3, self.reg.e),
                    0x9C => self.reg.h = self.opc_res(3, self.reg.h),
                    0x9D => self.reg.l = self.opc_res(3, self.reg.l),
                    0x9E => {
                        let hl = self.reg.get_hl();
                        let hl_v = self.mmu.borrow().get(hl);
                        let r = self.opc_res(3, hl_v);
                        self.mmu.borrow_mut().set(hl, r);
                    }
                    0x9F => self.reg.a = self.opc_res(3, self.reg.a),

                    0xA0 => self.reg.b = self.opc_res(4, self.reg.b),
                    0xA1 => self.reg.c = self.opc_res(4, self.reg.c),
                    0xA2 => self.reg.d = self.opc_res(4, self.reg.d),
                    0xA3 => self.reg.e = self.opc_res(4, self.reg.e),
                    0xA4 => self.reg.h = self.opc_res(4, self.reg.h),
                    0xA5 => self.reg.l = self.opc_res(4, self.reg.l),
                    0xA6 => {
                        let hl = self.reg.get_hl();
                        let hl_v = self.mmu.borrow().get(hl);
                        let r = self.opc_res(4, hl_v);
                        self.mmu.borrow_mut().set(hl, r);
                    }
                    0xA7 => self.reg.a = self.opc_res(4, self.reg.a),

                    0xA8 => self.reg.b = self.opc_res(5, self.reg.b),
                    0xA9 => self.reg.c = self.opc_res(5, self.reg.c),
                    0xAA => self.reg.d = self.opc_res(5, self.reg.d),
                    0xAB => self.reg.e = self.opc_res(5, self.reg.e),
                    0xAC => self.reg.h = self.opc_res(5, self.reg.h),
                    0xAD => self.reg.l = self.opc_res(5, self.reg.l),
                    0xAE => {
                        let hl = self.reg.get_hl();
                        let hl_v = self.mmu.borrow().get(hl);
                        let r = self.opc_res(5, hl_v);
                        self.mmu.borrow_mut().set(hl, r);
                    }
                    0xAF => self.reg.a = self.opc_res(5, self.reg.a),

                    0xB0 => self.reg.b = self.opc_res(6, self.reg.b),
                    0xB1 => self.reg.c = self.opc_res(6, self.reg.c),
                    0xB2 => self.reg.d = self.opc_res(6, self.reg.d),
                    0xB3 => self.reg.e = self.opc_res(6, self.reg.e),
                    0xB4 => self.reg.h = self.opc_res(6, self.reg.h),
                    0xB5 => self.reg.l = self.opc_res(6, self.reg.l),
                    0xB6 => {
                        let hl = self.reg.get_hl();
                        let hl_v = self.mmu.borrow().get(hl);
                        let r = self.opc_res(6, hl_v);
                        self.mmu.borrow_mut().set(hl, r);
                    }
                    0xB7 => self.reg.a = self.opc_res(6, self.reg.a),

                    0xB8 => self.reg.b = self.opc_res(7, self.reg.b),
                    0xB9 => self.reg.c = self.opc_res(7, self.reg.c),
                    0xBA => self.reg.d = self.opc_res(7, self.reg.d),
                    0xBB => self.reg.e = self.opc_res(7, self.reg.e),
                    0xBC => self.reg.h = self.opc_res(7, self.reg.h),
                    0xBD => self.reg.l = self.opc_res(7, self.reg.l),
                    0xBE => {
                        let hl = self.reg.get_hl();
                        let hl_v = self.mmu.borrow().get(hl);
                        let r = self.opc_res(7, hl_v);
                        self.mmu.borrow_mut().set(hl, r);
                    }
                    0xBF => self.reg.a = self.opc_res(7, self.reg.a),

                    0xC0 => self.reg.b = self.opc_set(0, self.reg.b),
                    0xC1 => self.reg.c = self.opc_set(0, self.reg.c),
                    0xC2 => self.reg.d = self.opc_set(0, self.reg.d),
                    0xC3 => self.reg.e = self.opc_set(0, self.reg.e),
                    0xC4 => self.reg.h = self.opc_set(0, self.reg.h),
                    0xC5 => self.reg.l = self.opc_set(0, self.reg.l),
                    0xC6 => {
                        let hl = self.reg.get_hl();
                        let hl_v = self.mmu.borrow().get(hl);
                        let r = self.opc_set(0, hl_v);
                        self.mmu.borrow_mut().set(hl, r);
                    }
                    0xC7 => self.reg.a = self.opc_set(0, self.reg.a),

                    0xC8 => self.reg.b = self.opc_set(1, self.reg.b),
                    0xC9 => self.reg.c = self.opc_set(1, self.reg.c),
                    0xCA => self.reg.d = self.opc_set(1, self.reg.d),
                    0xCB => self.reg.e = self.opc_set(1, self.reg.e),
                    0xCC => self.reg.h = self.opc_set(1, self.reg.h),
                    0xCD => self.reg.l = self.opc_set(1, self.reg.l),
                    0xCE => {
                        let hl = self.reg.get_hl();
                        let hl_v = self.mmu.borrow().get(hl);
                        let r = self.opc_set(1, hl_v);
                        self.mmu.borrow_mut().set(hl, r);
                    }
                    0xCF => self.reg.a = self.opc_set(1, self.reg.a),

                    0xD0 => self.reg.b = self.opc_set(2, self.reg.b),
                    0xD1 => self.reg.c = self.opc_set(2, self.reg.c),
                    0xD2 => self.reg.d = self.opc_set(2, self.reg.d),
                    0xD3 => self.reg.e = self.opc_set(2, self.reg.e),
                    0xD4 => self.reg.h = self.opc_set(2, self.reg.h),
                    0xD5 => self.reg.l = self.opc_set(2, self.reg.l),
                    0xD6 => {
                        let hl = self.reg.get_hl();
                        let hl_v = self.mmu.borrow().get(hl);
                        let r = self.opc_set(2, hl_v);
                        self.mmu.borrow_mut().set(hl, r);
                    }
                    0xD7 => self.reg.a = self.opc_set(2, self.reg.a),

                    0xD8 => self.reg.b = self.opc_set(3, self.reg.b),
                    0xD9 => self.reg.c = self.opc_set(3, self.reg.c),
                    0xDA => self.reg.d = self.opc_set(3, self.reg.d),
                    0xDB => self.reg.e = self.opc_set(3, self.reg.e),
                    0xDC => self.reg.h = self.opc_set(3, self.reg.h),
                    0xDD => self.reg.l = self.opc_set(3, self.reg.l),
                    0xDE => {
                        let hl = self.reg.get_hl();
                        let hl_v = self.mmu.borrow().get(hl);
                        let r = self.opc_set(3, hl_v);
                        self.mmu.borrow_mut().set(hl, r);
                    }
                    0xDF => self.reg.a = self.opc_set(3, self.reg.a),

                    0xE0 => self.reg.b = self.opc_set(4, self.reg.b),
                    0xE1 => self.reg.c = self.opc_set(4, self.reg.c),
                    0xE2 => self.reg.d = self.opc_set(4, self.reg.d),
                    0xE3 => self.reg.e = self.opc_set(4, self.reg.e),
                    0xE4 => self.reg.h = self.opc_set(4, self.reg.h),
                    0xE5 => self.reg.l = self.opc_set(4, self.reg.l),
                    0xE6 => {
                        let hl = self.reg.get_hl();
                        let hl_v = self.mmu.borrow().get(hl);
                        let r = self.opc_set(4, hl_v);
                        self.mmu.borrow_mut().set(hl, r);
                    }
                    0xE7 => self.reg.a = self.opc_set(4, self.reg.a),

                    0xE8 => self.reg.b = self.opc_set(5, self.reg.b),
                    0xE9 => self.reg.c = self.opc_set(5, self.reg.c),
                    0xEA => self.reg.d = self.opc_set(5, self.reg.d),
                    0xEB => self.reg.e = self.opc_set(5, self.reg.e),
                    0xEC => self.reg.h = self.opc_set(5, self.reg.h),
                    0xED => self.reg.l = self.opc_set(5, self.reg.l),
                    0xEE => {
                        let hl = self.reg.get_hl();
                        let hl_v = self.mmu.borrow().get(hl);
                        let r = self.opc_set(5, hl_v);
                        self.mmu.borrow_mut().set(hl, r);
                    }
                    0xEF => self.reg.a = self.opc_set(5, self.reg.a),

                    0xF0 => self.reg.b = self.opc_set(6, self.reg.b),
                    0xF1 => self.reg.c = self.opc_set(6, self.reg.c),
                    0xF2 => self.reg.d = self.opc_set(6, self.reg.d),
                    0xF3 => self.reg.e = self.opc_set(6, self.reg.e),
                    0xF4 => self.reg.h = self.opc_set(6, self.reg.h),
                    0xF5 => self.reg.l = self.opc_set(6, self.reg.l),
                    0xF6 => {
                        let hl = self.reg.get_hl();
                        let hl_v = self.mmu.borrow().get(hl);
                        let r = self.opc_set(6, hl_v);
                        self.mmu.borrow_mut().set(hl, r);
                    }
                    0xF7 => self.reg.a = self.opc_set(6, self.reg.a),

                    0xF8 => self.reg.b = self.opc_set(7, self.reg.b),
                    0xF9 => self.reg.c = self.opc_set(7, self.reg.c),
                    0xFA => self.reg.d = self.opc_set(7, self.reg.d),
                    0xFB => self.reg.e = self.opc_set(7, self.reg.e),
                    0xFC => self.reg.h = self.opc_set(7, self.reg.h),
                    0xFD => self.reg.l = self.opc_set(7, self.reg.l),
                    0xFE => {
                        let hl = self.reg.get_hl();
                        let hl_v = self.mmu.borrow().get(hl);
                        let r = self.opc_set(7, hl_v);
                        self.mmu.borrow_mut().set(hl, r);
                    }
                    0xFF => self.reg.a = self.opc_set(7, self.reg.a),
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
                    let hl_v = self.mmu.borrow().get(hl);
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
                    let hl_v = self.mmu.borrow().get(hl);
                    let new_hl_v = self.opc_dec(hl_v);
                    self.mmu.borrow_mut().set(hl, new_hl_v);
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
                    let hl_v = self.mmu.borrow().get(hl);
                    let new_hl_v = self.opc_inc(hl_v);
                    self.mmu.borrow_mut().set(hl, new_hl_v);
                }
                _ => {}
            },
            // INC rr
            0x03 | 0x23 | 0x13 | 0x33 => match opcode {
                0x03 => self.reg.set_bc(self.reg.get_bc().wrapping_add(1)),
                0x23 => self.reg.set_hl(self.reg.get_hl().wrapping_add(1)),
                0x13 => self.reg.set_de(self.reg.get_de().wrapping_add(1)),
                0x33 => self.reg.sp = self.reg.sp.wrapping_add(1),
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
                self.mmu.borrow_mut().set(address, a);
            }
            // LD A,(address)
            0x1a => {
                let de = self.reg.get_de();
                let de_v = self.mmu.borrow_mut().get(de);
                self.reg.a = de_v;
            }
            0xF2 => {
                let c = self.reg.c;
                let address = 0xFF00 | (c as u16);
                let address_v = self.mmu.borrow_mut().get(address);
                self.reg.a = address_v;
            }
            0x0a => {
                let bc = self.reg.get_bc();
                let bc_v = self.mmu.borrow_mut().get(bc);
                self.reg.a = bc_v;
            }
            0xF0 => {
                let address = 0xFF00 | (self.imm() as u16);
                let value = self.mmu.borrow_mut().get(address);
                self.reg.a = value;
            }
            0xFA => {
                let a16 = self.imm_word();
                let a16_v = self.mmu.borrow().get(a16);
                self.reg.a = a16_v;
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
                    self.reg.b = self.mmu.borrow().get(hl);
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
                    self.reg.c = self.mmu.borrow().get(hl);
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
                    self.reg.d = self.mmu.borrow().get(hl);
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
                    self.reg.e = self.mmu.borrow().get(hl);
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
                    self.reg.h = self.mmu.borrow().get(hl);
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
                    self.reg.l = self.mmu.borrow().get(hl);
                }
                0x6F => self.reg.l = self.reg.a,
                0x70 => {
                    let hl = self.reg.get_hl();
                    self.mmu.borrow_mut().set(hl, self.reg.b);
                }
                0x71 => {
                    let hl = self.reg.get_hl();
                    self.mmu.borrow_mut().set(hl, self.reg.c);
                }
                0x72 => {
                    let hl = self.reg.get_hl();
                    self.mmu.borrow_mut().set(hl, self.reg.d);
                }
                0x73 => {
                    let hl = self.reg.get_hl();
                    self.mmu.borrow_mut().set(hl, self.reg.e);
                }
                0x74 => {
                    let hl = self.reg.get_hl();
                    self.mmu.borrow_mut().set(hl, self.reg.h);
                }
                0x75 => {
                    let hl = self.reg.get_hl();
                    self.mmu.borrow_mut().set(hl, self.reg.l);
                }
                0x77 => {
                    let hl = self.reg.get_hl();
                    self.mmu.borrow_mut().set(hl, self.reg.a);
                }
                0x78 => self.reg.a = self.reg.b,
                0x79 => self.reg.a = self.reg.c,
                0x7A => self.reg.a = self.reg.d,
                0x7B => self.reg.a = self.reg.e,
                0x7C => self.reg.a = self.reg.h,
                0x7D => self.reg.a = self.reg.l,
                0x7E => {
                    let hl = self.reg.get_hl();
                    self.reg.a = self.mmu.borrow().get(hl);
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
                        self.mmu.borrow_mut().set(hl, d8);
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
            0xF9 => {
                let hl = self.reg.get_hl();
                self.reg.sp = hl;
            }
            // LD (a16) SP
            0x08 => {
                let a16 = self.imm_word();
                self.mmu.borrow_mut().set_word(a16, self.reg.sp);
            }
            // LD (HL+),A
            0x22 => {
                let hl = self.reg.get_hl();
                let a = self.reg.a;
                self.mmu.borrow_mut().set(hl, a);
                self.reg.set_hl(hl.wrapping_add(1));
            }
            // LD (HL-),A
            0x32 => {
                let hl = self.reg.get_hl();
                let a = self.reg.a;
                self.mmu.borrow_mut().set(hl, a);
                self.reg.set_hl(hl.wrapping_sub(1));
            }
            // LD A,(HL+)
            0x2A => {
                let hl = self.reg.get_hl();
                self.reg.a = self.mmu.borrow().get(hl);
                self.reg.set_hl(hl.wrapping_add(1));
            }
            // LD A,(HL-)
            0x3A => {
                let hl = self.reg.get_hl();
                self.reg.a = self.mmu.borrow().get(hl);
                self.reg.set_hl(hl.wrapping_sub(1));
            }
            // LD HL,SP+r8
            0xF8 => {
                let r8 = self.imm() as i8;
                let r8_ = (r8 as i16) as u16;
                let sp = self.reg.sp;
                let address = sp.wrapping_add(r8_);
                self.reg.set_hl(address);
                self.reg.set_flag(Z, false);
                self.reg.set_flag(N, false);
                self.reg.set_flag(H, (sp & 0x0f) + (r8_ & 0x0f) > 0x0f);
                self.reg.set_flag(C, (sp & 0xff) + (r8_ & 0xff) > 0xff);
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
                if is_jump {
                    let pc = self.stack_pop();
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
                    let hl_v = self.mmu.borrow().get(hl);
                    self.opc_sub(hl_v);
                }
                0x97 => self.opc_sub(self.reg.a),
                _ => {}
            },
            0xD6 => {
                let d8 = self.imm();
                self.opc_sub(d8);
            }
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
                    let hl_v = self.mmu.borrow().get(hl);
                    self.opc_sbc(hl_v);
                }
                0x9F => self.opc_sbc(self.reg.a),
                _ => {}
            },
            // SBC D8
            0xDE => {
                let d8 = self.imm();
                self.opc_sbc(d8);
            }
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
                    let hl_v = self.mmu.borrow().get(hl);
                    self.opc_and(hl_v);
                }
                0xA7 => self.opc_and(self.reg.a),
                _ => {}
            },
            0xE6 => {
                let d8 = self.imm();
                self.opc_and(d8);
            }
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
                    let hl_v = self.mmu.borrow().get(hl);
                    self.opc_xor(hl_v);
                }
                0xAF => self.opc_xor(self.reg.a),
                _ => {}
            },
            // XOR D8
            0xEE => {
                let d8 = self.imm();
                self.opc_xor(d8);
            }
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
                    let hl_v = self.mmu.borrow().get(hl);
                    self.opc_or(hl_v);
                }
                0xB7 => self.opc_or(self.reg.a),
                _ => {}
            },
            0xF6 => {
                let d8 = self.imm();
                self.opc_or(d8);
            }
            // STOP 0
            0x10 => {
                let _ = self.imm();
                // TODO
                // Halt CPU & LCD display until button pressed
            }
            // DAA
            0x27 => {
                // carry from low 4 bit
                let h = self.reg.get_flag(H);
                // carry from high 4 bit
                let c = self.reg.get_flag(C);
                // pre opcode is sub
                let n = self.reg.get_flag(N);

                let mut result = self.reg.a as u16;
                let mut adjust = 0 as u16;
                if n {
                    if h {
                        adjust |= 0x06;
                    }
                    if c {
                        adjust |= 0x60;
                    }
                    result = result.wrapping_sub(adjust);
                } else {
                    if (result > 0x99) || c {
                        adjust |= 0x60;
                    }
                    if ((result & 0x0F) > 9) || h {
                        adjust |= 0x06;
                    }
                    result = result.wrapping_add(adjust);
                }
                self.reg.a = (result & 0xFF) as u8;
                self.reg.set_flag(Z, self.reg.a == 0);
                self.reg.set_flag(H, false);
                // 这里C Flag 根据 高四位BCD是否进行了加减6的操作而更新
                self.reg.set_flag(C, adjust >= 0x60);
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
                self.is_halted = true;
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
            // RETI
            0xD9 => {
                let d16 = self.stack_pop();
                self.reg.pc = d16;
                self.ime = true;
            }
            // DI
            0xF3 => {
                self.ime_next = Some(false);
            }
            // EI
            0xFB => {
                self.ime_next = Some(true);
            }
            0xD3 | 0xDB | 0xDD | 0xE3 | 0xE4 | 0xEB | 0xEC | 0xED | 0xF4 | 0xFC | 0xFD => {
                panic!("this opcode not exist {:02x}", opcode);
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
        self.reg.set_flag(H, (a & 0x0f) + (value & 0x0f) > 0x0f);
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
            .set_flag(H, (a & 0x0f) + (value & 0x0f) + (c & 0x0f) > 0x0f);
        self.reg
            .set_flag(C, a as u16 + value as u16 + c as u16 > 0xff);
        self.reg.a = result;
    }
    fn opc_add_hl(&mut self, value: u16) {
        let hl = self.reg.get_hl();
        let result = hl.wrapping_add(value);
        self.reg.set_flag(N, false);
        self.reg
            .set_flag(H, (hl & 0xFFF) + (value & 0xFFF) > 0x0FFF);
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
        self.reg.set_flag(H, a & 0x0F < (value & 0x0F) + (c & 0x0F));
        self.reg.set_flag(C, (a as u16) < value as u16 + c as u16);
        self.reg.a = r;
    }
    fn opc_bit(&mut self, bit: u8, r: u8) {
        let z = ((1 << bit) & r) == 0;
        self.reg.set_flag(Z, z);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, true);
    }
    fn opc_sla(&mut self, n: u8) -> u8 {
        let r = n << 1;
        let c = n >> 7 == 0x01;
        self.reg.set_flag(Z, r == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, false);
        self.reg.set_flag(C, c);
        r
    }
    fn opc_sra(&mut self, n: u8) -> u8 {
        let c = n & 0x01 == 0x01;
        let msb = n >> 7;
        let r = n >> 1 | msb << 7;
        self.reg.set_flag(Z, r == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, false);
        self.reg.set_flag(C, c);
        r
    }
    fn opc_swap(&mut self, n: u8) -> u8 {
        let low = n & 0x0f;
        let high = (n & 0xf0) >> 4;
        let r = (low << 4) | high;
        self.reg.set_flag(Z, r == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, false);
        self.reg.set_flag(C, false);
        r
    }
    fn opc_srl(&mut self, n: u8) -> u8 {
        let r = n >> 1;
        let c = n & 0x01 == 0x01;
        self.reg.set_flag(Z, r == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, false);
        self.reg.set_flag(C, c);
        r
    }
    fn opc_res(&mut self, bit: u8, n: u8) -> u8 {
        let r = n & !(1 << bit);
        r
    }
    fn opc_set(&mut self, bit: u8, n: u8) -> u8 {
        let r = n | (1 << bit);
        r
    }
    fn stack_push(&mut self, value: u16) {
        self.reg.sp = self.reg.sp - 2;
        self.mmu.borrow_mut().set_word(self.reg.sp, value);
    }
    fn stack_pop(&mut self) -> u16 {
        let value = self.mmu.borrow_mut().get_word(self.reg.sp);
        // self.mmu.borrow_mut().set_word(self.reg.sp, 0);
        self.reg.sp = self.reg.sp + 2;
        value
    }
    fn imm(&mut self) -> u8 {
        let v = self.mmu.borrow().get(self.reg.pc);
        self.reg.pc += 1;
        v
    }
    fn imm_word(&mut self) -> u16 {
        let low = self.mmu.borrow().get(self.reg.pc);
        let high = self.mmu.borrow().get(self.reg.pc + 1);
        self.reg.pc += 2;
        u16_from_2u8(low, high)
    }
}
/*
FF04 Divider Register
 16384Hz 的频率 增长
 一秒执行16384Hz 2^14 HZ
 CPU 疫苗执行 2^22 Hz
FF05 TIMA
 FF07 的 频率 增长
 重置以FF06为准
FF06 TMA
FF07 TAC
 4096 Hz
 262144 Hz
 65536 Hz
 16384 Hz
*/
pub enum TimerClock {
    S0 = 1024,
    S1 = 16,
    S2 = 64,
    S3 = 256,
}
pub struct Timer {
    div_cycles: usize,
    tima_cycles: usize,
    mmu: Rc<RefCell<dyn Memory>>,
    cur_tac: TimerClock,
}
impl Timer {
    pub fn new(mmu: Rc<RefCell<dyn Memory>>) -> Self {
        Self {
            div_cycles: 0,
            tima_cycles: 0,
            mmu,
            cur_tac: TimerClock::S3,
        }
    }
    pub fn trick(&mut self) {
        self.trick_div();
        self.trick_tima();
    }
    fn trick_div(&mut self) {
        let time_clock = TimerClock::S3 as usize;
        if self.div_cycles == 0 {
            let mut div = self.mmu.borrow().get(0xFF04);
            div = div.wrapping_add(1);
            self.mmu.borrow_mut().set(0xFF04, div)
        }
        if self.div_cycles >= time_clock - 1 {
            self.div_cycles = 0;
        } else {
            self.div_cycles += 1;
        }
    }
    fn trick_tima(&mut self) {
        let tac = self.mmu.borrow().get(0xFF07);
        let time_enable = check_bit(tac, 2);
        if !time_enable {
            self.tima_cycles = 0;
            return;
        }
        let time_clock: usize = match self.cur_tac {
            TimerClock::S0 => 1024,
            TimerClock::S1 => 16,
            TimerClock::S2 => 64,
            TimerClock::S3 => 256,
        };
        if self.tima_cycles == 0 {
            let input_clock_select = match tac & 0x3 {
                0b00 => TimerClock::S0,
                0b01 => TimerClock::S1,
                0b10 => TimerClock::S2,
                0b11 => TimerClock::S3,
                _ => panic!("input_clock_select error"),
            };
            self.cur_tac = input_clock_select;
            let tma = self.mmu.borrow().get(0xFF06);
            let tima = self.mmu.borrow().get(0xFF05);
            match tima.checked_add(1) {
                Some(tima) => {
                    self.mmu.borrow_mut().set(0xFF05, tima);
                }
                None => {
                    self.mmu.borrow_mut().set(0xFF05, tma);
                    self.set_timer_interrupt();
                }
            };
        }
        if self.tima_cycles >= time_clock - 1 {
            self.tima_cycles = 0;
        } else {
            self.tima_cycles += 1;
        }
    }
    fn set_timer_interrupt(&mut self) {
        let d8 = self.mmu.borrow_mut().get(0xFF0F);
        self.mmu.borrow_mut().set(0xFF0F, d8 | 0b100);
    }
}
