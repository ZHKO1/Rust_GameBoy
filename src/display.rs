extern crate minifb;
use minifb::{Key, Window, WindowOptions};

pub struct Display {
    width: usize,
    height: usize,
    pub window: Window,
}

impl Display {
    pub fn init(width: usize, height: usize) -> Self {
        let mut window = Window::new(
            "Test - ESC to exit",
            width,
            height,
            WindowOptions::default(),
        )
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });
        window.limit_update_rate(Some(std::time::Duration::from_micros(16666)));
        Display {
            width,
            height,
            window: window,
        }
    }
    pub fn update_with_buffer(&mut self, buffer: &mut Vec<u32>) {
        self.window
            .update_with_buffer(&buffer, self.width, self.height)
            .unwrap();
    }
    pub fn is_open(&mut self) -> bool {
        self.window.is_open()
    }
    pub fn is_key_down(&mut self, key: Key) -> bool {
        self.window.is_key_down(key)
    }
}
