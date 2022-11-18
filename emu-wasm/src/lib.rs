mod utils;
use std::io::Read;

use rust_gameboy_core::gameboy::GameBoy as GameBoy_;
use rust_gameboy_core::gameboy::{HEIGHT, WIDTH};
use rust_gameboy_core::joypad::JoyPadKey as JoyPadKey_;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

extern crate web_sys;
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen]
pub enum JoyPadKey {
    Right,
    Left,
    Up,
    Down,
    A,
    B,
    Select,
    Start,
}
impl From<JoyPadKey> for JoyPadKey_ {
    fn from(joypad: JoyPadKey) -> Self {
        match joypad {
            JoyPadKey::Up => JoyPadKey_::Up,
            JoyPadKey::Down => JoyPadKey_::Down,
            JoyPadKey::Left => JoyPadKey_::Left,
            JoyPadKey::Right => JoyPadKey_::Right,
            JoyPadKey::A => JoyPadKey_::A,
            JoyPadKey::B => JoyPadKey_::B,
            JoyPadKey::Select => JoyPadKey_::Select,
            JoyPadKey::Start => JoyPadKey_::Start,
        }
    }
}

#[wasm_bindgen]
pub struct GameBoy {
    bios: Vec<u8>,
    rom: Vec<u8>,
    inner: Option<GameBoy_>,
}

#[wasm_bindgen]
impl GameBoy {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<GameBoy, JsValue> {
        utils::set_panic_hook();
        Ok(Self {
            bios: vec![],
            rom: vec![],
            inner: None,
        })
    }

    pub fn load_cartridge(&mut self, rom: Vec<u8>) {
        self.rom = rom;
    }

    pub fn load_bios(&mut self, bios: Vec<u8>) {
        self.bios = bios;
    }

    pub fn start(&mut self) {
        let cartridge = GameBoy_::get_cartridge(self.rom.clone());
        let inner = GameBoy_::new(self.bios.clone(), cartridge);
        self.inner = Some(inner);
    }

    pub fn frame(&mut self) -> *const u32 {
        if let Some(inner) = self.inner.as_mut() {
            while !inner.trick() {}
            let frame_buffer = inner.get_frame_buffer();
            frame_buffer.as_ptr()
        } else {
            panic!("Please execte gameboy.start().")
        }
    }

    pub fn is_gbc(&mut self) -> bool {
        let cartridge = GameBoy_::get_cartridge(self.rom.clone());
        cartridge.gbc_flag()
    }

    pub fn input(&mut self, key: JoyPadKey, is_pressed: bool) {
        let key_ = JoyPadKey_::from(key);
        if let Some(inner) = self.inner.as_mut() {
            inner.input(key_, is_pressed);
        }
    }

    pub fn lcd_width() -> usize {
        WIDTH
    }

    pub fn lcd_height() -> usize {
        HEIGHT
    }
}
