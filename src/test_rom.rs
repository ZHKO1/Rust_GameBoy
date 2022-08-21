use std::io::{self, Read};

pub fn start() {
  let result_rom = get_rom_buffer("./tests/SML.gb");
  let rom = match result_rom {
      Err(error) => panic!("read rom error{:?}", error),
      Ok(rom) => rom,
  };

  for data in rom[..].iter() {
      println!("{:02x}", data);
  }


  /*
  for data in rom[0x104..0x134].iter() {
      println!("{:02x}", data);
  }
  */

  let title = get_rom_title(&rom);
  println!("{:?}", title);
}

fn get_rom_buffer(path: &str) -> io::Result<Vec<u8>> {
  let mut rom = vec![];
  let mut file = std::fs::File::open(path)?;
  file.read_to_end(&mut rom)?;
  Ok(rom)
}

fn get_rom_slice(rom: &Vec<u8>, start: u16, end: u16) -> &[u8] {
  &rom[(start as usize)..((end + 1) as usize)]
}

fn get_rom_title(rom: &Vec<u8>) -> String {
  let start = 0x134;
  let end = match &rom[0x0143] {
      &0x80 => 0x013E,
      _ => 0x0143,
  };
  let str_buff = get_rom_slice(rom, start, end);
  let mut result = String::new();
  for ch in str_buff.iter() {
      result.push(*ch as char);
  }
  result
}

#[test]
fn test(){
  start();
}