use crate::cartridge::{from_vecu8, Stable};
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

struct JoyPadEvent {
    key: JoyPadKey,
    is_pressed: bool,
}
impl JoyPadEvent {
    fn new(key: JoyPadKey, is_pressed: bool) -> Self {
        Self { key, is_pressed }
    }
}

struct JoyPadInputs(Vec<JoyPadEvent>);

impl JoyPadInputs {
    pub fn new() -> Self {
        let up = JoyPadEvent::new(JoyPadKey::Up, false);
        let down = JoyPadEvent::new(JoyPadKey::Down, false);
        let left = JoyPadEvent::new(JoyPadKey::Left, false);
        let right = JoyPadEvent::new(JoyPadKey::Right, false);
        let a = JoyPadEvent::new(JoyPadKey::A, false);
        let b = JoyPadEvent::new(JoyPadKey::B, false);
        let select = JoyPadEvent::new(JoyPadKey::Select, false);
        let start = JoyPadEvent::new(JoyPadKey::Start, false);
        Self(vec![up, down, left, right, a, b, select, start])
    }
    pub fn input(&mut self, key: JoyPadKey, is_pressed: bool) {
        let index = match key {
            JoyPadKey::Up => 0,
            JoyPadKey::Down => 1,
            JoyPadKey::Left => 2,
            JoyPadKey::Right => 3,
            JoyPadKey::A => 4,
            JoyPadKey::B => 5,
            JoyPadKey::Select => 6,
            JoyPadKey::Start => 7,
        };
        self.0[index].is_pressed = is_pressed;
    }
}

pub struct GameBoy {
    pub mmu: Rc<RefCell<Mmu>>,
    ppu: PPU,
    cpu: Cpu,
    timer: Timer,
    inputs: JoyPadInputs,
}

impl GameBoy {
    pub fn new(bios: Vec<u8>, rom: Vec<u8>) -> Self {
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
        let cartridge = from_vecu8(rom);
        let gbc_flag = cartridge.gbc_flag();
        let mode = if gbc_flag { GameBoyMode::GBC } else { GameBoyMode::GB };
        let mmu = Mmu::new(mode, bios, cartridge);
        let rc_refcell_mmu = Rc::new(RefCell::new(mmu));
        let cpu = Cpu::new(mode, rc_refcell_mmu.clone(), skip_bios);
        let ppu = PPU::new(mode, rc_refcell_mmu.clone());
        let timer = Timer::new(mode, rc_refcell_mmu.clone());
        Self {
            mmu: rc_refcell_mmu.clone(),
            cpu,
            ppu,
            timer,
            inputs: JoyPadInputs::new(),
        }
    }
    pub fn trick(&mut self) -> bool {
        self.cpu.trick();
        self.timer.trick();
        let is_refresh = self.ppu.trick();
        for event in &self.inputs.0 {
            self.mmu
                .borrow_mut()
                .joypad
                .input(event.key.clone(), event.is_pressed);
        }
        is_refresh
    }
    pub fn get_frame_buffer(&self) -> &[u32; WIDTH * HEIGHT] {
        &self.ppu.frame_buffer
    }
    pub fn input(&mut self, key: JoyPadKey, is_pressed: bool) {
        self.inputs.input(key, is_pressed)
    }
    pub fn is_cartridge_gbc(&self) -> bool {
        self.mmu.borrow().cartridge.gbc_flag()
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
