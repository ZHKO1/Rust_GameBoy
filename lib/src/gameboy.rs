use crate::cartridge::{from_vecu8, Cartridge, Stable};
use crate::cpu::{Cpu, Timer};
use crate::gameboy_mode::GameBoyMode;
use crate::joypad::JoyPadKey;
use crate::mmu::Mmu;
use crate::ppu::PPU;
pub use crate::ppu::{HEIGHT, WIDTH};
use std::{cell::RefCell, rc::Rc};
/*
use std::fs::File;
use simplelog::*;
extern crate log;
extern crate simplelog;
*/

pub struct GameBoy {
    pub mmu: Rc<RefCell<Mmu>>,
    ppu: PPU,
    cpu: Cpu,
    timer: Timer,
}

impl GameBoy {
    pub fn new(bios: Vec<u8>, cartridge: Box<dyn Cartridge>) -> Self {
        /*
        CombinedLogger::init(vec![
              TermLogger::new(
                  LevelFilter::Warn,
                  Config::default(),
                  TerminalMode::Mixed,
                  ColorChoice::Auto,
              ),
              WriteLogger::new(
                  LevelFilter::Info,
                  Config::default(),
                  File::create("my_rust_binary.log").unwrap(),
              ),
          ])
          .unwrap();
          */
        let skip_bios = bios.is_empty();
        let gbc_flag = cartridge.gbc_flag();
        let mode = if gbc_flag {
            GameBoyMode::GBC
        } else {
            GameBoyMode::GB
        };
        let mmu = Mmu::new(mode, bios, cartridge);
        let rc_refcell_mmu = Rc::new(RefCell::new(mmu));
        let cpu = Cpu::new(rc_refcell_mmu.clone(), skip_bios);
        let ppu = PPU::new(rc_refcell_mmu.clone());
        let timer = Timer::new(mode, rc_refcell_mmu.clone());
        Self {
            mmu: rc_refcell_mmu.clone(),
            cpu,
            ppu,
            timer,
        }
    }
    pub fn trick(&mut self) -> bool {
        self.cpu.trick();
        self.timer.trick();
        let is_refresh = self.ppu.trick();
        is_refresh
    }
    pub fn flip(&mut self) -> bool {
        self.cpu.flip()
    }
    pub fn get_frame_buffer(&self) -> &[u32; WIDTH * HEIGHT] {
        &self.ppu.frame_buffer
    }
    pub fn input(&mut self, key: JoyPadKey, is_pressed: bool) {
        self.mmu.borrow_mut().joypad.input(key, is_pressed);
    }
    pub fn is_gbc(cartridge: Box<dyn Cartridge>) -> bool {
        cartridge.gbc_flag()
    }
    pub fn get_cartridge(rom: Vec<u8>) -> Box<dyn Cartridge> {
        from_vecu8(rom)
    }
}

impl Stable for GameBoy {
    fn save_sav(&self) -> Vec<u8> {
        self.mmu.borrow().save_sav()
    }
    fn load_sav(&mut self, ram: Vec<u8>) {
        self.mmu.borrow_mut().load_sav(ram);
    }
}
