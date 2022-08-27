use crate::memory::Memory;
use crate::ppu::PpuStatus::{DRAWING, HBLANK, OAMSCAN, VBLANK};
use crate::util::check_bit;
use std::collections::VecDeque;
use std::{cell::RefCell, rc::Rc};

const WIDTH: usize = 256;
const HEIGHT: usize = 144;

enum Color {
    WHITE = 0xE0F8D0,
    LIGHT_GRAY = 0x88C070,
    DARK_GRAY = 0x346856,
    BLACK_GRAY = 0x081820,
}

pub enum PpuStatus {
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
    fn get(&self, ly: u16) -> Vec<u8> {
        let lcdc = self.mmu.borrow().get(0xff40);
        // let window_title_map_area = check_bit(lcdc, 6);
        let bg_window_tile_area = check_bit(lcdc, 4);
        let bg_tile_map_area = check_bit(lcdc, 3);
        let (bg_map_start, bg_map_end): (u16, u16) = match bg_tile_map_area {
            true => (0x9C00, 0x9FFF),
            false => (0x9800, 0x9BFF),
        };
        let mut result: Vec<u8> = vec![];
        for draw_x in 0..WIDTH {
            let bg_map_x = draw_x as u16 / 8;
            let bg_map_y = ly / 8;
            let bg_map_index = bg_map_x + bg_map_y * 32;
            let bg_map_byte = self.mmu.borrow().get(bg_map_start + bg_map_index);
            let tile_index: u16 = if bg_window_tile_area {
                0x8000 + bg_map_byte as u16 * 8 * 2
            } else {
                (0x9000 as i32 + (bg_map_byte as i8) as i32 * 8 * 2) as u16
            };
            let tile_pixel_x = draw_x % 8;
            let tile_pixel_y = ly % 8;
            let tile_byte_low = self.mmu.borrow().get(tile_index + tile_pixel_y * 2);
            let tile_byte_high = self.mmu.borrow().get(tile_index + tile_pixel_y * 2 + 1);
            let pixel_bit = 8 - tile_pixel_x - 1;
            let pixel_low = tile_byte_low & (1 << pixel_bit) == (1 << pixel_bit);
            let pixel_high = tile_byte_high & (1 << pixel_bit) == (1 << pixel_bit);
            let pixel = (pixel_low as u8) + (pixel_high as u8) << 1;
            result.push(pixel);
        }
        return result;
    }
}

pub struct PPU {
    ly: u16,
    cycles: u32,
    status: PpuStatus,
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
                // println!("OAMSCAN");
                if self.cycles == 79 {
                    self.status = DRAWING;
                }
                self.cycles += 1;
            }
            DRAWING => {
                // println!("DRAWING");
                if self.cycles == 80 {
                    let array = self.fetcher.get(self.ly);
                    for (width, pixel) in array.iter().enumerate() {
                        self.pixel_array[(self.ly as usize * WIDTH + width) as usize] =
                            self.get_pixel_color(*pixel);
                    }
                }
                if self.cycles == 251 {
                    self.status = HBLANK;
                }
                self.cycles += 1;
            }
            HBLANK => {
                // println!("HBLANK");
                if self.cycles == 455 {
                    if self.ly == 143 {
                        self.status = VBLANK;
                    } else {
                        self.status = OAMSCAN;
                    }
                    self.ly += 1;
                    self.cycles = 0;
                } else {
                    self.cycles += 1;
                }
            }
            VBLANK => {
                // println!("VBLANK");
                if self.cycles == 455 {
                    if self.ly == 153 {
                        self.status = OAMSCAN;
                        self.ly = 0;
                    } else {
                        self.ly += 1;
                    }
                    self.cycles = 0;
                } else {
                    self.cycles += 1;
                }
            }
        }
    }
    fn get_pixel_color(&self, index: u8) -> u32 {
        let bg_palette = self.mmu.borrow().get(0xFF47);
        let color_value = match index {
            0 => bg_palette & 0b11,
            1 => (bg_palette & 0b1100) >> 2,
            2 => (bg_palette & 0b110000) >> 4,
            3 => (bg_palette & 0b11000000) >> 6,
            _ => {
                panic!("color index is out of range")
            }
        };
        match color_value {
            0 => Color::WHITE as u32,
            1 => Color::LIGHT_GRAY as u32,
            2 => Color::DARK_GRAY as u32,
            3 => Color::BLACK_GRAY as u32,
            _ => {
                panic!("color_value is out of range");
            }
        }
    }
    pub fn get_lcdc(&self) {
        let lcdc = self.mmu.borrow().get(0xFF40);
    }
}
