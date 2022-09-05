mod test {
    mod gb_test_roms {
        use crate::gameboy::GameBoy;
        #[test]
        fn test() {
            let bios_path = "";
            let rom_path = "tests/gb-test-roms/cpu_instrs/individual/01-special.gb";
            let mut gameboy = GameBoy::new(bios_path, rom_path);
            let expect = "01-special\n\n\nPassed";
            let mut cycle: usize = 0;
            while gameboy.mmu.borrow().log_msg.len() < expect.len() {
                gameboy.trick();
                cycle += 1;
                if cycle > 10743249 {
                  panic!("too long time");
                }
            }
            let str = &gameboy.mmu.borrow().log_msg;
            assert_eq!(&str[..], expect);
        }
    }
}
