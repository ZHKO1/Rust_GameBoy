// use chrono::*;
use rust_gameboy::cpu::Cpu;
use rust_gameboy::display::Display;
use rust_gameboy::mmu::Mmu;
use rust_gameboy::ppu::PPU;
use std::{cell::RefCell, rc::Rc};
fn main() {
    let mmu = Mmu::new("tests/SML.gb");
    let rc_refcell_mmu = Rc::new(RefCell::new(mmu));
    let mut cpu = Cpu::new(rc_refcell_mmu.clone());
    let mut ppu = PPU::new(rc_refcell_mmu.clone());
    let mut display = Display::init(160, 144);
    let mut cycle: u32 = 0;
    // let mut start_time = Local::now().time();
    // let mut frames = 0;
    while display.window.is_open() {
        /*
        if cycle == 0 {
            if frames == 59 {
                println!("60帧所耗时间 = {}", Local::now().time() - start_time);
                start_time = Local::now().time();
                frames = 0;
            }
        }
         */
        cycle += 1;
        cpu.trick();
        ppu.trick();
        let buffer = &ppu.frame_buffer;
        if cycle == 70224 {
            display.update_with_buffer(&mut buffer.to_vec());
            cycle = 0;
            // frames += 1;
        }
    }
}
