#[derive(Clone, Copy, Eq, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum GameBoyMode {
    GB,
    GBC,
}
