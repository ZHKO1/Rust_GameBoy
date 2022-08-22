pub fn u16_from_2u8(low: u8, high: u8) -> u16 {
    u16::from(low) + (u16::from(high) << 8)
}

pub fn u8u8_from_u16(value: u16) -> (u8, u8) {
  let value_low = (value & 0x00ff) as u8;
  let value_high = ((value & 0xff00) >> 8) as u8;
  (value_low, value_high)
}
