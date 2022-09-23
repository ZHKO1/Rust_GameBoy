use crate::cartridge::Stable;
use crate::cpu::{Cpu, Timer};
use crate::joypad::JoyPadKey;
use crate::mmu::Mmu;
use crate::ppu::PPU;
use std::{cell::RefCell, rc::Rc};

const WIDTH: usize = 160;
const HEIGHT: usize = 144;

pub struct GameBoy {
    pub mmu: Rc<RefCell<Mmu>>,
    ppu: PPU,
    cpu: Cpu,
    timer: Timer,
}

impl GameBoy {
    pub fn new(bios: Vec<u8>, rom: Vec<u8>) -> Self {
        let skip_boot = bios.is_empty();
        let mmu = Mmu::new(bios, rom);
        let rc_refcell_mmu = Rc::new(RefCell::new(mmu));
        let mut cpu = Cpu::new(rc_refcell_mmu.clone());
        if skip_boot {
            cpu.skip_bios();
        }
        let ppu = PPU::new(rc_refcell_mmu.clone());
        let timer = Timer::new(rc_refcell_mmu.clone());
        Self {
            mmu: rc_refcell_mmu.clone(),
            cpu,
            ppu,
            timer,
        }
    }
    pub fn trick(&mut self) {
        self.cpu.trick();
        self.timer.trick();
        self.ppu.trick();
    }
    pub fn get_frame_buffer(&self) -> &[u32; WIDTH * HEIGHT] {
        &self.ppu.frame_buffer
    }
    pub fn input(&mut self, key: JoyPadKey, is_pressed: bool) {
        self.mmu.borrow_mut().joypad.input(key, is_pressed);
    }
}

impl Stable for GameBoy {
    fn save_sav(&self) -> Vec<u8>{
        self.mmu.borrow().save_sav()  
    }
    fn load_sav(&mut self, ram: Vec<u8>){
        self.mmu.borrow_mut().load_sav(ram);        
    }
}