use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[repr(C)]
pub struct Header {
    pub version: u32,
    pub difficulty: u32,
    pub ships_defeated: u32,
    pub jumps_in_sector: u32,
    pub scrap_collected: u32,
    pub crew_recruited: u32,
}
