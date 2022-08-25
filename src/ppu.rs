use crate::memory::Memory;
use crate::ppu::PPU_STATUS::{DRAWING, HBLANK, OAMSCAN, VBLANK};
use std::{cell::RefCell, rc::Rc};

const WIDTH: usize = 256;
const HEIGHT: usize = 144;

pub enum PPU_STATUS {
    OAMSCAN = 2,
    DRAWING = 3,
    HBLANK = 0,
    VBLANK = 1,
}

pub struct PPU {
    ly: u32,
    cycles: u32,
    status: PPU_STATUS,
    memory: Rc<RefCell<dyn Memory>>,
    pub pixel_array: [u32; WIDTH * HEIGHT],
}
impl PPU {
    pub fn new(mmu: Rc<RefCell<dyn Memory>>) -> Self {
        PPU {
            ly: 0,
            cycles: 0,
            status: OAMSCAN,
            memory: mmu,
            pixel_array: [0; WIDTH * HEIGHT],
        }
    }
    pub fn trick(&mut self) {
        match self.status {
            OAMSCAN => {
                if self.cycles == 79 {
                    self.status = DRAWING;
                }
            }
            DRAWING => {
                if self.cycles == 251 {
                    self.status = HBLANK;
                }
            }
            HBLANK => {
                if self.cycles == 455 {
                    if self.ly == 143 {
                        self.status = VBLANK;
                    } else {
                        self.status = OAMSCAN;
                    }
                    self.ly += 1;
                }
            }
            VBLANK => {
                if self.cycles == 455 {
                    if self.ly == 153 {
                        self.status = OAMSCAN;
                        self.ly = 0;
                    }
                }
            }
        }
        self.cycles += 1;
    }
    pub fn get_ly(&self) {}
}
