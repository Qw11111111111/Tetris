use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::fs::File;

pub fn save(path: &Path, number: u64) -> io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(&number.to_le_bytes())?;
    Ok(())
}

pub fn read(path: &Path) -> io::Result<u64> {
    let mut file = File::open(path)?;
    let mut buffer = [0u8; 8];
    file.read_exact(&mut buffer)?;
    Ok(u64::from_le_bytes(buffer))
}