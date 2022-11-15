use crate::gameboy_mode::GameBoyMode;
use crate::interrupt::Interrupt;
use crate::interrupt::InterruptFlag::{VBlank as IVBlank, LCDSTAT as ILCDSTAT};
use crate::memory::Memory;
use crate::mmu::Mmu;
use crate::ppu::FetcherStatus::{GetTile, GetTileDataHigh, GetTileDataLow};
use crate::ppu::PixelType::{Sprite, Window, BG};
use crate::ppu::PpuStatus::{Drawing, HBlank, OAMScan, VBlank};
use crate::util::check_bit;
// use log::info;
use std::collections::VecDeque;
use std::{cell::RefCell, rc::Rc};

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;

enum Color {
    WHITE = 0xE0F8D0,
    LightGray = 0x88C070,
    DarkGray = 0x346856,
    BlackGray = 0x081820,
}

#[derive(Clone, Copy, PartialEq)]
enum PixelType {
    BG,
    Window,
    Sprite,
}

#[derive(Clone, Copy)]
struct Pixel {
    ptype: PixelType,
    pcolor: u8,
    pvalue: u8,
    bg_window_over_obj: bool,
    oam_priority: usize,
    bg_to_oam: bool,
}

impl Default for Pixel {
    fn default() -> Self {
        Self {
            ptype: BG,
            pcolor: 0,
            pvalue: 0,
            bg_window_over_obj: false,
            oam_priority: 40,
            bg_to_oam: false,
        }
    }
}

enum FetcherStatus {
    GetTile,
    GetTileDataLow,
    GetTileDataHigh,
}

trait Fetcher {
    fn new(mmu: Rc<RefCell<Mmu>>, scan_x: u8, scan_y: u8) -> Self
    where
        Self: Sized;
    fn trick(&mut self);
    fn get_tile(&mut self) -> u16;
    fn get_tile_data_low(&self) -> u8;
    fn get_tile_data_high(&self) -> u8;
    fn get_buffer(&mut self) -> Vec<Pixel>;
    fn get_color_index(&self, pvalue: u8) -> u8;
    fn buffer(&self) -> &[Pixel];
}

struct BGMapAttr {
    bg_to_oam: bool,
    y_flip: bool,
    x_flip: bool,
    vram_bank: bool,
    bg_palette: u8,
}
impl From<u8> for BGMapAttr {
    fn from(val: u8) -> Self {
        let bg_to_oam = check_bit(val, 7);
        let y_flip = check_bit(val, 6);
        let x_flip = check_bit(val, 5);
        let vram_bank = check_bit(val, 3);
        let bg_palette = val & 0x07;
        Self {
            bg_to_oam,
            y_flip,
            x_flip,
            vram_bank,
            bg_palette,
        }
    }
}

struct FetcherBg {
    mode: GameBoyMode,
    scan_x: u8,
    scan_y: u8,
    scx: u8,
    scy: u8,
    bg_map_attr: BGMapAttr,
    cycles: u16,
    mmu: Rc<RefCell<Mmu>>,
    status: FetcherStatus,
    tile_index: u16,
    tile_data_low: u8,
    tile_data_high: u8,
    buffer: Vec<Pixel>,
}
impl Fetcher for FetcherBg {
    fn new(mmu: Rc<RefCell<Mmu>>, scan_x: u8, scan_y: u8) -> Self {
        let mode = mmu.borrow().mode;
        Self {
            mode,
            scan_x,
            scan_y,
            scx: 0,
            scy: 0,
            bg_map_attr: BGMapAttr::from(0),
            mmu,
            cycles: 0,
            status: GetTile,
            tile_index: 0,
            tile_data_low: 0,
            tile_data_high: 0,
            buffer: Vec::with_capacity(8),
        }
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
                self.tile_data_high = self.get_tile_data_high();
                self.buffer = self.get_buffer();
                self.status = GetTile;
            }
        }
    }
    fn get_tile(&mut self) -> u16 {
        let bg_window_tile_data_area = self.mmu.borrow().ppu.lcdc.bg_window_tile_data_area;
        let bg_tile_map_area = self.mmu.borrow().ppu.lcdc.bg_tile_map_area;
        let bg_map_start: u16 = match bg_tile_map_area {
            true => 0x9C00,
            false => 0x9800,
        };
        self.scy = self.mmu.borrow().ppu.scy;
        self.scx = self.mmu.borrow().ppu.scx;
        let bg_map_x = (self.scan_x as u16 + self.scx as u16) % 256 / 8;
        let bg_map_y = (self.scan_y as u16 + self.scy as u16) % 256 / 8;
        let bg_map_index = bg_map_x + bg_map_y * 32;
        let bg_map_byte = self
            .mmu
            .borrow()
            .ppu
            .vram
            .get_by_bank(bg_map_start + bg_map_index, false);
        let tile_index: u16 = if bg_window_tile_data_area {
            0x8000 + bg_map_byte as u16 * 8 * 2
        } else {
            (0x9000 as i32 + (bg_map_byte as i8) as i32 * 8 * 2) as u16
        };
        if self.mode == GameBoyMode::GBC {
            let bg_map_attr_val = self
                .mmu
                .borrow()
                .ppu
                .vram
                .get_by_bank(bg_map_start + bg_map_index, true);
            self.bg_map_attr = BGMapAttr::from(bg_map_attr_val);
        }
        tile_index
    }
    fn get_tile_data_low(&self) -> u8 {
        let tile_index = self.tile_index;
        let mut tile_pixel_y = (self.scan_y as u16 + self.scy as u16) % 8;
        if self.mode == GameBoyMode::GBC {
            if self.bg_map_attr.y_flip {
                tile_pixel_y = (8 - 1) - tile_pixel_y;
            }
            self.mmu
                .borrow()
                .ppu
                .vram
                .get_by_bank(tile_index + tile_pixel_y * 2, self.bg_map_attr.vram_bank)
        } else {
            self.mmu
                .borrow()
                .ppu
                .vram
                .get_by_bank(tile_index + tile_pixel_y * 2, false)
        }
    }
    fn get_tile_data_high(&self) -> u8 {
        let tile_index = self.tile_index;
        let mut tile_pixel_y = (self.scan_y as u16 + self.scy as u16) % 8;
        if self.mode == GameBoyMode::GBC {
            if self.bg_map_attr.y_flip {
                tile_pixel_y = (8 - 1) - tile_pixel_y;
            }
            self.mmu.borrow().ppu.vram.get_by_bank(
                tile_index + tile_pixel_y * 2 + 1,
                self.bg_map_attr.vram_bank,
            )
        } else {
            self.mmu
                .borrow()
                .ppu
                .vram
                .get_by_bank(tile_index + tile_pixel_y * 2 + 1, false)
        }
    }
    fn get_buffer(&mut self) -> Vec<Pixel> {
        let mut result = Vec::new();
        let mut get_pixel_bit: Box<dyn Fn(u8) -> u8> = Box::new(|index: u8| 8 - index - 1);
        let buffer_index_start = (self.scan_x as u16 + self.scx as u16) % 8;
        let bg_window_enable = self.mmu.borrow().ppu.lcdc.bg_window_enable;
        if (self.mode == GameBoyMode::GBC) && self.bg_map_attr.x_flip {
            get_pixel_bit = Box::new(|index: u8| index);
        }
        for buffer_index in buffer_index_start..8 {
            let pixel_bit = get_pixel_bit(buffer_index as u8);
            let pixel_low = check_bit(self.tile_data_low, pixel_bit as u8);
            let pixel_high = check_bit(self.tile_data_high, pixel_bit as u8);
            let pvalue = (pixel_low as u8) | ((pixel_high as u8) << 1);
            let pcolor = self.get_color_index(pvalue);
            let pixel = if self.mode == GameBoyMode::GB {
                if !bg_window_enable {
                    Pixel {
                        ptype: BG,
                        pvalue: 0,
                        pcolor: 0,
                        ..Pixel::default()
                    }
                } else {
                    Pixel {
                        ptype: BG,
                        pvalue,
                        pcolor,
                        ..Pixel::default()
                    }
                }
            } else {
                Pixel {
                    ptype: BG,
                    pvalue,
                    pcolor,
                    bg_to_oam: self.bg_map_attr.bg_to_oam,
                    ..Pixel::default()
                }
            };
            result.push(pixel);
        }
        result
    }
    fn get_color_index(&self, pvalue: u8) -> u8 {
        if self.mode == GameBoyMode::GBC {
            let bg_palette = self.bg_map_attr.bg_palette;
            bg_palette * 4 * 2 + pvalue * 2
        } else {
            let palette = self.mmu.borrow().ppu.bgp;
            match pvalue {
                0 => palette & 0b11,
                1 => (palette & 0b1100) >> 2,
                2 => (palette & 0b110000) >> 4,
                3 => (palette & 0b11000000) >> 6,
                _ => {
                    panic!("color index is out of range {}", pvalue);
                }
            }
        }
    }
    fn buffer(&self) -> &[Pixel] {
        &self.buffer
    }
}

struct FetcherWindow {
    mode: GameBoyMode,
    scan_x: u8,
    wx: u8,
    wy: u8,
    window_internal_line_index: u8,
    bg_map_attr: BGMapAttr,
    cycles: u16,
    mmu: Rc<RefCell<Mmu>>,
    status: FetcherStatus,
    tile_index: u16,
    tile_data_low: u8,
    tile_data_high: u8,
    buffer: Vec<Pixel>,
}
impl FetcherWindow {
    fn set_window_internal_line_index(&mut self, window_internal_line_index: u8) {
        self.window_internal_line_index = window_internal_line_index;
    }
}
impl Fetcher for FetcherWindow {
    fn new(mmu: Rc<RefCell<Mmu>>, scan_x: u8, _: u8) -> Self {
        let mode = mmu.borrow().mode;
        Self {
            mode,
            scan_x,
            wx: 0,
            wy: 0,
            window_internal_line_index: 0,
            bg_map_attr: BGMapAttr::from(0),
            mmu,
            cycles: 0,
            status: GetTile,
            tile_index: 0,
            tile_data_low: 0,
            tile_data_high: 0,
            buffer: Vec::with_capacity(8),
        }
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
                self.tile_data_high = self.get_tile_data_high();
                self.buffer = self.get_buffer();
                self.status = GetTile;
            }
        }
    }
    fn get_tile(&mut self) -> u16 {
        let bg_window_tile_data_area = self.mmu.borrow().ppu.lcdc.bg_window_tile_data_area;
        let window_tile_map_area = self.mmu.borrow().ppu.lcdc.window_tile_map_area;
        let window_map_start: u16 = match window_tile_map_area {
            true => 0x9C00,
            false => 0x9800,
        };

        self.wy = self.mmu.borrow().ppu.wy;
        self.wx = self.mmu.borrow().ppu.wx;
        let bg_map_x = (self.scan_x as u16 + 7 - self.wx as u16) % 256 / 8;
        let bg_map_y = self.window_internal_line_index as u16 / 8;
        let bg_map_index = bg_map_x + bg_map_y * 32;
        let bg_map_byte = self
            .mmu
            .borrow()
            .ppu
            .vram
            .get_by_bank(window_map_start + bg_map_index, false);

        let tile_index: u16 = if bg_window_tile_data_area {
            0x8000 + bg_map_byte as u16 * 8 * 2
        } else {
            (0x9000 as i32 + (bg_map_byte as i8) as i32 * 8 * 2) as u16
        };
        if self.mode == GameBoyMode::GBC {
            let bg_map_attr_val = self
                .mmu
                .borrow()
                .ppu
                .vram
                .get_by_bank(window_map_start + bg_map_index, true);
            self.bg_map_attr = BGMapAttr::from(bg_map_attr_val);
        }
        tile_index
    }
    fn get_tile_data_low(&self) -> u8 {
        let tile_index = self.tile_index;
        let mut tile_pixel_y = self.window_internal_line_index as u16 % 8;
        if self.mode == GameBoyMode::GBC {
            if self.bg_map_attr.y_flip {
                tile_pixel_y = (8 - 1) - tile_pixel_y;
            }
            self.mmu
                .borrow()
                .ppu
                .vram
                .get_by_bank(tile_index + tile_pixel_y * 2, self.bg_map_attr.vram_bank)
        } else {
            self.mmu
                .borrow()
                .ppu
                .vram
                .get_by_bank(tile_index + tile_pixel_y * 2, false)
        }
    }
    fn get_tile_data_high(&self) -> u8 {
        let tile_index = self.tile_index;
        let mut tile_pixel_y = self.window_internal_line_index as u16 % 8;
        if self.mode == GameBoyMode::GBC {
            if self.bg_map_attr.y_flip {
                tile_pixel_y = (8 - 1) - tile_pixel_y;
            }
            self.mmu.borrow().ppu.vram.get_by_bank(
                tile_index + tile_pixel_y * 2 + 1,
                self.bg_map_attr.vram_bank,
            )
        } else {
            self.mmu
                .borrow()
                .ppu
                .vram
                .get_by_bank(tile_index + tile_pixel_y * 2 + 1, false)
        }
    }
    fn get_buffer(&mut self) -> Vec<Pixel> {
        let mut result = Vec::new();
        let mut get_pixel_bit: Box<dyn Fn(u8) -> u8> = Box::new(|index: u8| 8 - index - 1);
        if (self.mode == GameBoyMode::GBC) && self.bg_map_attr.x_flip {
            get_pixel_bit = Box::new(|index: u8| index);
        }
        let bg_window_enable = self.mmu.borrow().ppu.lcdc.bg_window_enable;
        let buffer_index_start = (self.scan_x as u16 + 7 - self.wx as u16) % 8;
        for buffer_index in buffer_index_start..8 {
            let pixel_bit = get_pixel_bit(buffer_index as u8);
            let pixel_low = check_bit(self.tile_data_low, pixel_bit as u8);
            let pixel_high = check_bit(self.tile_data_high, pixel_bit as u8);
            let pvalue = (pixel_low as u8) | ((pixel_high as u8) << 1);
            let pcolor = self.get_color_index(pvalue);
            let pixel = if self.mode == GameBoyMode::GB {
                if !bg_window_enable {
                    Pixel {
                        ptype: Window,
                        pvalue: 0,
                        pcolor: 0,
                        ..Pixel::default()
                    }
                } else {
                    Pixel {
                        ptype: Window,
                        pvalue,
                        pcolor,
                        ..Pixel::default()
                    }
                }
            } else {
                Pixel {
                    ptype: Window,
                    pvalue,
                    pcolor,
                    bg_to_oam: self.bg_map_attr.bg_to_oam,
                    ..Pixel::default()
                }
            };
            result.push(pixel);
        }
        result
    }
    fn get_color_index(&self, pvalue: u8) -> u8 {
        if self.mode == GameBoyMode::GBC {
            let bg_palette = self.bg_map_attr.bg_palette;
            bg_palette * 4 * 2 + pvalue * 2
        } else {
            let palette = self.mmu.borrow().ppu.bgp;
            match pvalue {
                0 => palette & 0b11,
                1 => (palette & 0b1100) >> 2,
                2 => (palette & 0b110000) >> 4,
                3 => (palette & 0b11000000) >> 6,
                _ => {
                    panic!("color index is out of range {}", pvalue);
                }
            }
        }
    }
    fn buffer(&self) -> &[Pixel] {
        &self.buffer
    }
}

struct FetcherSprite {
    mode: GameBoyMode,
    scan_x: u8,
    scan_y: u8,
    oam: OAM,
    cycles: u16,
    mmu: Rc<RefCell<Mmu>>,
    status: FetcherStatus,
    tile_index: u16,
    tile_data_low: u8,
    tile_data_high: u8,
    buffer: Vec<Pixel>,
}
impl FetcherSprite {
    fn set_oam(&mut self, oam: OAM) {
        self.oam = oam;
    }
}
impl Fetcher for FetcherSprite {
    fn new(mmu: Rc<RefCell<Mmu>>, scan_x: u8, scan_y: u8) -> Self {
        let mode = mmu.borrow().mode;
        Self {
            mode,
            scan_x,
            scan_y,
            oam: OAM::default(),
            mmu,
            cycles: 0,
            status: GetTile,
            tile_index: 0,
            tile_data_low: 0,
            tile_data_high: 0,
            buffer: Vec::with_capacity(8),
        }
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
                self.tile_data_high = self.get_tile_data_high();
                self.buffer = self.get_buffer();
                self.status = GetTile;
            }
        }
    }
    fn get_tile(&mut self) -> u16 {
        if self.mmu.borrow().ppu.lcdc.obj_size {
            0x8000 + (self.oam.tile_index as u16 & 0xFE) * 16
        } else {
            0x8000 + (self.oam.tile_index as u16) * 16
        }
    }
    fn get_tile_data_low(&self) -> u8 {
        let tile_index = self.tile_index;
        let obj_size = self.mmu.borrow().ppu.lcdc.obj_size;
        let height = if obj_size { 16 } else { 8 };
        let mut tile_pixel_y = (self.scan_y as u16 + 16 - self.oam.y as u16) % height;
        if self.oam.y_flip {
            tile_pixel_y = (height - 1) - tile_pixel_y;
        }
        self.mmu.borrow().ppu.vram.get_by_bank(
            tile_index + tile_pixel_y * 2,
            if self.mode == GameBoyMode::GB {
                false
            } else {
                self.oam.vram_bank
            },
        )
    }
    fn get_tile_data_high(&self) -> u8 {
        let tile_index = self.tile_index;
        let obj_size = self.mmu.borrow().ppu.lcdc.obj_size;
        let height = if obj_size { 16 } else { 8 };
        let mut tile_pixel_y = (self.scan_y as u16 + 16 - self.oam.y as u16) % height;
        if self.oam.y_flip {
            tile_pixel_y = (height - 1) - tile_pixel_y;
        }
        self.mmu.borrow().ppu.vram.get_by_bank(
            tile_index + tile_pixel_y * 2 + 1,
            if self.mode == GameBoyMode::GB {
                false
            } else {
                self.oam.vram_bank
            },
        )
    }
    fn get_buffer(&mut self) -> Vec<Pixel> {
        let mut result = Vec::new();
        let mut get_pixel_bit: Box<dyn Fn(u8) -> u8> = Box::new(|index: u8| 8 - index - 1);
        if self.oam.x_flip {
            get_pixel_bit = Box::new(|index: u8| index);
        }
        let buffer_index_start = (self.scan_x as u16 + 8 - self.oam.x as u16) % 8;
        for buffer_index in buffer_index_start..8 {
            let pixel_bit = get_pixel_bit(buffer_index as u8);
            let pixel_low = check_bit(self.tile_data_low, pixel_bit as u8);
            let pixel_high = check_bit(self.tile_data_high, pixel_bit as u8);
            let pvalue = (pixel_low as u8) | ((pixel_high as u8) << 1);
            let pcolor = self.get_color_index(pvalue);
            result.push(Pixel {
                ptype: Sprite,
                pvalue,
                pcolor,
                bg_window_over_obj: self.oam.bg_window_over_obj,
                oam_priority: self.oam.priority,
                ..Pixel::default()
            });
        }
        result
    }
    fn get_color_index(&self, pvalue: u8) -> u8 {
        if self.mode == GameBoyMode::GBC {
            let cpalette = self.oam.cpalette;
            cpalette * 4 * 2 + pvalue * 2
        } else {
            let palette = {
                if self.oam.palette {
                    self.mmu.borrow().ppu.op1
                } else {
                    self.mmu.borrow().ppu.op0
                }
            };
            match pvalue {
                0 => palette & 0b11,
                1 => (palette & 0b1100) >> 2,
                2 => (palette & 0b110000) >> 4,
                3 => (palette & 0b11000000) >> 6,
                _ => {
                    panic!("color index is out of range {}", pvalue);
                }
            }
        }
    }
    fn buffer(&self) -> &[Pixel] {
        &self.buffer
    }
}

struct OAM {
    y: u8,
    x: u8,
    tile_index: u8,
    bg_window_over_obj: bool,
    x_flip: bool,
    y_flip: bool,
    palette: bool,
    cpalette: u8,
    vram_bank: bool,
    priority: usize,
}
impl OAM {
    fn set(&mut self, y: u8, x: u8, tile_index: u8, priority: usize) {
        self.y = y;
        self.x = x;
        self.tile_index = tile_index;
        self.priority = priority;
    }
    fn is_scaned(&self, ly: u8, obj_size: bool) -> bool {
        let height = if obj_size { 16 } else { 8 };
        let y_start = self.y as i32 - 16;
        let y_end = self.y as i32 + height - 16;
        ((ly as i32) >= y_start) && ((ly as i32) < y_end) && (self.x != 0)
    }
}
impl From<u8> for OAM {
    fn from(val: u8) -> Self {
        let bg_window_over_obj = check_bit(val, 7);
        let y_flip = check_bit(val, 6);
        let x_flip = check_bit(val, 5);
        let palette = check_bit(val, 4);
        let vram_bank = check_bit(val, 3);
        let cpalette = val & 0x07;
        Self {
            bg_window_over_obj,
            y_flip,
            x_flip,
            palette,
            vram_bank,
            cpalette,
            ..Self::default()
        }
    }
}

impl Default for OAM {
    fn default() -> Self {
        Self {
            y: 0,
            x: 0,
            tile_index: 0,
            bg_window_over_obj: false,
            x_flip: false,
            y_flip: false,
            palette: false,
            cpalette: 0,
            vram_bank: false,
            priority: 40,
        }
    }
}

enum FifoTrick {
    BgWindow,
    Sprite,
}
struct FIFO {
    x: u8,
    y: u8,
    window_internal_line_counters: u8,
    status: FifoTrick,
    mmu: Rc<RefCell<Mmu>>,
    fetcher: Box<dyn Fetcher>,
    sprite_queue: VecDeque<Pixel>,
    queue: VecDeque<Pixel>,
    oam: Vec<OAM>,
}
impl FIFO {
    fn new(mmu: Rc<RefCell<Mmu>>) -> Self {
        let x = 0;
        let y = 0;
        let fetcher = Box::new(FetcherBg::new(mmu.clone(), x, y));
        Self {
            x,
            y,
            window_internal_line_counters: 0,
            mmu,
            status: FifoTrick::BgWindow,
            fetcher,
            sprite_queue: VecDeque::new(),
            queue: VecDeque::new(),
            oam: vec![],
        }
    }
    fn init(&mut self, y: u8) {
        self.x = 0;
        self.y = y;
        if y == 0 {
            self.window_internal_line_counters = 0;
        }
        if self.check_window(160) {
            self.window_internal_line_counters = self.window_internal_line_counters + 1;
        }
        self.sprite_queue.clear();
        self.queue.clear();
        self.oam.clear();
        self.status = FifoTrick::BgWindow;
        self.fetcher = self.get_fetcher_window_or_bg(self.check_window_or_bg(self.x), self.x, y);
    }
    fn set_oam(&mut self, oam: Vec<OAM>) {
        self.oam = oam;
    }
    fn trick(&mut self) -> Option<Pixel> {
        match self.status {
            FifoTrick::BgWindow => {
                let mut result = None;
                if self.queue.len() > 8 {
                    // 检查当前像素是否上层有Window或Sprite
                    let front = self.front().unwrap().to_owned();
                    let new_fetch_event = self.check_overlap(front.ptype, self.x);
                    if let Some(event) = new_fetch_event {
                        match event {
                            Window => {
                                self.status = FifoTrick::BgWindow;
                                self.queue.clear();
                                self.fetcher =
                                    self.get_fetcher_window_or_bg(Window, self.x, self.y);
                                return None;
                            }
                            Sprite => {
                                self.status = FifoTrick::Sprite;
                                let oam = self.oam_pop(self.x).unwrap();
                                let mut fetcher =
                                    Box::new(FetcherSprite::new(self.mmu.clone(), self.x, self.y));
                                fetcher.set_oam(oam);
                                self.fetcher = fetcher;
                                return None;
                            }
                            _ => {
                                panic!("ppu fifo trick error");
                            }
                        };
                    }
                    // 执行到这，无异常，正常压入弹出流程
                    self.x += 1;
                    result = self.pop_front();
                }
                if self.fetcher.buffer().len() > 0 {
                    if self.queue.len() <= 8 {
                        let buffer: Vec<Pixel> = self.fetcher.buffer().iter().map(|x| *x).collect();
                        for pixel in buffer {
                            self.push_back(pixel);
                        }
                        let fetcher_x = self.x + self.queue.len() as u8;
                        self.status = FifoTrick::BgWindow;
                        self.fetcher = self.get_fetcher_window_or_bg(
                            self.check_window_or_bg(fetcher_x),
                            fetcher_x,
                            self.y,
                        );
                    }
                } else {
                    self.fetcher.trick();
                }
                result
            }
            FifoTrick::Sprite => {
                if self.fetcher.buffer().len() > 0 {
                    let sprite_queue_len = self.sprite_queue.len();
                    for (index, new_sprite_pixel) in self.fetcher.buffer().iter().enumerate() {
                        if index < sprite_queue_len {
                            let old_sprite_pixel = self.sprite_queue[index];
                            if new_sprite_pixel.pvalue == 0 && old_sprite_pixel.pvalue == 0 {}
                            if new_sprite_pixel.pvalue == 0 && old_sprite_pixel.pvalue != 0 {}
                            if new_sprite_pixel.pvalue != 0 && old_sprite_pixel.pvalue == 0 {
                                self.sprite_queue[index] = *new_sprite_pixel;
                            }
                            if new_sprite_pixel.pvalue != 0 && old_sprite_pixel.pvalue != 0 {
                                if new_sprite_pixel.oam_priority < old_sprite_pixel.oam_priority {
                                    self.sprite_queue[index] = *new_sprite_pixel;
                                }
                            }
                        } else {
                            self.sprite_queue.push_back(*new_sprite_pixel);
                        }
                    }
                    self.status = FifoTrick::BgWindow;
                    let fetcher_x = self.x + self.queue.len() as u8;
                    self.fetcher = self.get_fetcher_window_or_bg(
                        self.check_window_or_bg(fetcher_x),
                        fetcher_x,
                        self.y,
                    );
                } else {
                    self.fetcher.trick();
                }
                None
            }
        }
    }
    fn check_overlap(&self, ptype: PixelType, x: u8) -> Option<PixelType> {
        match ptype {
            BG => {
                if self.check_window(x) {
                    Some(Window)
                } else if self.check_sprite(x) {
                    Some(Sprite)
                } else {
                    None
                }
            }
            Window => {
                if self.check_sprite(x) {
                    Some(Sprite)
                } else {
                    None
                }
            }
            Sprite => {
                if self.check_sprite(x) {
                    Some(Sprite)
                } else {
                    None
                }
            }
        }
    }
    fn check_window(&self, x: u8) -> bool {
        let window_enable = self.mmu.borrow().ppu.lcdc.window_enable;
        if !window_enable {
            return false;
        }
        let wy = self.mmu.borrow().ppu.wy;
        let wx = self.mmu.borrow().ppu.wx;
        (x + 7 >= wx) && (self.y >= wy)
    }
    fn check_sprite(&self, x: u8) -> bool {
        let obj_enable = self.mmu.borrow().ppu.lcdc.obj_enable;
        if !obj_enable {
            return false;
        }
        for oam in self.oam.iter() {
            if x + 8 >= oam.x && x < oam.x {
                return true;
            }
        }
        false
    }
    fn oam_pop(&mut self, x: u8) -> Option<OAM> {
        let mut oam_index = None;
        for (index, oam) in self.oam.iter().enumerate() {
            if x + 8 >= oam.x && x < oam.x {
                oam_index = Some(index);
                break;
            }
        }
        if !oam_index.is_none() {
            let index = oam_index.unwrap();
            Some(self.oam.remove(index))
        } else {
            None
        }
    }
    fn get_fetcher_window_or_bg(&self, ptype: PixelType, x: u8, y: u8) -> Box<dyn Fetcher> {
        let mmu = self.mmu.clone();
        match ptype {
            BG => Box::new(FetcherBg::new(mmu, x, y)),
            Window => {
                let mut fetcher = FetcherWindow::new(mmu, x, y);
                fetcher.set_window_internal_line_index(self.window_internal_line_counters - 1);
                Box::new(fetcher)
            }
            _ => panic!(""),
        }
    }
    fn check_window_or_bg(&self, x: u8) -> PixelType {
        if self.check_window(x) {
            Window
        } else {
            BG
        }
    }
    fn front(&mut self) -> Option<&Pixel> {
        self.sprite_queue.front().or(self.queue.front())
    }
    fn push_back(&mut self, pixel: Pixel) {
        self.queue.push_back(pixel);
    }
    fn pop_front(&mut self) -> Option<Pixel> {
        let sprite_pixel_option = self.sprite_queue.pop_front();
        match sprite_pixel_option {
            Some(sprite_pixel) => {
                let bg_pixel = self.queue.pop_front().unwrap();
                if self.mmu.borrow().mode == GameBoyMode::GBC {
                    let bg_window_enable = self.mmu.borrow().ppu.lcdc.bg_window_enable;
                    if !bg_window_enable {
                        if sprite_pixel.pvalue == 0 {
                            Some(bg_pixel)
                        } else {
                            Some(sprite_pixel)
                        }
                    } else {
                        if bg_pixel.bg_to_oam {
                            if bg_pixel.pvalue == 0 {
                                Some(sprite_pixel)
                            } else {
                                Some(bg_pixel)
                            }
                        } else {
                            if sprite_pixel.bg_window_over_obj {
                                if bg_pixel.pvalue == 0 {
                                    Some(sprite_pixel)
                                } else {
                                    Some(bg_pixel)
                                }
                            } else {
                                if sprite_pixel.pvalue == 0 {
                                    Some(bg_pixel)
                                } else {
                                    Some(sprite_pixel)
                                }
                            }
                        }
                    }
                } else {
                    if sprite_pixel.bg_window_over_obj {
                        if bg_pixel.pcolor == 0 {
                            Some(sprite_pixel)
                        } else {
                            Some(bg_pixel)
                        }
                    } else {
                        if sprite_pixel.pvalue == 0 {
                            Some(bg_pixel)
                        } else {
                            Some(sprite_pixel)
                        }
                    }
                }
            }
            None => self.queue.pop_front(),
        }
    }
}

#[derive(Clone, Copy)]
pub enum PpuStatus {
    OAMScan = 2,
    Drawing = 3,
    HBlank = 0,
    VBlank = 1,
}
pub struct PPU {
    mode: GameBoyMode,
    cycles: u32,
    fifo: FIFO,
    mmu: Rc<RefCell<Mmu>>,
    ly_buffer: Vec<u32>,
    lcd_enable: bool,
    pub frame_buffer: [u32; WIDTH * HEIGHT],
}
impl PPU {
    pub fn new(mmu: Rc<RefCell<Mmu>>) -> Self {
        let mode = mmu.borrow().mode;
        let fifo = FIFO::new(mmu.clone());
        let mut ppu = PPU {
            mode,
            cycles: 0,
            mmu,
            fifo,
            lcd_enable: true,
            ly_buffer: Vec::with_capacity(WIDTH),
            frame_buffer: [Color::WHITE as u32; WIDTH * HEIGHT],
        };
        ppu.set_mode(OAMScan);
        ppu
    }
    pub fn trick(&mut self) -> bool {
        let lcd_enable = self.mmu.borrow().ppu.lcdc.lcd_ppu_enable;
        let mut is_refresh = false;
        if !lcd_enable {
            if self.lcd_enable == lcd_enable {
                return is_refresh;
            }
            self.cycles = 0;
            self.ly_buffer = Vec::with_capacity(WIDTH);
            self.frame_buffer = [Color::WHITE as u32; WIDTH * HEIGHT];
            self.fifo = FIFO::new(self.mmu.clone());
            self.mmu.borrow_mut().ppu.reset_ly();
            self.mmu.borrow_mut().ppu.stat.mode_flag = HBlank;
            is_refresh = true;
        } else {
            let mode_flag = self.mmu.borrow().ppu.stat.mode_flag;
            match mode_flag {
                OAMScan => {
                    if self.cycles == 0 {
                        self.set_ly_interrupt();
                        self.set_mode_interrupt();
                        let ly = self.get_ly();
                        self.fifo.init(ly);
                        let oams = self.oam_scan();
                        self.fifo.set_oam(oams);
                    }
                    if self.cycles == 79 {
                        self.set_mode(Drawing);
                    }
                    self.cycles += 1;
                }
                Drawing => {
                    let pixel_option = self.fifo.trick();
                    if let Some(pixel) = pixel_option {
                        self.ly_buffer.push(self.get_pixel_color(pixel));
                        if self.ly_buffer.len() == WIDTH {
                            let ly = self.get_ly();
                            for (scan_x, pixel) in self.ly_buffer.iter().enumerate() {
                                self.frame_buffer[(ly as usize * WIDTH + scan_x) as usize] = *pixel;
                            }
                            self.set_mode(HBlank);
                            self.set_mode_interrupt();
                        }
                    } else {
                    }
                    self.cycles += 1;
                }
                HBlank => {
                    let ly = self.get_ly();
                    if self.cycles >= 455 {
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
                    let ly = self.get_ly();
                    if self.cycles == 0 {
                        self.set_ly_interrupt();
                        if ly == 144 {
                            is_refresh = true;
                            self.set_mode_interrupt();
                        }
                    }
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
        self.lcd_enable = lcd_enable;
        is_refresh
    }
    fn get_pixel_color(&self, pixel: Pixel) -> u32 {
        if self.mode == GameBoyMode::GBC {
            let rgb_memory = match pixel.ptype {
                BG | Window => self.mmu.borrow().ppu.bcp.memory,
                Sprite => self.mmu.borrow().ppu.ocp.memory,
            };
            let rgb_low = rgb_memory[pixel.pcolor as usize];
            let rgb_high = rgb_memory[pixel.pcolor as usize + 1];
            let color = u16::from_be_bytes([rgb_high, rgb_low]);
            let blue = ((color & 0x7C00) >> 10) as u32;
            let green = ((color & 0x03E0) >> 5) as u32;
            let red = (color & 0x001F) as u32;

            let hex_red = (red << 3) | (red >> 2);
            let hex_green = (green << 3) | (green >> 2);
            let hex_blue = (blue << 3) | (blue >> 2);

            let result = (hex_red << 16) | (hex_green << 8) | hex_blue;
            result
        } else {
            match pixel.pcolor {
                0 => Color::WHITE as u32,
                1 => Color::LightGray as u32,
                2 => Color::DarkGray as u32,
                3 => Color::BlackGray as u32,
                _ => {
                    panic!("color_value is out of range {}", pixel.pcolor);
                }
            }
        }
    }
    fn oam_scan(&self) -> Vec<OAM> {
        let ly = self.get_ly();
        let mut result = Vec::with_capacity(10);
        let obj_size = self.mmu.borrow().ppu.lcdc.obj_size;
        for index in 00..40 {
            let oam_address = 0xFE00 + (index as u16) * 4;
            let y = self.mmu.borrow().get(oam_address);
            let x = self.mmu.borrow().get(oam_address + 1);
            let tile_index = self.mmu.borrow().get(oam_address + 2);
            let flags = self.mmu.borrow().get(oam_address + 3);
            let mut oam = OAM::from(flags);
            oam.set(y, x, tile_index, index);
            if oam.is_scaned(ly, obj_size) {
                result.push(oam);
            }
            if result.len() == 10 {
                break;
            }
        }
        result
    }
    fn set_ly(&mut self, ly: u8) {
        self.mmu.borrow_mut().ppu.set_ly(ly);
    }
    fn set_ly_interrupt(&mut self) {
        self.mmu.borrow_mut().ppu.set_ly_interrupt();
    }
    fn get_ly(&self) -> u8 {
        self.mmu.borrow().ppu.get_ly()
    }
    fn set_mode(&mut self, mode: PpuStatus) {
        match mode {
            OAMScan => {
                self.ly_buffer = Vec::with_capacity(WIDTH);
            }
            Drawing => {}
            HBlank => {
                self.ly_buffer.clear();
            }
            VBlank => {}
        };
        self.mmu.borrow_mut().ppu.set_mode(mode);
    }
    fn set_mode_interrupt(&mut self) {
        self.mmu.borrow_mut().ppu.set_mode_interrupt();
    }
}

pub struct LCDC {
    pub lcd_ppu_enable: bool,
    pub window_tile_map_area: bool,
    pub window_enable: bool,
    pub bg_window_tile_data_area: bool,
    pub bg_tile_map_area: bool,
    pub obj_size: bool,
    pub obj_enable: bool,
    pub bg_window_enable: bool,
}
impl LCDC {
    fn new() -> Self {
        Self {
            lcd_ppu_enable: true,
            window_tile_map_area: true,
            window_enable: true,
            bg_window_tile_data_area: false,
            bg_tile_map_area: false,
            obj_size: false,
            obj_enable: true,
            bg_window_enable: true,
        }
    }
    fn get_complete_bit(&self, index: u8) -> u8 {
        match index {
            7 => (self.lcd_ppu_enable as u8) << index,
            6 => (self.window_tile_map_area as u8) << index,
            5 => (self.window_enable as u8) << index,
            4 => (self.bg_window_tile_data_area as u8) << index,
            3 => (self.bg_tile_map_area as u8) << index,
            2 => (self.obj_size as u8) << index,
            1 => (self.obj_enable as u8) << index,
            0 => (self.bg_window_enable as u8) << index,
            _ => panic!("get_complete_bit index out of range"),
        }
    }
}
impl Memory for LCDC {
    fn get(&self, index: u16) -> u8 {
        assert_eq!(index, 0xFF40);
        let lcd_ppu_enable = self.get_complete_bit(7);
        let window_tile_map_area = self.get_complete_bit(6);
        let window_enable = self.get_complete_bit(5);
        let bg_window_tile_data_area = self.get_complete_bit(4);
        let bg_tile_map_area = self.get_complete_bit(3);
        let obj_size = self.get_complete_bit(2);
        let obj_enable = self.get_complete_bit(1);
        let bg_window_enable = self.get_complete_bit(0);
        lcd_ppu_enable
            | window_tile_map_area
            | window_enable
            | bg_window_tile_data_area
            | bg_tile_map_area
            | obj_size
            | obj_enable
            | bg_window_enable
    }
    fn set(&mut self, index: u16, value: u8) {
        assert_eq!(index, 0xFF40);
        self.lcd_ppu_enable = check_bit(value, 7);
        self.window_tile_map_area = check_bit(value, 6);
        self.window_enable = check_bit(value, 5);
        self.bg_window_tile_data_area = check_bit(value, 4);
        self.bg_tile_map_area = check_bit(value, 3);
        self.obj_size = check_bit(value, 2);
        self.obj_enable = check_bit(value, 1);
        self.bg_window_enable = check_bit(value, 0);
    }
}

pub struct STAT {
    ly: Rc<RefCell<u8>>,
    lyc: Rc<RefCell<u8>>,
    pub lyc_ly_interrupt: bool,
    pub mode2_interrupt: bool,
    pub mode1_interrupt: bool,
    pub mode0_interrupt: bool,
    pub mode_flag: PpuStatus,
}
impl STAT {
    fn new(ly: Rc<RefCell<u8>>, lyc: Rc<RefCell<u8>>) -> Self {
        Self {
            ly,
            lyc,
            lyc_ly_interrupt: false,
            mode2_interrupt: false,
            mode1_interrupt: false,
            mode0_interrupt: false,
            mode_flag: OAMScan,
        }
    }
    fn get_complete_bit(&self, index: u8) -> u8 {
        match index {
            6 => (self.lyc_ly_interrupt as u8) << index,
            5 => (self.mode2_interrupt as u8) << index,
            4 => (self.mode1_interrupt as u8) << index,
            3 => (self.mode0_interrupt as u8) << index,
            2 => {
                let ly = *self.ly.borrow();
                let lyc = *self.lyc.borrow();
                ((ly == lyc) as u8) << index
            }
            _ => panic!("get_complete_bit index out of range"),
        }
    }
}
impl Memory for STAT {
    fn get(&self, index: u16) -> u8 {
        assert_eq!(index, 0xFF41);
        let lyc_ly_interrupt = self.get_complete_bit(6);
        let mode2_interrupt = self.get_complete_bit(5);
        let mode1_interrupt = self.get_complete_bit(4);
        let mode0_interrupt = self.get_complete_bit(3);
        let lyc_ly_flag = self.get_complete_bit(2);
        let mode_flag = self.mode_flag.clone() as u8;
        lyc_ly_interrupt
            | mode2_interrupt
            | mode1_interrupt
            | mode0_interrupt
            | lyc_ly_flag
            | mode_flag
    }
    fn set(&mut self, index: u16, value: u8) {
        assert_eq!(index, 0xFF41);
        self.lyc_ly_interrupt = check_bit(value, 6);
        self.mode2_interrupt = check_bit(value, 5);
        self.mode1_interrupt = check_bit(value, 4);
        self.mode0_interrupt = check_bit(value, 3);
    }
}

struct VRAM {
    mode: GameBoyMode,
    bank: u8,
    memory: [u8; (0x9FFF - 0x8000 + 1) * 2],
}
impl VRAM {
    fn new(mode: GameBoyMode) -> Self {
        Self {
            mode,
            bank: 0xFF,
            memory: [0; (0x9FFF - 0x8000 + 1) * 2],
        }
    }
    fn get_bank_index(&self) -> u8 {
        if self.mode == GameBoyMode::GBC {
            self.bank & 1
        } else {
            0
        }
    }
    fn get_by_bank(&self, index: u16, bank: bool) -> u8 {
        if self.mode == GameBoyMode::GBC {
            let bank_index = if bank { 1 } else { 0 };
            let index = index - 0x8000 + bank_index * (0x9FFF - 0x8000 + 1);
            self.memory[index as usize]
        } else {
            self.get(index)
        }
    }
}
impl Memory for VRAM {
    fn get(&self, index: u16) -> u8 {
        match index {
            0xFF4F => self.bank,
            0x8000..=0x9FFF => {
                let bank_index = self.get_bank_index() as u16;
                let index = index - 0x8000 + bank_index * (0x9FFF - 0x8000 + 1);
                self.memory[index as usize]
            }
            _ => panic!("VRAM out of range"),
        }
    }
    fn set(&mut self, index: u16, value: u8) {
        match index {
            0xFF4F => self.bank = value,
            0x8000..=0x9FFF => {
                let bank_index = self.get_bank_index() as u16;
                let index = index - 0x8000 + bank_index * (0x9FFF - 0x8000 + 1);
                self.memory[index as usize] = value;
            }
            _ => panic!("VRAM out of range"),
        }
    }
}

struct BCP {
    auto_increment: bool,
    address: u8,
    memory: [u8; 64],
}
impl BCP {
    fn new() -> Self {
        Self {
            auto_increment: false,
            address: 0,
            memory: [0; 64],
        }
    }
}
impl Memory for BCP {
    fn get(&self, index: u16) -> u8 {
        match index {
            0xFF68 => {
                let auto_increment_bit = if self.auto_increment { 1 } else { 0 };
                auto_increment_bit << 7 | self.address | 0b01000000
            }
            0xFF69 => self.memory[self.address as usize],
            _ => panic!("BCP out of range"),
        }
    }
    fn set(&mut self, index: u16, value: u8) {
        match index {
            0xFF68 => {
                self.auto_increment = check_bit(value, 7);
                self.address = value & 0x3F;
            }
            0xFF69 => {
                self.memory[self.address as usize] = value;
                if self.auto_increment {
                    self.address = self.address + 1;
                    if self.address == 0x40 {
                        self.address = 0;
                    }
                }
            }
            _ => panic!("BCP out of range"),
        }
    }
}

struct OCP {
    auto_increment: bool,
    address: u8,
    memory: [u8; 64],
}
impl OCP {
    fn new() -> Self {
        Self {
            auto_increment: false,
            address: 0,
            memory: [0; 64],
        }
    }
}
impl Memory for OCP {
    fn get(&self, index: u16) -> u8 {
        match index {
            0xFF6A => {
                let auto_increment_bit = if self.auto_increment { 1 } else { 0 };
                auto_increment_bit << 7 | self.address | 0b01000000
            }
            0xFF6B => self.memory[self.address as usize],
            _ => panic!("OCP out of range"),
        }
    }
    fn set(&mut self, index: u16, value: u8) {
        match index {
            0xFF6A => {
                self.auto_increment = check_bit(value, 7);
                self.address = value & 0x3F;
            }
            0xFF6B => {
                self.memory[self.address as usize] = value;
                if self.auto_increment {
                    self.address = self.address + 1;
                    if self.address == 0x40 {
                        self.address = 0;
                    }
                }
            }
            _ => panic!("OCP out of range"),
        }
    }
}

pub struct PpuMmu {
    mode: GameBoyMode,
    interrupt: Rc<RefCell<Interrupt>>,
    pub lcdc: LCDC,
    pub stat: STAT,
    ly: Rc<RefCell<u8>>,
    lyc: Rc<RefCell<u8>>,
    pub scy: u8,
    pub scx: u8,
    pub wx: u8,
    pub wy: u8,
    pub bgp: u8,
    pub op0: u8,
    pub op1: u8,
    vram: VRAM,
    bcp: BCP,
    ocp: OCP,
    oam: [u8; 0xFE9F - 0xFE00 + 1],
}
impl PpuMmu {
    pub fn new(mode: GameBoyMode, interrupt: Rc<RefCell<Interrupt>>) -> Self {
        let lcdc = LCDC::new();
        let ly = Rc::new(RefCell::new(0));
        let lyc = Rc::new(RefCell::new(0));
        let stat = STAT::new(ly.clone(), lyc.clone());
        let vram = VRAM::new(mode);
        let bcp = BCP::new();
        let ocp = OCP::new();
        Self {
            mode,
            interrupt,
            lcdc,
            stat,
            ly,
            lyc,
            scy: 0,
            scx: 0,
            wx: 0,
            wy: 0,
            bgp: 0,
            op0: 0,
            op1: 0,
            vram,
            bcp,
            ocp,
            oam: [0; 0xFE9F - 0xFE00 + 1],
        }
    }
    pub fn set_mode(&mut self, mode: PpuStatus) {
        self.stat.mode_flag = mode;
    }
    pub fn set_mode_interrupt(&mut self) {
        let mode = self.stat.mode_flag;
        match mode {
            OAMScan => {
                let enable = self.stat.mode2_interrupt;
                if enable {
                    self.interrupt.borrow_mut().set_flag(ILCDSTAT);
                }
            }
            VBlank => {
                let enable = self.stat.mode1_interrupt;
                if enable {
                    self.interrupt.borrow_mut().set_flag(ILCDSTAT);
                }
                self.interrupt.borrow_mut().set_flag(IVBlank);
            }
            HBlank => {
                let enable = self.stat.mode0_interrupt;
                if enable {
                    self.interrupt.borrow_mut().set_flag(ILCDSTAT);
                }
            }
            Drawing => {}
        };
    }
    pub fn set_ly(&mut self, ly: u8) {
        *self.ly.borrow_mut() = ly;
    }
    pub fn set_ly_interrupt(&mut self) {
        let ly = self.get_ly();
        let lyc = self.get_lyc();
        if ly == lyc {
            let enable = self.stat.lyc_ly_interrupt;
            if enable {
                self.interrupt.borrow_mut().set_flag(ILCDSTAT);
            }
        }
    }
    pub fn reset_ly(&mut self) {
        *self.ly.borrow_mut() = 0;
    }
    pub fn get_ly(&self) -> u8 {
        *self.ly.borrow()
    }
    pub fn set_lyc(&mut self, ly: u8) {
        *self.lyc.borrow_mut() = ly;
    }
    pub fn get_lyc(&self) -> u8 {
        *self.lyc.borrow()
    }
}
impl Memory for PpuMmu {
    fn get(&self, index: u16) -> u8 {
        match index {
            0xFF40 => self.lcdc.get(index),
            0xFF41 => self.stat.get(index),
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.get_ly(),
            0xFF45 => self.get_lyc(),
            0xFF47 => self.bgp,
            0xFF48 => self.op0,
            0xFF49 => self.op1,
            0xFF4A => self.wy,
            0xFF4B => self.wx,
            0xFF4F | 0x8000..=0x9FFF => self.vram.get(index),
            0xFF68 | 0xFF69 => self.bcp.get(index),
            0xFF6A | 0xFF6B => self.ocp.get(index),
            0xFE00..=0xFE9F => self.oam[(index - 0xFE00) as usize],
            _ => panic!("PpuMmu out of range"),
        }
    }
    fn set(&mut self, index: u16, value: u8) {
        match index {
            0xFF40 => self.lcdc.set(index, value),
            0xFF41 => self.stat.set(index, value),
            0xFF42 => self.scy = value,
            0xFF43 => self.scx = value,
            0xFF44 => {}
            0xFF45 => self.set_lyc(value),
            0xFF47 => self.bgp = value,
            0xFF48 => self.op0 = value,
            0xFF49 => self.op1 = value,
            0xFF4A => self.wy = value,
            0xFF4B => self.wx = value,
            0xFF4F | 0x8000..=0x9FFF => self.vram.set(index, value),
            0xFF68 | 0xFF69 => self.bcp.set(index, value),
            0xFF6A | 0xFF6B => self.ocp.set(index, value),
            0xFE00..=0xFE9F => self.oam[(index - 0xFE00) as usize] = value,
            _ => panic!("PpuMmu out of range"),
        }
    }
}
