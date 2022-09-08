use crate::memory::Memory;
use crate::ppu::FetcherStatus::{GetTile, GetTileDataHigh, GetTileDataLow};
use crate::ppu::PixelType::{Sprite, Window, BG};
use crate::ppu::PpuStatus::{Drawing, HBlank, OAMScan, VBlank};
use crate::util::check_bit;
use std::collections::VecDeque;
use std::{cell::RefCell, rc::Rc};

const WIDTH: usize = 160;
const HEIGHT: usize = 144;

enum Color {
    WHITE = 0xE0F8D0,
    LightGray = 0x88C070,
    DarkGray = 0x346856,
    BlackGray = 0x081820,
}

pub enum PpuStatus {
    OAMScan = 2,
    Drawing = 3,
    HBlank = 0,
    VBlank = 1,
}

#[derive(Clone, Copy)]
enum PixelType {
    BG,
    Window,
    Sprite,
}

#[derive(Clone, Copy)]
struct Pixel {
    ptype: PixelType,
    pvalue: u8,
}

enum FetcherStatus {
    GetTile,
    GetTileDataLow,
    GetTileDataHigh,
}

struct Fetcher {
    scan_x: u8,
    scan_y: u8,
    scx: u8,
    scy: u8,
    cycles: u16,
    mmu: Rc<RefCell<dyn Memory>>,
    status: FetcherStatus,
    tile_index: u16,
    tile_data_low: u8,
    tile_dada_high: u8,
    buffer: Vec<Pixel>,
}
impl Fetcher {
    fn new(mmu: Rc<RefCell<dyn Memory>>) -> Self {
        Fetcher {
            scan_x: 0,
            scan_y: 0,
            scx: 0,
            scy: 0,
            mmu,
            cycles: 0,
            status: GetTile,
            tile_index: 0,
            tile_data_low: 0,
            tile_dada_high: 0,
            buffer: Vec::new(),
        }
    }
    fn init(&mut self, x: u8, y: u8) {
        self.scan_x = x;
        self.scan_y = y;

        self.scx = 0;
        self.scy = 0;

        self.cycles = 0;
        self.status = GetTile;

        self.tile_index = 0;
        self.tile_data_low = 0;
        self.tile_dada_high = 0;

        self.buffer = Vec::new();
    }
    fn trick(&mut self) {
        if self.cycles == 1 {
            self.cycles = 0;
            return;
        }
        self.cycles += 1;
        match self.status {
            GetTile => {
                self.tile_index = self.get_tile();
                self.status = GetTileDataLow;
            }
            GetTileDataLow => {
                self.tile_data_low = self.get_tile_data_low();
                self.status = GetTileDataHigh;
            }
            GetTileDataHigh => {
                self.tile_dada_high = self.get_tile_data_high();
                self.buffer = self.get_buffer();
                self.status = GetTile;
            }
        }
    }
    fn get_tile(&mut self) -> u16 {
        let lcdc = self.mmu.borrow().get(0xFF40);
        self.scy = self.mmu.borrow().get(0xFF42);
        self.scx = self.mmu.borrow().get(0xFF43);
        let bg_window_tile_area = check_bit(lcdc, 4);
        let bg_tile_map_area = check_bit(lcdc, 3);
        let (bg_map_start, _): (u16, u16) = match bg_tile_map_area {
            true => (0x9C00, 0x9FFF),
            false => (0x9800, 0x9BFF),
        };
        let bg_map_x = (self.scan_x as u16 + self.scx as u16) % 256 / 8;
        let bg_map_y = (self.scan_y as u16 + self.scy as u16) % 256 / 8;
        let bg_map_index = bg_map_x + bg_map_y * 32;
        let bg_map_byte = self.mmu.borrow().get(bg_map_start + bg_map_index);
        let tile_index: u16 = if bg_window_tile_area {
            0x8000 + bg_map_byte as u16 * 8 * 2
        } else {
            (0x9000 as i32 + (bg_map_byte as i8) as i32 * 8 * 2) as u16
        };
        tile_index
    }
    fn get_tile_data_low(&self) -> u8 {
        let tile_index = self.tile_index;
        let tile_pixel_y = (self.scan_y as u16 + self.scy as u16) % 8;
        let tile_byte_low = self.mmu.borrow().get(tile_index + tile_pixel_y * 2);
        tile_byte_low
    }
    fn get_tile_data_high(&self) -> u8 {
        let tile_index = self.tile_index;
        let tile_pixel_y = (self.scan_y as u16 + self.scy as u16) % 8;
        let tile_byte_high = self.mmu.borrow().get(tile_index + tile_pixel_y * 2 + 1);
        tile_byte_high
    }
    fn get_buffer(&mut self) -> Vec<Pixel> {
        let mut result = Vec::new();
        let buffer_index_start = (self.scan_x as u16 + self.scx as u16) % 8;
        for buffer_index in buffer_index_start..8 {
            let pixel_bit = 8 - buffer_index - 1;
            let pixel_low = self.tile_data_low & (1 << pixel_bit) == (1 << pixel_bit);
            let pixel_high = self.tile_dada_high & (1 << pixel_bit) == (1 << pixel_bit);
            let pvalue = (pixel_low as u8) | ((pixel_high as u8) << 1);
            result.push(Pixel { ptype: BG, pvalue });
        }
        result
    }
}

struct FIFO {
    scan_x: u8,
    scan_y: u8,
    fetcher: Fetcher,
    queue: VecDeque<Pixel>,
}

impl FIFO {
    fn new(mmu: Rc<RefCell<dyn Memory>>) -> Self {
        let fetcher = Fetcher::new(mmu.clone());
        FIFO {
            scan_x: 0,
            scan_y: 0,
            fetcher,
            queue: VecDeque::new(),
        }
    }
    fn init(&mut self, y: u8) {
        self.scan_x = 0;
        self.scan_y = y;
        self.fetcher.init(0, y);
    }
    fn trick(&mut self) -> Option<Pixel> {
        let result = if self.queue.len() > 8 {
            self.pop_front()
        } else {
            None
        };
        if self.fetcher.buffer.len() > 0 {
            if self.queue.len() <= 8 {
                self.scan_x += self.fetcher.buffer.len() as u8;
                for pixel in self.fetcher.buffer.clone().into_iter() {
                    self.push_back(pixel);
                }
                self.fetcher.init(self.scan_x, self.scan_y);
            }
        } else {
            self.fetcher.trick();
        }
        result
    }
    fn push_back(&mut self, pixel: Pixel) {
        self.queue.push_back(pixel);
    }
    fn pop_front(&mut self) -> Option<Pixel> {
        self.queue.pop_front()
    }
    fn clear(&mut self) {
        self.queue.clear();
    }
}

pub struct PPU {
    cycles: u32,
    status: PpuStatus,
    fifo: FIFO,
    mmu: Rc<RefCell<dyn Memory>>,
    ly_buffer: Vec<u32>,
    pub frame_buffer: [u32; WIDTH * HEIGHT],
}
impl PPU {
    pub fn new(mmu: Rc<RefCell<dyn Memory>>) -> Self {
        let fifo = FIFO::new(mmu.clone());
        let mut ppu = PPU {
            cycles: 0,
            status: OAMScan,
            mmu,
            fifo,
            ly_buffer: Vec::new(),
            frame_buffer: [0; WIDTH * HEIGHT],
        };
        ppu.set_mode(OAMScan);
        ppu
    }
    pub fn trick(&mut self) {
        match self.status {
            OAMScan => {
                if self.cycles == 79 {
                    self.set_mode(Drawing);
                }
                self.cycles += 1;
            }
            Drawing => {
                let pixel_option = self.fifo.trick();
                if let Some(pixel) = pixel_option {
                    self.ly_buffer.push(self.get_pixel_color(pixel.pvalue));
                    if self.ly_buffer.len() == WIDTH {
                        let ly = self.get_ly();
                        for (scan_x, pixel) in self.ly_buffer.iter().enumerate() {
                            self.frame_buffer[(ly as usize * WIDTH + scan_x) as usize] = *pixel;
                        }
                        self.set_mode(HBlank);
                    }
                } else {
                }
                self.cycles += 1;
            }
            HBlank => {
                let ly = self.get_ly();
                if self.cycles == 455 {
                    if ly == 143 {
                        self.set_mode(VBlank);
                    } else {
                        self.set_mode(OAMScan);
                    }
                    self.set_ly(ly + 1);
                    self.cycles = 0;
                } else {
                    self.cycles += 1;
                }
            }
            VBlank => {
                self.set_vblank_interrupt();
                let ly = self.get_ly();
                if self.cycles == 455 {
                    if ly == 153 {
                        self.set_mode(OAMScan);
                        self.set_ly(0);
                    } else {
                        self.set_ly(ly + 1);
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
                panic!("color index is out of range {}", index);
            }
        };
        match color_value {
            0 => Color::WHITE as u32,
            1 => Color::LightGray as u32,
            2 => Color::DarkGray as u32,
            3 => Color::BlackGray as u32,
            _ => {
                panic!("color_value is out of range {}", color_value);
            }
        }
    }
    fn set_ly(&mut self, ly: u8) {
        self.mmu.borrow_mut().set(0xFF44, ly);
    }
    fn get_ly(&self) -> u8 {
        self.mmu.borrow().get(0xFF44)
    }
    fn set_vblank_interrupt(&mut self) {
        let d8 = self.mmu.borrow_mut().get(0xFF0F);
        self.mmu.borrow_mut().set(0xFF0F, d8 | 0x1);
    }
    fn set_mode(&mut self, mode: PpuStatus) {
        let value;
        match mode {
            OAMScan => {
                self.ly_buffer = Vec::new();
                let ly = self.get_ly();
                self.fifo.init(ly);

                value = 0b10;
            }
            Drawing => {
                value = 0b11;
            }
            HBlank => {
                self.ly_buffer.clear();
                self.fifo.clear();

                value = 0b00;
            }
            VBlank => {
                value = 0b01;
            }
        };
        self.status = mode;
        let d8 = self.mmu.borrow_mut().get(0xFF41);
        let d8 = d8 & 0b11111100 | value;
        self.mmu.borrow_mut().set(0xFF41, d8);
    }
}
