use crate::cartridge::{Cartridge, Stable};
use crate::gameboy_mode::GameBoyMode;
use crate::interrupt::Interrupt;
use crate::joypad::JoyPad;
use crate::memory::Memory;
use crate::ppu::PpuMmu;
use std::cell::RefCell;
use std::rc::Rc;

struct MemoryBlock {
    memory: [u8; 0xFFFF - 0x8000 + 1],
    start: u16,
    end: u16,
}

impl MemoryBlock {
    fn new() -> Self {
        let start = 0x8000;
        let end = 0xFFFF;
        MemoryBlock {
            start,
            end,
            memory: [0; 0xFFFF - 0x8000 + 1],
        }
    }
}

impl Memory for MemoryBlock {
    fn get(&self, index: u16) -> u8 {
        if index < self.start || index > self.end {
            return 0xFF;
        }
        self.memory[(index - self.start) as usize]
    }
    fn set(&mut self, index: u16, value: u8) {
        if index < self.start || index > self.end {
            return;
        }
        self.memory[(index - self.start) as usize] = value;
    }
}

struct HDMA {
    source_high: u8,
    source_low: u8,
    destination_high: u8,
    destination_low: u8,
    length_mode_start: u8,
}
impl HDMA {
    fn new() -> Self {
        Self {
            source_high: 0xFF,
            source_low: 0xFF,
            destination_high: 0xFF,
            destination_low: 0xFF,
            length_mode_start: 0xFF,
        }
    }
    fn get_source_destination_length(&self) -> (u16, u16, u16) {
        let source = u16::from_be_bytes([self.source_high, self.source_low]) & 0xFFF0;
        let destination =
            (u16::from_be_bytes([self.destination_high, self.destination_low]) | 0x8000) & 0xFFF0;
        let length = ((self.length_mode_start & 0x7F) as u16 + 1) << 4;
        (source, destination, length)
    }
}
impl Memory for HDMA {
    fn get(&self, index: u16) -> u8 {
        match index {
            0xFF51 => self.source_high,
            0xFF52 => self.source_low,
            0xFF53 => self.destination_high,
            0xFF54 => self.destination_low,
            0xFF55 => self.length_mode_start,
            _ => panic!("HDMA get index not in 0xFF51~0xFF55"),
        }
    }
    fn set(&mut self, index: u16, value: u8) {
        match index {
            0xFF51 => {
                self.source_high = value;
            }
            0xFF52 => {
                self.source_low = value;
            }
            0xFF53 => {
                self.destination_high = value;
            }
            0xFF54 => {
                self.destination_low = value;
            }
            0xFF55 => {
                self.length_mode_start = value;
            }
            _ => {}
        }
    }
}

struct WRAM {
    bank: u8,
    memory: [u8; (0xDFFF - 0xD000 + 1) * 8],
}
impl WRAM {
    fn new() -> Self {
        Self {
            bank: 1,
            memory: [0; (0xDFFF - 0xD000 + 1) * 8],
        }
    }
}
impl Memory for WRAM {
    fn get(&self, index: u16) -> u8 {
        match index {
            0xFF70 => self.bank | 0xF8,
            0xC000..=0xCFFF => self.memory[index as usize - 0xC000],
            0xD000..=0xDFFF => {
                let bank = if self.bank == 0 { 1 } else { self.bank } as usize;
                let memory_index = (index as usize - 0xD000) + (0xDFFF - 0xD000 + 1) * bank;
                self.memory[memory_index]
            }
            _ => {
                panic!("Out Range of WRAM")
            }
        }
    }
    fn set(&mut self, index: u16, value: u8) {
        match index {
            0xFF70 => {
                self.bank = value & 0x07;
            }
            0xC000..=0xCFFF => {
                self.memory[index as usize - 0xC000] = value;
            }
            0xD000..=0xDFFF => {
                let bank = if self.bank == 0 { 1 } else { self.bank } as usize;
                let memory_index = (index as usize - 0xD000) + (0xDFFF - 0xD000 + 1) * bank;
                self.memory[memory_index] = value;
            }
            _ => {
                panic!("Out Range of WRAM")
            }
        }
    }
}

pub struct Speed {
    pub current_speed: bool,
    prepare_switch: bool,
    memory: u8,
}
impl Speed {
    fn new() -> Self {
        Self {
            current_speed: false,
            prepare_switch: false,
            memory: 0xFF,
        }
    }
    pub fn switch(&mut self) {
        if self.prepare_switch {
            self.current_speed = !self.current_speed;
            self.prepare_switch = false;
        }
    }
}
impl Memory for Speed {
    fn get(&self, index: u16) -> u8 {
        assert_eq!(index, 0xFF4D);
        let current_speed = if self.current_speed { 1 } else { 0 } << 7;
        let prepare_switch = if self.prepare_switch { 0x01 } else { 0x00 };
        (self.memory & 0x7E) | current_speed | prepare_switch
    }
    fn set(&mut self, index: u16, value: u8) {
        assert_eq!(index, 0xFF4D);
        self.memory = value;
        self.prepare_switch = (self.memory & 0x01) == 0x01;
    }
}

pub struct Mmu {
    pub mode: GameBoyMode,
    boot: [u8; 0x100],
    pub cartridge: Box<dyn Cartridge>,
    pub joypad: JoyPad,
    pub ppu: PpuMmu,
    interrupt: Rc<RefCell<Interrupt>>,
    wram: WRAM,
    other: MemoryBlock,
    hdma: HDMA,
    pub speed: Speed,
    pub log_msg: Vec<u8>,
}

impl Mmu {
    pub fn new(mode: GameBoyMode, bios: Vec<u8>, cartridge: Box<dyn Cartridge>) -> Self {
        let other = MemoryBlock::new();
        let mut boot = [0; 0x100];
        let skip_boot = bios.is_empty();
        if !skip_boot {
            boot.copy_from_slice(&bios[..0x100]);
        }
        let interrupt = Interrupt::new();
        let rc_refcell_interrupt = Rc::new(RefCell::new(interrupt));
        let joypad = JoyPad::new(rc_refcell_interrupt.clone());
        let ppu = PpuMmu::new(mode, rc_refcell_interrupt.clone());
        let hdma = HDMA::new();
        let speed = Speed::new();
        let wram = WRAM::new();
        let mut mmu = Self {
            mode,
            boot,
            cartridge,
            other,
            joypad,
            ppu,
            wram,
            interrupt: rc_refcell_interrupt,
            hdma,
            speed,
            log_msg: vec![],
        };
        if skip_boot {
            mmu.set(0xFF50, 1);
        }
        mmu.set(0xFF40, 0b11100011);
        mmu
    }
    pub fn is_boot(&self) -> bool {
        let v = self.get(0xFF50);
        v == 0
    }
    fn dma(&mut self, value: u8) {
        if value > 0xdf {
            return;
        }
        for index in 0x00..=0x9F {
            let source = ((value as u16) << 8) | index;
            let destination = 0xFE00 | index;
            let source_v = self.get(source);
            self.set(destination, source_v);
        }
    }
    fn hdma(&mut self) {
        let (source, destination, length) = self.hdma.get_source_destination_length();
        if source > destination {
            return;
        }
        for index in 0x0000..=length {
            let s = source + index;
            let d = destination + index;
            let s_v = self.get(s);
            self.set(d, s_v);
        }
        self.hdma.set(0xFF55, 0xFF);
    }
    pub fn bind_event(&mut self, index: u16, value: u8) {
        match index {
            0xFF02 => {
                if value == 0x81 {
                    let v = self.get(0xFF01);
                    self.log_msg.push(v);
                }
            }
            0xFF46 => {
                self.dma(value);
            }
            0xFF55 => {
                if self.mode == GameBoyMode::GBC {
                    self.hdma();
                }
            }
            _ => {}
        };
    }
}
impl Memory for Mmu {
    fn get(&self, index: u16) -> u8 {
        match index {
            0x0000..=0x00FF => {
                if self.is_boot() {
                    self.boot[index as usize]
                } else {
                    self.cartridge.get(index)
                }
            }
            0x0100..=0x7FFF => self.cartridge.get(index),
            0x8000..=0x9FFF => self.ppu.get(index),
            0xA000..=0xBFFF => self.cartridge.get(index),
            0xFE00..=0xFE9F => self.ppu.get(index),
            0xFF00 => self.joypad.get(index),
            0xFF0F => self.interrupt.borrow().get(index),
            0xFF40..=0xFF45 | 0xFF47..=0xFF4B | 0xFF4F => self.ppu.get(index),
            0xFF4D => {
                if self.mode == GameBoyMode::GBC {
                    self.speed.get(index)
                } else {
                    self.other.get(index)
                }
            }
            0xFF51..=0xFF55 => {
                if self.mode == GameBoyMode::GBC {
                    self.hdma.get(index)
                } else {
                    self.other.get(index)
                }
            }
            0xFF68 | 0xFF69 | 0xFF6A | 0xFF6B => {
                if self.mode == GameBoyMode::GBC {
                    self.ppu.get(index)
                } else {
                    self.other.get(index)
                }
            }
            0xFF70 | 0xC000..=0xDFFF => {
                if self.mode == GameBoyMode::GBC {
                    self.wram.get(index)
                } else {
                    self.other.get(index)
                }
            }
            0xFF74 => {
                if self.mode == GameBoyMode::GBC {
                    self.other.get(index)
                } else {
                    0xFF
                }
            }
            _ => self.other.get(index),
        }
    }
    fn set(&mut self, index: u16, value: u8) {
        match index {
            0x0000..=0x7FFF => self.cartridge.set(index, value),
            0x8000..=0x9FFF => self.ppu.set(index, value),
            0xA000..=0xBFFF => self.cartridge.set(index, value),
            0xFE00..=0xFE9F => self.ppu.set(index, value),
            0xFF00 => self.joypad.set(index, value),
            0xFF0F => self.interrupt.borrow_mut().set(index, value),
            0xFF40..=0xFF45 | 0xFF47..=0xFF4B | 0xFF4F => self.ppu.set(index, value),
            0xFF4D => {
                if self.mode == GameBoyMode::GBC {
                    self.speed.set(index, value)
                } else {
                    self.other.set(index, value)
                }
            }
            0xFF51..=0xFF55 => {
                if self.mode == GameBoyMode::GBC {
                    self.hdma.set(index, value)
                } else {
                    self.other.set(index, value)
                }
            }
            0xFF68 | 0xFF69 | 0xFF6A | 0xFF6B => {
                if self.mode == GameBoyMode::GBC {
                    self.ppu.set(index, value)
                } else {
                    self.other.set(index, value)
                }
            }
            0xFF70 | 0xC000..=0xDFFF => {
                if self.mode == GameBoyMode::GBC {
                    self.wram.set(index, value)
                } else {
                    self.other.set(index, value)
                }
            }
            0xFF74 => {
                if self.mode == GameBoyMode::GBC {
                    self.other.set(index, value)
                } else {
                }
            }
            _ => self.other.set(index, value),
        }
        self.bind_event(index, value);
    }
}

impl Stable for Mmu {
    fn save_sav(&self) -> Vec<u8> {
        self.cartridge.save_sav()
    }
    fn load_sav(&mut self, ram: Vec<u8>) {
        self.cartridge.load_sav(ram);
    }
}
