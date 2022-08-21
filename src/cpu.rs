use std::io::{self, Read};

struct Registers {
    AF: u16,
    BC: u16,
    DE: u16,
    HL: u16,
    SP: u16,
    PC: u16,
}

pub struct Cpu {
    reg: Registers,
    cycles: usize,
    // memory: Vec<u8>,
    programs: Vec<u8>,
}

impl Cpu {
    pub fn new(path: &str) -> Self {
        let reg = Registers {
            AF: 0,
            BC: 0,
            DE: 0,
            HL: 0,
            SP: 0,
            PC: 0,
        };
        Cpu {
            reg: reg,
            programs: get_boot_rom(path).unwrap(),
            cycles: 0,
        }
    }
    pub fn run(&mut self) {
        println!("{:x}", self.programs.len());
        while usize::from(self.reg.PC) < self.programs.len() {
            self.step();
        }
    }
    pub fn step(&mut self) {
        let opcode = self.get_opcode();
        self.run_opcode(opcode);
    }
    fn get_opcode(&self) -> u8 {
        self.get_u8(self.reg.PC)
    }
    fn get_u8(&self, pc: u16) -> u8 {
        self.programs[pc as usize]
    }
    fn run_opcode(&mut self, opcode: u8) {
        println!("{:02x}", opcode);
        match opcode {
            0x31 => {
                let cycle = 12;
                let d16: u16 = self.get_u8(self.reg.PC + 1) as u16
                    + ((self.get_u8(self.reg.PC + 2) as u16) << 8);
                println!("{:04x}", d16);
                self.reg.PC += 3;
            }
            _ => {
                println!("unkown opcode");
                self.reg.PC += 1;
            }
        }
    }
}

fn get_boot_rom(path: &str) -> io::Result<Vec<u8>> {
    let mut rom = vec![];
    let mut file = std::fs::File::open(path)?;
    file.read_to_end(&mut rom)?;
    Ok(rom)
}

#[test]
fn test() {
    let mut cpu = Cpu::new("./tests/DMG_ROM.bin");
    cpu.run();
}
