use crate::interrupt::Interrupt;
use crate::interrupt::InterruptFlag::{VBlank as IVBlank, LCDSTAT as ILCDSTAT};
use crate::memory::Memory;
use crate::mmu::Mmu;
use crate::ppu::FetcherStatus::{GetTile, GetTileDataHigh, GetTileDataLow};
use crate::ppu::PixelType::{Sprite, Window, BG};
use crate::ppu::PpuStatus::{Drawing, HBlank, OAMScan, VBlank};
use crate::util::check_bit;
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
    wx: u8,
    wy: u8,
    cycles: u16,
    oam_x: u8,
    oam_y: u8,
    oam_priority: usize,
    x_flip: bool,
    y_flip: bool,
    palette: bool,
    bg_window_over_obj: bool,
    ptype: PixelType,
    mmu: Rc<RefCell<Mmu>>,
    status: FetcherStatus,
    tile_index: u16,
    tile_data_low: u8,
    tile_data_high: u8,
    buffer: Vec<Pixel>,
}
impl Fetcher {
    fn new(mmu: Rc<RefCell<Mmu>>) -> Self {
        Fetcher {
            scan_x: 0,
            scan_y: 0,
            scx: 0,
            scy: 0,
            wx: 0,
            wy: 0,
            oam_x: 0,
            oam_y: 0,
            x_flip: false,
            y_flip: false,
            palette: false,
            oam_priority: 40,
            bg_window_over_obj: false,
            ptype: BG,
            mmu,
            cycles: 0,
            status: GetTile,
            tile_index: 0,
            tile_data_low: 0,
            tile_data_high: 0,
            buffer: Vec::new(),
        }
    }
    fn init(&mut self, ptype: PixelType, x: u8, y: u8) {
        self.scan_x = x;
        self.scan_y = y;

        self.scx = 0;
        self.scy = 0;

        self.wx = 0;
        self.wy = 0;

        self.oam_x = 0;
        self.oam_y = 0;

        self.x_flip = false;
        self.y_flip = false;
        self.palette = false;
        self.bg_window_over_obj = false;
        self.ptype = ptype;

        self.cycles = 0;
        self.status = GetTile;

        self.tile_index = 0;
        self.tile_data_low = 0;
        self.tile_data_high = 0;

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
        let window_tile_map_area = self.mmu.borrow().ppu.lcdc.window_tile_map_area;
        let window_map_start: u16 = match window_tile_map_area {
            true => 0x9C00,
            false => 0x9800,
        };

        match self.ptype {
            BG => {
                self.scy = self.mmu.borrow().ppu.scy;
                self.scx = self.mmu.borrow().ppu.scx;
                let bg_map_x = (self.scan_x as u16 + self.scx as u16) % 256 / 8;
                let bg_map_y = (self.scan_y as u16 + self.scy as u16) % 256 / 8;
                let bg_map_index = bg_map_x + bg_map_y * 32;
                let bg_map_byte = self.mmu.borrow().get(bg_map_start + bg_map_index);
                let tile_index: u16 = if bg_window_tile_data_area {
                    0x8000 + bg_map_byte as u16 * 8 * 2
                } else {
                    (0x9000 as i32 + (bg_map_byte as i8) as i32 * 8 * 2) as u16
                };
                tile_index
            }
            Window => {
                self.wy = self.mmu.borrow().ppu.wy;
                self.wx = self.mmu.borrow().ppu.wx;
                let bg_map_x = (self.scan_x as u16 + 7 - self.wx as u16) % 256 / 8;
                let bg_map_y = (self.scan_y as u16 - self.wy as u16) % 256 / 8;
                let bg_map_index = bg_map_x + bg_map_y * 32;
                let bg_map_byte = self.mmu.borrow().get(window_map_start + bg_map_index);
                let tile_index: u16 = if bg_window_tile_data_area {
                    0x8000 + bg_map_byte as u16 * 8 * 2
                } else {
                    (0x9000 as i32 + (bg_map_byte as i8) as i32 * 8 * 2) as u16
                };
                tile_index
            }
            Sprite => self.tile_index,
        }
    }
    fn get_tile_data_low(&self) -> u8 {
        let tile_index = self.tile_index;
        match self.ptype {
            BG => {
                let tile_pixel_y = (self.scan_y as u16 + self.scy as u16) % 8;
                let tile_byte_low = self.mmu.borrow().get(tile_index + tile_pixel_y * 2);
                tile_byte_low
            }
            Window => {
                let tile_pixel_y = (self.scan_y as u16 - self.wy as u16) % 8;
                let tile_byte_low = self.mmu.borrow().get(tile_index + tile_pixel_y * 2);
                tile_byte_low
            }
            Sprite => {
                let mut tile_pixel_y = (self.scan_y as u16 - (self.oam_y - 16) as u16) % 8;
                if self.y_flip {
                    tile_pixel_y = (8 - 1) - tile_pixel_y;
                }
                let tile_byte_low = self.mmu.borrow().get(tile_index + tile_pixel_y * 2);
                tile_byte_low
            }
        }
    }
    fn get_tile_data_high(&self) -> u8 {
        let tile_index = self.tile_index;
        match self.ptype {
            BG => {
                let tile_pixel_y = (self.scan_y as u16 + self.scy as u16) % 8;
                let tile_byte_high = self.mmu.borrow().get(tile_index + tile_pixel_y * 2 + 1);
                tile_byte_high
            }
            Window => {
                let tile_pixel_y = (self.scan_y as u16 - self.wy as u16) % 8;
                let tile_byte_high = self.mmu.borrow().get(tile_index + tile_pixel_y * 2 + 1);
                tile_byte_high
            }
            Sprite => {
                let mut tile_pixel_y = (self.scan_y as u16 - (self.oam_y - 16) as u16) % 8;
                if self.y_flip {
                    tile_pixel_y = (8 - 1) - tile_pixel_y;
                }
                let tile_byte_high = self.mmu.borrow().get(tile_index + tile_pixel_y * 2 + 1);
                tile_byte_high
            }
        }
    }
    fn get_buffer(&mut self) -> Vec<Pixel> {
        let mut result = Vec::new();
        let mut get_pixel_bit: Box<dyn Fn(u8) -> u8> = Box::new(|index: u8| 8 - index - 1);
        let buffer_index_start = match self.ptype {
            BG => (self.scan_x as u16 + self.scx as u16) % 8,
            Window => (self.scan_x as u16 + 7 - self.wx as u16) % 8,
            Sprite => {
                if self.x_flip {
                    get_pixel_bit = Box::new(|index: u8| index);
                }
                (self.scan_x as u16 + 8 - self.oam_x as u16) % 8
            }
        };
        for buffer_index in buffer_index_start..8 {
            let pixel_bit = get_pixel_bit(buffer_index as u8);
            let pixel_low = check_bit(self.tile_data_low, pixel_bit as u8);
            let pixel_high = check_bit(self.tile_data_high, pixel_bit as u8);
            let pvalue = (pixel_low as u8) | ((pixel_high as u8) << 1);
            let pcolor = self.get_color_index(self.ptype, pvalue, self.palette);
            result.push(Pixel {
                ptype: self.ptype,
                pvalue,
                pcolor,
                bg_window_over_obj: self.bg_window_over_obj,
                oam_priority: self.oam_priority,
            });
        }
        result
    }
    fn get_color_index(&self, ptype: PixelType, pvalue: u8, is_obp1: bool) -> u8 {
        let palette = match ptype {
            BG | Window => self.mmu.borrow().ppu.bgp,
            Sprite => {
                if is_obp1 {
                    self.mmu.borrow().ppu.op1
                } else {
                    self.mmu.borrow().ppu.op0
                }
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

#[derive(Clone, Copy)]
struct OAM {
    y: u8,
    x: u8,
    tile_index: u8,
    bg_window_over_obj: bool,
    x_flip: bool,
    y_flip: bool,
    palette: bool,
    priority: usize,
}
impl OAM {
    fn new(y: u8, x: u8, tile_index: u8, flags: u8, priority: usize) -> Self {
        Self {
            y,
            x,
            tile_index,
            bg_window_over_obj: check_bit(flags, 7),
            x_flip: check_bit(flags, 5),
            y_flip: check_bit(flags, 6),
            palette: check_bit(flags, 4),
            priority,
        }
    }
    fn is_scaned(&self, ly: u8) -> bool {
        if self.y < 16 {
            return false;
        }
        let y_start = self.y as i32 - 16;
        let y_end = self.y as i32 + 8 - 16;
        ((ly as i32) >= y_start) && ((ly as i32) < y_end) && (self.x != 0)
    }
}

enum FifoTrick {
    BgWindow,
    Sprite,
}
struct FIFO {
    x: u8,
    y: u8,
    status: FifoTrick,
    mmu: Rc<RefCell<Mmu>>,
    fetcher: Fetcher,
    sprite_queue: VecDeque<Pixel>,
    queue: VecDeque<Pixel>,
    oam: Vec<OAM>,
}
impl FIFO {
    fn new(mmu: Rc<RefCell<Mmu>>) -> Self {
        let fetcher = Fetcher::new(mmu.clone());
        FIFO {
            x: 0,
            y: 0,
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
        self.sprite_queue.clear();
        self.queue.clear();
        self.oam.clear();
        self.status = FifoTrick::BgWindow;
        self.fetcher.init(self.get_fetcher_ptype(self.x), self.x, y);
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
                    let front = self.front().unwrap();
                    let new_fetch_event = self.check_overlap(front.ptype, self.x);
                    if let Some(event) = new_fetch_event {
                        match event {
                            Window => {
                                self.queue.clear();
                                self.fetcher.init(Window, self.x, self.y);
                                return None;
                            }
                            Sprite => {
                                self.status = FifoTrick::Sprite;
                                self.fetcher.init(Sprite, self.x, self.y);
                                let oam = self.oam_pop(self.x).unwrap();
                                self.fetcher.oam_x = oam.x;
                                self.fetcher.oam_y = oam.y;
                                self.fetcher.oam_priority = oam.priority;
                                self.fetcher.x_flip = oam.x_flip;
                                self.fetcher.y_flip = oam.y_flip;
                                self.fetcher.bg_window_over_obj = oam.bg_window_over_obj;
                                self.fetcher.palette = oam.palette;
                                self.fetcher.tile_index = 0x8000 + (oam.tile_index as u16) * 16;
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
                if self.fetcher.buffer.len() > 0 {
                    if self.queue.len() <= 8 {
                        for pixel in self.fetcher.buffer.clone().into_iter() {
                            self.push_back(pixel);
                        }
                        let fetcher_x = self.x + self.queue.len() as u8;
                        self.fetcher
                            .init(self.get_fetcher_ptype(fetcher_x), fetcher_x, self.y);
                    }
                } else {
                    self.fetcher.trick();
                }
                result
            }
            FifoTrick::Sprite => {
                if self.fetcher.buffer.len() > 0 {
                    let sprite_queue_len = self.sprite_queue.len();
                    for (index, pixel) in self.fetcher.buffer.iter().enumerate() {
                        if index < sprite_queue_len {
                            let origin_pixel = self.sprite_queue[index];
                            if pixel.pvalue != 0 && pixel.oam_priority < origin_pixel.oam_priority {
                                self.queue[index] = *pixel;
                            }
                        } else {
                            self.sprite_queue.push_back(*pixel);
                        }
                    }
                    self.status = FifoTrick::BgWindow;
                    let fetcher_x = self.x + self.queue.len() as u8;
                    self.fetcher
                        .init(self.get_fetcher_ptype(fetcher_x), fetcher_x, self.y);
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
    fn get_fetcher_ptype(&self, x: u8) -> PixelType {
        if self.check_window(x) {
            Window
        } else {
            BG
        }
    }
    fn push_back(&mut self, pixel: Pixel) {
        self.queue.push_back(pixel);
    }
    fn front(&mut self) -> Option<Pixel> {
        let sprite_pixel_option = self.sprite_queue.front();
        match sprite_pixel_option {
            Some(sprite_pixel) => Some(*sprite_pixel),
            None => self.queue.front().map(|x| *x),
        }
    }
    fn pop_front(&mut self) -> Option<Pixel> {
        let sprite_pixel_option = self.sprite_queue.pop_front();
        match sprite_pixel_option {
            Some(sprite_pixel) => {
                let bg_pixel = self.queue.pop_front().unwrap();
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
            None => self.queue.pop_front(),
        }
    }
    fn clear(&mut self) {
        self.queue.clear();
        self.oam.clear();
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
    cycles: u32,
    fifo: FIFO,
    mmu: Rc<RefCell<Mmu>>,
    ly_buffer: Vec<u32>,
    lcd_enable: bool,
    pub frame_buffer: [u32; WIDTH * HEIGHT],
}
impl PPU {
    pub fn new(mmu: Rc<RefCell<Mmu>>) -> Self {
        let fifo = FIFO::new(mmu.clone());
        let mut ppu = PPU {
            cycles: 0,
            mmu,
            fifo,
            lcd_enable: true,
            ly_buffer: Vec::new(),
            frame_buffer: [Color::WHITE as u32; WIDTH * HEIGHT],
        };
        ppu.set_mode(OAMScan);
        ppu
    }
    pub fn trick(&mut self) {
        let lcd_enable = self.mmu.borrow().ppu.lcdc.lcd_ppu_enable;
        if !lcd_enable {
            if self.lcd_enable == lcd_enable {
                return;
            }
            self.cycles = 0;
            self.ly_buffer = Vec::new();
            self.frame_buffer = [Color::WHITE as u32; WIDTH * HEIGHT];
            self.fifo = FIFO::new(self.mmu.clone());
            self.mmu.borrow_mut().ppu.reset_ly();
        } else {
            if self.lcd_enable != lcd_enable {
                self.mmu.borrow_mut().ppu.stat.mode_flag = OAMScan;
            }
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
                        self.ly_buffer.push(self.get_pixel_color(pixel.pcolor));
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
                    let ly = self.get_ly();
                    if self.cycles == 0 {
                        self.set_ly_interrupt();
                        if ly == 144 {
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
    }
    fn get_pixel_color(&self, color_value: u8) -> u32 {
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
    fn oam_scan(&self) -> Vec<OAM> {
        let ly = self.get_ly();
        let mut result = vec![];
        for index in 00..40 {
            let oam_address = 0xFE00 + (index as u16) * 4;
            let y = self.mmu.borrow().get(oam_address);
            let x = self.mmu.borrow().get(oam_address + 1);
            let tile_index = self.mmu.borrow().get(oam_address + 2);
            let flags = self.mmu.borrow().get(oam_address + 3);
            let oam = OAM::new(y, x, tile_index, flags, index);
            if oam.is_scaned(ly) {
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
                self.ly_buffer = Vec::new();
            }
            Drawing => {}
            HBlank => {
                self.ly_buffer.clear();
                self.fifo.clear();
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

pub struct PpuMmu {
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
    vram: [u8; 0x9FFF - 0x8000 + 1],
    oam: [u8; 0xFE9F - 0xFE00 + 1],
}
impl PpuMmu {
    pub fn new(interrupt: Rc<RefCell<Interrupt>>) -> Self {
        let lcdc = LCDC::new();
        let ly = Rc::new(RefCell::new(0));
        let lyc = Rc::new(RefCell::new(0));
        let stat = STAT::new(ly.clone(), lyc.clone());
        Self {
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
            vram: [0; 0x9FFF - 0x8000 + 1],
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
            0xff40 => self.lcdc.get(index),
            0xff41 => self.stat.get(index),
            0xff42 => self.scy,
            0xff43 => self.scx,
            0xff44 => self.get_ly(),
            0xff45 => self.get_lyc(),
            0xff47 => self.bgp,
            0xff48 => self.op0,
            0xff49 => self.op1,
            0xff4A => self.wy,
            0xff4B => self.wx,
            0x8000..=0x9FFF => self.vram[(index - 0x8000) as usize],
            0xFE00..=0xFE9F => self.oam[(index - 0xFE00) as usize],
            _ => panic!("PpuMmu out of range"),
        }
    }
    fn set(&mut self, index: u16, value: u8) {
        match index {
            0xff40 => self.lcdc.set(index, value),
            0xff41 => self.stat.set(index, value),
            0xff42 => self.scy = value,
            0xff43 => self.scx = value,
            0xff44 => {}
            0xff45 => self.set_lyc(value),
            0xff47 => self.bgp = value,
            0xff48 => self.op0 = value,
            0xff49 => self.op1 = value,
            0xff4A => self.wy = value,
            0xff4B => self.wx = value,
            0x8000..=0x9FFF => self.vram[(index - 0x8000) as usize] = value,
            0xFE00..=0xFE9F => self.oam[(index - 0xFE00) as usize] = value,
            _ => panic!("PpuMmu out of range"),
        }
    }
}
