extern crate minifb;
use minifb::{Key, Window, WindowOptions};

pub fn show_pixel_window() {
    const WIDTH: usize = 640;
    const HEIGHT: usize = 360;
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });
    window.limit_update_rate(Some(std::time::Duration::from_micros(166000)));
    while window.is_open() && !window.is_key_down(Key::Escape) {
        for i in buffer.iter_mut() {
            *i = 0xffffff00; // write something more funny here!
                             // j = j.wrapping_add(1);
        }
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}

#[test]
fn test(){
  
}