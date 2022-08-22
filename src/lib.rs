pub mod cpu;
pub mod memory;
pub mod cartridge;
pub mod mmu;
pub mod util;
pub mod display;
pub mod test_rom;

use cpu::Cpu;
use mmu::Mmu;
use std::{cell::RefCell, rc::Rc};
#[test]
fn test() {
    let mmu = Mmu::new();
    let rc_refcell_mmu = Rc::new(RefCell::new(mmu));
    let mut cpu = Cpu::new(rc_refcell_mmu.clone());
    cpu.run();

}