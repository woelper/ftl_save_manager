use anyhow::{Error, Result};

use byteorder::{ByteOrder, LittleEndian,ReadBytesExt};
use positioned_io::{self, ReadAt};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Sav {
    pub header: Header,
    pub player_name_len: u32,
    // pub player_name: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Header {
    pub version: i32,
    pub ships_defeated: i32,
    pub jumps_in_sector: i32,
    pub scrap_collected: i32,
    pub crew_recruited: i32,
    pub xxx: i32,
}

/// Get the FTL save file
pub fn get_save_info(filepath: &PathBuf) -> Result<String> {
    let file = std::fs::File::open(&filepath)?;

    // enough for i32
    let mut buf = vec![0; 4];
    // 32 is where the length of the ship string is stored
    file.read_exact_at(32, &mut buf)?;
    let ship_len:i32 = LittleEndian::read_i32(&buf);

    let mut buf = vec![0; ship_len as usize];
    file.read_exact_at(36, &mut buf)?;

    Ok(String::from_utf8_lossy(&buf).to_string().replace("?", ""))
}
