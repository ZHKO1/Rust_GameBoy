use crate::cpu::Cpu;
use crate::mmu::Mmu;
use crate::ppu::PPU;
use std::{cell::RefCell, path::Path, rc::Rc};

const WIDTH: usize = 160;
const HEIGHT: usize = 144;

pub struct GameBoy {
    mmu: Rc<RefCell<Mmu>>,
    ppu: PPU,
    cpu: Cpu,
}

impl GameBoy {
    pub fn new(bios_path: impl AsRef<Path>, rom_path: impl AsRef<Path>) -> Self {
        let skip_boot = bios_path.as_ref().to_str().unwrap().is_empty();
        let mmu = Mmu::new(bios_path, rom_path);
        let rc_refcell_mmu = Rc::new(RefCell::new(mmu));
        let mut cpu = Cpu::new(rc_refcell_mmu.clone());
        if skip_boot {
            cpu.skip_bios();
        }
        let ppu = PPU::new(rc_refcell_mmu.clone());
        Self {
            mmu: rc_refcell_mmu.clone(),
            cpu,
            ppu,
        }
    }
    pub fn trick(&mut self) {
        self.cpu.trick();
        self.ppu.trick();
    }
    pub fn get_frame_buffer(&self) -> &[u32; WIDTH * HEIGHT] {
        &self.ppu.frame_buffer
    }
}
