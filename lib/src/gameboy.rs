use crate::cartridge::{from_vecu8, Cartridge, Stable};
use crate::cpu::{Cpu, Timer};
use crate::gameboy_mode::GameBoyMode;
use crate::joypad::JoyPadKey;
use crate::mmu::{CartridgeProxy, Mmu};
use crate::ppu::PPU;
pub use crate::ppu::{HEIGHT, WIDTH};
use bincode::Error;
use std::ops::Deref;
use std::{cell::RefCell, rc::Rc};
/*
use std::fs::File;
use simplelog::*;
extern crate log;
extern crate simplelog;
*/

#[derive(Default)]
pub struct GameBoyStatus {
    other_status: Vec<u8>,
    mmu_status: Vec<u8>,
    ram: Vec<u8>,
    cartridge_status: Vec<u8>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct GameBoy {
    #[serde(skip)]
    pub mmu: Rc<RefCell<Mmu>>,
    #[serde(skip)]
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

    pub fn load(
        &self,
        status: &GameBoyStatus,
        cartridge: Box<dyn Cartridge>,
    ) -> Result<Self, Error> {
        let mut gameboy: Self = bincode::deserialize_from(status.other_status.as_slice())?;
        let mut mmu: Mmu = bincode::deserialize_from(status.mmu_status.as_slice())?;
        mmu.cartridge = CartridgeProxy { content: cartridge };
        let rc_refcell_mmu = Rc::new(RefCell::new(mmu));
        gameboy.mmu = rc_refcell_mmu.clone();
        gameboy.cpu.mmu = rc_refcell_mmu.clone();
        gameboy.ppu = PPU::new(rc_refcell_mmu.clone());
        gameboy.timer.mmu = rc_refcell_mmu.clone();
        gameboy.load_sav(status.ram.clone());
        gameboy
            .mmu
            .borrow_mut()
            .cartridge
            .content
            .load_status(status.cartridge_status.clone());
        Ok(gameboy)
    }

    pub fn save(&self) -> Result<GameBoyStatus, Error> {
        let mut other_data = Vec::new();
        bincode::serialize_into(&mut other_data, &self)?;
        let mut mmu_data = Vec::new();
        bincode::serialize_into(&mut mmu_data, self.mmu.borrow().deref())?;
        let ram = self.save_sav();
        let cartridge_status = self.mmu.borrow().cartridge.content.save_status();
        Ok(GameBoyStatus {
            other_status: other_data,
            mmu_status: mmu_data,
            ram,
            cartridge_status,
        })
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
