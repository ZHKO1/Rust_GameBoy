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
            fn test_03_op_sp_hl() {
                let bios_path = "";
                let rom_path = "tests/gb-test-roms/cpu_instrs/individual/03-op sp,hl.gb";
                let mut gameboy = GameBoy::new(bios_path, rom_path);
                let expect = "03-op sp,hl\n\n\nPassed";
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
            fn test_04_op_r_imm() {
                let bios_path = "";
                let rom_path = "tests/gb-test-roms/cpu_instrs/individual/04-op r,imm.gb";
                let mut gameboy = GameBoy::new(bios_path, rom_path);
                let expect = "04-op r,imm\n\n\nPassed";
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
            fn test_05_op_rp() {
                let bios_path = "";
                let rom_path = "tests/gb-test-roms/cpu_instrs/individual/05-op rp.gb";
                let mut gameboy = GameBoy::new(bios_path, rom_path);
                let expect = "05-op rp\n\n\nPassed";
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
            fn test_06_ld_r_r() {
                let bios_path = "";
                let rom_path = "tests/gb-test-roms/cpu_instrs/individual/06-ld r,r.gb";
                let mut gameboy = GameBoy::new(bios_path, rom_path);
                let expect = "06-ld r,r\n\n\nPassed";
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
