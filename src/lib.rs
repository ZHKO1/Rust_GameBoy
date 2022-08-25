pub mod cartridge;
pub mod cpu;
pub mod display;
pub mod memory;
pub mod mmu;
pub mod ppu;
pub mod test_rom;
pub mod util;

use cpu::Cpu;
use display::Display;
use mmu::Mmu;
use ppu::PPU;
use std::{cell::RefCell, rc::Rc};
#[test]
fn test() {
    let mmu = Mmu::new();
    let rc_refcell_mmu = Rc::new(RefCell::new(mmu));
    let mut cpu = Cpu::new(rc_refcell_mmu.clone());
    let mut ppu = PPU::new(rc_refcell_mmu.clone());
    let mut display = Display::init(256, 144);
    let mut cycle: u32 = 0;
    while display.window.is_open() {
        cycle += 1;
        cpu.trick();
        ppu.trick();
        let buffer = &ppu.pixel_array;
        if cycle == 70224 {
            display.update_with_buffer(&mut buffer.to_vec());
            cycle = 0;
        }
    }
}
