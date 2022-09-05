mod test {
    mod gb_test_roms {

        mod cpu_instrs {
            use crate::gameboy::GameBoy;
            #[test]
            fn test_01_special() {
                let bios_path = "";
                let rom_path = "tests/gb-test-roms/cpu_instrs/individual/01-special.gb";
                let mut gameboy = GameBoy::new(bios_path, rom_path);
                let expect = "01-special\n\n\nPassed";
                let mut cycle: usize = 0;
                while gameboy.mmu.borrow().log_msg.len() < expect.len() {
                    gameboy.trick();
                    cycle += 1;
                    if cycle > 103647161 {
                        panic!("too long time");
                    }
                }
                let str = &gameboy.mmu.borrow().log_msg;
                assert_eq!(&str[..], expect);
            }

            
            #[test]
            fn test_09_op_r_r() {
                let bios_path = "";
                let rom_path = "tests/gb-test-roms/cpu_instrs/individual/09-op r,r.gb";
                let mut gameboy = GameBoy::new(bios_path, rom_path);
                let expect = "09-op r,r\n\n\nPassed";
                let mut cycle: usize = 0;
                while gameboy.mmu.borrow().log_msg.len() < expect.len() {
                    gameboy.trick();
                    cycle += 1;
                    if cycle > 103647161 {
                        panic!("too long time");
                    }
                }
                let str = &gameboy.mmu.borrow().log_msg;
                assert_eq!(&str[..], expect);
            }

            #[test]
            fn test_10_bit_ops() {
                let bios_path = "";
                let rom_path = "tests/gb-test-roms/cpu_instrs/individual/10-bit ops.gb";
                let mut gameboy = GameBoy::new(bios_path, rom_path);
                let expect = "10-bit ops\n\n\nPassed";
                let mut cycle: usize = 0;
                while gameboy.mmu.borrow().log_msg.len() < expect.len() {
                    gameboy.trick();
                    cycle += 1;
                    if cycle > 103647161 {
                        panic!("too long time");
                    }
                }
                let str = &gameboy.mmu.borrow().log_msg;
                assert_eq!(&str[..], expect);
            }

            #[test]
            fn test_11_op_a_hl() {
                let bios_path = "";
                let rom_path = "tests/gb-test-roms/cpu_instrs/individual/11-op a,(hl).gb";
                let mut gameboy = GameBoy::new(bios_path, rom_path);
                let expect = "11-op a,(hl)\n\n\nPassed";
                let mut cycle: usize = 0;
                while gameboy.mmu.borrow().log_msg.len() < expect.len() {
                    gameboy.trick();
                    cycle += 1;
                    if cycle > 103647161 {
                        panic!("too long time");
                    }
                }
                let str = &gameboy.mmu.borrow().log_msg;
                assert_eq!(&str[..], expect);
            }
        }
    }
}
