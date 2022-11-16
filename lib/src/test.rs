mod test {
    mod gb_test_roms {
        macro_rules! test {
            ($func: ident, $path:expr, $game:expr, $expect:expr) => {
                #[test]
                fn $func() {
                    use crate::gameboy::GameBoy;
                    use crate::util::read_rom;
                    use std::time::SystemTime;
                    let bios_path = "";
                    let rom_path = format!("{}{}{}{}", "tests/gb-test-roms/", $path, $game, ".gb");
                    let bios = read_rom(bios_path).unwrap_or(vec![]);
                    let rom = read_rom(rom_path).unwrap();
                    let mut gameboy = GameBoy::new(bios, rom);
                    let expect = format!("{}", $expect);
                    let expect = expect.as_bytes().to_vec();
                    let start = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    let mut clocks: usize = 0;
                    while gameboy.mmu.borrow().log_msg.len() < expect.len() {
                        gameboy.trick();
                        clocks += 1;
                        if clocks >= 100000 {
                            let nowtime = SystemTime::now()
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap()
                                .as_secs();
                            if nowtime - start > 60 * 10 {
                                panic!("too long time");
                            }
                            clocks = 0;
                        }
                    }
                    let str = &gameboy.mmu.borrow().log_msg;
                    assert_eq!(&str[..], &expect[..]);
                }
            };
            ($func: ident, $path:expr, $game:expr) => {
                test!($func, $path, $game, format!("{}\n\n\nPassed", $game));
            };
        }
        mod cpu_instrs {
            test!(test_01_special, "cpu_instrs/individual/", "01-special");
            test!(
                test_02_interrupts,
                "cpu_instrs/individual/",
                "02-interrupts"
            );
            test!(test_03_op_sp_hl, "cpu_instrs/individual/", "03-op sp,hl");
            test!(test_04_op_r_imm, "cpu_instrs/individual/", "04-op r,imm");
            test!(test_05_op_rp, "cpu_instrs/individual/", "05-op rp");
            test!(test_06_ld_r_r, "cpu_instrs/individual/", "06-ld r,r");
            test!(
                test_07_jr_jp_call_ret_rst,
                "cpu_instrs/individual/",
                "07-jr,jp,call,ret,rst"
            );
            test!(
                test_08_misc_instrs,
                "cpu_instrs/individual/",
                "08-misc instrs"
            );
            test!(test_09_op_r_r, "cpu_instrs/individual/", "09-op r,r");
            test!(test_10_bit_ops, "cpu_instrs/individual/", "10-bit ops");
            test!(test_11_op_a_hl, "cpu_instrs/individual/", "11-op a,(hl)");

            test!(test_cpu_instrs, "cpu_instrs/", "cpu_instrs", "cpu_instrs\n\n01:ok  02:ok  03:ok  04:ok  05:ok  06:ok  07:ok  08:ok  09:ok  10:ok  11:ok  \n\nPassed all tests");
        }
        mod instr_timing {
            test!(test_instr_timing, "instr_timing/", "instr_timing");
        }
    }

    mod mooneye_test_suite {
        macro_rules! test {
            ($func: ident, $path:expr, $game:expr) => {
                #[test]
                fn $func() {
                    use crate::gameboy::GameBoy;
                    use crate::util::read_rom;
                    use std::time::SystemTime;
                    let bios_path = "";
                    let rom_path = format!("{}{}{}{}", "tests/mts/", $path, $game, ".gb");
                    let bios = read_rom(bios_path).unwrap_or(vec![]);
                    let rom = read_rom(rom_path).unwrap();
                    let mut gameboy = GameBoy::new(bios, rom);
                    let expect = vec![3, 5, 8, 13, 21, 34];
                    let start = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    let mut clocks: usize = 0;
                    while gameboy.mmu.borrow().log_msg.len() < expect.len() {
                        gameboy.trick();
                        clocks += 1;
                        if clocks >= 100000 {
                            let nowtime = SystemTime::now()
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap()
                                .as_secs();
                            if nowtime - start > 60 * 10 {
                                panic!("too long time");
                            }
                            clocks = 0;
                        }
                    }
                    let str = &gameboy.mmu.borrow().log_msg;
                    assert_eq!(&str[..], &expect[..]);
                }
            };
        }
        mod emulator_only {
            mod mbc1 {
                test!(bits_bank1, "emulator-only/mbc1/", "bits_bank1");
                test!(bits_bank2, "emulator-only/mbc1/", "bits_bank2");
                test!(bits_mode, "emulator-only/mbc1/", "bits_mode");
                test!(bits_ramg, "emulator-only/mbc1/", "bits_ramg");
                test!(rom_1mb, "emulator-only/mbc1/", "rom_1Mb");
                test!(rom_2mb, "emulator-only/mbc1/", "rom_2Mb");
                test!(rom_4mb, "emulator-only/mbc1/", "rom_4Mb");
                // test!(rom_8mb, "emulator-only/mbc1/", "rom_8Mb");
                // test!(rom_16mb, "emulator-only/mbc1/", "rom_16Mb");
                test!(rom_512kb, "emulator-only/mbc1/", "rom_512kb");
            }
            mod mbc2 {
                test!(bits_ramg, "emulator-only/mbc2/", "bits_ramg");
                test!(bits_romb, "emulator-only/mbc2/", "bits_romb");
                test!(bits_unused, "emulator-only/mbc2/", "bits_unused");
                test!(ram, "emulator-only/mbc2/", "ram");
                test!(rom_1mb, "emulator-only/mbc2/", "rom_1Mb");
                test!(rom_2mb, "emulator-only/mbc2/", "rom_2Mb");
                test!(rom_512kb, "emulator-only/mbc2/", "rom_512kb");
            }

            mod mbc5 {
                test!(rom_1mb, "emulator-only/mbc5/", "rom_1Mb");
                test!(rom_2mb, "emulator-only/mbc5/", "rom_2Mb");
                test!(rom_4mb, "emulator-only/mbc5/", "rom_4Mb");
                test!(rom_8mb, "emulator-only/mbc5/", "rom_8Mb");
                test!(rom_16mb, "emulator-only/mbc5/", "rom_16Mb");
                test!(rom_32mb, "emulator-only/mbc5/", "rom_32Mb");
                test!(rom_64mb, "emulator-only/mbc5/", "rom_64Mb");
                test!(rom_512kb, "emulator-only/mbc5/", "rom_512kb");
            }
        }
    }

    mod dmg_acid2 {

        #[test]
        fn check_frame_buffer() {
            use crate::gameboy::GameBoy;
            use crate::util::read_rom;
            use std::fs::File;
            use std::io::Read;
            // use std::io::Write;
            use std::path::Path;
            use std::time::SystemTime;

            let expect = get_expect();
            let bios_path = "";
            let rom_path = format!("tests/dmg_acid2/dmg-acid2.gb");
            let bios = read_rom(bios_path).unwrap_or(vec![]);
            let rom = read_rom(rom_path).unwrap();
            let mut gameboy = GameBoy::new(bios, rom);
            let start = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let buffer: String;
            let mut clocks: usize = 0;
            loop {
                gameboy.trick();
                clocks += 1;
                if clocks >= 100000 {
                    let nowtime = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    if nowtime - start > 3 {
                        let frame_buffer = gameboy.get_frame_buffer();
                        buffer = frame_buffer
                            .iter()
                            .map(|x| x.to_string())
                            .collect::<Vec<String>>()
                            .join(",");
                        break;
                    }
                    clocks = 0;
                }
            }
            assert_eq!(buffer, expect);

            fn get_expect() -> String {
                let path = Path::new("tests/dmg_acid2/expect");
                let display = path.display();
                let mut file = match File::open(&path) {
                    Err(why) => panic!("File.open {display} Err:{}", why),
                    Ok(file) => file,
                };
                let mut expect = String::new();
                match file.read_to_string(&mut expect) {
                    Err(_) => panic!("file {display} read_to_string"),
                    Ok(_) => {}
                };
                expect
            }
        }
    }
}
