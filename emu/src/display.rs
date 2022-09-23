extern crate minifb;
use minifb::{Key, Window, WindowOptions};

pub struct Display {
    width: usize,
    height: usize,
    pub window: Window,
}

impl Display {
    pub fn init(width: usize, height: usize) -> Self {
        let mut option = WindowOptions::default();
        option.resize = true;
        let c_scale = 2;
        option.scale = match c_scale {
            1 => minifb::Scale::X1,
            2 => minifb::Scale::X2,
            4 => minifb::Scale::X4,
            8 => minifb::Scale::X8,
            _ => panic!("Supported scale: 1, 2, 4 or 8"),
        };
        let mut window = Window::new(
            "Test - ESC to exit",
            width,
            height,
            option,
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
