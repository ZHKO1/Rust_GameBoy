mod utils;
use rust_gameboy_core::gameboy::GameBoy as Gameboy_;
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
pub struct Gameboy {
    inner: Gameboy_,
}

#[wasm_bindgen]
impl Gameboy {
    #[wasm_bindgen(constructor)]
    pub fn new(bios: Vec<u8>, rom: Vec<u8>) -> Result<Gameboy, JsValue> {
        utils::set_panic_hook();
        let inner = Gameboy_::new(bios, rom);
        Ok(Self { inner })
    }

    pub fn frame(&mut self) -> *const u32 {
        while !self.inner.trick() {}
        let frame_buffer = self.inner.get_frame_buffer();
        frame_buffer.as_ptr()
    }

    pub fn input(&mut self, key: JoyPadKey, is_pressed: bool) {
        let key_ = JoyPadKey_::from(key);
        self.inner.input(key_, is_pressed)
    }

    pub fn lcd_width() -> usize {
        WIDTH
    }

    pub fn lcd_height() -> usize {
        HEIGHT
    }
}
