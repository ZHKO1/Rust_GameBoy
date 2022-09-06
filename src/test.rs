mod test {
    mod gb_test_roms {

        mod cpu_instrs {
            macro_rules! test {
                ($func: ident, $x:expr) => {
                    #[test]
                    fn $func() {
                        use crate::gameboy::GameBoy;
                        let bios_path = "";
                        let rom_path = format!(
                            "{}{}{}",
                            "tests/gb-test-roms/cpu_instrs/individual/", $x, ".gb"
                        );
                        let mut gameboy = GameBoy::new(bios_path, rom_path);
                        let expect = format!("{}\n\n\nPassed", $x);
                        let mut cycle: usize = 0;
                        while gameboy.mmu.borrow().log_msg.len() < expect.len() {
                            gameboy.trick();
                            cycle += 1;
                            if cycle > 103647161 {
                                panic!("too long time");
                            }
                        }
                        let str = &gameboy.mmu.borrow().log_msg;
                        assert_eq!(&str[..], &expect[..]);
                    }
                };
            }
            test!(test_01_special, "01-special");
            test!(test_02_interrupts, "02-interrupts");
            test!(test_03_op_sp_hl, "03-op sp,hl");
            test!(test_04_op_r_imm, "04-op r,imm");
            test!(test_05_op_rp, "05-op rp");
            test!(test_06_ld_r_r, "06-ld r,r");
            test!(test_07_jr_jp_call_ret_rst, "07-jr,jp,call,ret,rst");
            test!(test_08_misc_instrs, "08-misc instrs");
            test!(test_09_op_r_r, "09-op r,r");
            test!(test_10_bit_ops, "10-bit ops");
            test!(test_11_op_a_hl, "11-op a,(hl)");
        }
    }
}
