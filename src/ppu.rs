use crate::memory::Memory;
use crate::ppu::PPU_STATUS::{DRAWING, HBLANK, OAMSCAN, VBLANK};
use crate::util::check_bit;
use std::collections::VecDeque;
use std::{cell::RefCell, rc::Rc};

const WIDTH: usize = 256;
const HEIGHT: usize = 144;

pub enum PPU_STATUS {
    OAMSCAN = 2,
    DRAWING = 3,
    HBLANK = 0,
    VBLANK = 1,
}

struct Pixel {}

struct Fetcher {
    index: u16,
    mmu: Rc<RefCell<dyn Memory>>,
    // result: Vec<u8>,
}
impl Fetcher {
    fn new(mmu: Rc<RefCell<dyn Memory>>) -> Self {
        Fetcher { index: 0, mmu }
    }
    fn get(&self, ly: u32) {
        let lcdc = self.mmu.borrow().get(0xff40);
        // let window_title_map_area = check_bit(lcdc, 6);
        let bg_window_tile_area = check_bit(lcdc, 4);
        let bg_tile_map_area = check_bit(lcdc, 3);
        let (bg_map_start, bg_map_end): (u16, u16) = match bg_tile_map_area {
            true => (0x9C00, 0x9FFF),
            false => (0x9800, 0x9BFF),
        };
        for draw_x in 0..=WIDTH {
            /*
                32 * 32 Title
             */
        }
    }
}
// 获取地图，
pub struct PPU {
    ly: u32,
    cycles: u32,
    status: PPU_STATUS,
    fetcher: Fetcher,
    fifo: VecDeque<u8>,
    mmu: Rc<RefCell<dyn Memory>>,
    pub pixel_array: [u32; WIDTH * HEIGHT],
}
impl PPU {
    pub fn new(mmu: Rc<RefCell<dyn Memory>>) -> Self {
        let fetcher = Fetcher::new(mmu.clone());
        PPU {
            ly: 0,
            cycles: 0,
            status: OAMSCAN,
            mmu: mmu,
            fetcher,
            fifo: VecDeque::new(),
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
                // 80开始
                // TODO 绘制每一行的像素
                // 直接绘制背景，不过高度从0画到144
                if self.cycles == 80 {
                    self.fetcher.get(self.ly);
                }
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
    pub fn get_lcdc(&self) {
        let lcdc = self.mmu.borrow().get(0xFF40);
    }
}
